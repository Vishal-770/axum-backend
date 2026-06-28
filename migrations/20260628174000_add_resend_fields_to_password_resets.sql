-- Add resend tracking fields to password_resets table
ALTER TABLE password_resets 
ADD COLUMN resend_count INT DEFAULT 0 NOT NULL,
ADD COLUMN last_sent_at TIMESTAMPTZ DEFAULT NOW() NOT NULL;
