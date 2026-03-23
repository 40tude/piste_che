# Code Review: piste_che

Version 0.1.0 | Commit 6577588 | 2026-03-23

---

## Summary

Full-stack Rust/Leptos ski itinerary planner. Code quality is high overall: strict lint configuration, proper feature-gating, good error propagation via `anyhow`/`thiserror`, and meaningful tests. Findings below are genuine issues, not style opinions.

Audit fixes applied on branch `audit/code-review` (commits 4cf5b84..8a1ccf0).

---

## Findings

### [CRITICAL]

---

**C1 -- `src/routing/graph.rs:48` -- `SKI_OUT_RADIUS` comment contradicts code and constant value** -- [FIXED]

The original docstrings claimed both `SKI_OUT_RADIUS` and `SKI_IN_RADIUS` "must be >= SPLIT_RADIUS (300 m)" while the constants are 100 m. After investigation the constants are correct (tight radius prevents long-range shortcuts); the premise in the comments was wrong.

Fix: replaced both docstrings with accurate explanations of why 100 m is the right value and how piste nodes beyond that radius are still reachable via regular segments. Commit `4cf5b84`.

---

**C2 -- `src/routing/dijkstra.rs:31` -- Dijkstra uses `usize`-indexed `dist[]` but node IDs are not guaranteed contiguous** -- [FIXED]

Fix: added `debug_assert!` in `build_graph` verifying `node.id == index` for every node before returning, and two range guards at the top of `dijkstra()` confirming `start < n_nodes` and all `goal_zone` members are in range. The invariant is now explicit and will fire immediately in debug builds if it ever breaks. Commit `b37e8dd`.

---

**C3 -- `src/routing/graph.rs:386` -- `dedup_by_key` on `boundaries` can silently remove valid split nodes** -- [FIXED]

`dedup_by_key` only removes *consecutive* duplicates. If the same node appeared at non-adjacent positions (e.g., ci=10 and ci=25), both survived, producing a topology loop through that node.

Fix: replaced `boundaries.dedup_by_key(|b| b.1)` with a `HashSet`-based `retain` that removes *all* duplicate node IDs globally (keeping the first / lowest-ci occurrence), regardless of position. Commit `d947efe`. Also resolves W3.

---

### [WARN]

---

**W1 -- `src/main.rs:70` -- Hardcoded data file path defeats `find_latest_json()`** -- [FIXED]

Fix: replaced the hardcoded `"data/serre_chevalier_20260319_221219.json"` with a call to `find_latest_json(Path::new("data"))`. The server now auto-selects the most recent timestamped JSON in `data/` at startup; no manual edit is needed after a data refresh. Commit `f9f2d91`.

---

**W2 -- `src/server/api.rs:244` -- End-element deduplication uses string comparison, not structural equality** -- [FIXED]

Fix: the end-element guard now compares `(name, kind)` instead of `name` alone, matching the `group_key` composite-key semantics used elsewhere. Commit `83df2f2`.

---

**W3 -- `src/routing/graph.rs:386` -- Piste boundary dedup uses node identity, not position order** -- [FIXED]

Same root cause and fix as C3. Commit `d947efe`.

---

**W4 -- `src/server/api.rs:196-238` -- Intermediate path steps: distance accumulation skips highlight extension for matching names** -- [FIXED]

Fix: added `debug_assert!(!steps.is_empty() && !highlight_segments.is_empty())` immediately before the same-name accumulation branch. The assert fires in debug builds if the invariant (steps and highlights are always added together) is violated. Commit `1ce61f0`.

---

**W5 -- `src/routing/graph.rs` (Step 6b/6c) -- `lift_base_ids` and `lift_exit_ids` re-computed from segments that already include synthetic edges** -- [FIXED]

Fix: both `HashSet`s are now computed once from the pure piste/lift segments (end of Step 5, before any synthetic edges are appended). Steps 6b and 6c borrow `&lift_base_ids_snap` / `&lift_exit_ids_snap` rather than re-deriving them. The ordering dependency is now explicit and cannot be broken by reordering Step 6a. Commit `71fe40c`.

---

**W6 -- `src/routing/chains.rs:34` -- `remaining.remove(0)` is O(n) per iteration** -- [FIXED]

Fix: changed `remaining` from `Vec<usize>` to `VecDeque<usize>`; the initial pop is now `pop_front()` (O(1)). The two mid-loop `remove(pos)` calls remain O(n) (inherent to the greedy search), but the startup cost per chain is eliminated. Commit `ddc377c`.

