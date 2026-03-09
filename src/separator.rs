#![allow(unused_assignments)]
use std::env;
use std::sync::OnceLock;

/// Cache for the separator to avoid expensive System Calls (env::var)
/// and String allocations during repetitive serialization tasks.
static THOUSANDS_SEP: OnceLock<char> = OnceLock::new();

/// Returns the preferred thousands separator character with lazy initialization.
pub fn get_thousands_separator() -> char {
    *THOUSANDS_SEP.get_or_init(|| {
        // 1. High priority: Compile-time features (fixed at build time)
        #[cfg(feature = "thousands-sep-comma")]
        {
            return ',';
        }
        #[cfg(feature = "thousands-sep-dot")]
        {
            return '.';
        }
        #[cfg(feature = "thousands-sep-space")]
        {
            return ' ';
        }

        // 2. Low priority: Runtime environment check
        if let Ok(lc_numeric) = env::var("LC_NUMERIC") {
            // Locales like pt_BR or de_DE usually use '.'
            if !lc_numeric.starts_with("en_") {
                return '.';
            }
        }

        // Default to US standard
        ','
    })
}
