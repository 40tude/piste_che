# ARCHITECTURE.md

Version 0.1.0 | Commit 7dd61a8 | 2026-03-23

---

## 1. Project Overview

**Piste Che** is a ski itinerary planner for the Serre Chevalier ski area (French Alps). Personal/portfolio project.

What it delivers:
- Interactive Leaflet map showing pistes and lifts with difficulty/lift-type filters
- Dijkstra shortest-route computation between two named lifts
- Step-by-step itinerary panel with distances and map highlight overlay

![Screenshot](docs/img00.webp)

**Why full-stack Rust:**
1. Learning the Leptos/WASM ecosystem hands-on
2. Shared types between SSR and WASM via `src/models.rs` -- one file, zero duplication
3. Performance and memory safety for the graph-routing pipeline

---

## 2. Data Pipeline

See `resort_generator` sub workspace.

Current file: `data/serre_chevalier_20260319_221219.json`.

`src/routing/data.rs` exposes `find_latest_json()` to auto-select the most recent timestamped file in `data/`. `src/main.rs` calls it at startup -- no manual path update is needed after a data refresh. Future work: integrate `get_data` and `get_elevation` CLIs as Cargo sub-workspaces.

---

## 3. Tech Stack

| Layer | Crate | Version | Role |
|---|---|---|---|
| UI framework | leptos | 0.7 | SSR + WASM hydration |
| Web server | axum | 0.7 | HTTP routing, server functions |
| Async runtime | tokio | 1 | Single-threaded (`current_thread`) |
| Map | leptos-leaflet | 0.9 | Leaflet.js wrapper |
| Map tiles | OpenStreetMap | -- | No API key required |
| Serialization | serde + serde_json | 1 | DTOs, JSON I/O |
| Allocator | mimalloc | 0.1 | Replaces system allocator for throughput |
| Build | cargo-leptos | 0.3.x | Orchestrates SSR + WASM builds |
| Deploy | Heroku | -- | Rust buildpack (Linux binary only) |

**Feature flags** (Cargo.toml):
- `ssr` (default) -- server binary + routing pipeline
- `hydrate` -- WASM client only; routing code stripped from bundle

---

## 4. Architecture

### SSR + WASM Hydration

1. Server renders full HTML and streams it to the browser.
2. Browser loads `piste_che.wasm`; WASM re-attaches reactive Leptos state to the existing DOM.

### Application State

`AppState` (defined in `src/server/mod.rs`) holds nodes, segments, route elements, and the adjacency map. It is built once at startup, wrapped in `Arc<>`, and injected into each request via Leptos context. Cloning is cheap (Arc clone).

### API Surface (`src/server/api.rs`)

| Method | Path | Handler | Response |
|---|---|---|---|
| GET | `/api/get_area` | Axum route in `main.rs` | `AreaResponse` (nodes, segments, dropdown list) |
| POST | `/api/get_area` | Leptos `#[server]` stub | same (used by WASM client) |
| POST | `/api/compute_route` | Leptos `#[server]`, JSON codec | `RouteResponse` (steps, distances, highlight coords) |

`GET /api/get_area` is registered as a plain Axum route so REST clients and integration tests can use the idiomatic HTTP method. The Leptos POST stub calls the same `build_area_response()` helper.

### Module Feature Gating

`src/routing/` is compiled only under `ssr`. The `#[cfg(feature = "ssr")]` guards in `src/server/api.rs` strip all routing logic from the WASM bundle, keeping it small.

---

## 5. Key Design Decisions

### 5.1 Shared Types (`src/models.rs`)

Eight DTOs compiled for both `ssr` and `hydrate`:

| Type | Direction |
|---|---|
| `AreaNode` | server -> client |
| `AreaSegment` | server -> client |
| `SelectableElement` | server -> client |
| `AreaResponse` | server -> client |
| `RouteRequest` | client -> server |
| `RouteStep` | server -> client |
| `HighlightSegment` | server -> client |
| `RouteResponse` | server -> client |

All derive `Serialize + Deserialize`. Leptos `#[server]` handles transport automatically.

### 5.2 Graph Construction -- Repairing Real-World OSM Data

