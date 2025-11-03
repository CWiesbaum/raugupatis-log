-- Add lessons_learned field to fermentations table
ALTER TABLE fermentations ADD COLUMN lessons_learned TEXT;

-- Create taste_profiles table to track multiple taste profiles over time
CREATE TABLE taste_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fermentation_id INTEGER NOT NULL,
    profile_text TEXT NOT NULL,
    tasted_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fermentation_id) REFERENCES fermentations (id) ON DELETE CASCADE
);

-- Performance index for taste profiles
CREATE INDEX idx_taste_profiles_fermentation ON taste_profiles(fermentation_id, tasted_at DESC);
