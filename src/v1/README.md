# Version 1 (v1) API Modules

This directory contains the entire module boundary for the **Version 1 (v1)** REST API. All routes, controllers (handlers), business logic (services), and data schemas (DTOs and models) specific to v1 are grouped inside self-contained feature directories.

---

## Directory Structure

```text
src/v1/
├── mod.rs                # Entrypoint. Merges and nests all v1 routers.
├── auth/                 # Authentication module (Sign-up, OTP Verification, Resend OTP, Login, RTR, Password Recovery)
│   ├── mod.rs            # Exposes auth submodules
│   ├── claims.rs         # JWT claims structs (AccessClaims, RefreshClaims)
│   ├── jwt.rs            # Token signing utilities
│   ├── errors.rs         # Domain errors (AuthError → HTTP status code mapping)
│   ├── middleware.rs     # require_auth middleware — reads JWT secret from AppState.config
│   ├── dtos.rs           # Request/Response schemas (LoginDto, SignUpDto, ResendOtpDto, etc.)
│   ├── routes.rs         # Router mappings (/v1/auth/*)
│   ├── rate_limit.md     # Rate limiting architecture documentation
│   ├── handlers/         # HTTP controllers translating requests to service calls
│   └── services/         # Business logic (Argon2 hashing, OTPs, transactional DB locks)
├── session/              # Session Management module
│   ├── mod.rs
│   ├── dtos.rs           # SessionResponseDto
│   ├── routes.rs         # Router mappings (/v1/sessions/*)
│   ├── handlers/         # Controllers for listing and revoking sessions
│   └── services/         # Queries computing active session family details
└── user/                 # User Profile module
    ├── mod.rs
    ├── model.rs          # User entity model matching the DB table
    ├── dtos.rs           # UserMeResponse DTO
    ├── routes.rs         # Router mapping (/v1/me)
    ├── handlers.rs       # Controller exposing me_handler
    └── services.rs       # Service retrieving profile data
```

---

## V1 Router Integration

All v1 routes are assembled in [mod.rs](file:///home/vishal/Projects/axum-backend/src/v1/mod.rs) and mounted under the `/v1` prefix in the main application router (`src/app/mod.rs`). The router receives a cloned `AppState` that carries the database pool, Redis connection, mail service, and pre-loaded `AuthConfig`:

```rust
pub fn v1_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes::auth_routes(state.clone()))
        .merge(user::routes::user_routes(state.clone()))
        .nest("/sessions", session::routes::session_routes(state))
}
```

---

## Middleware Stack (per request)

```text
Global Rate Limiter (100 RPS / IP)
        ↓
require_auth (for protected routes only — reads JWT from AppState.config)
        ↓
Route-Specific Rate Limiter (per-endpoint business limits)
        ↓
Handler
```

---

## Shared Dependencies

While domain logic is compartmentalized inside `src/v1/`, all modules share utilities defined at `src/` root:

* **Auth Config**: [src/config/auth_config.rs](file:///home/vishal/Projects/axum-backend/src/config/auth_config.rs) — `AuthConfig` struct holding JWT secrets pre-loaded at boot (eliminates `env::var` mutex contention on every request).
* **Mail Config**: [src/config/mail_config.rs](file:///home/vishal/Projects/axum-backend/src/config/mail_config.rs) — SMTP transport setup and `MailService`.
* **Database Access**: [src/database/](file:///home/vishal/Projects/axum-backend/src/database) — connection pool (`PgPool`) and `AppState` definition.
* **Utility Helpers**: [src/utils/](file:///home/vishal/Projects/axum-backend/src/utils) — `send_email`, random OTP generation, SHA-256 helpers.
* **Global Error Router**: [src/errors/](file:///home/vishal/Projects/axum-backend/src/errors) — `AppError` enum with unified HTTP response mapping.
