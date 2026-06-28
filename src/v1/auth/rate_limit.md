# V1 Rate Limiting Architecture

This document describes the design, configuration, and execution details of the two-layer rate limiting system built into the Axum backend.

---

## 1. Core Architecture

The rate limiting system consists of two sequential middleware layers sharing a **Redis** backend:

1. **Global Rate Limiter (100 RPS per IP)**: Applied globally at the root router level — the outermost shield against floods and DoS.
2. **Route-Specific Rate Limiter**: Applied per sub-router to enforce business-level limits on authentication, sessions, and profile routes.

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
│ Authentication Middleware (require_auth)      │   [Decodes JWT from AppState.config — no env::var]
└──────────────────────┬───────────────────────┘
                       │ (Passed)
                       ▼
┌──────────────────────────────────────────────┐
│ Route-Specific Rate Limiter                  │   [Extracts custom keys: User ID, Email, IP, Token]
└──────────────────────┬───────────────────────┘
                       │ (Passed)
                       ▼
               Target Handler
```

---

## 2. Sliding Window Log Algorithm (Atomic Lua Script)

To guarantee exact rate limiting without race conditions under concurrent requests, limits are managed atomically inside a **single Redis round-trip** using a **Sliding Window Log** algorithm backed by Redis Sorted Sets.

### Sorted Set (`ZSET`) Storage
Each rate limiting key maps to a Redis Sorted Set:
* **Score**: Unix timestamp of the request in milliseconds (`now`).
* **Member**: A unique string `"<timestamp>:<UUID>"` so multiple requests within the same millisecond are stored as distinct elements.

### Lua Script Execution
```lua
local key    = KEYS[1]
local now    = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local limit  = tonumber(ARGV[3])
local member = ARGV[4]

-- 1. Evict timestamps that have fallen outside the sliding window
redis.call('ZREMRANGEBYSCORE', key, '-inf', now - window)

-- 2. Count active requests within the current window
local current_requests = redis.call('ZCARD', key)

if current_requests < limit then
    -- 3. Record the current request
    redis.call('ZADD', key, now, member)
    -- 4. Auto-expire the key once the window closes
    redis.call('EXPIRE', key, math.ceil(window / 1000))
    return 1  -- Allowed
else
    return 0  -- Rate Limited
end
```

This Lua script runs atomically on the Redis server. No Rust-side locking, no race conditions.

---

## 3. Rate Limit Rules Reference

| Endpoint | Key | Limit | Window | Purpose | Redis Key Pattern |
| :--- | :--- | :---: | :---: | :--- | :--- |
| **All Endpoints** (Global) | IP | **100** | 1 sec | General abuse / DoS protection | `rl:global:<ip>` |
| `POST /v1/auth/sign-up` | IP | **5** | 1 min | Prevent account creation spam | `rl:signup:<ip>` |
| `POST /v1/auth/verify-email` | Email | **10** | 10 min | Allow OTP retries, prevent brute force | `rl:verifyemail:<email>` |
| `POST /v1/auth/resend-otp` | IP + Email | **3** | 1 min | Throttle sign-up OTP resend requests | `rl:login:<ip>:<email>` |
| `POST /v1/auth/login` | IP + Email | **5** | 1 min | Protect credentials from brute force | `rl:login:<ip>:<email>` |
| `POST /v1/auth/refresh` | Refresh Token ID (`jti`) | **30** | 1 min | Prevent refresh token abuse | `rl:refresh:<jti>` |
| `POST /v1/auth/forgot-password` | Email + IP | **3** | 15 min | Prevent email spam | `rl:forgotpassword:<email>:<ip>` |
| `POST /v1/auth/resend-reset-otp` | IP + Email | **3** | 1 min | Throttle password reset OTP resend requests | `rl:login:<ip>:<email>` |
| `POST /v1/auth/reset-password` | Reset Token | **5** | 15 min | Prevent OTP guessing | `rl:resetpassword:<token>` |
| `POST /v1/auth/logout` | User ID | **30** | 1 min | Prevent logout abuse | `rl:logout:<user_id>` |
| `GET /v1/sessions` | User ID | **60** | 1 min | Normal session listing | `rl:sessions:<user_id>` |
| `GET /v1/sessions/current` | User ID | **120** | 1 min | SPA polling support | `rl:currentsession:<user_id>` |
| `DELETE /v1/sessions/{family_id}` | User ID | **20** | 1 min | Prevent session revoke spam | `rl:revokesession:<user_id>` |
| `POST /v1/sessions/logout-all` | User ID | **5** | 5 min | Protect security-critical action | `rl:logoutall:<user_id>` |
| `GET /v1/me` | User ID | **120** | 1 min | Normal profile polling | `rl:me:<user_id>` |

---

## 4. Key Extraction Strategies

1. **IP**: Parses `x-forwarded-for` (first value in the CSV list) or `x-real-ip` headers, falling back to `"127.0.0.1"`.
   > ⚠️ **Production Note**: Ensure your reverse proxy (Nginx, Cloudflare, etc.) sanitizes these headers before forwarding requests, otherwise an attacker can spoof their IP and bypass all IP-based limits.

2. **Email / Reset Token**: The request body is buffered once, deserialized as JSON, and the target field extracted. The buffered bytes are reconstructed back into a new `Body` stream so downstream handlers can read the body normally.

3. **Refresh Token ID (`jti`)**: Reads the `refresh_token` cookie, decodes the JWT using the secret from `AppState.config.jwt_refresh_secret` (no `env::var` call), and extracts the unique `jti` UUID claim.

4. **User ID**: Reads the `ClaimsExtension` injected into request extensions by the `require_auth` middleware that runs before the route-specific rate limiter.

---

## 5. AppState Config — No Per-Request `env::var`

Both the global and route-specific rate limiters receive the full `AppState` via `State<AppState>` extractor. JWT secrets are pre-loaded at server boot into `AppState.config` (an `AuthConfig` struct):

```rust
pub struct AuthConfig {
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
}
```

This means **zero `std::env::var` calls on the hot path**. In Rust, `env::var` locks a global mutex internally — doing it on every request causes thread contention under high concurrency and throttles throughput.

---

## 6. Error Responses

When a rate limit is hit, the middleware short-circuits the pipeline and immediately returns:
* **HTTP Status**: `429 Too Many Requests`
* **JSON Payload**:
  ```json
  {
    "error": "Too many requests. Please try again later."
  }
  ```
