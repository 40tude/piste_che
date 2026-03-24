Create a GitHub issue on this project with the following details

# Add `debug_graph` CLI binary for visual graph debugging

## Context

The routing pipeline in `src/routing/` has ~12 tuning constants (`CROSSING_RADIUS`, `TRAVERSE_RADIUS`, `SKI_IN_RADIUS`, etc.) and a complex 7-step graph construction. Currently there is **no way to visually verify** why routing behaves unexpectedly (missing traverses, wrong crossings, etc.).

**Goal:** a CLI binary that reuses routing code as-is and generates a single self-contained HTML map via Leaflet.js.

## Approach

Add a new binary `src/bin/debug_graph.rs` to the existing crate.

- Imports routing code directly (no duplication)
- Calls `build_graph()` and optionally `dijkstra()`
- Writes `temp/debug_graph.html` with an embedded Leaflet.js map

**Why `[[bin]]` in the same crate:** avoids a sub-crate, shares all routing types/constants directly, and `default = ["ssr"]` means no extra feature flags needed.

## Files to modify

### `Cargo.toml`

Uncomment the existing `[[bin]]` block and add a new entry:

```toml
[[bin]]
name = "piste_che_server"
path = "src/main.rs"
required-features = ["ssr"]

[[bin]]
name = "debug_graph"
path = "src/bin/debug_graph.rs"
required-features = ["ssr"]
```

### `src/bin/debug_graph.rs` (new file)

**Config block** at top (user edits to tune):

- `START_NODE` / `END_NODE`: `Option<usize>` — `None` skips routing
- `SHOW_TRAVERSES` / `SHOW_SKI_IN` / `SHOW_SKI_OUT`: `bool`
- `RADIUS_DISPLAY`: `Option<f64>` — draws circles at this radius on lift nodes

**`main()` flow:**

1. `find_latest_json(Path::new("data"))` → path
2. `OsmData::load(&path)` → osm
3. `build_graph(&osm)` → `(nodes, segments, _route_elements)`
4. `adjacency_from_segments(&segments)` → adj
5. If `START_NODE` / `END_NODE` set → call `dijkstra(start, &arrival_zone(end), ...)` → route segment IDs
6. Call `generate_html(...)` → write `temp/debug_graph.html`
7. Print counts to stdout

**Imports** (mirror pattern from `src/main.rs`):

```rust
use piste_che::routing::{
    OsmData, adjacency_from_segments, arrival_zone, build_graph, dijkstra,
    data::find_latest_json,
};
```

**HTML generation** (pure string formatting, no new deps):

- Segments split into 5 GeoJSON `FeatureCollection`s by kind: piste, lift, traverse, ski-in, ski-out
- Nodes as GeoJSON `Point`s with popup: id, lat/lon/ele, connected segment names
- Route as separate GeoJSON `LineString` collection (magenta, weight 5)
- Radius circles: Leaflet `L.circle` objects at lift-base/lift-exit nodes, radius = `RADIUS_DISPLAY`
- Layer toggles: `<input type="checkbox">` per layer in a floating `#controls` div
- Piste color by difficulty: green / blue / red / black
- GeoJSON coords: swap to `[lon, lat]` (GeoJSON convention)

## How to run

```powershell
cargo run --bin debug_graph
# Output: temp/debug_graph.html
```

Open `temp/debug_graph.html` in any browser. Toggle layers in the top-right panel.

## Verification

- [ ] `cargo build --bin debug_graph` — clean compile
- [ ] Run → file written with correct node/segment counts printed to stdout
- [ ] Open HTML → map centered on Serre Chevalier, all segment types visible
- [ ] Set `START_NODE` / `END_NODE` → route overlaid in magenta
- [ ] Set `RADIUS_DISPLAY = Some(50.0)` → circles appear at lift nodes matching `CROSSING_RADIUS`
