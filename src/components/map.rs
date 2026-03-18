// Rust guideline compliant 2026-02-16
//
// SkiMap component -- renders ski segments and the computed route overlay
// using leptos-leaflet declarative components.

use crate::models::AreaSegment;
use leptos::prelude::*;
use leptos_leaflet::prelude::*;

/// Map color for a segment given its kind and difficulty.
fn segment_color(kind: &str, difficulty: &str) -> &'static str {
    if kind == "lift" {
        return "#f59e0b"; // amber
    }
    match difficulty {
        "novice" => "#22c55e",               // green
        "easy" => "#3b82f6",                 // blue
        "intermediate" => "#ef4444",          // red
        "advanced" | "freeride" => "#1e293b", // black
        _ => "#94a3b8",                      // slate (unknown)
    }
}

/// Interactive Leaflet map with ski segments and optional route highlight.
///
/// Segment opacity is 0.2 when the segment's difficulty or lift type is in the
/// respective excluded set, giving visual feedback for active filters.
#[component]
pub fn SkiMap(
    segments: ReadSignal<Vec<AreaSegment>>,
    route_coords: ReadSignal<Vec<Vec<[f64; 2]>>>,
    excluded_difficulties: ReadSignal<Vec<String>>,
    excluded_lift_types: ReadSignal<Vec<String>>,
) -> impl IntoView {
    // Center on Serre Chevalier ski area.
    // 44.9403, 6.5063 = approximate center of the resort.
    let center = Position::new(44.9403, 6.5063);

    view! {
        <MapContainer
            style="height:100%;width:100%;"
            center=center
            zoom=13.0
            set_view=true
        >
            <TileLayer
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                attribution="&copy; OpenStreetMap contributors"
            />

            // Ski segments -- collect_view avoids keyed-For attribute parsing issues.
            // Re-renders reactively when segments or filter signals change.
            {move || {
                let excl_diff = excluded_difficulties.get();
                let excl_lift = excluded_lift_types.get();
                segments
                    .get()
                    .into_iter()
                    .map(|seg| {
                        let opacity: f64 = if seg.kind == "lift" {
                            // Lift type filtering uses difficulty field (stored as lift subtype).
                            if excl_lift.contains(&seg.difficulty) { 0.2 } else { 1.0 }
                        } else {
                            if excl_diff.contains(&seg.difficulty) { 0.2 } else { 1.0 }
                        };
                        let color = segment_color(&seg.kind, &seg.difficulty).to_string();
                        let positions: Vec<Position> = seg
                            .coords
                            .iter()
                            .map(|c| Position::new(c[0], c[1]))
                            .collect();
                        // Wrap static values in Signal::derive for leptos-leaflet 0.9 API.
                        let positions_sig =
                            Signal::derive(move || positions.clone());
                        let color_sig = Signal::derive(move || color.clone());
                        let weight_sig = Signal::derive(|| Some(3.0_f64));
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

            // Route highlight overlay (yellow, weight 6, rendered on top).
            {move || {
                route_coords
                    .get()
                    .into_iter()
                    .map(|coords| {
                        let positions: Vec<Position> =
                            coords.iter().map(|c| Position::new(c[0], c[1])).collect();
                        let positions_sig =
                            Signal::derive(move || positions.clone());
                        let color_sig = Signal::derive(|| "#facc15".to_string());
                        let weight_sig = Signal::derive(|| Some(6.0_f64));
                        let opacity_sig = Signal::derive(|| Some(1.0_f64));
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
        </MapContainer>
    }
}
