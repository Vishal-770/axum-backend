-- Add resend tracking fields to email_otp table
ALTER TABLE email_otp 
ADD COLUMN resend_count INT DEFAULT 0 NOT NULL,
ADD COLUMN last_sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;
