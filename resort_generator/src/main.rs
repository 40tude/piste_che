// cargo run -p resort_generator -- --resort "Serre Chevalier"
// Rust guideline compliant 2026-02-16
use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::Parser;
use mimalloc::MiMalloc;
use tracing::event;

mod elevation;
mod http;
mod merge;
mod overpass;
mod types;

use http::ReqwestClient;
use types::{Element, ResortConfig};

// Use mimalloc as the global allocator for improved allocation performance
// (see application guidelines M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// CLI arguments for the resort data generator.
#[derive(Debug, Parser)]
#[command(
    name = "resort_generator",
    about = "Fetch and merge trail + elevation data for a ski resort"
)]
struct Args {
    /// Human-readable resort name (e.g., "Serre Chevalier").
    #[arg(long, required = true)]
    resort: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if let Err(err) = run(args).await {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    let config = ResortConfig::from_name(&args.resort);
    let client = ReqwestClient::new().context("failed to build HTTP client")?;

    // Stage 1: fetch trail geometry from Overpass.
    event!(
        name: "pipeline.fetch_trails.start",
        tracing::Level::INFO,
        resort = config.resort_name,
        "fetching trails for resort {{resort}}"
    );
    let query = overpass::build_query(&config.resort_name);
    let mut osm = overpass::fetch_trails(&client, query)
        .await
        .with_context(|| {
            format!(
                "Overpass API returned no trails for resort \"{}\"",
                config.resort_name
            )
        })?;

    // Stage 2: extract node coordinates.
    event!(
        name: "pipeline.extract_nodes.start",
        tracing::Level::INFO,
        "extracting node coordinates"
    );
    let node_coords: Vec<(u64, f64, f64)> = osm
        .elements
        .iter()
        .filter_map(|e| {
            if let Element::Node(n) = e {
                Some((n.id, n.lat, n.lon))
            } else {
                None
            }
        })
        .collect();
    event!(
        name: "pipeline.extract_nodes.done",
        tracing::Level::INFO,
        count = node_coords.len(),
        "extracted {{count}} node coordinates"
    );

    // Stage 3: fetch elevation data from IGN.
    event!(
        name: "pipeline.fetch_elevation.start",
        tracing::Level::INFO,
        "fetching elevation data from IGN"
    );
    let elevations = elevation::fetch_elevation(&client, node_coords)
        .await
        .context("IGN elevation fetch failed")?;
    event!(
        name: "pipeline.fetch_elevation.done",
        tracing::Level::INFO,
        count = elevations.len(),
        "received {{count}} elevation values"
    );

    // Stage 4: merge elevation into the OSM response.
    event!(name: "pipeline.merge.start", tracing::Level::INFO, "merging elevation data");
    merge::merge_elevation(&mut osm, elevations);

    // Stage 5: validate that all way-referenced nodes have elevation.
    event!(name: "pipeline.validate.start", tracing::Level::INFO, "validating elevation completeness");
    merge::validate_completeness(&osm).context("elevation completeness validation failed")?;

    // Stage 6: write output file.
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{timestamp}.json", config.filename_slug);
    let output_path = config.output_dir.join(&filename);

    // Abort if the output file already exists (FR-007: MUST NOT overwrite).
    if output_path.exists() {
        bail!("output file already exists: {}", output_path.display());
    }

    std::fs::create_dir_all(&config.output_dir).with_context(|| {
        format!(
            "failed to create output directory: {}",
            config.output_dir.display()
        )
    })?;

    event!(
        name: "pipeline.write.start",
        tracing::Level::INFO,
        file.path = %output_path.display(),
        "writing output file to {{file.path}}"
    );

    let json = serde_json::to_string_pretty(&osm).context("failed to serialize OSM data")?;
    std::fs::write(&output_path, json)
        .with_context(|| format!("failed to write output file: {}", output_path.display()))?;

    event!(
        name: "pipeline.write.success",
        tracing::Level::INFO,
        file.path = %output_path.display(),
        "output written to {{file.path}}"
    );

    // Print the output path to stdout as specified in contracts/cli.md.
    println!("{}", output_path.display());

    Ok(())
}
