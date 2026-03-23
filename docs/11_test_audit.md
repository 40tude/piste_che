# Testing Gap Analysis

Version 0.1.0 | Commit 6577588 | 2026-03-23 | Updated 2026-03-23 (tests implemented)

---

## Inventory of Existing Tests

| File | Kind | Count | Function names |
|---|---|---|---|
| `src/routing/dijkstra.rs` | unit | 6 | same_start_end, disconnected, simple_path, excluded_difficulty_blocks, excluded_lift_blocks, excluded_difficulty_alternate |
| `resort_generator/src/overpass.rs` | unit | 1 | build_query_contains_resort_name_and_landuse |
| `resort_generator/src/elevation.rs` | unit (mock) | 1 | batching_110_nodes_calls_mock_3_times |
| `resort_generator/src/types.rs` | unit | 2 | resort_config_serre_chevalier, resort_config_mont_blanc_2000 |
| `resort_generator/src/merge.rs` | unit | 2 | validate_completeness_returns_err, validate_completeness_passes |
| `tests/integration.rs` | integration | 4 | get_area_returns_valid, compute_route_valid_request, compute_route_same_start_end, compute_route_unknown_element |

**Total before this pass: 16 tests** across 6 files.
**Total after this pass: 63 unit tests + 7 integration tests** across 8 files.

---

## Module-by-Module Analysis

---

### `src/routing/dijkstra.rs`

**Tested:**
- Start already in goal zone -> `Some(vec![])`
- No path (empty graph) -> `None`
- Single-segment happy path -> `Some([0])`
- Excluded difficulty blocks only path -> `None`
- Excluded lift type blocks only path -> `None`
- Excluded difficulty with alternate route -> path found, blocked segment absent

**Not tested:**
- Weight comparison: shortest physical path chosen over a longer one (Dijkstra correctness)
- Lift weight (50 m fixed) vs piste weight (haversine): a lift + piste route beats a piste-only route that is > 50 m longer
- Traverse weight (10x penalty): traverse path is avoided when a piste alternative exists
- Multi-hop path reconstruction (3+ nodes, 2+ segments) -- existing tests only verify 0 or 1 segment paths
- Goal zone with multiple nodes: Dijkstra stops at the first settled member, not necessarily the cheapest-to-reach
- `segment_length()` edge cases: zero coords -> 0.0, single coord -> 0.0, two coords -> haversine distance

**Redundant / too implementation-specific:**
- None; existing tests are behavior-level and appropriate.

**Verdict:** Missing coverage for weight semantics (the most critical correctness property of Dijkstra) and multi-hop reconstruction.

---

### `src/routing/graph.rs`

**Tested:** Nothing. Zero unit tests.

**Not tested:**
- `build_polylines()`: single way -> polyline with correct coords and direction
- Direction normalization: lift stored top-to-base gets reversed to base-to-top; piste stored base-to-top gets reversed to top-to-base; flat segment (equal elevation) is left as-is
- `is_closed_polygon()` filtering: a closed way is excluded from polylines
- `element_kind()` classification: `aerialway=chair_lift` -> `"lift"`, `aerialway=goods` -> `"?"`, `piste:type=downhill` -> `"piste"`, neither -> `"?"`
- `adjacency_from_segments()`: correct node-to-segments mapping; node with no outgoing edges absent from map
- `arrival_zone()`: includes goal node and ski-in sources; excludes traverse sources; empty segments -> zone contains only goal node
- Split detection: interior coord near an endpoint candidate generates a split
- Cluster deduplication: two candidates within `CLUSTER_RADIUS` produce one node, not two
- Lift boundary restriction: interior lift coords do not generate boundary nodes
- `entry_exit()`: piste start = higher end; lift start = lower end; handles equal elevation

**Verdict:** Highest coverage gap in the codebase. The 7-step graph pipeline has zero tests; correctness is only validated end-to-end through integration tests, which cannot isolate individual pipeline steps.

---

### `src/routing/chains.rs`

**Tested:** Nothing. Zero unit tests.

**Not tested:**
- Single way -> one chain with `reversed=false`
- Two ways connected tail-to-head -> merged into one chain
- Two ways connected but the second must be reversed -> `reversed=true` on the second segment
- Three ways forming a line -> one chain
- Two disconnected groups -> two separate chains
- Way with empty `nodes` list -> handled gracefully (uses `unwrap_or(0)`)

**Verdict:** Complete coverage gap. `build_chains` is the entry point for all graph topology; a bug here silently produces broken polylines.

