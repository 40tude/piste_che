# Quickstart: Map, Filters & Shortest Route

## Prerequisites

- Rust stable 1.85+ (edition 2024 support)
- cargo-leptos: `cargo install cargo-leptos`
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`

## Build

```powershell
# Development (watch mode with hot-reload)
cargo leptos watch

# Release build (single binary + WASM bundle)
cargo leptos build --release
```

## Run

```powershell
# Default port (from Cargo.toml site-addr)
cargo leptos watch

# Custom port via environment variable (takes precedence)
$env:PORT='3000'; cargo leptos watch

# Release binary with CLI flag
./target/release/piste_che --port 3000
```

Open browser at `http://localhost:3000`.

## Test

```powershell
# All tests (unit + integration)
cargo test

# Integration tests only (requires server running)
cargo test --test integration
```

## Project Layout

| Path | Purpose |
|------|---------|
| `src/routing/` | Prototype routing module (graph, Dijkstra, data loader) |
| `src/server/` | Server functions (get_area, compute_route) |
| `src/components/` | Leptos UI components (map, filters, selector, itinerary, tabs) |
| `src/models.rs` | Shared API DTOs |
| `data/*.json` | Ski area data (OSM-derived) |
| `style/main.css` | Application styles |
| `public/` | Static assets (Leaflet JS/CSS) |

## Data Flow

1. **Startup**: `main.rs` loads JSON, calls `build_graph()`, stores result in Axum state
2. **Page load**: Leptos SSR renders HTML, WASM hydrates, calls `get_area()` server function
3. **Map init**: Leaflet initialized on hydrated `<div>`, segments drawn as polylines
4. **Route request**: User selects start/end + filters, calls `compute_route()` server function
5. **Display**: Route highlighted on map, itinerary panel populated with steps

## Heroku Deploy

```powershell
# Procfile already present
heroku create --buildpack emk/rust
git push heroku 001-map-filters-route:main
```
