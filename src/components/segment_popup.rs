// Rust guideline compliant 2026-03-19
//
// Popup overlay shown when the user clicks a piste or lift on the map.
// Displays name, type, difficulty/lift subtype, seat count, duration,
// click coordinates, and interpolated altitude.

use crate::models::AreaSegment;
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Data shown in the segment popup after a map click.
#[derive(Debug, Clone)]
pub struct PopupData {
    pub name: String,
    /// `"lift"` or `"piste"`.
    pub kind: String,
    /// Aerialway sub-type for lifts; piste difficulty for pistes.
    pub difficulty: String,
    /// Seat count per cabin/chair, lifts only.
    pub occupancy: Option<u32>,
    /// Ride duration in minutes, lifts only.
    pub duration_min: Option<u32>,
    /// Latitude of the click point (decimal degrees).
    pub lat: f64,
    /// Longitude of the click point (decimal degrees).
    pub lon: f64,
    /// Altitude interpolated from the two nearest segment coords (metres).
    pub alt_m: f64,
    /// Total length of the segment (sum of haversine distances, metres).
    pub length_m: u32,
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Haversine distance between two WGS-84 points, in metres.
///
/// Uses the mean spherical Earth radius (6 371 000 m).
fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    R * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

/// Project point P onto segment AB; return squared distance and parameter t in [0, 1].
///
/// Uses a flat-earth approximation (degree units) sufficient for short segments (< 10 km).
fn project_point_onto_segment(
    plat: f64,
    plon: f64,
    alat: f64,
    alon: f64,
    blat: f64,
    blon: f64,
) -> (f64, f64) {
    let abx = blon - alon;
    let aby = blat - alat;
    let apx = plon - alon;
    let apy = plat - alat;
    let len2 = abx * abx + aby * aby;
    if len2 < f64::EPSILON {
        // Degenerate zero-length segment.
        let d = haversine(plat, plon, alat, alon);
        return (d * d, 0.0);
    }
    let t = ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0);
    let cx = alon + t * abx;
    let cy = alat + t * aby;
    let d = haversine(plat, plon, cy, cx);
    (d * d, t)
}

/// Find the nearest piste or lift segment to the click point.
///
/// Returns a `PopupData` when any segment is within 30 m of the click.
///
/// # Click threshold
///
/// 30 m is generous enough to catch a click on a 3-6 px wide polyline at
/// zoom 13 while ignoring clicks on empty terrain.
pub fn nearest_segment(lat: f64, lon: f64, segments: &[AreaSegment]) -> Option<PopupData> {
    // 30 m -- generous for 3-6 px polylines at zoom 13.
    const CLICK_THRESHOLD_M: f64 = 30.0;

    let mut best_dist2 = f64::MAX;
    let mut best_t = 0.0_f64;
    let mut best_seg: Option<&AreaSegment> = None;
    let mut best_coord_idx: usize = 0;

    for seg in segments.iter().filter(|s| s.kind == "piste" || s.kind == "lift") {
        let coords = &seg.coords;
        if coords.len() < 2 {
            continue;
        }
        for i in 0..coords.len() - 1 {
            let (alat, alon) = (coords[i][0], coords[i][1]);
            let (blat, blon) = (coords[i + 1][0], coords[i + 1][1]);
            let (d2, t) = project_point_onto_segment(lat, lon, alat, alon, blat, blon);
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_t = t;
                best_seg = Some(seg);
                best_coord_idx = i;
            }
        }
    }

    let seg = best_seg?;
    if best_dist2.sqrt() > CLICK_THRESHOLD_M {
        return None;
    }

    // Linear interpolation of altitude between the two bracketing coords.
    let c0 = &seg.coords[best_coord_idx];
    let c1 = &seg.coords[best_coord_idx + 1];
    let alt_m = c0[2] + best_t * (c1[2] - c0[2]);

    // Total segment length (sum of consecutive haversine distances).
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "length in metres, always positive and < 50 km"
    )]
    let length_m = seg
        .coords
        .windows(2)
        .map(|w| haversine(w[0][0], w[0][1], w[1][0], w[1][1]))
        .sum::<f64>()
        .round() as u32;

    Some(PopupData {
        name: seg.name.clone(),
        kind: seg.kind.clone(),
        difficulty: seg.difficulty.clone(),
        occupancy: seg.occupancy,
        duration_min: seg.duration_min,
        lat,
        lon,
        alt_m,
        length_m,
    })
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