---

### `src/routing/data.rs`

**Tested:** Nothing. Zero unit tests.

**Not tested:**
- `OsmData::load()`: valid JSON file -> correct `ways` and `nodes` counts
- `OsmData::load()`: file not found -> `Err` with path in message
- `OsmData::load()`: invalid JSON -> `Err`
- `RawWay::name()`: `name` tag present; `piste:name` fallback when `name` absent; both absent -> `None`
- `RawWay::group_key()`: lift -> `"Name [chair_lift 6p]"` format; lift without occupancy -> no `" p"` suffix; piste -> name only
- `RawWay::is_closed_polygon()`: first == last with > 1 node -> true; open way -> false; single node -> false
- `RawWay::element_kind()`: `aerialway=gondola` -> `"lift"`; `aerialway=goods` -> `"?"`; `aerialway=construction` -> `"?"`; `piste:type=downhill` -> `"piste"`; neither -> `"?"`
- `RawWay::difficulty()`: aerialway present -> returns aerialway value; piste:difficulty present -> returns value; absent -> `"-"`
- `RawWay::occupancy()`: parseable integer -> `Some(n)`; non-parseable string -> `None`; absent -> `None`
- `RawWay::duration_min()`: plain integer `"8"` -> `Some(8)`; `"PT8M"` -> `Some(8)`; `"PT1H30M"` -> `Some(90)`; `"PT"` (malformed) -> `None`; absent -> `None`
- `haversine()`: identical coords -> 0.0; known distance between two real coordinates
- `find_latest_json()`: empty directory -> `Err`; directory with multiple files -> most recent alphabetically; `request.json` excluded

**Verdict:** Complete coverage gap. The parsing helpers are pure functions that are trivially unit-testable without filesystem access (tag maps only).

---

### `src/server/api.rs`

**Tested (via integration):**
- `get_area` response shape and `selectable_elements` kind constraint
- `compute_route` with valid pair: 200, response has required fields
- `compute_route` same start/end (non-lift): error field set
- `compute_route` unknown element names: vacuous assertion (see REMOVE section)

**Not tested:**
- `build_area_response()`: `selectable_elements` excludes non-lift route elements
- `element_distance()`: sums segment lengths for all matching name+kind segments; returns 0 for an element with no matching segments
- `element_highlight()`: coord extraction from multiple segments of the same element
- `compute_route` mode != `"short"` -> `error` field set, `steps` empty
- `compute_route` start == end for a lift element (circuit case) -> no error, route attempted
- `compute_route` with excluded difficulty that removes all paths -> `error` field set (`"No route found"`)
- `compute_route` with valid route: `total_distance_m` equals sum of step `distance_m` values
- `compute_route` with valid route: `highlight_segments` is non-empty when `steps` is non-empty
- `compute_route` content-type: POST body must be `application/json`; sending form-encoded returns error

**Verdict:** Integration tests cover structural validity but not route computation correctness or error field semantics.

---

### `src/components/segment_popup.rs`

**Tested:** Nothing. Zero unit tests.

**Not tested (all pure functions, testable without WASM):**
- `haversine()`: identical points -> 0.0; known pair
- `project_point_onto_segment()`: point on segment midpoint -> t = 0.5; point before A -> t = 0.0; point past B -> t = 1.0; degenerate zero-length segment -> t = 0.0
- `nearest_segment()`: click within 30 m -> `Some(PopupData)`; click 31 m away -> `None`; empty segment list -> `None`; segment with < 2 coords -> skipped
- `nearest_segment()`: altitude interpolated correctly at t = 0.0 (-> c0[2]), t = 1.0 (-> c1[2]), t = 0.5 (-> midpoint)
- `nearest_segment()`: `length_m` sums all sub-segments of the same name+kind

**Verdict:** Complete coverage gap. The popup geometry is client-side logic; unit tests require no browser or server.

---

### `src/components/map.rs`

**Tested:** Nothing. Zero unit tests.

**Not tested (pure functions, testable without WASM):**
- `segment_color()`: `kind="lift"` -> amber regardless of difficulty; each named difficulty; unknown difficulty -> slate
- `route_bearing()`: fewer than 2 coords -> 0.0; north (dlat > 0, dlon = 0) -> 0.0; east (dlat = 0, dlon > 0) -> 90.0
- `route_midpoint()`: empty coords -> `None`; 1 coord -> `Some([0])`; 3 coords -> `Some([1])`
- `arrow_class()`: bearing 0 -> `"route-arrow-0"`; boundary values 22, 23, 67, 68 (sector transitions); bearing 360 == bearing 0

