# Axum Rust Backend

A version-partitioned modular, production-ready Rust backend server built using the Axum web framework, SQLx for PostgreSQL database interactions (Neon support), Argon2id password hashing, JWT token rotation (RTR), Redis-backed rate limiting, and full session lineage tracking.

---

## Documentation Quick Links

* 🗄️ **[Database Management & Migrations](file:///home/vishal/Projects/axum-backend/database/README.md)**: Guide to running the Postgres container, managing database tables, using SQLx CLI migrations, and offline metadata setup.
* 📦 **[V1 API Architecture](file:///home/vishal/Projects/axum-backend/src/v1/README.md)**: Details the modular version 1 code architecture, directory structure, module boundaries, routing layout, and middleware.
* 🔒 **[Rate Limiting Architecture](file:///home/vishal/Projects/axum-backend/src/v1/auth/rate_limit.md)**: Explains the two-layer rate limiting system (global + per-route), the Redis sliding window Lua algorithm, and all configured limits.
* 🔐 **[Auth Module](file:///home/vishal/Projects/axum-backend/src/v1/auth/README.md)**: Complete breakdown of all authentication flows — sign-up, OTP verification, login/logout, refresh token rotation (RTR), and password recovery with resend mechanisms.

---

## Architecture Overview

This project implements a **Version-Partitioned Modular Architecture**. Code is grouped by API version and then by feature module:

* `src/v1/` contains all the logic, controllers, and services for the Version 1 API.
* Under `src/v1/`, code is divided into self-contained feature modules:
  * `auth/`: Complete registration, OTP verification, resend OTP, sign-in/out, RTR token rotation, and password recovery (forgot/reset/resend reset OTP) flows.
  * `session/`: Querying active sessions and revoking device logins.
  * `user/`: Core user profile queries.
* Shared cross-cutting logic like configs, database state, error mappings, and helpers remain outside `v1` at `src/` root to be shared across all API versions.

Adding a future `v2` is as simple as creating `src/v2/` — the stable `v1` API is not touched.

---

## Key Technical Highlights

| Concern | Approach |
|---|---|
| Password hashing | **Argon2id** (memory-hard, brute-force resistant) |
| Session tokens | **JWT** — short-lived Access Token (15 min) + long-lived Refresh Token (7 days) |
| Token rotation | **RTR** (Refresh Token Rotation) with SHA-256 token hashing in DB |
| Reuse detection | Revoking entire `family_id` lineage on replay |
| Rate limiting | **Redis ZSET Sliding Window** via atomic Lua script — 2 layers (global + per-route) |
| Config loading | JWT secrets loaded **once at boot** into `AppState.config` — no per-request `env::var` mutex contention |
| OTP audit trail | OTPs flagged `used_at = NOW()` (not deleted) — full audit history preserved |
| DB index | Partial index on `email_otp(email) WHERE used_at IS NULL` — fast active OTP lookups |
| Resend OTP | 60-second cooldown + maximum 3 resends — enforced transactionally for both sign-up and password reset flows |

---

## Prerequisites

- **Rust**: Ensure the Rust toolchain (Cargo) is installed.
- **Redis**: A running Redis instance (local or hosted).
- **Docker**: Needed to run the PostgreSQL database locally (optional if using Neon).

---

## Quick Start

### 1. Configure Environment
Create a `.env` file at the project root:
```env
DATABASE_URL="postgresql://username:password@host/dbname?sslmode=require"
REDIS_URL="redis://127.0.0.1:6379"
JWT_ACCESS_SECRET="your-access-secret"
JWT_REFRESH_SECRET="your-refresh-secret"
SMTP_USER="smtp-username"
SMTP_PASS="smtp-password"
```

### 2. Run Database Migrations
```bash
sqlx migrate run
```
*Note: The Axum server also runs pending migrations automatically on startup.*

### 3. Run the Server
```bash
cargo run
```
The server starts listening on port `3000`.

---

## Versioned API Endpoints (v1)

### Authentication
| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/auth/sign-up` | Register a new user account (dispatches OTP to email) |
| `POST` | `/v1/auth/verify-email` | Verify registration using the 6-digit OTP |
| `POST` | `/v1/auth/resend-otp` | Resend sign-up OTP (60s cooldown, max 3 resends) |
| `POST` | `/v1/auth/login` | Authenticate credentials, set access + refresh token cookies |
| `POST` | `/v1/auth/refresh` | Rotate expired access token using RTR (replay-protected) |
| `POST` | `/v1/auth/forgot-password` | Request a password reset OTP (email enumeration protected) |
| `POST` | `/v1/auth/resend-reset-otp` | Resend password reset OTP (60s cooldown, max 3 resends) |
| `POST` | `/v1/auth/reset-password` | Reset password using the OTP + reset token |
| `POST` | `/v1/auth/logout` | Log out current active session |

### Sessions *(requires authentication)*
| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/sessions` | List all active sessions for the current user |
| `GET` | `/v1/sessions/current` | Details for the currently active session |
| `DELETE` | `/v1/sessions/{family_id}` | Revoke a specific device session |
| `POST` | `/v1/sessions/logout-all` | Terminate all active sessions on all devices |

### User *(requires authentication)*
| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/me` | Retrieve profile details for the logged-in user |
