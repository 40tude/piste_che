// Rust guideline compliant 2026-03-19
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Raw OSM deserialization types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OsmRoot {
    pub elements: Vec<RawElement>,
}

/// One element from an Overpass JSON response.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RawElement {
    Way(RawWay),
    Node(RawNode),
    /// Catches "relation" and any future element types.
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct RawWay {
    pub id: u64,
    #[serde(default)]
    pub nodes: Vec<u64>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawNode {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    /// Elevation in metres, present only in `_ele.json` enriched files.
    #[serde(default)]
    pub ele: Option<f32>,
}

// ---------------------------------------------------------------------------
// Parsed dataset
// ---------------------------------------------------------------------------

/// All ways and a lookup table of node coordinates from one Overpass dump.
#[derive(Debug)]
pub struct OsmData {
    pub ways: Vec<RawWay>,
    /// Node coordinates indexed by OSM node ID.
    pub nodes: HashMap<u64, RawNode>,
}

impl OsmData {
    /// Load and parse an Overpass JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Reading {}", path.display()))?;
        let root: OsmRoot = serde_json::from_str(&content).context("Parsing JSON")?;

        let mut ways = Vec::new();
        let mut nodes = HashMap::new();

        for element in root.elements {
            match element {
                RawElement::Way(way) => ways.push(way),
                RawElement::Node(node) => {
                    nodes.insert(node.id, node);
                }
                RawElement::Other => {}
            }
        }

        Ok(Self { ways, nodes })
    }
}

// ---------------------------------------------------------------------------
// Helpers on RawWay
// ---------------------------------------------------------------------------

impl RawWay {
    /// Returns the display name of this way.
    ///
    /// Reads the `name` tag first; falls back to `piste:name` for ways that
    /// use the piste-specific tag instead of (or in addition to) the generic one.
    pub fn name(&self) -> Option<&str> {
        self.tags
            .get("name")
            .or_else(|| self.tags.get("piste:name"))
            .map(String::as_str)
    }

    /// Returns the grouping key used to aggregate ways into a named element.
    ///
    /// Two ways belong to the same named element only when their group keys match.
    ///
    /// # Why a composite key for lifts?
    ///
    /// OSM sometimes assigns the same `name` to two physically distinct lift
    /// installations at the same resort (e.g. "Eychauda" for both the 6-seat
    /// `chair_lift` and a platter serving the same area).  Using `name` alone
    /// would merge them into a single element with mixed ways -- wrong topology.
    ///
    /// To separate them we combine:
    ///   - `aerialway`           -- distinguishes `chair_lift` from platter, etc.
    ///   - `aerialway:occupancy` -- distinguishes a 4-seat from a 6-seat chair
    ///     (omitted from the key when absent).
    ///
    /// `aerialway:capacity` (throughput in persons/hour) is intentionally
    /// excluded: it is an operational figure that can differ between segments
    /// of the same installation, which would spuriously split a single lift.
    ///
    /// Pistes are grouped by name only: difficulty can legitimately vary along
    /// a route, so it must not split the group.
    pub fn group_key(&self) -> Option<String> {
        let name = self.name()?;

        if let Some(aerialway) = self.tags.get("aerialway") {
            // Lift: append type and, when present, occupancy.
            let mut key = format!("{name} [{aerialway}");
            if let Some(occ) = self.tags.get("aerialway:occupancy") {
                use std::fmt::Write as _;
                let _: Result<(), _> = write!(key, " {occ}p");
            }
            key.push(']');
            Some(key)
        } else {
            // Piste (or unknown kind): name alone.
            Some(name.to_string())
        }
    }

    /// Returns `true` when this way is a closed polygon (first node == last node).
    ///
    /// In OSM, a piste is sometimes mapped twice: as an open linear way for
    /// routing and as a closed polygon to draw its visual footprint (area=yes).
    /// Closed polygons must be excluded from topology analysis because they
    /// have no distinct start/end and cannot be directed arcs in a graph.
    pub fn is_closed_polygon(&self) -> bool {
        self.nodes.len() > 1 && self.nodes.first() == self.nodes.last()
    }

