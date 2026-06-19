# Authentication Services

This module handles the core business logic for user authentication, registration, email verification, password management, and session token rotation.

---

## Services Overview

### 1. Sign Up Service (`sign_up_service.rs`)
* **Purpose**: Registers a new user account.
* **Logic**:
  1. Normalizes the email (lowercased and trimmed).
  2. Checks if a user already exists with the given email.
     - If user exists but is verified, returns an error (`UserAlreadyExists`).
     - If user exists but is unverified, it regenerates the OTP, sends a new email, and updates their password hash.
  3. If new, hashes the password using **Argon2id** (`argon2::password_hash`).
  4. Inserts the new unverified user into the `users` table.
  5. Generates a cryptographically secure 6-digit numeric OTP.
  6. Inserts the OTP into the `email_otp` table with a 10-minute expiration.
  7. Sends a verification email to the user with the OTP.

### 2. Verify Email Service (`verify_email_service.rs`)
* **Purpose**: Verifies a user's email using the OTP received after signing up.
* **Logic**:
  1. Looks up the latest OTP entry in `email_otp` for the given email.
  2. Validates that the OTP exists, matches, and has not expired (`expires_at > NOW()`).
  3. Updates the user's status to `verified = true` in the `users` table.

### 3. Login Service (`login_service.rs`)
* **Purpose**: Authenticates credentials and starts a new session.
* **Logic**:
  1. Retrieves the user record by normalized email.
  2. Verifies the password using Argon2id.
  3. Checks if the user is verified.
  4. Generates a unique `family_id` (UUID) representing the new session/lineage.
  5. Creates a JWT Access Token containing `user_id` and `family_id` (expires in 15 minutes).
  6. Creates a JWT Refresh Token containing a unique `jti` (UUID) and the `family_id` (expires in 7 days).
  7. Hashes the Refresh Token (`SHA-256`) and inserts it into the `refresh_tokens` table with the metadata (`device_name`, `user_agent`, `ip_address`, `expires_at`, and `family_id`).

### 4. Refresh Service (`refresh_service.rs`)
* **Purpose**: Performs **Refresh Token Rotation (RTR)** to issue a new pair of tokens when the access token expires. Implements **Replay Protection**.
* **Logic**:
  1. Decodes the refresh token (validates signature, but ignores expiration during validation so that we can still detect reuse/revocation of expired tokens if necessary).
  2. Hashes the incoming token string with SHA-256.
  3. Retrieves the token record from the `refresh_tokens` database matching the `jti` (primary key).
  4. Performs **Security Checks**:
     - Verifies `record.user_id` matches the token's claim user id.
     - Checks if the token is already expired.
     - Checks if the token has already been revoked (`revoked_at IS NOT NULL`).
  5. **Replay Protection / Theft Detection**:
     - If the token has **already been revoked**, it indicates a potential theft/replay attack (someone is trying to use an old rotated token).
     - **Action**: Immediately revokes the *entire token family* by setting `revoked_at = NOW()` for all rows with the same `family_id`. This kills the active session for the legitimate user and the attacker, forcing a re-login.
  6. **Normal Rotation**:
     - If the token is active and valid, it soft-revokes the current token: `UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL`.
     - Generates a new Refresh Token carrying the *same* `family_id` (to maintain the lineage) but a new `jti`.
     - Generates a new Access Token.
     - Hashes and inserts the new Refresh Token into the database.

### 5. Forgot Password Service (`forgot_password_service.rs`)
* **Purpose**: Initiates the password recovery flow.
* **Logic**:
  1. Looks up the user by email. If the user does not exist, it returns a generic success message to prevent user enumeration attacks.
  2. Generates a secure, random **Reset Token** (32 bytes) and a 6-digit numeric **OTP**.
  3. Hashes both the Reset Token and OTP using SHA-256.
  4. Inserts a record into the `password_resets` table containing `user_id`, `token_hash`, `otp_hash`, `attempt_count = 0`, and `expires_at = 15 minutes from now`.
  5. Sends the OTP code to the user's email.
  6. Returns the raw, unhashed Reset Token to the client (to be stored in browser memory/state).

### 6. Reset Password Service (`reset_password_service.rs`)
* **Purpose**: Completes the password recovery flow using the OTP and Reset Token.
* **Logic**:
  1. Hashes the user's input OTP and Reset Token.
  2. Retrieves the reset record from the `password_resets` table by `token_hash`.
  3. **Brute Force Protection**:
     - Increments the `attempt_count` in the database.
     - If `attempt_count > 5`, rejects the request immediately.
  4. Verifies the OTP hash matches, the record is not expired, and has not already been used (`used_at IS NULL`).
  5. Marks the reset record as used: `used_at = NOW()`.
  6. Hashes the new password using Argon2id and updates the user's password in the `users` table.
  7. **Security Invalidation**: Soft-revokes all existing active sessions/refresh tokens for the user (`UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL`) so that any leaked sessions on other devices are immediately terminated.

### 7. Logout Service (`logout_service.rs`)
* **Purpose**: Terminates the current active session.
* **Logic**:
  1. Decodes the refresh token to extract its `jti` (ignores expiration so users can log out even if the token has expired).
  2. Updates the database: `UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL`.
  3. Returns success (always idempotent; does not return errors if already logged out or if the token is invalid/not found).
