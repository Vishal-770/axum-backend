# Version 1 Session Module

This module encapsulates all active session queries, details, and revocations.

---

## Folders & Layout

```text
src/v1/session/
├── mod.rs                # Exports dtos, services, handlers, routes.
├── dtos.rs               # Defines SessionResponseDto returned to the client.
├── routes.rs             # Mappings for /v1/sessions/* (requires authentication).
├── handlers/             # HTTP controllers for handling session requests.
└── services/             # Session database queries and invalidation logic.
```

---

## Session Concept

A **Session** is modeled as a refresh token lineage identified by a unique `family_id` (UUID). Instead of exposing raw database token records:
- **Get All Sessions (`GET /v1/sessions`)**: Queries the `refresh_tokens` table for active tokens (`revoked_at IS NULL AND expires_at > NOW()`) belonging to the user. Groups them by `family_id` to list distinct logins.
  - `session_id` is the `family_id`.
  - `created_at` represents the session start time, computed using a fast subquery for the oldest `created_at` timestamp in the lineage family:
    ```sql
    (SELECT MIN(created_at) FROM refresh_tokens rt2 WHERE rt2.family_id = rt.family_id)
    ```
  - `last_seen_at` is the time the active token was last used/issued.
  - `current` is a boolean flagging if the session matches the one currently invoking the request.
- **Get Current Session (`GET /v1/sessions/current`)**: Returns metadata for the session currently making the API call.
- **Revoke Session (`DELETE /v1/sessions/{family_id}`)**:
  - Validates session ownership: checks if the requested `family_id` belongs to the authenticated user.
  - If owned, soft-revokes all refresh tokens under that lineage (`UPDATE refresh_tokens SET revoked_at = NOW() WHERE family_id = $1`). This terminates access for the targeted device on its next refresh attempt.
- **Logout All Sessions (`POST /v1/sessions/logout-all`)**: Soft-revokes all active refresh tokens belonging to the authenticated user.
