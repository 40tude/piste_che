// Rust guideline compliant 2026-02-16
//
// Server functions -- auto-registered by Leptos under the `/api/` prefix.
// This module compiles for both `ssr` and `hydrate`; the `#[server]` macro
// generates full implementations under `ssr` and HTTP stubs under `hydrate`.

// Return types must be visible to both SSR and hydrate for deserialization.
use crate::models::{AreaResponse, RouteResponse};
use leptos::prelude::*;
use leptos::server_fn::codec::Json;

// SSR-only: routing types, application state, and internal model constructors.
#[cfg(feature = "ssr")]
use {
    crate::models::{AreaNode, AreaSegment, HighlightSegment, RouteStep, SelectableElement},
    crate::routing::{arrival_zone, dijkstra, segment_length, RouteElement, Segment},
    crate::server::AppState,
    std::sync::Arc,
};

/// Returns full ski area data: nodes, segments, and selectable lift elements.
///
/// Called by the WASM client (POST).  `GET /api/get_area` is handled by a
/// dedicated Axum route in `main.rs` so that non-Leptos callers (integration
/// tests, REST clients) can use the idiomatic HTTP method.
#[server(endpoint = "get_area")]
pub async fn get_area() -> Result<AreaResponse, ServerFnError> {
    let state = use_context::<Arc<AppState>>()
        .ok_or_else(|| ServerFnError::new("AppState not in context"))?;
    Ok(build_area_response(&state))
}

/// Assembles [`AreaResponse`] from application state.
///
/// Shared by the Leptos server function (POST) and the Axum GET handler so
/// both paths return identical data without duplicating logic.
#[cfg(feature = "ssr")]
pub fn build_area_response(state: &AppState) -> AreaResponse {
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
            occupancy: s.occupancy,
            duration_min: s.duration_min,
            coords: s.coords.iter().map(|c| [c[0], c[1], c[2]]).collect(),
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

    AreaResponse {
        nodes,
        segments,
        selectable_elements,
    }
}

/// Computes the shortest route between two named elements with optional filters.
///
/// Returns a [`RouteResponse`] with steps, total distance, and highlight
/// coordinates.  Sets `error` instead of failing the server function for
/// expected no-route conditions (same point, disconnected graph).
#[server(endpoint = "compute_route", input = Json, output = Json)]
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
            highlight_segments: vec![],
            error: Some(format!("Mode '{mode}' is not implemented; use 'short'")),
        });
    }

    // Same start/end guard.
    // Exception: if the element is a lift, allow routing from summit back to
    // base (circuit: ride up, ski down, return to departure station).
    if start == end {
        let is_lift = state
            .route_elements
            .iter()
            .any(|e| e.name == start && e.kind == "lift");
        if !is_lift {
            return Ok(RouteResponse {
                steps: vec![],
                total_distance_m: 0,
                highlight_segments: vec![],
                error: Some("Start and end points are the same".to_string()),
            });
        }
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

    let goal_zone = arrival_zone(end_el.start_node);

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
        } else {
            // Same-named piste split into sub-segments at junctions: accumulate
            // distance and extend the highlight rather than discarding sub-segment data.
            // steps and highlight_segments are always in sync: a new name always
            // pushes to both, so they can never be empty here.
            debug_assert!(
                !steps.is_empty() && !highlight_segments.is_empty(),
                "steps and highlight_segments must be non-empty before accumulating same-name segment"
            );
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "distance in metres, always positive and < 50 km"
            )]
            let dist = segment_length(seg).round() as u32;
            if let Some(last_step) = steps.last_mut() {
                last_step.distance_m = last_step.distance_m.saturating_add(dist);
            }
            total_distance_m = total_distance_m.saturating_add(dist);
            if let Some(last_hs) = highlight_segments.last_mut() {
                last_hs.coords.extend(seg.coords.iter().map(|c| [c[0], c[1]]));
            }
        }
    }

    // End element is always shown as the last step (unless already appended)
    // so the user knows where the itinerary terminates.
    //
    // The arrival is the *departure station* of a lift, not a segment the user
    // actually rides.  Its distance is therefore intentionally excluded from
    // `total_distance_m`.  Adding it would over-count: for a circular route
    // (START == END, e.g. "Ecole de Frejus") the same lift would be counted
    // twice -- once at the start (ridden up) and once at the end (just arrived
    // at the base, not ridden again).
    //
    // Compare (name, kind) not name alone: two elements can share a display
    // name while having different kinds (e.g., "Eychauda" piste vs lift).
    let last_matches_end = steps.last().is_some_and(|s| {
        s.name.as_str() == end_el.name.as_str() && s.kind.as_str() == end_el.kind.as_str()
    });
    if !last_matches_end {
        let end_dist = element_distance(end_el, &state.segments);
        steps.push(RouteStep {
            name: end_el.name.clone(),
            kind: end_el.kind.clone(),
            difficulty: end_el.difficulty.clone(),
            distance_m: end_dist,
        });
        // NOTE: end_dist is deliberately NOT added to total_distance_m.
        // See the block comment above.
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
/// Haversine distance for a route step (lifts and pistes).
fn element_distance(el: &RouteElement, segments: &[Segment]) -> u32 {
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
