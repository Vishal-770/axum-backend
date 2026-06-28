# Version 1 User Module

This module handles core user database entity mappings, profile handlers, and services.

---

## Folders & Layout

```text
src/v1/user/
├── mod.rs                # Exports model, dtos, services, handlers, routes.
├── model.rs              # Database User entity struct.
├── dtos.rs               # Defines UserMeResponse.
├── routes.rs             # Route definitions for GET /v1/me (requires authentication).
├── handlers.rs           # Axum controller handler for profiles.
└── services.rs           # Profile retrieval logic.
```

---

## Logic & Flow

### User Profile (`GET /v1/me`)
* **Authentication**: Requires a valid `access_token` cookie. The router applies the `require_auth` middleware which parses the token and injects the `ClaimsExtension` into request extensions.
* **Handler (`me_handler`)**: Extracts the authenticated `user_id` from request extensions and the `AppState`, calling the `get_me` service.
* **Service (`get_me`)**:
  - Queries the `users` table for the user record by ID:
    ```sql
    SELECT id, email, username, password, verified, created_at, updated_at FROM users WHERE id = $1
    ```
  - Maps the database entity safely to `UserMeResponse` to avoid exposing the password hash.
