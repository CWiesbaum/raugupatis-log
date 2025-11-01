-- Add preferred temperature unit to users table
-- Default to Fahrenheit to match existing behavior
ALTER TABLE users ADD COLUMN preferred_temp_unit TEXT NOT NULL DEFAULT 'fahrenheit' CHECK (preferred_temp_unit IN ('fahrenheit', 'celsius'));