**Verdict:** Moderate gap. These are trivial pure functions with no dependencies; tests would be small and high-value for catching future color/bearing changes.

---

### `resort_generator/src/overpass.rs`

**Tested:**
- Query string contains resort name and `landuse=winter_sports`

**Not tested:**
- `fetch_trails` mock: returns 0 ways -> `bail!`
- `fetch_trails` mock: returns 0 nodes -> `bail!`
- `fetch_trails` mock: HTTP error -> error propagated
- `fetch_trails` mock: malformed JSON body -> deserialization error

**Verdict:** Only the query builder is tested. The fetch function's validation logic (way/node count guards) has no tests.

---

### `resort_generator/src/elevation.rs`

**Tested:**
- 110 nodes -> 3 mock HTTP calls (batch split correctness)

**Not tested:**
- Retry on first failure, success on second attempt: result is `Ok`
- All 3 retries exhausted -> error returned with batch number in message
- IGN returns wrong elevation count for batch -> `bail!`
- Last batch does not trigger `tokio::time::sleep` (boundary: `batch_idx + 1 < total_batches`)
- 0 nodes input -> `Ok(vec![])` immediately
- Exactly 50 nodes -> 1 batch, no inter-batch delay

**Verdict:** Batch splitting is tested but retry logic and error paths are not. The backoff retry is the most operationally critical code path for a flaky external API.

---

### `resort_generator/src/merge.rs`

**Tested:**
- Missing elevation (`ele = None`) -> error listing node ID
- All nodes have elevation -> `Ok`

