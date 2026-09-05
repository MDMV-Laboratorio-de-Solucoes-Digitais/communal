#![deny(unsafe_code)]
#![deny(missing_docs)]

//! WebAssembly bindings for the communal community detection framework.
//!
//! Exposes the Leiden algorithm as a single `detect_communities` function that
//! can be called from JavaScript or other WebAssembly hosts.

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use wasm_bindgen::prelude::*;

/// Detect communities using the Leiden algorithm (WASM entry point).
///
/// # Arguments
///
/// * `edges` - A JavaScript value that deserializes to `Vec<(u32, u32, f64)>`
///   representing the edge list (source, target, weight).
/// * `node_count` - The total number of nodes in the graph.
///
/// # Returns
///
/// A `JsValue` containing the membership vector (`Vec<u32>`) where element `i`
/// is the community ID assigned to node `i`.
#[wasm_bindgen]
pub fn detect_communities(edges: JsValue, node_count: usize) -> Result<JsValue, JsValue> {
    let edge_vec: Vec<(u32, u32, f64)> = serde_wasm_bindgen::from_value(edges)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse edges: {e}")))?;

    let graph = CsrGraph::from_edges(&edge_vec, node_count);
    let config = LeidenConfig::default();
    let detector = Leiden::new(config);

    let partition = detector
        .detect(&graph)
        .map_err(|e| JsValue::from_str(&format!("Detection failed: {e}")))?;

    serde_wasm_bindgen::to_value(partition.membership_vec())
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {e}")))
}
