# V1 Rate Limiting Architecture

This document describes the design, configuration, and execution details of the rate limiting layer built into the Axum backend.

---

## 1. Core Architecture

The rate limiting system consists of two sequential layers integrated with a shared **Redis database**:
1. **Global Rate Limiter (100 RPS per IP)**: Applied globally at the root router level to shield the application server.
2. **Route-Specific Rate Limiter**: Applied selectively to sub-routers to enforce specific business limits on authentication, sessions, and profile routes.

```text
Incoming Request
       │
       ▼
┌──────────────────────────────────────────────┐
│ Global IP Rate Limiter (100 RPS per IP)      │   [Registered in app_router]
└──────────────────────┬───────────────────────┘
                       │ (Passed)
                       ▼
┌──────────────────────────────────────────────┐
│ Authentication Middleware (require_auth)     │   [Decodes JWT cookies]
└──────────────────────┬───────────────────────┘
                       │ (Passed)
                       ▼
┌──────────────────────────────────────────────┐
│ Route-Specific Rate Limiter                  │   [Extracts custom keys: User ID, Email, etc.]
└──────────────────────┬───────────────────────┘
                       │ (Passed)
                       ▼
               Target Handler
```

---

## 2. Sliding Window Log Algorithm (Atomic Lua script)

To guarantee exact rate limiting without race conditions under concurrent requests, limits are managed atomically inside a single Redis round-trip using a **Sliding Window Log** algorithm.

### Sorted Set (`ZSET`) Storage
Each rate limiting key maps to a Redis Sorted Set:
* **Score**: Unix timestamp of the request in milliseconds (`now`).
* **Member**: A unique string of format `timestamp:UUID` (to ensure multiple requests within the same millisecond are stored as distinct elements).

### Lua script Execution
```lua
local key = KEYS[1]
local now = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local limit = tonumber(ARGV[3])
local member = ARGV[4]

-- 1. Remove old timestamps outside of the sliding window
redis.call('ZREMRANGEBYSCORE', key, '-inf', now - window)

-- 2. Count active requests in current window
local current_requests = redis.call('ZCARD', key)

if current_requests < limit then
    -- 3. Record current request
    redis.call('ZADD', key, now, member)
    -- 4. Set key TTL (auto-expires once window closes)
    redis.call('EXPIRE', key, math.ceil(window / 1000))
    return 1 -- Allowed
else
    return 0 -- Rate limited
end
```

---

## 3. Rate Limit Rules Reference

| Endpoint | Key | Limit | Window | Purpose | Redis Key Pattern |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **All Endpoints** (Global) | IP | **100** | 1 sec | General abuse protection | `rl:global:<ip>` |
| `POST /v1/auth/sign-up` | IP | **5** | 1 min | Prevent signup spam | `rl:signup:<ip>` |
| `POST /v1/auth/verify-email` | Email | **10** | 10 min | Prevent OTP brute force | `rl:verifyemail:<email>` |
| `POST /v1/auth/login` | IP + Email | **5** | 1 min | Protect credentials | `rl:login:<ip>:<email>` |
| `POST /v1/auth/refresh` | Refresh Token ID | **30** | 1 min | Prevent token abuse | `rl:refresh:<jti>` |
| `POST /v1/auth/forgot-password` | Email + IP | **3** | 15 min | Prevent email spam | `rl:forgotpassword:<email>:<ip>` |
| `POST /v1/auth/reset-password` | Reset Token | **5** | 15 min | Prevent OTP guessing | `rl:resetpassword:<token>` |
| `POST /v1/auth/logout` | User | **30** | 1 min | Prevent logout abuse | `rl:logout:<user_id>` |
| `GET /v1/sessions` | User | **60** | 1 min | Normal session listing | `rl:sessions:<user_id>` |
| `GET /v1/sessions/current` | User | **120** | 1 min | Normal SPA polling | `rl:currentsession:<user_id>` |
| `DELETE /v1/sessions/{family_id}`| User | **20** | 1 min | Prevent session revoke spam | `rl:revokesession:<user_id>` |
| `POST /v1/sessions/logout-all` | User | **5** | 5 min | Protect security-critical action | `rl:logoutall:<user_id>` |
| `GET /v1/me` | User | **120** | 1 min | Normal profile polling | `rl:me:<user_id>` |

---

## 4. Key Extraction Strategies

1. **IP**: Parses `x-forwarded-for` (first value in csv) or `x-real-ip` headers, falling back to connection remote IP.
2. **Email / Reset Token**: Buffers the request body conditionally, deserializes JSON, and extracts the target field. Reconstructs the body stream afterwards so downstream handlers can consume it.
3. **Refresh Token ID (`jti`)**: Reads the `refresh_token` cookie, decodes the JWT, and extracts the unique `jti` UUID.
4. **User ID**: Reads the `ClaimsExtension` populated by the `require_auth` middleware.

---

## 5. Error Responses

When a rate limit is hit, the application short-circuits the pipeline and immediately returns:
* **HTTP Status**: `429 Too Many Requests`
* **JSON Payload**:
  ```json
  {
    "error": "Too many requests. Please try again later."
  }
  ```
