// Rust guideline compliant 2026-02-16
//
// Server functions -- auto-registered by Leptos under the `/api/` prefix.
// This module compiles for both `ssr` and `hydrate`; the `#[server]` macro
// generates full implementations under `ssr` and HTTP stubs under `hydrate`.

// Return types must be visible to both SSR and hydrate for deserialization.
use crate::models::{AreaResponse, HighlightSegment, RouteResponse};
use leptos::prelude::*;
use leptos::server_fn::codec::Json;

// SSR-only: routing types, application state, and internal model constructors.
#[cfg(feature = "ssr")]
use {
    crate::models::{AreaNode, AreaSegment, RouteStep, SelectableElement},
    crate::routing::{arrival_zone, dijkstra, segment_length, RouteElement, Segment},
    crate::server::AppState,
    std::sync::Arc,
};

/// Returns full ski area data: nodes, segments, and selectable lift elements.
///
/// Called once on page load; the client caches the response for the session.
#[server(GetUrl)]
pub async fn get_area() -> Result<AreaResponse, ServerFnError> {
    let state = use_context::<Arc<AppState>>()
        .ok_or_else(|| ServerFnError::new("AppState not in context"))?;

    let nodes: Vec<AreaNode> = state
        .nodes
        .iter()
        .map(|n| AreaNode {
            id: n.id,
            lat: n.coord[0],
            lon: n.coord[1],
            alt: n.coord[2],
        })
        .collect();

    let segments: Vec<AreaSegment> = state
        .segments
        .iter()
        .map(|s| AreaSegment {
            id: s.id,
            name: s.name.clone(),
            kind: s.kind.clone(),
            difficulty: s.difficulty.clone(),
            // Drop elevation from coords; only lat/lon needed by the map.
            coords: s.coords.iter().map(|c| [c[0], c[1]]).collect(),
        })
        .collect();

    // Only lift-type RouteElements appear in the start/end dropdowns (FR-004).
    let selectable_elements: Vec<SelectableElement> = state
        .route_elements
        .iter()
        .filter(|e| e.kind == "lift")
        .map(|e| SelectableElement {
            name: e.name.clone(),
            kind: e.kind.clone(),
            difficulty: e.difficulty.clone(),
        })
        .collect();

    tracing::event!(
        name: "api.get_area.success",
        tracing::Level::INFO,
        nodes.count = nodes.len(),
        segments.count = segments.len(),
        selectable.count = selectable_elements.len(),
        "get_area: {{nodes.count}} nodes, {{segments.count}} segments, {{selectable.count}} selectable",
    );

    Ok(AreaResponse {
        nodes,
        segments,
        selectable_elements,
    })
}

