-- Add is_locked field to users table for admin to lock/unlock user accounts
ALTER TABLE users ADD COLUMN is_locked INTEGER NOT NULL DEFAULT 0;

-- Create index for efficient locked user queries
CREATE INDEX idx_users_locked ON users(is_locked);
