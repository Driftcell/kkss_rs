-- Add membership expiration column to users
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS membership_expires_at TIMESTAMPTZ NULL;

-- Optional index to speed up expiration checks
CREATE INDEX IF NOT EXISTS idx_users_membership_expires_at
    ON users(membership_expires_at)
    WHERE membership_expires_at IS NOT NULL;
