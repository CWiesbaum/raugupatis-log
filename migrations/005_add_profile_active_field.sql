-- Add is_active field to fermentation_profiles table
-- This allows soft deletion/deactivation of profiles
-- Deactivated profiles cannot be selected for new fermentations
-- but existing fermentations still reference them

ALTER TABLE fermentation_profiles ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1));

-- Create index for faster queries filtering by active status
CREATE INDEX idx_fermentation_profiles_active ON fermentation_profiles(is_active);
