// Rust guideline compliant 2026-02-16
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Root structure of the Overpass API JSON response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverpassResponse {
    pub version: f64,
    pub generator: String,
    pub osm3s: Osm3s,
    pub elements: Vec<Element>,
}

/// Overpass API metadata block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Osm3s {
    pub timestamp_osm_base: String,
    pub copyright: String,
}

/// A tagged OSM element: either a Way or a Node.
///
/// Discriminated by the `"type"` field in the JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Way(Way),
    Node(Node),
}

/// An OSM Way: ordered list of node references plus tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Way {
    pub id: u64,
    pub nodes: Vec<u64>,
    pub tags: HashMap<String, String>,
}

/// An OSM Node: geographic point with optional elevation.
///
/// The `ele` field is absent before elevation enrichment and present after.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    /// Elevation in meters; None before enrichment, Some after.
    /// IGN sentinel value -99999 is treated as a valid elevation value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ele: Option<f32>,
}

/// IGN Altimetrie API response.
///
/// Elevations are returned in the same order as the request coordinates.
#[derive(Debug, Clone, Deserialize)]
pub struct ElevationResponse {
    pub elevations: Vec<f64>,
}

/// Runtime configuration derived from CLI arguments.
#[derive(Debug, Clone)]
pub struct ResortConfig {
    /// Human-readable resort name, passed directly to the Overpass area query.
    pub resort_name: String,
    /// Lowercase slug derived from `resort_name`; spaces replaced with underscores.
    pub filename_slug: String,
    /// Output directory; hardcoded to `data/` relative to the working directory.
    pub output_dir: PathBuf,
}

impl ResortConfig {
    /// Create a `ResortConfig` from a human-readable resort name.
    ///
    /// The slug is derived by lowercasing and replacing spaces with underscores.
    ///
    /// # Examples
    ///
    /// ```
    /// # use resort_generator::types::ResortConfig;
    /// let cfg = ResortConfig::from_name("Serre Chevalier");
    /// assert_eq!(cfg.resort_name, "Serre Chevalier");
    /// assert_eq!(cfg.filename_slug, "serre_chevalier");
    /// ```
    pub fn from_name(name: &str) -> Self {
        Self {
            resort_name: name.to_owned(),
            // Lowercase and replace spaces with underscores per FR-001 slug rules.
            filename_slug: name.to_lowercase().replace(' ', "_"),
            // Hardcoded relative output dir per spec assumptions.
            output_dir: PathBuf::from("data"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resort_config_serre_chevalier() {
        let cfg = ResortConfig::from_name("Serre Chevalier");
        assert_eq!(cfg.resort_name, "Serre Chevalier");
        assert_eq!(cfg.filename_slug, "serre_chevalier");
    }

    #[test]
    fn resort_config_mont_blanc_2000() {
        let cfg = ResortConfig::from_name("Mont Blanc 2000");
        assert_eq!(cfg.filename_slug, "mont_blanc_2000");
    }
}
