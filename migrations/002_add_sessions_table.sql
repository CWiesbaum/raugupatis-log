-- Add sessions table for server-side session management
-- This migration creates the sessions table required by tower-sessions-rusqlite-store

CREATE TABLE IF NOT EXISTS tower_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    data TEXT NOT NULL,
    expiry_date INTEGER NOT NULL
);

-- Index for expiry date to efficiently clean up expired sessions
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON tower_sessions(expiry_date);
