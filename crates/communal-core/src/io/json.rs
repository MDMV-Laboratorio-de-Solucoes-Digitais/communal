//! JSON graph format support.
//!
//! Placeholder module for future JSON serialization/deserialization.

/// Parses a JSON-encoded graph representation.
///
/// Currently a placeholder that always returns an unsupported format error.
///
/// # Errors
///
/// Always returns [`crate::error::GraphError::UnsupportedFormat`] since JSON
/// parsing is not yet implemented.
pub fn parse_json(_input: &str) -> Result<(), crate::error::GraphError> {
    Err(crate::error::GraphError::UnsupportedFormat {
        format: "json".to_string(),
    })
}
