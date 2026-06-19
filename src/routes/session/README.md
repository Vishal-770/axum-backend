# Session Router

This module configures the router and defines the routing table for all session-related endpoints.

---

## Security Layer

All routes in this router are protected by the `require_auth` middleware (applied using `Router::route_layer`):
- Checks for the presence of a valid HTTP-only `access_token` cookie.
- Decodes and validates the access token claims.
- Populates the request extensions with `ClaimsExtension` containing `user_id` and `family_id`.
- Rejects unauthenticated requests with `401 Unauthorized`.

---

## Route Mappings

All routes defined in this router are mounted under the `/sessions` path prefix in the main application router:

| Method | Path | Handler Function | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/sessions` | `get_all_handler` | Get all active sessions for the user |
| `GET` | `/sessions/current` | `get_current_handler` | Get details of the current session |
| `DELETE` | `/sessions/{family_id}` | `revoke_handler` | Revoke a specific session (by family ID) |
| `POST` | `/sessions/logout-all` | `logout_all_handler` | Revoke all active sessions for the user |

---

## Route Details

### Revoke Session (`DELETE /sessions/{family_id}`)
* Uses the **Axum 0.8** template parameter syntax: `{family_id}`.
* Binds the path segment directly to a `Uuid` parameter in the handler.
* Terminating the session is safe and verifies ownership before executing database mutations.