/// Computes the shortest route between two named elements with optional filters.
///
/// Returns a [`RouteResponse`] with steps, total distance, and highlight
/// coordinates.  Sets `error` instead of failing the server function for
/// expected no-route conditions (same point, disconnected graph).
#[server(input = Json, output = Json)]
pub async fn compute_route(
    start: String,
    end: String,
    excluded_difficulties: Vec<String>,
    excluded_lift_types: Vec<String>,
    mode: String,
) -> Result<RouteResponse, ServerFnError> {
    let state = use_context::<Arc<AppState>>()
        .ok_or_else(|| ServerFnError::new("AppState not in context"))?;

    tracing::event!(
        name: "api.compute_route.start",
        tracing::Level::INFO,
        route.start = %start,
        route.end = %end,
        route.mode = %mode,
        "compute_route: {{route.start}} -> {{route.end}} (mode={{route.mode}})",
    );

    // Validate mode -- only "short" is implemented.
    if mode != "short" {
        return Ok(RouteResponse {
            steps: vec![],
            total_distance_m: 0,
            highlight_coords: vec![],
            error: Some(format!("Mode '{mode}' is not implemented; use 'short'")),
        });
    }

    // Same start/end guard.
    if start == end {
        return Ok(RouteResponse {
            steps: vec![],
            total_distance_m: 0,
            highlight_coords: vec![],
            error: Some("Start and end points are the same".to_string()),
        });
    }

    // Resolve named elements.
    let start_el = state
        .route_elements
        .iter()
        .find(|e| e.name == start)
        .ok_or_else(|| ServerFnError::new(format!("Unknown start element: '{start}'")))?;

    let end_el = state
        .route_elements
        .iter()
        .find(|e| e.name == end)
        .ok_or_else(|| ServerFnError::new(format!("Unknown end element: '{end}'")))?;

    // Build excluded slices for Dijkstra.
    let excl_diff: Vec<&str> = excluded_difficulties.iter().map(String::as_str).collect();
    let excl_lift: Vec<&str> = excluded_lift_types.iter().map(String::as_str).collect();

    let goal_zone = arrival_zone(end_el.start_node, &state.nodes);

    let mid_path = dijkstra(
        start_el.end_node,
        &goal_zone,
        state.nodes.len(),
        &state.segments,
        &state.adjacency,
        &excl_diff,
        &excl_lift,
    );

    let Some(mid) = mid_path else {
        return Ok(RouteResponse {
            steps: vec![],
            total_distance_m: 0,
            highlight_segments: vec![],
            error: Some(format!("No route found between '{start}' and '{end}'")),
        });
    };

    // Start element is always step 1.
    let start_dist = element_distance(start_el, &state.segments);
    let mut steps: Vec<RouteStep> = vec![RouteStep {
        name: start_el.name.clone(),
        kind: start_el.kind.clone(),
        difficulty: start_el.difficulty.clone(),
        distance_m: start_dist,
    }];
    let mut highlight_segments: Vec<HighlightSegment> =
        vec![element_highlight(start_el, &state.segments)];
    let mut total_distance_m: u32 = start_dist;

    // Append intermediate named elements (skip synthetic edges; merge consecutive same-name).
    let mut last_name = start_el.name.as_str();
    for &sid in &mid {
        let seg = &state.segments[sid];
        if matches!(seg.kind.as_str(), "traverse" | "ski-out" | "ski-in") {
            continue;
        }
        if seg.name != last_name {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "distance in metres, always positive and < 50 km"
            )]
            let dist = segment_length(seg).round() as u32;
            steps.push(RouteStep {
                name: seg.name.clone(),
                kind: seg.kind.clone(),
                difficulty: seg.difficulty.clone(),
                distance_m: dist,
            });
            highlight_segments.push(HighlightSegment {
                coords: seg.coords.iter().map(|c| [c[0], c[1]]).collect(),
                kind: seg.kind.clone(),
                difficulty: seg.difficulty.clone(),
            });
            total_distance_m = total_distance_m.saturating_add(dist);
            last_name = seg.name.as_str();
        }
    }

    // End element is always the last step (unless already appended).
    if steps.last().map(|s| s.name.as_str()) != Some(end_el.name.as_str()) {
        let end_dist = element_distance(end_el, &state.segments);
        steps.push(RouteStep {
            name: end_el.name.clone(),
            kind: end_el.kind.clone(),
            difficulty: end_el.difficulty.clone(),
            distance_m: end_dist,
        });
        highlight_segments.push(element_highlight(end_el, &state.segments));
        total_distance_m = total_distance_m.saturating_add(end_dist);
    }

    tracing::event!(
        name: "api.compute_route.success",
        tracing::Level::INFO,
        route.steps = steps.len(),
        route.total_distance_m = total_distance_m,
        "compute_route: {{route.steps}} steps, {{route.total_distance_m}} m total",
    );

    Ok(RouteResponse {
        steps,
        total_distance_m,
        highlight_segments,
        error: None,
    })
}

// ---------------------------------------------------------------------------
// Helpers (ssr-only, called inside server function bodies)
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
/// Approximate distance for a route step: 50 m for lifts, haversine sum for pistes.
fn element_distance(el: &RouteElement, segments: &[Segment]) -> u32 {
    if el.kind == "lift" {
        // 50 m flat cost matching Dijkstra's lift weight constant.
        return 50;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "distance in metres, always positive and < 50 km"
    )]
    segments
        .iter()
        .filter(|s| s.name == el.name && s.kind == el.kind)
        .map(|s| segment_length(s).round() as u32)
        .fold(0u32, u32::saturating_add)
}

#[cfg(feature = "ssr")]
/// Collect coords + kind/difficulty for a route element's segments (highlight overlay).
fn element_highlight(el: &RouteElement, segments: &[Segment]) -> HighlightSegment {
    let coords = segments
        .iter()
        .filter(|s| s.name == el.name && s.kind == el.kind)
        .flat_map(|s| s.coords.iter().map(|c| [c[0], c[1]]))
        .collect();
    HighlightSegment {
        coords,
        kind: el.kind.clone(),
        difficulty: el.difficulty.clone(),
    }
}
