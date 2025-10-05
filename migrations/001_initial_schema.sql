-- Initial schema for Raugupatis Log
-- This migration creates the basic tables needed for the application

-- Users table with role-based permissions
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user', 'admin')),
    experience_level TEXT DEFAULT 'beginner' CHECK (experience_level IN ('beginner', 'intermediate', 'advanced')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Fermentation profiles (templates) table
CREATE TABLE fermentation_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL,
    min_days INTEGER NOT NULL,
    max_days INTEGER NOT NULL,
    temp_min REAL NOT NULL,
    temp_max REAL NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Active/completed fermentations table
CREATE TABLE fermentations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    profile_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    start_date DATETIME NOT NULL,
    target_end_date DATETIME,
    actual_end_date DATETIME,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'failed')),
    success_rating INTEGER CHECK (success_rating BETWEEN 1 AND 5),
    notes TEXT,
    ingredients_json TEXT, -- JSON formatted ingredients list
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (profile_id) REFERENCES fermentation_profiles (id)
);

-- Temperature logs table (optimized for time-series data)
CREATE TABLE temperature_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fermentation_id INTEGER NOT NULL,
    recorded_at DATETIME NOT NULL,
    temperature REAL NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fermentation_id) REFERENCES fermentations (id) ON DELETE CASCADE
);

-- Photo documentation table
CREATE TABLE fermentation_photos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fermentation_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    caption TEXT,
    taken_at DATETIME NOT NULL,
    stage TEXT DEFAULT 'progress' CHECK (stage IN ('start', 'progress', 'end')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fermentation_id) REFERENCES fermentations (id) ON DELETE CASCADE
);

-- Performance indexes
CREATE INDEX idx_temp_logs_fermentation_time ON temperature_logs(fermentation_id, recorded_at);
CREATE INDEX idx_fermentations_user_status ON fermentations(user_id, status);
CREATE INDEX idx_fermentations_profile ON fermentations(profile_id);
CREATE INDEX idx_photos_fermentation ON fermentation_photos(fermentation_id);

-- Insert default fermentation profiles
INSERT INTO fermentation_profiles (name, type, min_days, max_days, temp_min, temp_max, description) VALUES 
('Pickles', 'vegetable', 3, 7, 65.0, 75.0, 'Salt brine fermented pickles - cucumbers, carrots, or other vegetables'),
('Kombucha', 'beverage', 7, 14, 68.0, 78.0, 'SCOBY-based fermented tea with first and second fermentation'),
('Kimchi', 'vegetable', 3, 5, 65.0, 75.0, 'Korean fermented cabbage with chili paste and aromatics'),
('Sauerkraut', 'vegetable', 14, 28, 65.0, 72.0, 'Dry salt fermented cabbage - traditional German style'),
('Sourdough Starter', 'bread', 5, 7, 70.0, 80.0, 'Wild yeast and bacteria culture for bread making'),
('Kefir (Milk)', 'dairy', 1, 1, 68.0, 76.0, 'Kefir grains fermented milk - 12-24 hour cycle'),
('Water Kefir', 'beverage', 1, 3, 68.0, 76.0, 'Water kefir grains fermented sugar water with fruit');