// Rust guideline compliant 2026-02-16
use anyhow::{Context, Result, bail};
use tracing::event;

use crate::http::HttpClient;
use crate::types::OverpassResponse;

/// Overpass API endpoint.
///
/// Public instance at overpass-api.de; no authentication required per spec assumptions.
const OVERPASS_ENDPOINT: &str = "https://overpass-api.de/api/interpreter";

/// Build an Overpass QL query that returns piste and aerialway ways plus all
/// their nodes for the given resort name.
///
/// The resort name is injected verbatim into the area filter. The
/// `landuse=winter_sports` tag scopes the query to ski areas per research.md R1.
pub fn build_query(resort_name: &str) -> String {
    format!(
        "[out:json][timeout:25];\narea[\"name\"=\"{resort_name}\"][\"landuse\"=\"winter_sports\"]->.a;\n(\n  way[\"piste:type\"=\"downhill\"](area.a);\n  way[\"aerialway\"](area.a);\n);\nout body;\n>;\nout skel qt;"
    )
}

/// Fetch trail geometry from the Overpass API for a given query.
///
/// Validates that the response contains at least one Way and one Node.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the response cannot be
/// deserialized, or the response contains no trails or no nodes.
pub async fn fetch_trails(client: &impl HttpClient, query: String) -> Result<OverpassResponse> {
    event!(
        name: "overpass.fetch.start",
        tracing::Level::INFO,
        "fetching trail geometry from Overpass API"
    );

    let body = client
        .post(OVERPASS_ENDPOINT.to_owned(), query)
        .await
        .context("Overpass API request failed")?;

    let response: OverpassResponse =
        serde_json::from_str(&body).context("failed to deserialize Overpass response")?;

    let way_count = response.elements.iter().filter(|e| matches!(e, crate::types::Element::Way(_))).count();
    let node_count = response.elements.iter().filter(|e| matches!(e, crate::types::Element::Node(_))).count();

    if way_count == 0 {
        bail!("no trails found: Overpass returned zero Way elements");
    }
    if node_count == 0 {
        bail!("no nodes found: Overpass returned zero Node elements");
    }

    event!(
        name: "overpass.fetch.success",
        tracing::Level::INFO,
        way_count,
        node_count,
        "fetched {{way_count}} ways and {{node_count}} nodes"
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_query_contains_resort_name_and_landuse() {
        let query = build_query("Serre Chevalier");
        assert!(query.contains("\"name\"=\"Serre Chevalier\""));
        assert!(query.contains("landuse\"=\"winter_sports\""));
    }
}
