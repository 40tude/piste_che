// Rust guideline compliant 2026-03-19
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

/// A directed segment projected for map rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaSegment {
    pub id: usize,
    pub name: String,
    pub kind: String,
    pub difficulty: String,
    /// Seat count per cabin/chair (aerialway:occupancy), lifts only.
    pub occupancy: Option<u32>,
    /// Ride duration in minutes (aerialway:duration), lifts only.
    pub duration_min: Option<u32>,
    /// `[lat, lon, elevation_m]` triples along the arc.
    pub coords: Vec<[f64; 3]>,
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
    /// Haversine distance in metres.
    pub distance_m: u32,
}

/// One highlighted segment in the route overlay (preserves kind + difficulty for coloring).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSegment {
    pub coords: Vec<[f64; 2]>,
    pub kind: String,
    pub difficulty: String,
}

/// Server-to-client route computation result.
///
/// On failure `steps` is empty and `error` contains the reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub steps: Vec<RouteStep>,
    pub total_distance_m: u32,
    /// One entry per named element in the route; carries kind+difficulty for natural coloring.
    pub highlight_segments: Vec<HighlightSegment>,
    pub error: Option<String>,
}