    /// `"lift"` for passenger aerialways, `"piste"` for downhill/nordic pistes, `"?"` otherwise.
    ///
    /// Not every `aerialway` tag represents a ski lift.  OSM also uses this tag
    /// for infrastructure that carries goods or equipment rather than skiers:
    ///
    /// - `aerialway=goods`        -- freight cable (e.g. CATEX avalanche-control launchers)
    /// - `aerialway=construction` -- cable under construction, not yet in service
    ///
    /// These must be excluded from the "lift" category so they do not appear in
    /// lift lists or lift maps.  All other aerialway values (`chair_lift`, gondola,
    /// `drag_lift`, platter, `magic_carpet`, etc.) are genuine passenger lifts.
    pub fn element_kind(&self) -> &str {
        // Aerialway values that carry goods or are not yet in service, not skiers.
        const NON_PASSENGER_AERIALWAY: &[&str] = &["goods", "construction"];

        if let Some(aerialway) = self.tags.get("aerialway") {
            if NON_PASSENGER_AERIALWAY.contains(&aerialway.as_str()) {
                "?"
            } else {
                "lift"
            }
        } else if self.tags.contains_key("piste:type") {
            "piste"
        } else {
            "?"
        }
    }

    /// For lifts: the aerialway sub-type.  For pistes: `piste:difficulty`.
    pub fn difficulty(&self) -> &str {
        if self.tags.contains_key("aerialway") {
            self.tags.get("aerialway").map_or("?", String::as_str)
        } else {
            self.tags
                .get("piste:difficulty")
                .map_or("-", String::as_str)
        }
    }

    /// Seat count per cabin/chair from `aerialway:occupancy`, lifts only.
    ///
    /// Returns `None` when the tag is absent or cannot be parsed as an integer.
    pub fn occupancy(&self) -> Option<u32> {
        self.tags
            .get("aerialway:occupancy")
            .and_then(|v| v.trim().parse().ok())
    }

    /// Ride duration in minutes from `aerialway:duration`, lifts only.
    ///
    /// Handles both plain integers ("8") and ISO 8601 duration strings ("PT8M").
    /// Returns `None` when the tag is absent or the format is not recognized.
    pub fn duration_min(&self) -> Option<u32> {
        let raw = self.tags.get("aerialway:duration")?;
        let s = raw.trim();
        // ISO 8601 subset: "PT<n>M" (minutes only) or "PT<n>H<m>M".
        if let Some(inner) = s.strip_prefix("PT") {
            // Try "PT<n>M" first.
            if let Some(mins) = inner.strip_suffix('M') {
                if let Ok(m) = mins.parse::<u32>() {
                    return Some(m);
                }
            }
            // Try "PT<h>H<m>M".
            if let Some(h_pos) = inner.find('H') {
                let hours: u32 = inner[..h_pos].parse().ok()?;
                let rest = &inner[h_pos + 1..];
                let mins: u32 = rest.strip_suffix('M').and_then(|m| m.parse().ok()).unwrap_or(0);
                return Some(hours * 60 + mins);
            }
            return None;
        }
        // Plain integer: treat as minutes.
        s.parse().ok()
    }
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Haversine distance between two WGS-84 coordinates, in metres.
///
/// Uses the mean spherical Earth radius (6 371 000 m).
pub fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    // Earth mean spherical radius in metres (WGS-84 approximation)
    const R: f64 = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    R * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

// ---------------------------------------------------------------------------
// Input-file discovery
// ---------------------------------------------------------------------------

/// Returns the path of the most recent timestamped JSON file in `dir`,
/// excluding `request.json`.
pub fn find_latest_json(dir: &Path) -> Result<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("Reading directory {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n != "request.json")
        })
        .collect();

    files.sort();
    files.pop().context("No data JSON files found in data/")
}
