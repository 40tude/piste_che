# Research: Map, Filters & Shortest Route

## R1: Leptos Rendering Mode (CSR vs SSR)

**Decision**: Leptos SSR with Axum via cargo-leptos

**Rationale**: SSR produces a single binary (server + WASM client), simplifies Heroku deployment (one Procfile entry), and is the idiomatic Leptos+Axum integration path. Server functions bridge client/server with zero boilerplate API definitions.

**Alternatives considered**:
- CSR with trunk: Simpler mental model, but requires two build steps, manual static file serving, and CORS setup. No single binary.
- CSR embedded in Axum: Possible via include_bytes!, but cargo-leptos handles this automatically for SSR.

**Key constraint**: The map (Leaflet) is 100% client-side. SSR renders the shell HTML; Leaflet initializes only after WASM hydration via `create_effect`. This is a well-supported pattern in Leptos.

## R2: Leptos + Leaflet.js Integration

**Decision**: Primary: `leptos-leaflet` crate for declarative map components. Fallback: thin JS glue module for anything the crate does not cover.

**Rationale**: The `leptos-leaflet` crate (by Headless Studio, `headless-studio/leptos-leaflet` on GitHub) provides declarative Leptos components wrapping Leaflet.js: `<MapContainer>`, `<TileLayer>`, `<Polyline>`, `<Polygon>`, `<Marker>`, `<Popup>`. Polyline supports reactive `positions`, `color`, `weight`, `opacity` props -- exactly what we need for ski segments and route highlighting. The crate handles SSR safety internally (no-ops on server).

**Primary pattern (leptos-leaflet)**:

```rust
use leptos::*;
use leptos_leaflet::*;

#[component]
pub fn SkiMap(
    segments: Signal<Vec<AreaSegment>>,
    route_coords: Signal<Vec<Vec<Position>>>,
) -> impl IntoView {
    view! {
        <MapContainer
            style="height:100vh;width:100%;"
            center=Position::new(44.9403, 6.5063)
            zoom=13.0
            set_view=true
        >
            <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"/>
            // Ski segments with difficulty-based colors
            <For
                each=move || segments.get()
                key=|seg| seg.id
                children=move |seg| {
                    view! {
                        <Polyline
                            positions=seg.positions()
                            color=seg.color()
                            weight=3.0
                            opacity=seg.opacity()
                        />
                    }
                }
            />
            // Route highlight overlay
            <For
                each=move || route_coords.get()
                key=|coords| coords.len()
                children=move |coords| {
                    view! { <Polyline positions=coords color="yellow" weight=6.0/> }
                }
            />
        </MapContainer>
    }
}
```

**Fallback pattern (JS glue)**: If `leptos-leaflet` lacks support for dimming/opacity changes or other advanced styling, a thin `public/js/map.js` module called via `#[wasm_bindgen(module = "/public/js/map.js")]` can handle those specific operations.

**HTML setup**: Leaflet CSS/JS loaded in `index.html` (before hydration):
```html
<link rel="stylesheet" href="/leaflet.css" />
<script src="/leaflet.js"></script>
```

**Alternatives considered**:
- JS glue only (no leptos-leaflet): More boilerplate, manual reactivity wiring. Reserve for fallback.
- Direct web-sys DOM manipulation: Verbose and unergonomic for Leaflet's object-oriented API.
- Full wasm-bindgen bindings for Leaflet: Too much surface area for MVP.

## R3: cargo-leptos Project Setup

**Decision**: Single crate with `ssr`/`hydrate` feature flags, built by cargo-leptos 0.2+

**Rationale**: cargo-leptos is the official build tool for Leptos SSR projects. It compiles the server binary (with `ssr` feature) and WASM client (with `hydrate` feature) from the same crate in one command.

**Cargo.toml structure**:

```toml
[package]
name = "piste_che"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = { version = "0.7", features = [] }
leptos_meta = { version = "0.7" }
leptos_router = { version = "0.7" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen = "0.2"
# ... other shared deps

[dependencies.leptos_axum]
version = "0.7"
optional = true

[features]
hydrate = ["leptos/hydrate"]
ssr = [
    "dep:leptos_axum",
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
]

[package.metadata.leptos]
output-name = "piste_che"
site-root = "target/site"
site-pkg-dir = "pkg"
style-file = "style/main.css"
assets-dir = "public"
site-addr = "127.0.0.1:3000"
reload-port = 3001
end2end-cmd = "cargo test --test integration"
```

**Build commands**:
- `cargo leptos watch` -- dev mode with hot-reload
- `cargo leptos build --release` -- production build
- Output: `target/server/release/piste_che` (binary) + `target/site/` (static assets)

**Alternatives considered**:
- Cargo workspace (server + frontend crates): More separation but double the boilerplate. cargo-leptos handles the dual-target compilation seamlessly.
- trunk (CSR only): No SSR, no server functions, requires separate Axum binary.

## R4: Server Function State Access

**Decision**: `leptos_routes_with_context` + `provide_context` / `use_context`