OSM ski data is fragmented: disconnected polylines, GPS noise, named elements split across multiple ways. `build_graph()` in `src/routing/graph.rs` repairs this in 7 steps.

**Tuning constants (all in metres):**

| Constant | Value | Purpose |
|---|---|---|
| `CLUSTER_RADIUS` | 25 | Identity threshold; deduplicates nearby nodes |
| `SPLIT_RADIUS` | 300 | Interior junction detection radius |
| `SPLIT_MAX_ALT` | 100 | Max altitude diff for junction match |
| `CROSSING_RADIUS` | 50 | Piste-to-piste crossing detection |
| `CROSSING_MAX_ALT` | 5 | Max altitude diff for crossing match |
| `TRAVERSE_RADIUS` | 100 | Max distance for synthetic flat traverse edge |
| `TRAVERSE_MAX_ALT` | 5 | Max altitude diff for traverse edge |
| `SKI_OUT_RADIUS` | 100 | Lift-exit to piste ski-out edge radius |
| `SKI_OUT_MAX_ALT` | 10 | Max descent allowed for ski-out edge |
| `SKI_IN_RADIUS` | 100 | Piste to lift-base ski-in edge radius |
| `SKI_IN_MAX_ALT` | 10 | Max ascent allowed for ski-in edge |

**Pipeline:**

1. **Build polylines** -- chain OSM ways by `group_key`, normalize direction: lifts base->summit, pistes summit->base.
2. **Collect endpoint candidates** -- push head/tail of every polyline.
3. **Split detection (3 passes):**
   - Pass A: any piste interior point within `SPLIT_RADIUS`/`SPLIT_MAX_ALT` of an existing candidate becomes a split.
   - Pass C: piste-to-piste interior crossings within `CROSSING_RADIUS`/`CROSSING_MAX_ALT`; both points added.
   - Pass B: merge raw splits within `CLUSTER_RADIUS` to avoid duplicate split nodes.
4. **Cluster nodes** -- deduplicate all candidates within `CLUSTER_RADIUS` -> final `Vec<Node>`.
5. **Build directed segments** -- one segment per consecutive node pair on each polyline. Lifts: endpoint-only (no mid-lift boarding).
6. **Synthetic edges (3 sub-steps):**
   - 6a. Traverse: bidirectional edges between any two nodes within `TRAVERSE_RADIUS`/`TRAVERSE_MAX_ALT`.
   - 6b. Ski-out: directed edge from each lift exit to nearest piste node within `SKI_OUT_RADIUS`. One edge per reachable piste name; excludes other lift nodes.
   - 6c. Ski-in: directed edge from nearest piste node to each lift base within `SKI_IN_RADIUS`. One edge per reachable piste name; excludes other lift nodes.
7. **Build RouteElements** -- one `RouteElement` per named element (entry/exit node pair).

### 5.3 Routing Algorithm (`src/routing/dijkstra.rs`)

Standard Dijkstra with asymmetric edge weights (all in metres):

| Edge kind | Weight |
|---|---|
| `lift` | 50 (fixed, incentivizes uphill travel) |
| `traverse` | 10 x haversine distance (heavy penalty) |
| `piste`, `ski-out`, `ski-in` | haversine distance |

Filters (`excluded_difficulties`, `excluded_lift_types`) are applied at traversal time, not post-filter.

Goal zone is a `HashSet<usize>` of destination nodes (built by `arrival_zone()`). Dijkstra stops at the first settled goal-zone member.

### 5.4 Leptos/WASM Gotchas

**Gotcha 1 -- Single-threaded Tokio** (`src/main.rs:55`)

```rust
#[tokio::main(flavor = "current_thread")]
```

`leptos-leaflet` uses `SendWrapper<T>` internally, which panics when dereferenced from a different thread. `current_thread` ensures the async runtime never spawns work onto another thread.

**Gotcha 2 -- WASM filename mismatch** (`src/main.rs:138-141`)

`cargo-leptos 0.3.x` renames `piste_che_bg.wasm` -> `piste_che.wasm` on disk but the generated JS still requests `piste_che_bg.wasm`. Fixed with an Axum route alias -- no manual file copy needed:

