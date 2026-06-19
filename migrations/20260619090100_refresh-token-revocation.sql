-- Add soft-revocation and token family tracking to refresh_tokens.
--
-- revoked_at: NULL = active, non-NULL = rotated away or explicitly revoked.
--             Keeping the row (instead of deleting) means we can detect reuse:
--             if a rotated token (revoked_at != NULL) is presented again, that's
--             a strong signal of theft or replay.
--
-- family_id:  Groups all tokens produced from a single login into one "family".
--             On reuse detection we revoke the entire family, kicking every session
--             that descended from the original compromised token.

ALTER TABLE refresh_tokens
    ADD COLUMN revoked_at TIMESTAMPTZ NULL,
    ADD COLUMN family_id  UUID        NOT NULL DEFAULT gen_random_uuid();

-- Fast lookup for family-wide revocation (reuse detection)
CREATE INDEX idx_refresh_tokens_family_id
    ON refresh_tokens(family_id);

-- Fast lookup for active tokens per user (WHERE revoked_at IS NULL)
CREATE INDEX idx_refresh_tokens_user_revoked
    ON refresh_tokens(user_id, revoked_at);
