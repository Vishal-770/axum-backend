# Session Services

This module handles the queries and mutations for active user sessions. Sessions are tracked via token lineages using the `family_id` column in the `refresh_tokens` table.

---

## Sessions Concept

Instead of exposing raw refresh token records to the frontend, this API models sessions.
* A **Session** represents a single login event (e.g., logging in from Chrome on a laptop, or Safari on an iPhone).
* A session consists of a lineage of refresh tokens rotated over time, all sharing a single `family_id`.
* A session is active if there is at least one active (unrevoked and unexpired) refresh token in that family.

---

## Services Overview

### 1. Get All Sessions (`get_all_service.rs`)
* **Purpose**: Retrieves a list of all active sessions for a user.
* **Logic**:
  1. Performs a query on `refresh_tokens` to find active tokens belonging to the `user_id`.
  2. Active criteria: `revoked_at IS NULL AND expires_at > NOW()`.
  3. Groups the tokens by `family_id` (so each session is listed only once, regardless of how many times it was rotated).
  4. Returns metadata for each session:
     - `session_id`: The `family_id` identifying the lineage.
     - `device_name`: The device identifier.
     - `ip_address`: The IP address of the active session.
     - `created_at`: The start time of the session lineage, resolved via a subquery identifying the minimum `created_at` timestamp for that `family_id`:
       ```sql
       (SELECT MIN(created_at) FROM refresh_tokens rt2 WHERE rt2.family_id = rt.family_id)
       ```
     - `last_seen_at`: The creation time of the currently active token in that lineage.
     - `current`: A boolean indicating if the session matches the current request's session (`family_id` from the JWT claims extension).

### 2. Get Current Session (`get_current_service.rs`)
* **Purpose**: Retrieves details of the session currently in use.
* **Logic**:
  1. Queries the active token in the database matching the user's current request `family_id`.
  2. Returns the same metadata format as `get_all_service` with `current = true`.

### 3. Revoke Session (`revoke_service.rs`)
* **Purpose**: Logs out a specific session (e.g., revoking a session from another device).
* **Logic**:
  1. Verifies ownership of the target session by checking that the `family_id` belongs to the requesting user:
     ```sql
     SELECT user_id FROM refresh_tokens WHERE family_id = $1 LIMIT 1
     ```
  2. If the session belongs to a different user, throws an `Unauthorized` error.
  3. Soft-revokes all active tokens in the lineage:
     ```sql
     UPDATE refresh_tokens 
     SET revoked_at = NOW() 
     WHERE family_id = $1 
       AND user_id = $2 
       AND revoked_at IS NULL
     ```

### 4. Logout All Sessions (`logout_all_service.rs`)
* **Purpose**: Logs out the user from all sessions on all devices.
* **Logic**:
  1. Soft-revokes every active refresh token belonging to the `user_id`:
     ```sql
     UPDATE refresh_tokens 
     SET revoked_at = NOW() 
     WHERE user_id = $1 
       AND revoked_at IS NULL
     ```
