-- Add used_at column to email_otp table
ALTER TABLE email_otp ADD COLUMN used_at TIMESTAMP;
