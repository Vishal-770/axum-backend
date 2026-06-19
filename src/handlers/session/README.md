# Session Handlers

This directory contains the HTTP handlers for querying and revoking active user sessions. All endpoints in this module require active authentication and are protected by the JWT authentication middleware.

---

## Accessing Session Context

The handlers retrieve authentication context from Axum request extensions, which are populated by the `require_auth` middleware:
* `ClaimsExtension` contains:
  - `user_id`: The ID of the authenticated user.
  - `family_id`: The `family_id` (session ID) associated with the current access token.

---

## Handlers Reference

### 1. Get All Sessions (`get_all.rs`)
* **Endpoint**: `GET /sessions`
* **Security**: Requires a valid `access_token` in cookies.
* **Response**: `200 OK` with JSON array of `SessionResponseDto`:
  ```json
  [
    {
      "session_id": "9369cc4e-b5c4-4b57-a3cf-b3d41ab36c53",
      "device_name": "Chrome on Fedora",
      "ip_address": "127.0.0.1",
      "created_at": "2026-06-19T10:00:00Z",
      "last_seen_at": "2026-06-19T11:45:00Z",
      "current": true
    }
  ]
  ```

### 2. Get Current Session (`get_current.rs`)
* **Endpoint**: `GET /sessions/current`
* **Security**: Requires a valid `access_token` in cookies.
* **Response**: `200 OK` with a single `SessionResponseDto` representing the session that initiated the current request.

### 3. Revoke Session (`revoke.rs`)
* **Endpoint**: `DELETE /sessions/{family_id}`
* **Security**: Requires a valid `access_token` in cookies.
* **Path Parameters**: `{family_id}` (UUID identifying the target session/lineage to terminate).
* **Response**: `200 OK` on success.
* **Security Invariant**: The handler queries the ownership of the session to ensure that users can only revoke their own sessions (returns `401 Unauthorized` if a user attempts to delete a session belonging to another user).

### 4. Logout All (`logout_all.rs`)
* **Endpoint**: `POST /sessions/logout-all`
* **Security**: Requires a valid `access_token` in cookies.
* **Response**: `200 OK` on success. Revokes every active session belonging to the user.
