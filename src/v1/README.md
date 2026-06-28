# Version 1 (v1) API Modules

This directory contains the entire module boundary for the **Version 1 (v1)** REST API. All routes, controllers (handlers), business logic (services), and data boundaries (DTOs and models) specific to v1 are grouped inside self-contained feature directories.

---

## Directory Structure

```text
src/v1/
├── mod.rs                # Entrypoint. Merges and nests all v1 routers.
├── auth/                 # Authentication module (Sign-up, Verification, Session Tokens)
│   ├── mod.rs            # Exposes auth submodules (jwt, claims, middleware, errors)
│   ├── claims.rs         # JWT claims structs (AccessClaims, RefreshClaims)
│   ├── jwt.rs            # Token signing utilities
│   ├── errors.rs         # Domain errors (AuthError status code mapping)
│   ├── middleware.rs     # require_auth middleware & ClaimsExtension
│   ├── dtos.rs           # Login/Signup/Forgot/Reset DTO request/response schemas
│   ├── routes.rs         # Router mappings (/v1/auth/*)
│   ├── handlers/         # Controllers translating HTTP inputs to services
│   └── services/         # Business logic functions (Argon2 hashes, OTPs, transactional db checks)
├── session/              # Session Management module
│   ├── mod.rs
│   ├── dtos.rs           # SessionResponseDto
│   ├── routes.rs         # Router mappings (/v1/sessions/*)
│   ├── handlers/         # Controllers for listing/deleting sessions
│   └── services/         # Queries calculating active session family details
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

The v1 router configures and encapsulates all v1 endpoints under a centralized namespace. The router is exported in [mod.rs](file:///home/vishal/Projects/axum-backend/src/v1/mod.rs) and mounted under the `/v1` prefix in the main application router:

```rust
use axum::Router;
use crate::database::db_state::AppState;

pub fn v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes::auth_routes())
        .merge(user::routes::user_routes())
        .nest("/sessions", session::routes::session_routes())
}
```

---

## Shared Dependencies

While domain logic is compartmentalized inside `src/v1/`, modules consume shared utility interfaces defined outside `v1`:
* **State Configs**: [src/config/](file:///home/vishal/Projects/axum-backend/src/config) provides application configuration and mail transport setups.
* **Database Access**: [src/database/](file:///home/vishal/Projects/axum-backend/src/database) manages connection pooling and db state.
* **Utility Scripts**: [src/utils/](file:///home/vishal/Projects/axum-backend/src/utils) exports send_email SMTP logic, random OTP generation, and sha256 helpers.
* **Global Error Router**: [src/errors/](file:///home/vishal/Projects/axum-backend/src/errors) provides the parent application error handling interface (`AppError`).
