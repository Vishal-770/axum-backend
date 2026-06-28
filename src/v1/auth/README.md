# Version 1 Auth Module

This module encapsulates all authentication, registration, OTP verification, session token management, and password recovery logic for the v1 API.

---

## Folder Layout

```text
src/v1/auth/
├── mod.rs                # Exports submodules: claims, jwt, errors, middleware, dtos, services, handlers, routes.
├── claims.rs             # Defines AccessClaims and RefreshClaims JWT payload structs.
├── jwt.rs                # Generates and signs Access & Refresh JWTs using HmacSHA256.
├── errors.rs             # AuthError enum mapping domain errors to HTTP status codes and JSON bodies.
├── middleware.rs         # require_auth middleware — reads JWT access secret from AppState.config (no env::var per request).
├── dtos.rs               # Request/Response schemas (LoginDto, SignUpDto, ResendOtpDto, ResendResetOtpDto, etc.).
├── routes.rs             # Auth router mapping all paths under the /auth prefix.
├── rate_limit.md         # Rate limiting architecture documentation.
├── handlers/             # HTTP controllers receiving requests, validating DTOs, writing cookies.
│   ├── sign_up.rs
│   ├── verify_email.rs
│   ├── resend_otp.rs
│   ├── login.rs
│   ├── refresh.rs
│   ├── forgot_password.rs
│   ├── resend_reset_otp.rs
│   ├── reset_password.rs
│   └── logout.rs
└── services/             # Business logic (DB queries, Argon2 hashing, OTP generation, transactional locks).
    ├── sign_up_service.rs
    ├── verify_email_service.rs
    ├── resend_otp_service.rs
    ├── login_service.rs
    ├── refresh_service.rs
    ├── forgot_password_service.rs
    ├── resend_reset_otp_service.rs
    ├── reset_password_service.rs
    └── logout_service.rs
```

---

## Key Flows

### 1. Registration & Email Verification

1. **Sign Up (`POST /v1/auth/sign-up`)**:
   - Validates input and checks for email conflicts.
   - If an unverified user already exists for that email, their credentials are overwritten (re-registration).
   - Hashes password using **Argon2id** and inserts the user row.
   - Generates a cryptographically random **6-digit OTP**, stores it in `email_otp` (`created_at`, `expires_at`, `resend_count = 0`, `last_sent_at = NOW()`, `used_at = NULL`).
   - Dispatches the OTP to the user's email.

2. **Verify Email (`POST /v1/auth/verify-email`)**:
   - Fetches the latest OTP for the email where `used_at IS NULL`.
   - Validates the submitted OTP against the stored value.
   - On success: sets `user.verified = true`, stamps `email_otp.used_at = NOW()` (record preserved for audit), and commits in a transaction.

3. **Resend Sign-Up OTP (`POST /v1/auth/resend-otp`)**:
   - Validates the email exists as an unverified user.
   - Fetches the active OTP record (where `used_at IS NULL`) inside a **serializable transaction**.
   - Enforces a **60-second cooldown** via `last_sent_at`.
   - Enforces a **maximum of 3 resends** via `resend_count`.
   - On success: increments `resend_count`, updates `last_sent_at`, generates a new OTP, overwrites the `otp` value, and sends the email.

---

### 2. Login, RTR, and Logout

1. **Login (`POST /v1/auth/login`)**:
   - Verifies password using **Argon2id**.
   - Generates short-lived Access JWT (15 min) and long-lived Refresh JWT (7 days) under a fresh `family_id` UUID.
   - SHA-256 hashes the Refresh Token and stores it in `refresh_tokens`.
   - Sets `access_token` and `refresh_token` cookies with `HttpOnly`, `Secure`, `SameSite=Lax`.
   - JWT secrets are read from `AppState.config` (loaded once at boot — no per-request `env::var` mutex lock).

2. **Refresh Token Rotation (RTR) (`POST /v1/auth/refresh`)**:
   - Decodes the refresh token and extracts `jti` / `sub`.
   - Loads the DB token row. If the row has `revoked_at IS NOT NULL`, **reuse is detected** — the server immediately revokes all tokens with the same `family_id` and returns `401`.
   - Verifies SHA-256 hash matches and token is not expired.
   - Soft-revokes the old token, issues a new Access + Refresh Token pair carrying the same `family_id`, saves the new hash.

3. **Logout (`POST /v1/auth/logout`)**:
   - Extracts `jti` from the refresh token cookie.
   - Soft-revokes the token row (`revoked_at = NOW()`).
   - Clears both cookies from the client.

---

### 3. Password Recovery (Forgot / Reset / Resend)

1. **Forgot Password (`POST /v1/auth/forgot-password`)**:
   - Protected against **email enumeration**: always returns `200 OK` with a generic message regardless of whether the email exists.
   - If the email exists and is verified: generates a high-entropy 64-char hex **Reset Token** and a **6-digit OTP**.
   - Hashes both with SHA-256 and inserts a record in `password_resets` (15-min expiry, `resend_count = 0`, `last_sent_at = NOW()`).
   - Dispatches OTP to email. The raw `reset_token` is embedded in the response so the client can reference it later.

2. **Resend Password Reset OTP (`POST /v1/auth/resend-reset-otp`)**:
   - Client submits `email` + `reset_token`.
   - Fetches the active reset record (where `used_at IS NULL`) and verifies token hash.
   - Enforces a **60-second cooldown** via `last_sent_at`.
   - Enforces a **maximum of 3 resends** via `resend_count`.
   - On success: increments `resend_count`, updates `last_sent_at`, generates and stores a new OTP hash, and emails the new code.

3. **Reset Password (`POST /v1/auth/reset-password`)**:
   - Client submits `reset_token`, `otp`, and `new_password`.
   - Hashes the token and retrieves the record.
   - Increments `attempt_count` (capped at 5 to prevent OTP brute force).
   - Verifies the OTP hash, checks expiry, then stamps `used_at = NOW()`.
   - Updates the user's password using **Argon2id**, then soft-revokes all active refresh tokens to force re-login on all devices.

---

## Authentication Middleware (`require_auth`)

Defined in [middleware.rs](file:///home/vishal/Projects/axum-backend/src/v1/auth/middleware.rs):

- Registered via `from_fn_with_state` so it can access `AppState`.
- Extracts the `access_token` cookie from the request.
- Reads `JWT_ACCESS_SECRET` directly from `state.config.jwt_access_secret` — **no per-request `env::var` call**.
- Validates token signature, expiration, and claims.
- Populates `ClaimsExtension` (`user_id`, `family_id`) into request extensions for downstream handlers.

---

## OTP Audit Trail

OTP records (`email_otp`, `password_resets`) are **never deleted** upon use. Instead, a `used_at` timestamp is set. This allows:
- Full oversight of OTP issuance and usage patterns.
- Customer support investigation of authentication events.
- Detection of replay attempts (used OTPs with `used_at IS NOT NULL` are rejected).

A partial index speeds up all active-OTP queries:
```sql
CREATE INDEX idx_email_otp_email_active ON email_otp(email) WHERE used_at IS NULL;
```
