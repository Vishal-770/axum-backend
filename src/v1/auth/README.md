# Version 1 Auth Module

This module encapsulates all authentication, registration, token generation, and password recovery logic for the v1 API.

---

## Folder Layout

```text
src/v1/auth/
├── mod.rs                # Exports submodules: claims, jwt, errors, middleware, dtos, services, handlers, routes.
├── claims.rs             # Defines AccessClaims and RefreshClaims structs.
├── jwt.rs                # Generates and signs Access & Refresh JWTs using HmacSHA256.
├── errors.rs             # AuthError enum mapping security errors to HTTP status codes and JSON bodies.
├── middleware.rs         # require_auth middleware extracting & verifying JWTs from cookies.
├── dtos.rs               # Request/Response schemas (e.g. LoginDto, SignUpDto, AuthResponse).
├── routes.rs             # Auth router mapping paths under the /auth prefix.
├── handlers/             # HTTP Controllers receiving requests & writing cookies.
└── services/             # Business logic execution (database queries, cryptography, OTP mailer).
```

---

## Key Flows

### 1. Registration & Verification
1. **Sign Up (`POST /v1/auth/sign-up`)**: Receives payload, validates input, checks for conflicts. If unverified user exists, overwrites password/details. Hashes password using **Argon2id** and inserts a new row. Generates a 6-digit OTP code, stores it with a 10-minute expiry in `email_otp`, and dispatches a verification email.
2. **Verify Email (`POST /v1/auth/verify-email`)**: Compares client OTP against latest DB OTP record. On success, updates user status `verified = true`, deletes the OTP record, and commits.

### 2. Login, RTR, and Logout
1. **Login (`POST /v1/auth/login`)**: Verifies password hash using Argon2id. Generates short-lived Access JWT (15-min) and long-lived Refresh JWT (7-day) under a fresh `family_id` UUID. Hashes Refresh Token with SHA-256 and records it in `refresh_tokens`. Cookies (`access_token`, `refresh_token`) are set with `HTTP-only`, `Secure`, and `SameSite=Lax`.
2. **Refresh Token Rotation (RTR) (`POST /v1/auth/refresh`)**: Implements replay protection:
   - Decodes the refresh token (validates signature, bypasses expiry check for reuse detection).
   - Computes SHA-256 hash and retrieves the token row from the database.
   - If the token is **already revoked**, reuse is detected. The server immediately revokes all tokens matching the lineage `family_id` to block the attacker and logs out the victim.
   - If valid, soft-revokes the old token, issues a new Access & Refresh Token carrying the same `family_id` but a fresh `jti` (rotation), and saves the new hash.
3. **Logout (`POST /v1/auth/logout`)**: Extracts `jti` from refresh token cookie, soft-revokes the token in the database (`revoked_at = NOW()`), and clears cookies from the client.

### 3. Password Recovery (Forgot/Reset)
1. **Forgot Password (`POST /v1/auth/forgot-password`)**: Generates a high-entropy, 64-char hex **Reset Token** and a numeric 6-digit **OTP**. Hashes both with SHA-256 and inserts a recovery record in `password_resets` (15-min expiry). Dispatches OTP to email and returns the raw Reset Token in the HTTP response.
2. **Reset Password (`POST /v1/auth/reset-password`)**: Clients submit the new password, OTP, and Reset Token. The server hashes inputs, retrieves the request by `token_hash`, increments the `attempt_count` (capped at 5 to prevent brute force), verifies the OTP, marks the reset as used, updates the password using Argon2id, and invalidates all active user sessions/refresh tokens to force a re-login.

---

## Authentication Middleware (`require_auth`)

Defined in [middleware.rs](file:///home/vishal/Projects/axum-backend/src/v1/auth/middleware.rs):
- Extracts the `access_token` cookie.
- Validates token signature, expiration, and claims.
- Populates the request extensions with `ClaimsExtension` containing `user_id` and `family_id` (so subsequent handlers can identify the user and session lineage).
