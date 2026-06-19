-- Add index on token_hash for efficient lookup in the atomic UPDATE...RETURNING query.
-- The reset flow now hashes the incoming OTP and matches it directly against this column
-- rather than scanning by user_id + email.
CREATE INDEX idx_password_resets_token_hash ON password_resets(token_hash);
