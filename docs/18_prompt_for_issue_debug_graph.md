Avec claude dans un browser

Peux tu m'aider à créer le texte d'une issue GitHub en utilisant les éléments ci-dessous




Ideas for debug_graph CLI Visualizer:
Add a CLI binary debug_graph to the piste_che crate that loads OSM data, runs build_graph(), optionally runs dijkstra() with hardcoded start/end nodes, and writes a self-contained temp/debug_graph.html with a Leaflet.js map showing all nodes and segments colored by kind, configurable radius circles at lift nodes, and a route overlay.


Plan for debug_graph CLI Visualizer:

Context
The routing pipeline in src/routing/ has ~12 tuning constants (CROSSING_RADIUS, TRAVERSE_RADIUS, SKI_IN_RADIUS, etc.) and complex 7-step graph construction. Currently
there is no way to visually verify why routing behaves unexpectedly (missing traverses, wrong crossings, etc.). Goal: a CLI binary that reuses routing code as-is and
generates a single self-contained HTML map.

Approach
Add a new binary src/bin/debug_graph.rs to the existing crate. It imports routing code directly (no duplication), calls build_graph() and optionally dijkstra(), then
writes temp/debug_graph.html with an embedded Leaflet.js map.

Why this approach: [[bin]] in same crate avoids a sub-crate, shares all routing types/constants directly, and default = ["ssr"] means no extra feature flags needed.

Files to Modify

Cargo.toml

Uncomment existing [[bin]] block and add new entry:
[[bin]]
name = "piste_che_server"
path = "src/main.rs"
required-features = ["ssr"]

[[bin]]
name = "debug_graph"
path = "src/bin/debug_graph.rs"
required-features = ["ssr"]

src/bin/debug_graph.rs (new file)

Config block at top (user edits to tune):
// START_NODE / END_NODE: Option<usize> -- None skips routing
// SHOW_TRAVERSES / SHOW_SKI_IN / SHOW_SKI_OUT: bool
// RADIUS_DISPLAY: Option<f64> -- draws circles at this radius on lift nodes

main() flow:
1. find_latest_json(Path::new("data")) -> path
2. OsmData::load(&path) -> osm
3. build_graph(&osm) -> (nodes, segments, _route_elements)
4. adjacency_from_segments(&segments) -> adj
5. If START/END set: call dijkstra(start, &arrival_zone(end), ...) -> route seg IDs
6. Call generate_html(...) -> write temp/debug_graph.html
7. Print counts to stdout

Imports (mirror pattern from src/main.rs):
use piste_che::routing::{
    OsmData, adjacency_from_segments, arrival_zone, build_graph, dijkstra,
    data::find_latest_json,
};

HTML generation (pure string formatting, no new deps):
- Segments split into 5 GeoJSON FeatureCollections by kind: piste, lift, traverse, ski-in, ski-out
- Nodes as GeoJSON Points with popup: id, lat/lon/ele, connected segment names
- Route as separate GeoJSON LineString collection (magenta, weight 5)
- Radius circles: Leaflet L.circle objects at lift-base/lift-exit nodes, radius = RADIUS_DISPLAY
- Layer toggles: <input type="checkbox"> per layer in a floating #controls div
- Piste color by difficulty: green/blue/red/black
- GeoJSON coords: swap to [lon, lat] (GeoJSON convention)

How to Run

```powershell
cargo run --bin debug_graph
# Output: temp/debug_graph.html
Open temp/debug_graph.html in any browser. Toggle layers in top-right panel.
```

Verification

1. cargo build --bin debug_graph -- clean compile
2. Run -> file written with correct node/segment counts printed
3. Open HTML -> map centered on Serre Chevalier, all segment types visible
4. Set START_NODE / END_NODE -> route overlaid in magenta
5. Set RADIUS_DISPLAY = Some(50.0) -> circles appear at lift nodes matching CROSSING_RADIUS