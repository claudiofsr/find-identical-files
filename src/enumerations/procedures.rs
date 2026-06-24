use crate::FIFError;
use std::fmt;

/// Represents the stages of the file deduplication pipeline.
///
/// The process follows a "Successive Filtering" strategy:
/// 1. Compare by Size (Fastest, many false positives).
/// 2. Compare by Header/First Bytes (Fast, filters out most unique files).
/// 3. Compare by Full Content Hash (Slower, definitive proof of identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Procedure {
    /// Step 1: Group files sharing the exact same byte count.
    Size = 1,
    /// Step 2: Group files sharing the same hash of their initial bytes (e.g., first 1KB).
    FirstBytes = 2,
    /// Step 3: Group files sharing the same hash of their entire content.
    EntireFile = 3,
}

impl Procedure {
    /// Validates if the number of files in a group satisfies the filter criteria.
    ///
    /// # Logic:
    /// - During **Size** and **FirstBytes** stages, we only care about the `min` frequency.
    ///   We keep any group that *could* potentially be a duplicate.
    /// - During the **EntireFile** stage (final result), we apply both `min` and `max`
    ///   constraints provided by the user to finalize the report.
    pub fn is_valid_frequency(&self, count: usize, min: usize, max: usize) -> bool {
        match self {
            // Preliminary stages: keep candidates meeting the minimum threshold.
            Procedure::Size | Procedure::FirstBytes => count >= min,
            // Final stage: apply strict bounds for the final output.
            Procedure::EntireFile => count >= min && count <= max,
        }
    }

    /// Returns a human-readable description of what this stage filters.
    pub fn description(&self) -> &'static str {
        match self {
            Procedure::Size => "Number of files of identical size",
            Procedure::FirstBytes => "Number of files with identical first bytes",
            Procedure::EntireFile => "Number of files with identical hashes",
        }
    }
}

/// Facilitates safe conversion from raw numbers (e.g., from CLI args or config) to Enum variants.
impl TryFrom<u8> for Procedure {
    type Error = FIFError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Procedure::Size),
            2 => Ok(Procedure::FirstBytes),
            3 => Ok(Procedure::EntireFile),
            // Returns our custom error variant instead of a simple String.
            _ => Err(FIFError::InvalidProcedure(value)),
        }
    }
}

impl fmt::Display for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Displays as "Step 1", "Step 2", etc.
        write!(f, "Step {}", *self as u8)
    }
}
