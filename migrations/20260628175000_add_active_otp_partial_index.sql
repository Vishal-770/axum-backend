-- Create partial index on active email verification OTPs
CREATE INDEX idx_email_otp_email_active ON email_otp(email) WHERE used_at IS NULL;
