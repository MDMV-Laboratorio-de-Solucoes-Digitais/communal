//! GML (Graph Modelling Language) format support.
//!
//! Placeholder module for future GML serialization/deserialization.

/// Parses a GML-encoded graph representation.
///
/// Currently a placeholder that always returns an unsupported format error.
pub fn parse_gml(_input: &str) -> Result<(), crate::error::GraphError> {
    Err(crate::error::GraphError::UnsupportedFormat {
        format: "gml".to_string(),
    })
}
