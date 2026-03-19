// Rust guideline compliant 2026-03-19
//
// SkiMap component -- renders ski segments and the computed route overlay
// using leptos-leaflet declarative components.
//
// Map click handling:
//   A `MapEvents::mouse_click` callback receives the Leaflet `MouseEvent`,
//   extracts the lat/lon, finds the nearest segment client-side, interpolates
//   the altitude, then updates `popup_info` to show the `SegmentPopup`.

use crate::components::segment_popup::{PopupData, nearest_segment};
use crate::models::{AreaSegment, HighlightSegment};
use leptos::prelude::*;
use leptos_leaflet::leaflet::MouseEvent;
use leptos_leaflet::prelude::*;

/// Map color for a segment given its kind and difficulty.
fn segment_color(kind: &str, difficulty: &str) -> &'static str {
    if kind == "lift" {
        return "#f59e0b"; // amber
    }
    match difficulty {
        "novice" => "#22c55e",                // green
        "easy" => "#3b82f6",                  // blue
        "intermediate" => "#ef4444",          // red
        "advanced" | "freeride" => "#1e293b", // black
        _ => "#94a3b8",                       // slate (unknown)
    }
}

/// Bearing in degrees (0 = north, 90 = east) from first to last coord.
///
/// Uses a flat-Earth approximation sufficient for a small ski resort.
fn route_bearing(coords: &[[f64; 2]]) -> f64 {
    if coords.len() < 2 {
        return 0.0;
    }
    let first = coords[0];
    let last = coords[coords.len() - 1];
    let dlat = last[0] - first[0];
    let dlon = last[1] - first[1];
    // atan2(dlon, dlat) yields a clockwise bearing from north in degrees.
    dlon.atan2(dlat).to_degrees()
}

/// Middle coordinate of a segment for arrow marker placement.
fn route_midpoint(coords: &[[f64; 2]]) -> Option<[f64; 2]> {
    if coords.is_empty() {
        return None;
    }
    Some(coords[coords.len() / 2])
}

/// CSS class encoding the arrow direction quantized to 8 sectors of 45 degrees.
///
/// The leptos-leaflet `rotation` prop relies on `Effect::watch(immediate=false)`,
/// which never fires for a static signal.  Instead we bake the rotation into a
/// CSS class so the `::before` pseudo-element carries the correct transform.
fn arrow_class(bearing: f64) -> &'static str {
    // Map bearing (0=N, 90=E, 180=S, 270=W) to one of 8 sectors.
    let b = (bearing.round() as i32).rem_euclid(360) as u32;
    let sector = (b + 22) / 45 % 8;
    match sector {
        0 => "route-arrow route-arrow-0",
        1 => "route-arrow route-arrow-45",
        2 => "route-arrow route-arrow-90",
        3 => "route-arrow route-arrow-135",
        4 => "route-arrow route-arrow-180",
        5 => "route-arrow route-arrow-225",
        6 => "route-arrow route-arrow-270",
        7 => "route-arrow route-arrow-315",
        _ => "route-arrow route-arrow-0",
    }
}