```rust
.route_service(
    "/pkg/piste_che_bg.wasm",
    ServeFile::new("site/pkg/piste_che.wasm"),
)
```

**Gotcha 3 -- Leptos config on Heroku** (`src/main.rs:96`)

```rust
let conf = get_configuration(Some("Cargo.toml")).context("Reading Leptos configuration")?;
```

Using `None` causes a panic on Heroku because the process working directory differs from the dev working directory. Passing `Some("Cargo.toml")` resolves the path relative to the binary location.

---

## 6. File Structure

```
piste_che/
|-- Cargo.toml                  # workspace; [package.metadata.leptos] config
|-- Procfile                    # Heroku: web: ./piste_che
|-- src/
|   |-- main.rs                 # server entry point (ssr only); gotchas 1/2/3
|   |-- lib.rs                  # crate root; re-exports routing module
|   |-- app.rs                  # Leptos App component + router
|   |-- models.rs               # 8 shared DTOs (ssr + hydrate)
|   |-- components/
|   |   |-- mod.rs
|   |   |-- map.rs              # Leaflet map component
|   |   |-- filters.rs          # difficulty/lift-type filter panel
|   |   |-- selector.rs         # start/end dropdown selectors
|   |   |-- itinerary.rs        # step-by-step itinerary panel
|   |   |-- mode_tabs.rs        # routing mode tab bar (Short/Sport/Safe)
|   |   `-- segment_popup.rs    # click-to-inspect popup (name, kind, length, altitude)
|   |-- server/
|   |   |-- mod.rs              # AppState definition
|   |   `-- api.rs              # get_area + compute_route server functions
|   `-- routing/                # ssr-only graph pipeline
|       |-- mod.rs
|       |-- data.rs             # OsmData loader, find_latest_json(), haversine
|       |-- chains.rs           # way-chaining helper for build_polylines
|       |-- graph.rs            # build_graph() 7-step pipeline + tuning constants
|       `-- dijkstra.rs         # Dijkstra + segment_length + arrival_zone
|-- data/
|   `-- serre_chevalier_20260319_221219.json  # current elevation-enriched OSM dump
|-- docs/
|   `-- img00.webp              # app screenshot
|-- specs/
|   `-- 001-map-filters-route/  # feature specs and task lists
|-- style/
|   `-- main.css                # global stylesheet
|-- public/                     # static assets (leaflet.js, leaflet.css, icons)
|-- site/                       # cargo-leptos output (committed for Heroku)
|   `-- pkg/
|       |-- piste_che.js
|       |-- piste_che.wasm      # cargo-leptos rename of piste_che_bg.wasm
|       |-- piste_che.css
|       `-- ...
`-- tests/
    `-- integration/            # integration tests (require running server)
```

---

## 7. Deployment

Heroku's Rust buildpack cannot run `cargo leptos build` (no WASM toolchain). The `site/` bundle must be committed locally and pushed with the source.

**Standard deploy workflow:**

```powershell
# 1. Build WASM bundle and server binary locally
cargo leptos build --release

# 2. Commit the generated assets
git add site/
git commit -m "deploy: rebuild assets"

# 3. Push -- buildpack recompiles server binary for Linux; WASM bundle comes from site/
git push heroku main
```

**Port resolution** (`src/main.rs:42-47`): reads `$env:PORT` (set by Heroku), falls back to `--port` CLI arg (default 3000). Server always binds `0.0.0.0`.

**Local development:**

```powershell
cargo leptos watch
```

Then open `http://localhost:3000`.

---

## 8. Future Work

- Sport and Safe routing modes (mode tabs exist; Dijkstra weights to implement)
- Integrate `get_data` + `get_elevation` as Cargo sub-workspaces
- CI/CD automation for WASM build + Heroku deploy
- `Option<f32>` elevation propagation through pipeline (currently 0.0 sentinel for missing data)

---

## 9. Local Setup

See **README.md** for full prerequisites and build commands.

Quick summary:
- Rust stable 1.85+ (edition 2024)
- `rustup target add wasm32-unknown-unknown`
- `winget install StrawberryPerl.StrawberryPerl` (required by Leptos on Windows)
- `cargo install cargo-leptos`
