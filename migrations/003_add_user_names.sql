-- Add first_name and last_name columns to users table
-- Both fields are optional (nullable)

ALTER TABLE users ADD COLUMN first_name TEXT;
ALTER TABLE users ADD COLUMN last_name TEXT;
