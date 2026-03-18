// Rust guideline compliant 2026-02-16
//
// Routing module -- server-only (`#[cfg(feature = "ssr")]` gate applied in lib.rs).
// Re-exports the public interface consumed by `server/api.rs`.

pub mod chains;
pub mod data;
pub mod dijkstra;
pub mod graph;

pub use data::OsmData;
pub use dijkstra::{dijkstra, segment_length};
pub use graph::{
    adjacency_from_segments, arrival_zone, build_graph, Node, RouteElement, Segment,
};
