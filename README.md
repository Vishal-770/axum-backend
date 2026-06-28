# Axum Rust Backend

A version-partitioned modular, production-ready Rust backend server built using the Axum web framework, SQLx for PostgreSQL database interactions (Neon support), Argon2id password hashing, and session lineage tracking.

---

## Documentation Quick Links

* 🗄️ **[Database Management & Migrations](file:///home/vishal/Projects/axum-backend/database/README.md)**: Guide to running the Postgres container, managing database tables, using SQLx CLI migrations, and offline metadata setup.
* 📦 **[V1 API Architecture](file:///home/vishal/Projects/axum-backend/src/v1/README.md)**: Details the modular version 1 code architecture, directory structure, module boundaries, routing layout, and middleware.

---

## Architecture Overview

This project implements a **Version-Partitioned Modular Architecture**. Rather than separating the codebase globally by layers (such as keeping all services in one folder and all handlers in another), code is grouped by API version and then by feature module:
* `src/v1/` contains all the logic, controllers, and services for the Version 1 API.
* Under `src/v1/`, code is divided into self-contained feature modules:
  * `auth/`: Complete registration, verification, sign-in/out, token rotation (RTR), and password recovery flows.
  * `session/`: Querying active sessions and revoking device logins.
  * `user/`: Core user profile queries.
* Shared cross-cutting logic like configs, database state, error mappings, and helpers remain outside `v1` at the root of `src/` to be shared.

This allows the backend to scale cleanly: adding a `v2` in the future is as simple as creating `src/v2/` without changing or breaking the stable `v1` API.

---

## Prerequisites

- **Rust**: Ensure the Rust toolchain (Cargo) is installed.
- **Docker**: Needed to run the PostgreSQL database locally (optional if using Neon).

---

## Quick Start

### 1. Configure Environment
Verify or create a `.env` file at the root of the project:
```env
DATABASE_URL="postgresql://username:password@host/dbname?sslmode=require"
JWT_ACCESS_SECRET="your-access-secret"
JWT_REFRESH_SECRET="your-refresh-secret"
SMTP_USER="smtp-username"
SMTP_PASS="smtp-password"
```

### 2. Run Database Migrations
If using `sqlx-cli`:
```bash
sqlx migrate run
```
*Note: The Axum server is also configured to run migrations automatically on startup.*

### 3. Run the Server
Launch the Axum backend server:
```bash
cargo run
```
The server starts listening on port `3000`.

---

## Versioned API Endpoints (v1)

### Authentication
* `POST /v1/auth/sign-up`: Register a new unverified user account.
* `POST /v1/auth/verify-email`: Confirm registration using the 6-digit email OTP.
* `POST /v1/auth/login`: Authenticate credentials, set access and refresh token cookies.
* `POST /v1/auth/refresh`: Rotate expired access token using Refresh Token Rotation (RTR).
* `POST /v1/auth/forgot-password`: Request a password recovery code (OTP sent to email).
* `POST /v1/auth/reset-password`: Reset password using OTP and recovery reset token.
* `POST /v1/auth/logout`: Log out of the current active session.

### Sessions
* `GET /v1/sessions`: Retrieve all active sessions for the user.
* `GET /v1/sessions/current`: Show details for the current active session.
* `DELETE /v1/sessions/{family_id}`: Revoke a specific session.
* `POST /v1/sessions/logout-all`: Terminate all active sessions on all devices.

### User
* `GET /v1/me`: Retrieve details for the logged-in user.