/// Interactive Leaflet map with ski segments, optional route highlight, and click popup.
///
/// When a route is active (`route_segments` non-empty):
/// - All background segments are dimmed (weight 4, opacity 0.50).
/// - Route segments are rendered as a white halo + difficulty-colored top layer.
/// - Directional arrow markers are placed at each segment midpoint.
///
/// When no route is active, segments render normally with filter-aware opacity.
///
/// Clicking any piste or lift polyline (within 30 m) shows a `SegmentPopup`
/// with name, type, difficulty/lift-subtype, coordinates, and interpolated altitude.
#[component]
pub fn SkiMap(
    segments: ReadSignal<Vec<AreaSegment>>,
    route_segments: ReadSignal<Vec<HighlightSegment>>,
    excluded_difficulties: ReadSignal<Vec<String>>,
    excluded_lift_types: ReadSignal<Vec<String>>,
    popup_info: RwSignal<Option<PopupData>>,
) -> impl IntoView {
    // Center on Serre Chevalier ski area.
    // 44.9403, 6.5063 = approximate center of the resort.
    let center = Position::new(44.9403, 6.5063);

    // Build MapEvents with a mouse_click handler.
    // The handler reads segments from the signal, finds the nearest one,
    // and writes the result into popup_info.
    // Clicking empty terrain (>30 m from any segment) sets None, closing
    // any open popup.
    let map_events = MapEvents::new().mouse_click(move |e: MouseEvent| {
        let ll = e.lat_lng();
        let lat = ll.lat();
        let lon = ll.lng();
        let segs = segments.get_untracked();
        let popup = nearest_segment(lat, lon, &segs);
        popup_info.set(popup);
    });

    view! {
        <MapContainer
            style="height:100%;width:100%;"
            center=center
            zoom=13.0
            set_view=true
            events=map_events
        >
            <TileLayer
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                attribution="&copy; OpenStreetMap contributors"
            />

            // Background ski segments.
            // Dimmed uniformly when a route is shown; otherwise natural color + filter opacity.
            {move || {
                let excl_diff = excluded_difficulties.get();
                let excl_lift = excluded_lift_types.get();
                let has_route = !route_segments.get().is_empty();
                segments
                    .get()
                    .into_iter()
                    .filter(|seg| seg.kind == "piste" || seg.kind == "lift")
                    .map(|seg| {
                        let (color, weight, opacity): (&str, f64, f64) = if has_route {
                            // Route active: keep natural color but dim weight + opacity.
                            (segment_color(&seg.kind, &seg.difficulty), 4.0, 0.50)
                        } else {
                            // Normal: natural color, opacity reduced only for filtered-out types.
                            let op = if seg.kind == "lift" {
                                if excl_lift.contains(&seg.difficulty) { 0.2 } else { 1.0 }
                            } else if excl_diff.contains(&seg.difficulty) {
                                0.2
                            } else {
                                1.0
                            };
                            (segment_color(&seg.kind, &seg.difficulty), 3.0, op)
                        };
                        let color = color.to_string();
                        let positions: Vec<Position> = seg
                            .coords
                            .iter()
                            .map(|c| Position::new(c[0], c[1]))
                            .collect();
                        let positions_sig = Signal::derive(move || positions.clone());
                        let color_sig = Signal::derive(move || color.clone());
                        let weight_sig = Signal::derive(move || Some(weight));
                        let opacity_sig = Signal::derive(move || Some(opacity));
                        view! {
                            <Polyline
                                positions=positions_sig
                                color=color_sig
                                weight=weight_sig
                                opacity=opacity_sig
                            />
                        }
                    })
                    .collect_view()
            }}

            // Route overlay -- three passes: white halos, colored lines, direction arrows.
            {move || {
                let segs = route_segments.get();

                // Pass 1: white halo beneath the colored line (Google Maps glow effect).
                let halos = segs
                    .iter()
                    .map(|hs| {
                        let positions: Vec<Position> =
                            hs.coords.iter().map(|c| Position::new(c[0], c[1])).collect();
                        let pos_sig = Signal::derive(move || positions.clone());
                        view! {
                            <Polyline
                                positions=pos_sig
                                color=Signal::derive(|| "#ffffff".to_string())
                                weight=Signal::derive(|| Some(10.0_f64))
                                opacity=Signal::derive(|| Some(0.6_f64))
                            />
                        }
                    })
                    .collect_view();

                // Pass 2: difficulty-colored line on top of the halo.
                let lines = segs
                    .iter()
                    .map(|hs| {
                        let color = segment_color(&hs.kind, &hs.difficulty).to_string();
                        let positions: Vec<Position> =
                            hs.coords.iter().map(|c| Position::new(c[0], c[1])).collect();
                        let pos_sig = Signal::derive(move || positions.clone());
                        let color_sig = Signal::derive(move || color.clone());
                        view! {
                            <Polyline
                                positions=pos_sig
                                color=color_sig
                                weight=Signal::derive(|| Some(6.0_f64))
                                opacity=Signal::derive(|| Some(1.0_f64))
                            />
                        }
                    })
                    .collect_view();

                // Pass 3: directional arrow at each segment midpoint.
                // Direction is encoded in the CSS class (see `arrow_class`), not via the
                // `rotation` prop, which does not fire for static signals.
                let arrows = segs
                    .iter()
                    .filter_map(|hs| {
                        let mid = route_midpoint(&hs.coords)?;
                        let class = arrow_class(route_bearing(&hs.coords)).to_string();
                        let pos = JsSignal::derive_local(move || Position::new(mid[0], mid[1]));
                        Some(view! {
                            <Marker
                                position=pos
                                icon_class=Signal::derive(move || Some(class.clone()))
                                icon_size=Signal::derive(|| Some((24.0_f64, 24.0_f64)))
                                icon_anchor=Signal::derive(|| Some((12.0_f64, 12.0_f64)))
                            />
                        })
                    })
                    .collect_view();

                view! { {halos} {lines} {arrows} }
            }}
        </MapContainer>
    }
}
