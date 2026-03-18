// Rust guideline compliant 2026-02-16
//
// `server` module is compiled for BOTH ssr and hydrate:
//   - `api` contains `#[server]` functions whose stubs are needed on the client.
//   - `AppState` and routing imports are gated behind `ssr` only.

pub mod api;

#[cfg(feature = "ssr")]
use crate::routing::graph::{Node, RouteElement, Segment};
#[cfg(feature = "ssr")]
use std::collections::HashMap;

/// Precomputed ski domain graph held in `Arc` for cheap per-request cloning.
///
/// Populated once at startup from `data/*.json`.  Immutable after init so no
/// locking is needed.
#[cfg(feature = "ssr")]
#[derive(Debug)]
pub struct AppState {
    pub nodes: Vec<Node>,
    pub segments: Vec<Segment>,
    pub route_elements: Vec<RouteElement>,
    /// Adjacency list: node ID -> segment IDs departing from that node.
    pub adjacency: HashMap<usize, Vec<usize>>,
}
