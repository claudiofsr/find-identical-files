use std::env;

/// Returns the preferred thousands separator character.
/// Precedence: LC_NUMERIC env var > compile-time feature > default (comma).
pub fn get_thousands_separator() -> char {
    let mut chosen_separator: char = ','; // Default to comma

    // 1. Check LC_NUMERIC environment variable.
    if let Ok(lc_numeric) = env::var("LC_NUMERIC") {
        if lc_numeric.starts_with("en_") {
            // Anglo-Saxon countries (e.g., UK, USA) typically use ','.
            chosen_separator = ',';
        } else {
            // Assume other locales, especially many European ones, use '.' as thousands separator.
            // This is a simplification and not universally true for all non-en_ locales.
            chosen_separator = '.';
        }
    }

    // 2. Check compile-time features.
    #[cfg(feature = "thousands-sep-comma")]
    {
        chosen_separator = ','; // Override with comma if feature enabled.
    }

    #[cfg(feature = "thousands-sep-dot")]
    {
        chosen_separator = '.'; // Override with dot if feature enabled.
    }

    #[cfg(feature = "thousands-sep-space")]
    {
        chosen_separator = ' '; // Override with space if feature enabled.
    }

    chosen_separator
}