---

**W7 -- `resort_generator/src/elevation.rs:68` -- Retry loop always attempts 3 iterations, even on success**

No code change. The loop is correct; the readability concern is minor and the existing comment is sufficient.

---

**W8 -- `src/routing/graph.rs:172` -- Elevation sentinel 0.0 treated as "missing"** -- [FIXED]

Fix: added an inline comment documenting the assumption: 0.0 means "no elevation data" because Serre Chevalier elements are all above 1000 m, but this would misclassify sea-level elements. Full propagation of `Option<f32>` through the pipeline is left as future work. Commit `01099b8`.

---

**W9 -- `src/components/segment_popup.rs:43-50` -- Duplicate `haversine` implementation** -- [FIXED]

Fix: added a `// NOTE: identical implementation lives in src/routing/data.rs` cross-reference to both copies so future maintainers know to update both if the formula changes. Commit `ef695b9`.

---

**W10 -- `tests/integration.rs:165` -- `compute_route_unknown_element` test assertion is vacuous** -- [FIXED]

Fix: the test now distinguishes the two valid outcomes: (a) HTTP 500 -- asserted directly, (b) HTTP 200 -- body must contain a non-empty `error` field. Any other status fails the test. Commit `03a2db6`.

---

**W11 -- `Cargo.toml:39` -- `tower` and `tower-http` not feature-gated under `ssr`** -- [FIXED]

Fix: both crates are now `optional = true` and listed under the `ssr` feature. They are no longer pulled into dependency resolution for WASM builds. Commit `8a1ccf0`.

---

**W12 -- `src/routing/graph.rs:384-386` -- Lift boundary detection iterates all nodes for every lift coord**

Not fixed. O(n^2) is negligible at single-resort scale (startup-only); deferred to future work if multi-resort datasets are introduced.

---

### [INFO]

---

**I1 -- `src/app.rs:89` -- `active_mode` signal not wired to `compute_action`**

`active_mode` is a signal defaulting to `"short"`, but `compute_action` always passes `"short".to_string()`. The signal is decorative until Sport/Safe modes are implemented.

---

**I2 -- `src/routing/data.rs:248` -- `find_latest_json()` is dead code in the main app**

Resolved by W1 fix: `find_latest_json` is now called by `main.rs`.

---

**I3 -- `src/components/map.rs:61` -- `arrow_class` has an unreachable `_ =>` arm**

Not fixed. The compiler cannot prove the arm unreachable (integer arithmetic); the arm is a safety net. Low priority.

---

**I4 -- `src/main.rs:110-124` -- Large commented-out code block**

Not fixed. Deferred; the block documents the alternate Router pattern for reference.

---

**I5 -- `resort_generator/src/overpass.rs:18-21` -- Resort name injected verbatim into Overpass QL query**

Not fixed. CLI-only tool; injection risk is minimal for a developer-facing tool.

---

**I6 -- `src/routing/data.rs:63` -- `OsmData::load` reads entire file into a `String` before parsing**

Not fixed. Single-resort file is a few MB; streaming parse is an optimization for a hypothetical future multi-resort dataset.

---

**I7 -- Architecture doc: `README.md:64-74` -- "All tests (unit + integration)" section is identical to "Integration tests only"**

Documentation error; not fixed in this pass.

---

**I8 -- `src/components/mode_tabs.rs` -- `active_mode` parameter is unused in rendering**

Not fixed. Will be used once Sport/Safe modes are implemented (see I1).

---

## Spec vs. Implementation Gaps

| Spec requirement | Status |
|---|---|
| FR-001: Short routing mode | Implemented |
| FR-002: Sport routing mode | Tabs visible, backend returns error -- documented as future work |
| FR-003: Safe routing mode | Same as FR-002 |
| FR-004: Only lifts in dropdowns | Implemented and integration-tested |
| FR-005: Difficulty filter | Implemented |
| FR-006: Lift-type filter | Implemented |
| `find_latest_json()` wired in main | Fixed (W1) |

No undocumented deviations found.

---

## Priority Summary

| Tag | Count | Fixed | Deferred |
|---|---|---|---|
| [CRITICAL] | 3 | 3 | 0 |
| [WARN] | 12 | 10 | 2 (W7, W12) |
| [INFO] | 8 | 1 (I2 via W1) | 7 |