/// Human-readable lift subtype label.
fn lift_label(difficulty: &str) -> &'static str {
    match difficulty {
        "chair_lift" => "Chair lift",
        "gondola" => "Gondola",
        "cable_car" => "Cable car",
        "drag_lift" => "Drag lift",
        "platter" => "Platter",
        "magic_carpet" => "Magic carpet",
        _ => "Lift",
    }
}

/// Human-readable piste difficulty label.
fn difficulty_label(difficulty: &str) -> &'static str {
    match difficulty {
        "novice" => "Novice",
        "easy" => "Easy",
        "intermediate" => "Intermediate",
        "advanced" => "Advanced",
        "freeride" => "Freeride",
        _ => "Unknown",
    }
}

/// CSS color for the piste difficulty dot.
fn difficulty_color(difficulty: &str) -> &'static str {
    match difficulty {
        "novice" => "#22c55e",
        "easy" => "#3b82f6",
        "intermediate" => "#ef4444",
        "advanced" | "freeride" => "#1e293b",
        _ => "#94a3b8",
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Fixed-position popup card shown after a map click on a piste or lift.
///
/// Renders nothing when `info` is `None`.  The user dismisses the card by
/// clicking the X button, which sets `info` back to `None`.
#[component]
pub fn SegmentPopup(info: RwSignal<Option<PopupData>>) -> impl IntoView {
    view! {
        {move || {
            info.get().map(|data| {
                let kind = data.kind.clone();
                let is_lift = kind == "lift";

                let detail = if is_lift {
                    let label = lift_label(&data.difficulty);
                    let occ = data.occupancy.map_or(String::new(), |n| format!(" - {n}p"));
                    let dur = data.duration_min.map_or(String::new(), |m| format!(" - {m} min"));
                    format!("{label}{occ}{dur}")
                } else {
                    difficulty_label(&data.difficulty).to_string()
                };

                let dot_color = if is_lift {
                    String::new()
                } else {
                    difficulty_color(&data.difficulty).to_string()
                };

                let badge_class = if is_lift {
                    "popup-badge popup-badge-lift"
                } else {
                    "popup-badge popup-badge-piste"
                };
                let badge_text = if is_lift { "Lift" } else { "Piste" };

                view! {
                    <div class="segment-popup">
                        <div class="popup-header">
                            <span class=badge_class>{badge_text}</span>
                            <span class="popup-name">{data.name.clone()}</span>
                            <button
                                class="popup-close"
                                on:click=move |_| info.set(None)
                            >
                                "\u{00D7}"
                            </button>
                        </div>
                        <div class="popup-detail">
                            {if !is_lift {
                                view! {
                                    <span
                                        class="popup-dot"
                                        style=format!("background:{dot_color}")
                                    />
                                }.into_any()
                            } else {
                                view! { <span/> }.into_any()
                            }}
                            <span>{detail}</span>
                        </div>
                        <div class="popup-coords">
                            {format!("{:.4}\u{00B0} N  {:.4}\u{00B0} E", data.lat, data.lon)}
                        </div>
                        <div class="popup-alt">
                            {format!("alt: {} m", data.alt_m.round() as i32)}
                        </div>
                        <div class="popup-alt">
                            {format!("len: {} m", data.length_m)}
                        </div>
                    </div>
                }
            })
        }}
    }
}
