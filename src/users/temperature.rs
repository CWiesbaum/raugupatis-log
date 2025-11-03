/// Temperature conversion utilities
/// All temperatures in the database are stored in Fahrenheit
use crate::users::models::TemperatureUnit;

/// Convert temperature to the user's preferred unit
/// If the user prefers Fahrenheit, returns the value as-is
/// If the user prefers Celsius, converts from Fahrenheit to Celsius
pub fn convert_temp_for_display(temp_fahrenheit: f64, preferred_unit: &TemperatureUnit) -> f64 {
    match preferred_unit {
        TemperatureUnit::Fahrenheit => temp_fahrenheit,
        TemperatureUnit::Celsius => fahrenheit_to_celsius(temp_fahrenheit),
    }
}

/// Convert temperature from user's preferred unit to Fahrenheit for storage
/// If the user prefers Fahrenheit, returns the value as-is
/// If the user prefers Celsius, converts from Celsius to Fahrenheit
pub fn convert_temp_for_storage(temp_value: f64, preferred_unit: &TemperatureUnit) -> f64 {
    match preferred_unit {
        TemperatureUnit::Fahrenheit => temp_value,
        TemperatureUnit::Celsius => celsius_to_fahrenheit(temp_value),
    }
}

/// Convert Fahrenheit to Celsius
/// Formula: (°F - 32) × 5/9 = °C
pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Convert Celsius to Fahrenheit
/// Formula: (°C × 9/5) + 32 = °F
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

/// Get the temperature unit symbol
pub fn get_unit_symbol(unit: &TemperatureUnit) -> &str {
    match unit {
        TemperatureUnit::Fahrenheit => "°F",
        TemperatureUnit::Celsius => "°C",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fahrenheit_to_celsius() {
        // Water freezing point
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);

        // Water boiling point
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);

        // Room temperature
        let room_temp_c = fahrenheit_to_celsius(68.0);
        assert!((room_temp_c - 20.0).abs() < 0.1);

        // Fermentation temp range
        let ferment_temp_c = fahrenheit_to_celsius(70.0);
        assert!((ferment_temp_c - 21.11).abs() < 0.1);
    }

    #[test]
    fn test_celsius_to_fahrenheit() {
        // Water freezing point
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);

        // Water boiling point
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);

        // Room temperature
        let room_temp_f = celsius_to_fahrenheit(20.0);
        assert_eq!(room_temp_f, 68.0);

        // Fermentation temp range
        let ferment_temp_f = celsius_to_fahrenheit(21.0);
        assert!((ferment_temp_f - 69.8).abs() < 0.1);
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_f = 75.0;
        let celsius = fahrenheit_to_celsius(original_f);
        let back_to_f = celsius_to_fahrenheit(celsius);
        assert!((original_f - back_to_f).abs() < 0.0001);
    }

    #[test]
    fn test_convert_temp_for_display_fahrenheit() {
        let temp = 75.0;
        let result = convert_temp_for_display(temp, &TemperatureUnit::Fahrenheit);
        assert_eq!(result, 75.0);
    }

    #[test]
    fn test_convert_temp_for_display_celsius() {
        let temp_f = 68.0; // Room temperature
        let result = convert_temp_for_display(temp_f, &TemperatureUnit::Celsius);
        assert!((result - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_convert_temp_for_storage_fahrenheit() {
        let temp = 75.0;
        let result = convert_temp_for_storage(temp, &TemperatureUnit::Fahrenheit);
        assert_eq!(result, 75.0);
    }

    #[test]
    fn test_convert_temp_for_storage_celsius() {
        let temp_c = 20.0; // Room temperature in Celsius
        let result = convert_temp_for_storage(temp_c, &TemperatureUnit::Celsius);
        assert_eq!(result, 68.0);
    }

    #[test]
    fn test_get_unit_symbol() {
        assert_eq!(get_unit_symbol(&TemperatureUnit::Fahrenheit), "°F");
        assert_eq!(get_unit_symbol(&TemperatureUnit::Celsius), "°C");
    }
}
