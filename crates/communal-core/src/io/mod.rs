//! Graph input/output utilities.
//!
//! This module provides parsers and serializers for common graph formats
//! including edge lists, JSON, GML, and CSV.

/// CSV graph format support.
pub mod csv;
/// Edge list parsing and construction.
pub mod edgelist;
/// GML (Graph Modelling Language) format support.
pub mod gml;
/// JSON graph format support.
pub mod json;
/// Graph and partition serialization.
pub mod serialize;
