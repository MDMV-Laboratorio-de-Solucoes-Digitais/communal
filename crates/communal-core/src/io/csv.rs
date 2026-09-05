//! CSV graph format support.
//!
//! Placeholder module for future CSV serialization/deserialization.

/// Parses a CSV-encoded graph representation.
///
/// Currently a placeholder that always returns an unsupported format error.
pub fn parse_csv(_input: &str) -> Result<(), crate::error::GraphError> {
    Err(crate::error::GraphError::UnsupportedFormat {
        format: "csv".to_string(),
    })
}