**Not tested:**
- `merge_elevation()`: node present in elevations map -> `ele` field set correctly
- `merge_elevation()`: node not in elevations map -> `ele` field unchanged (not overwritten with 0)
- `validate_completeness()`: way references a node ID absent from elements entirely (not just missing ele)
- `validate_completeness()`: no ways -> `Ok` (no required nodes)
- `validate_completeness()`: nodes unreferenced by any way are ignored (they don't cause a failure)

**Verdict:** The `merge_elevation` function itself has no tests; only `validate_completeness` is partially covered.

---

### `resort_generator/src/types.rs`

**Tested:**
- `ResortConfig::from_name()`: slug derivation for two resort names

**Not tested:**
- `ResortConfig::from_name()`: name with special characters (hyphens, accented letters) -- slug rule only specifies space -> underscore
- `ElevationResponse` deserialization: valid JSON -> `elevations` vec populated
- `Node` serde: `ele = None` is not serialized (`skip_serializing_if`); `ele = Some(1000.0)` is serialized

**Verdict:** Minimal; `ResortConfig` slug tests cover the only non-trivial logic.

---

## Tests to REMOVE or SIMPLIFY

### REMOVE: `compute_route_unknown_element` (integration.rs:145)
```rust
assert!(
    resp.status().is_server_error() || resp.status().is_success(),
    ...
);
```
Accepts any 2xx or 5xx status, which is every normal HTTP response. Tests nothing. Should be replaced with a specific assertion (see MISSING below).

### SIMPLIFY: `compute_route_valid_request` (integration.rs:72)
The test comment acknowledges "The route might not exist between any random pair of lifts" and then only checks that `steps` and `highlight_segments` keys exist in the JSON. The test is essentially a shape-only check on an endpoint that might return a no-route error. Should use a known routable pair (or skip if route not found), and add an assertion that when `error` is null, `steps` is non-empty.

---

## Missing Tests -- Priority Order

### P1: Critical correctness (routing pipeline)

1. **`graph.rs` -- `adjacency_from_segments`**: 2-3 segments -> correct adj map, node with no outgoing edges absent -- [IMPLEMENTED]
2. **`graph.rs` -- `arrival_zone`**: returns goal + ski-in sources; excludes traverse sources -- [IMPLEMENTED]
3. **`dijkstra.rs` -- weight semantics**: two paths exist; shorter physical distance wins; lift (50 m) beats a piste > 50 m -- [IMPLEMENTED]
4. **`graph.rs` -- `entry_exit`**: piste start = higher coord; lift start = lower coord -- [IMPLEMENTED]
5. **`chains.rs` -- reversal**: two ways where second must be reversed -> `reversed=true` -- [IMPLEMENTED]

### P2: Data parsing (resort_generator)

6. **`data.rs` -- `RawWay::group_key`**: lift key format including/excluding occupancy; piste key = name only -- [IMPLEMENTED]
7. **`data.rs` -- `RawWay::duration_min`**: ISO 8601 `"PT8M"`, `"PT1H30M"`, plain `"8"`, malformed -- [IMPLEMENTED]
8. **`data.rs` -- `RawWay::element_kind`**: `goods`/`construction` -> `"?"`; passenger aerialway -> `"lift"` -- [IMPLEMENTED]
9. **`merge.rs` -- `merge_elevation`**: node gets correct elevation; node absent from map unchanged -- DEFERRED (requires mock infrastructure)
10. **`overpass.rs` -- `fetch_trails` validation**: 0 ways -> bail; 0 nodes -> bail -- DEFERRED (requires HTTP mocking)

### P2 additions (also implemented)

- **`data.rs` -- `haversine`**: identical coords -> 0.0; known ~100 m pair -- [IMPLEMENTED]
- **`dijkstra.rs` -- `segment_length`**: 0 coords, 1 coord, 2 known-distance coords -- [IMPLEMENTED]
- **`dijkstra.rs` -- multi-hop path reconstruction**: 3-segment chain path -- [IMPLEMENTED]
- **`dijkstra.rs` -- traverse penalty**: traverse avoided when piste alternative exists -- [IMPLEMENTED]
- **`chains.rs` -- single/merged/disconnected chains**: 5 chain-building scenarios -- [IMPLEMENTED]

### P3: Geometry helpers (client-side)

11. **`segment_popup.rs` -- `project_point_onto_segment`**: before-A, on-segment, past-B, degenerate -- [IMPLEMENTED]
12. **`segment_popup.rs` -- `nearest_segment`**: within threshold -> Some; beyond threshold -> None -- [IMPLEMENTED]
13. **`segment_popup.rs` -- altitude interpolation**: t=0.5 midpoint interpolation -- [IMPLEMENTED]
14. **`map.rs` -- `arrow_class`**: boundary values at sector transitions (22/23, 67/68, etc.) -- [IMPLEMENTED]

### P3 additions (also implemented)

- **`map.rs` -- `segment_color`**: lift=amber, easy=blue, unknown=slate -- [IMPLEMENTED]
- **`map.rs` -- `route_bearing`**: no coords, north, east -- [IMPLEMENTED]
- **`map.rs` -- `route_midpoint`**: empty, 1-coord, 3-coord -- [IMPLEMENTED]
- **`segment_popup.rs` -- `nearest_segment`**: empty list, single-coord segment skipped -- [IMPLEMENTED]

### P4: Integration test improvements

15. **Replace** `compute_route_unknown_element`: assert `status == 500` OR `body["error"]` is non-null and non-empty -- [IMPLEMENTED] (W10 fix, commit 03a2db6)
16. **Add** `compute_route_mode_not_short`: POST with `mode: "sport"` -> 200, `error` contains "not implemented" -- [IMPLEMENTED]
17. **Add** `compute_route_difficulty_filter_blocks_all`: POST with all difficulties excluded -> 200, `error` contains "No route" -- [IMPLEMENTED]
18. **Add** `get_area_segment_coords_have_three_elements`: every coord in segments must be `[lat, lon, alt]` (length 3) -- [IMPLEMENTED]

---

## Summary Table

| Module | Tests exist | Coverage | Remaining gaps |
|---|---|---|---|
| `routing/dijkstra.rs` | Yes (12) | Good | None significant |
| `routing/graph.rs` | Yes (7) | Good | None significant |
| `routing/chains.rs` | Yes (5) | Good | None significant |
| `routing/data.rs` | Yes (10) | Good | find_latest_json (filesystem) |
| `server/api.rs` | Partial (integration) | Shape + mode + filter | Circuit case, distance sums |
| `components/segment_popup.rs` | Yes (7) | Good | None significant |
| `components/map.rs` | Yes (10) | Good | None significant |
| `resort_generator/overpass.rs` | Partial (1) | Query only | fetch validation (needs HTTP mock) |
| `resort_generator/elevation.rs` | Partial (1) | Batching only | retry logic (needs HTTP mock) |
| `resort_generator/merge.rs` | Partial (2) | validate only | merge_elevation (needs mock) |
| `resort_generator/types.rs` | Yes (2) | Adequate | serde round-trips |
| `tests/integration.rs` | Yes (7) | Structural + mode + filter + coord shape | Known routable pair |
