# Version 1 Session Module

This module encapsulates all active session queries, session details, and session revocations for the v1 API.

---

## Folder Layout

```text
src/v1/session/
├── mod.rs                # Exports dtos, services, handlers, routes.
├── dtos.rs               # Defines SessionResponseDto returned to the client.
├── routes.rs             # Mappings for /v1/sessions/* (requires authentication via require_auth middleware).
├── handlers/             # HTTP controllers for handling session requests.
│   ├── get_all.rs        # GET /v1/sessions
│   ├── get_current.rs    # GET /v1/sessions/current
│   ├── revoke.rs         # DELETE /v1/sessions/{family_id}
│   └── logout_all.rs     # POST /v1/sessions/logout-all
└── services/             # Session database queries and invalidation logic.
```

---

## Authentication & Rate Limiting

All session routes are protected by two middleware layers applied in `routes.rs`:

1. **`require_auth`** — decodes the `access_token` cookie using the JWT secret from `AppState.config` (no per-request `env::var` call), and injects `ClaimsExtension` (`user_id`, `family_id`) into request extensions.
2. **Route-specific rate limiter** — enforces per-user request caps for each endpoint.

---

## Session Concept

A **Session** in this system represents a unique login lineage tracked by a `family_id` UUID. Every time a refresh token is rotated (RTR), the new token inherits the same `family_id`, keeping the entire device session traceable from its first login.

---

## Endpoint Behaviour

### `GET /v1/sessions`
Retrieves all active sessions for the authenticated user.
- Queries `refresh_tokens` for rows where `revoked_at IS NULL AND expires_at > NOW()`.
- Groups by `family_id` to represent each distinct device login once.
- Returns:
  - `session_id` → the `family_id`
  - `created_at` → oldest `created_at` in the family lineage (the original login time):
    ```sql
    (SELECT MIN(created_at) FROM refresh_tokens rt2 WHERE rt2.family_id = rt.family_id)
    ```
  - `last_seen_at` → the most recent token issue time in the lineage.
  - `current` → `true` if this session matches the one currently making the request.

### `GET /v1/sessions/current`
Returns metadata for the session that is currently making the API call, identified by the `family_id` extracted from the access token claims.

### `DELETE /v1/sessions/{family_id}`
Revokes a specific device session.
- Validates session ownership: ensures the requested `family_id` belongs to the authenticated user.
- Soft-revokes all refresh tokens in that lineage:
  ```sql
  UPDATE refresh_tokens SET revoked_at = NOW() WHERE family_id = $1
  ```
- The targeted device will be logged out on its next refresh attempt.

### `POST /v1/sessions/logout-all`
Terminates all active sessions across all devices for the authenticated user.
- Soft-revokes every non-revoked refresh token belonging to the user:
  ```sql
  UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL
  ```