**Rationale**: The graph (built at startup from JSON) is large and readonly. It lives in `Arc<AppState>`. Using `leptos_routes_with_context` injects it into Leptos's context system for every request. Server functions retrieve it with `use_context()`. This is the most Leptos-idiomatic pattern and avoids direct Axum extractor coupling.

**Pattern**:

```rust
// AppState holds the precomputed graph
struct AppState {
    nodes: Vec<Node>,
    segments: Vec<Segment>,
    route_elements: Vec<RouteElement>,
    adjacency: HashMap<usize, Vec<usize>>,
}

// In main.rs (ssr feature) -- provide context per request
let state = Arc::new(AppState { /* from build_graph() */ });
let app = Router::new()
    .leptos_routes_with_context(
        &leptos_options,
        routes,
        {
            let state = state.clone();
            move || provide_context(state.clone())  // Arc clone per request (cheap)
        },
        App,
    )
    .with_state(leptos_options);

// In server function -- retrieve via use_context
#[server(input = GetUrl, output = Json, prefix = "/api")]
pub async fn get_area() -> Result<AreaResponse, ServerFnError> {
    let state = use_context::<Arc<AppState>>()
        .ok_or_else(|| ServerFnError::new("AppState not provided"))?;
    // Use state.nodes, state.segments, etc.
    Ok(AreaResponse { /* ... */ })
}
```

**Key points**:
- `Arc<AppState>` is cheaply cloneable (just reference count bump)
- State is readonly after startup -- no Mutex/RwLock needed
- `use_context()` works in any server function without imports from axum
- Server functions default to `/api/` prefix (matches constitution)
- Encoding options: `GetUrl` for read queries, `PostUrl`/`Json` for mutations

**Server function encoding options**:
| Input | Method | Use case |
|-------|--------|----------|
| `GetUrl` | GET | `get_area` (cacheable, idempotent) |
| `PostUrl` | POST | `compute_route` (has filter params) |
| `Json` | POST | Alternative for complex request bodies |

**Alternatives considered**:
- `leptos_axum::extract()`: Works but couples server functions to Axum extractors. Less portable.
- Global static (OnceCell/LazyLock): Simpler but harder to test and doesn't follow dependency injection.

## R5: Prototype Module Integration

**Decision**: Copy 4 source files into `src/routing/`, extract Dijkstra into `dijkstra.rs`

**Rationale**: The prototype is in a separate workspace (`020_serre_che_1/get_itinerary`). Constitution Principle I mandates integration "as-is" without rewriting. We copy the 4 files and make minimal changes:

1. `data.rs` -- as-is (OsmData, JSON parsing, haversine)
2. `graph.rs` -- as-is (Node, Segment, RouteElement, build_graph)
3. `chains.rs` -- as-is (chain building)
4. `dijkstra.rs` -- extracted from `main.rs` (dijkstra function + segment_length)

**Changes required**:
- `mod.rs` re-exports public types and functions
- `#[cfg(feature = "ssr")]` gate on the routing module (server-only)
- Remove `mimalloc` global allocator (not needed in web server context, optional optimization)
- Add `pub` visibility where needed for server function access
- No logic changes

**Existing tests**: The prototype has no formal test files (main.rs is a CLI binary). Constitution says "existing tests MUST be preserved" -- since there are none, this is trivially satisfied. New tests will be written per TDD (Principle III).

## R6: Difficulty and Lift Type Mapping

**Decision**: Map OSM values to UI labels at the API boundary (in DTOs)

**Rationale**: The prototype uses raw OSM values internally ("novice", "easy", "intermediate", "advanced" for difficulties; "chair_lift", "gondola", "platter", etc. for lift types). The UI needs user-friendly labels and color codes. Mapping happens in the server function response serialization, not in the routing module.

**Piste difficulty mapping**:
| OSM | UI label | Map color | Hex |
|-----|----------|-----------|-----|
| novice | Green | green | #22c55e |
| easy | Blue | blue | #3b82f6 |
| intermediate | Red | red | #ef4444 |
| advanced | Black | black | #1e293b |
| freeride | Freeride | black dashed | #1e293b |

**Lift type mapping**:
| OSM aerialway | Filter category | UI label |
|---------------|-----------------|----------|
| chair_lift | chairlift | Chairlift |
| gondola | gondola | Gondola |
| platter, drag_lift | drag_lift | Drag lift |
| cable_car | cable_car | Cable car |
| magic_carpet | (always included) | Magic carpet |

**Filter behavior**: Checkboxes use the filter category. When unchecked, all segments with matching `difficulty` (for pistes) or `aerialway` sub-type (for lifts) are excluded from Dijkstra traversal. Synthetic edges (traverse, ski-out, ski-in) are always included.

## R7: Port Configuration

**Decision**: clap `--port` flag with `PORT` env var override

**Rationale**: Constitution mandates Heroku convention (PORT env var takes precedence).

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "3000")]
    port: u16,
}

fn resolve_port(cli: &Cli) -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(cli.port)
}
```

This must override the cargo-leptos `site-addr` at runtime. The server binds to `0.0.0.0:{port}` (required for Heroku).
