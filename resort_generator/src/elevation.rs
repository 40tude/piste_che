// Rust guideline compliant 2026-02-16
use anyhow::{Context, Result, bail};
use tracing::event;

use crate::http::HttpClient;
use crate::types::ElevationResponse;

/// IGN Altimetrie REST endpoint.
const IGN_ENDPOINT: &str =
    "https://data.geopf.fr/altimetrie/1.0/calcul/alti/rest/elevation.json";

/// IGN resource identifier for the worldwide elevation model.
const IGN_RESOURCE: &str = "ign_rge_alti_wld";

/// Maximum coordinates per IGN request (URL length constraint, research.md R2).
const BATCH_SIZE: usize = 50;

/// Inter-batch delay in milliseconds to respect IGN rate limits (research.md R2).
const INTER_BATCH_DELAY_MS: u64 = 200;

/// Exponential backoff delays in seconds: 2 s, 4 s, 8 s (research.md R2).
const BACKOFF_DELAYS_SECS: [u64; 3] = [2, 4, 8];

/// Fetch elevation for a list of nodes from the IGN Altimetrie API.
///
/// Nodes are chunked into batches of 50. Each batch is retried up to 3 times
/// with exponential backoff (2 s, 4 s, 8 s). A 200 ms delay is applied between
/// successful batches to respect IGN rate limits.
///
/// Returns a `Vec<(node_id, elevation_f32)>` in the same order as the input.
///
/// The IGN sentinel value -99999 is treated as a valid elevation value.
///
/// # Errors
///
/// Returns an error if any batch fails after all retries, or if the API returns
/// an unexpected number of elevation values for a batch.
pub async fn fetch_elevation(
    client: &impl HttpClient,
    nodes: Vec<(u64, f64, f64)>,
) -> Result<Vec<(u64, f32)>> {
    let total_nodes = nodes.len();
    let batches: Vec<_> = nodes.chunks(BATCH_SIZE).collect();
    let total_batches = batches.len();
    let mut result: Vec<(u64, f32)> = Vec::with_capacity(total_nodes);

    for (batch_idx, batch) in batches.iter().enumerate() {
        let batch_num = batch_idx + 1;
        event!(
            name: "elevation.batch.start",
            tracing::Level::INFO,
            batch_num,
            total_batches,
            node_count = batch.len(),
            "fetching elevation batch {{batch_num}}/{{total_batches}} ({{node_count}} nodes)"
        );

        let lats: String = batch.iter().map(|(_, lat, _)| lat.to_string()).collect::<Vec<_>>().join("|");
        let lons: String = batch.iter().map(|(_, _, lon)| lon.to_string()).collect::<Vec<_>>().join("|");

        let url = format!(
            "{IGN_ENDPOINT}?lon={lons}&lat={lats}&resource={IGN_RESOURCE}&zonly=true"
        );

        let mut last_err = None;
        let mut response_text: Option<String> = None;

        for (attempt, &delay_secs) in BACKOFF_DELAYS_SECS.iter().enumerate() {
            match client.get(url.clone()).await {
                Ok(text) => {
                    response_text = Some(text);
                    break;
                }
                Err(err) => {
                    event!(
                        name: "elevation.batch.retry",
                        tracing::Level::WARN,
                        batch_num,
                        total_batches,
                        attempt = attempt + 1,
                        delay_secs,
                        error = %err,
                        "batch {{batch_num}} attempt {{attempt}} failed, retrying in {{delay_secs}}s"
                    );
                    last_err = Some(err);
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                }
            }
        }

        let text = response_text.ok_or_else(|| {
            last_err.take().unwrap_or_else(|| anyhow::anyhow!("unknown error"))
        }).with_context(|| {
            format!("IGN elevation fetch failed for batch {batch_num}/{total_batches} (3 retries exhausted)")
        })?;

        let elevation_response: ElevationResponse =
            serde_json::from_str(&text).with_context(|| {
                format!("failed to deserialize IGN response for batch {batch_num}/{total_batches}")
            })?;

        if elevation_response.elevations.len() != batch.len() {
            bail!(
                "IGN returned {} elevations for batch {batch_num}/{total_batches}, expected {}",
                elevation_response.elevations.len(),
                batch.len()
            );
        }

        for ((node_id, _, _), &ele) in batch.iter().zip(elevation_response.elevations.iter()) {
            #[expect(clippy::cast_possible_truncation, reason = "f64->f32 precision loss is acceptable for elevation meters")]
            result.push((*node_id, ele as f32));
        }

        // Apply inter-batch delay except after the last batch.
        if batch_idx + 1 < total_batches {
            tokio::time::sleep(tokio::time::Duration::from_millis(INTER_BATCH_DELAY_MS)).await;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::MockHttpClient;

    #[tokio::test]
    async fn batching_110_nodes_calls_mock_3_times() {
        // 110 nodes -> 3 batches: 50 + 50 + 10
        let nodes: Vec<(u64, f64, f64)> =
            (0..110u64).map(|i| (i, i as f64 * 0.001, i as f64 * 0.001)).collect();

        let mut mock = MockHttpClient::new();

        // First batch (50 nodes)
        mock.expect_get().times(1).returning(|_url| {
            let body = serde_json::json!({ "elevations": vec![1000.0f64; 50] }).to_string();
            Ok(body)
        });

        // Second batch (50 nodes)
        mock.expect_get().times(1).returning(|_url| {
            let body = serde_json::json!({ "elevations": vec![1000.0f64; 50] }).to_string();
            Ok(body)
        });

        // Third batch (10 nodes)
        mock.expect_get().times(1).returning(|_url| {
            let body = serde_json::json!({ "elevations": vec![1000.0f64; 10] }).to_string();
            Ok(body)
        });

        let result = fetch_elevation(&mock, nodes).await.expect("fetch_elevation failed");
        assert_eq!(result.len(), 110);
    }
}
