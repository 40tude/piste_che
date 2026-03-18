// Rust guideline compliant 2026-02-16
//
// Shared API DTOs -- compiled for both `ssr` (server) and `hydrate` (WASM client).
// All types derive Serialize + Deserialize for server function transport.

use serde::{Deserialize, Serialize};

/// A graph node simplified for frontend consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaNode {
    pub id: usize,
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
}

/// A directed segment projected for map rendering (elevation stripped).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaSegment {
    pub id: usize,
    pub name: String,
    pub kind: String,
    pub difficulty: String,
    /// `[lat, lon]` pairs along the arc (no elevation, saves bandwidth).
    pub coords: Vec<[f64; 2]>,
}

/// One entry in the start/end dropdown (lift-type elements only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectableElement {
    pub name: String,
    pub kind: String,
    pub difficulty: String,
}

/// Top-level response for `get_area`.  Wraps all map + dropdown data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaResponse {
    pub nodes: Vec<AreaNode>,
    pub segments: Vec<AreaSegment>,
    pub selectable_elements: Vec<SelectableElement>,
}

/// Client-to-server route computation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    pub start: String,
    pub end: String,
    pub excluded_difficulties: Vec<String>,
    pub excluded_lift_types: Vec<String>,
    /// Routing mode.  Only `"short"` is implemented; others return an error.
    pub mode: String,
}

/// One step in the itinerary panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    pub name: String,
    pub kind: String,
    pub difficulty: String,
    /// Approximate distance: lifts use 50 m flat cost; pistes use haversine length.
    pub distance_m: u32,
}

/// Server-to-client route computation result.
///
/// On failure `steps` is empty and `error` contains the reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub steps: Vec<RouteStep>,
    pub total_distance_m: u32,
    /// One polyline per segment in the route for map overlay.
    pub highlight_coords: Vec<Vec<[f64; 2]>>,
    pub error: Option<String>,
}
