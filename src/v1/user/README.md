# Version 1 User Module

This module handles core user entity mappings, profile handlers, and services for the v1 API.

---

## Folder Layout

```text
src/v1/user/
├── mod.rs                # Exports model, dtos, services, handlers, routes.
├── model.rs              # Database User entity struct (maps to the `users` table).
├── dtos.rs               # Defines UserMeResponse (safe user data without password hash).
├── routes.rs             # Route definitions for GET /v1/me (requires authentication).
├── handlers.rs           # Axum controller handler (me_handler).
└── services.rs           # Profile retrieval service (get_me).
```

---

## Authentication & Rate Limiting

The `/v1/me` route is protected by two middleware layers applied in `routes.rs`:

1. **`require_auth`** (`from_fn_with_state`) — decodes the `access_token` cookie using `AppState.config.jwt_access_secret` (cached at boot, no per-request `env::var` call), and injects `ClaimsExtension` into request extensions.
2. **Route-specific rate limiter** (`from_fn_with_state`) — enforces a cap of 120 requests per minute per user ID.

---

## Logic & Flow

### User Profile (`GET /v1/me`)

1. **`require_auth` middleware**: Validates the `access_token` cookie, injects `ClaimsExtension { user_id, family_id }` into request extensions.
2. **`me_handler`**: Extracts `user_id` from `ClaimsExtension` and the `AppState` (database pool), then calls `get_me(user_id, db)`.
3. **`get_me` service**:
   - Queries the `users` table by ID:
     ```sql
     SELECT id, email, username, password, verified, created_at, updated_at
     FROM users
     WHERE id = $1
     ```
   - Maps the result to `UserMeResponse`, which **excludes the password hash** for safe serialization.

### `UserMeResponse`
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "username": "johndoe",
  "verified": true,
  "created_at": "2026-06-01T10:00:00Z",
  "updated_at": "2026-06-28T18:00:00Z"
}
```
