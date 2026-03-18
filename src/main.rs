// Rust guideline compliant 2026-02-16
//
// Server binary entry point -- compiled only when the `ssr` feature is active
// (cargo-leptos sets `bin-features = ["ssr"]`).

use mimalloc::MiMalloc;

/// Use mimalloc as the global allocator for improved allocation performance.
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use anyhow::{Context, Result};
use axum::Router;
use clap::Parser;
use leptos::config::get_configuration;
use leptos::prelude::provide_context;
use leptos_axum::{generate_route_list, LeptosRoutes};
use piste_che::{
    app::App,
    routing::{adjacency_from_segments, build_graph, OsmData},
    server::AppState,
};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(about = "Piste Che -- Serre Chevalier ski itinerary web server")]
struct Cli {
    /// TCP port to listen on.  `PORT` env var takes precedence (Heroku convention).
    #[arg(long, default_value_t = 3000)]
    port: u16,
}

/// Returns the effective port.
///
/// The `PORT` environment variable (set by Heroku) overrides the CLI flag.
fn resolve_port(cli: &Cli) -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(cli.port)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Structured logging: level from RUST_LOG env var, default "info".
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let port = resolve_port(&cli);

    // Load the bundled ski area data file.
    // Path is relative to the working directory (project root when running
    // with `cargo leptos watch` or from the release binary).
    let data_path = std::path::Path::new("data/20260315_164849_ele.json");
    let osm = OsmData::load(data_path)
        .with_context(|| format!("Loading {}", data_path.display()))?;

    let (nodes, segments, route_elements) = build_graph(&osm);
    let adjacency = adjacency_from_segments(&segments);

    tracing::event!(
        name: "app.startup.graph_loaded",
        tracing::Level::INFO,
        nodes.count = nodes.len(),
        segments.count = segments.len(),
        route_elements.count = route_elements.len(),
        "graph loaded: {{nodes.count}} nodes, {{segments.count}} segments",
    );

    let state = Arc::new(AppState {
        nodes,
        segments,
        route_elements,
        adjacency,
    });

    // Leptos configuration from Cargo.toml `[package.metadata.leptos]`.
    let conf = get_configuration(None).context("Reading Leptos configuration")?;
    let mut leptos_options = conf.leptos_options;

    // Override the configured address with the resolved port so `PORT` env var
    // and `--port` CLI flag both work at runtime.
    leptos_options.site_addr.set_port(port);
    let addr = leptos_options.site_addr;

    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let state = Arc::clone(&state);
                move || provide_context(Arc::clone(&state))
            },
            App,
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    tracing::event!(
        name: "app.startup.listening",
        tracing::Level::INFO,
        server.address = %addr,
        "server listening on {{server.address}}",
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Binding to {addr}"))?;

    axum::serve(listener, app).await.context("Serving")?;

    Ok(())
}

// ---------------------------------------------------------------------------
// SSR shell
// ---------------------------------------------------------------------------

/// Returns the initial HTML document shell for server-side rendering.
///
/// `file_and_error_handler` streams this view to HTML internally;
/// returning `IntoView` (not `IntoResponse`) is required by its signature.
fn shell(options: leptos::config::LeptosOptions) -> impl leptos::prelude::IntoView {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="stylesheet" href="/leaflet.css"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
