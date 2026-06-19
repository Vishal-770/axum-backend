-- Extend password_resets with:
--   otp_hash      — separate secret sent to the user's email; both token_hash
--                   AND otp_hash must match to consume a reset request.
--   attempt_count — incremented on each wrong OTP; request is invalidated
--                   after 5 failed attempts to prevent brute-force.

ALTER TABLE password_resets
    ADD COLUMN otp_hash       TEXT NOT NULL DEFAULT '',
    ADD COLUMN attempt_count  INT  NOT NULL DEFAULT 0;

-- Remove bootstrap default — every new row will supply a real otp_hash.
ALTER TABLE password_resets ALTER COLUMN otp_hash DROP DEFAULT;

-- Partial index: only active (not-yet-used) rows need fast token_hash lookups.
CREATE INDEX idx_password_resets_token_hash_active
    ON password_resets(token_hash)
    WHERE used_at IS NULL;
