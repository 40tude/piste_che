# Tasks: Map, Filters & Shortest Route

**Input**: Design documents from `/specs/001-map-filters-route/`
**Branch**: `001-map-filters-route`
**Stack**: Rust 1.85+ (edition 2024), Leptos 0.7 SSR, Axum, cargo-leptos, leptos-leaflet

**Tests**: Included -- constitution Principle III (Test-First NON-NEGOTIABLE) and plan.md TDD declaration require it.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete-task dependencies)
- **[Story]**: User story label (US1-US4)
- Exact file paths included in every task description

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Create project skeleton and obtain external assets before any Rust code is written.

- [X] T001 Create `Cargo.toml` with `[lib]` (`crate-type = ["cdylib", "rlib"]`), all dependencies (leptos 0.7, leptos_axum optional/ssr, leptos_router, leptos-leaflet, axum, tokio full, serde+derive, serde_json, clap+derive, tracing, tracing-subscriber, wasm-bindgen, thiserror, anyhow), feature flags `hydrate`/`ssr`, and `[package.metadata.leptos]` (output-name=piste_che, style-file=style/main.css, assets-dir=public, site-addr=127.0.0.1:3000)
- [X] T002 Create directory skeleton: `src/routing/`, `src/server/`, `src/components/`, `style/`, `public/`, `data/`, `tests/` with stub files (`src/lib.rs`, `src/app.rs`, `src/main.rs`, `src/models.rs`) so cargo can parse the crate
- [X] T003 [P] Create `Procfile` containing `web: ./target/release/piste_che`
- [X] T004 [P] Download Leaflet 1.9.x `leaflet.js` and `leaflet.css` into `public/` (self-hosted, no CDN; see R2 in research.md)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before any user story is implemented.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T005 Copy prototype routing source files from `020_serre_che_1/get_itinerary/src/` into `src/routing/`: `data.rs` (OsmData, JSON parsing, haversine), `graph.rs` (Node, Segment, RouteElement, build_graph), `chains.rs` (chain building) -- copy as-is per Principle I
- [X] T006 Extract the Dijkstra algorithm and `segment_length` helper from the prototype `main.rs` into `src/routing/dijkstra.rs` -- no logic changes, only file separation
- [X] T007 Create `src/routing/mod.rs`: add `#[cfg(feature = "ssr")]` gate on the entire module, and pub re-export `Node`, `Segment`, `RouteElement`, `build_graph`, `dijkstra` for server function access
- [X] T008 Create `src/models.rs` with all shared DTOs (derive Serialize+Deserialize): `AreaNode {id, lat, lon, alt}`, `AreaSegment {id, name, kind, difficulty, coords: Vec<[f64;2]>}`, `SelectableElement {name, kind, difficulty}`, `AreaResponse {nodes: Vec<AreaNode>, segments: Vec<AreaSegment>, selectable_elements: Vec<SelectableElement>}`, `RouteRequest {start, end, excluded_difficulties: Vec<String>, excluded_lift_types: Vec<String>, mode}`, `RouteStep {name, kind, difficulty, distance_m: u32}`, `RouteResponse {steps: Vec<RouteStep>, total_distance_m: u32, highlight_coords: Vec<Vec<[f64;2]>>, error: Option<String>}`
- [X] T009 Create `src/lib.rs` with: module declarations (`pub mod app`, `pub mod components`, `pub mod models`, `#[cfg(feature = "ssr")] pub mod routing`, `#[cfg(feature = "ssr")] pub mod server`), the `#[wasm_bindgen(start)]` hydrate entry point calling `leptos::mount_to_body(App)`
- [X] T010 Create `src/components/mod.rs` with: `pub mod map`, `pub mod filters`, `pub mod selector`, `pub mod itinerary`, `pub mod mode_tabs`
- [X] T011 Create `src/app.rs` with the root `App` component: layout shell (sidebar div + map div, no logic yet), `<Router>` wrapping a single `<Route path="/" view=HomePage/>`, and `HomePage` as an empty placeholder returning the layout

**Checkpoint**: Foundation ready -- routing module compiles under `ssr` feature, models crate builds, lib.rs resolves all modules.

---

## Phase 3: User Story 1 - View Ski Area Map (Priority: P1) - MVP

**Goal**: Skier opens the app and sees the full Serre Chevalier ski area on an interactive map with runs color-coded by difficulty and lifts in a distinct style.

**Independent Test**: Start server, open `http://localhost:3000`, verify: map renders with OSM tiles, runs visible in green/blue/red/black, lifts visually distinct from runs, pan and zoom work.

> **NOTE: Write test T012 FIRST and verify it FAILS before implementing T013-T020**

- [X] T012 [US1] Write integration test for `GET /api/get_area` in `tests/integration.rs`: assert HTTP 200, response body deserializes to `AreaResponse`, `nodes` is non-empty, `segments` is non-empty, all `selectable_elements` have `kind == "lift"`
- [X] T013 [US1] Create `AppState` struct in `src/main.rs` (ssr-gated) with fields `nodes: Vec<Node>`, `segments: Vec<Segment>`, `route_elements: Vec<RouteElement>`; implement startup JSON loading: read `data/20260315_164849_ele.json`, call `load_data()` then `build_graph()`, store result in `Arc<AppState>`
- [X] T014 [US1] Implement port resolution in `src/main.rs`: `clap` `#[derive(Parser)]` struct with `--port u16` default 3000, `resolve_port()` checks `PORT` env var first (Heroku convention per R7); bind Axum on `0.0.0.0:{port}`
- [X] T015 [US1] Initialize `tracing_subscriber` in `src/main.rs` `main()`; add `#[instrument]` or manual `tracing::info!` spans in `src/server/api.rs` for each server function call logging function name, node/segment counts
- [X] T016 [US1] Implement `get_area()` server function in `src/server/api.rs`: `use_context::<Arc<AppState>>()`, map internal `segments` to `Vec<AreaSegment>` (drop elevation from coords), map internal `nodes` to `Vec<AreaNode>`, filter `route_elements` to lifts-only for `selectable_elements`; return `AreaResponse`
- [X] T017 [US1] Create `src/server/mod.rs` with `pub mod api`; verify that `#[server]` macros in `api.rs` are sufficient for auto-registration in Leptos 0.7 (no explicit `.register()` calls needed -- that pattern is deprecated since Leptos 0.6)
- [X] T018 [US1] Create `src/components/map.rs` with `SkiMap` component using `leptos-leaflet`: `<MapContainer center=(44.9403,6.5063) zoom=13.0>`, `<TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"/>`, `<For each=segments>` rendering one `<Polyline>` per segment with color from difficulty (`novice`->#22c55e, `easy`->#3b82f6, `intermediate`->#ef4444, `advanced`/#freeride->#1e293b, lifts->#f59e0b dashed)
- [X] T019 [US1] Wire `src/app.rs` `HomePage`: call `get_area()` via `create_resource(|| (), |_| get_area())`, extract segments signal, pass to `<SkiMap segments=.../>` in the map div; show loading spinner while resource is pending
- [X] T020 [US1] Add layout CSS in `style/main.css`: full-viewport flex layout, sidebar 320px fixed width, map fills remaining width at 100vh, difficulty color variables, lift polyline style

**Checkpoint**: US1 fully functional -- map visible with colored runs and lifts. `cargo test --test integration` test T012 passes.

---

## Phase 4: User Story 2 - Compute Shortest Route (Priority: P2)

**Goal**: Skier selects start/end from dropdowns, clicks Calculate, sees shortest route highlighted on map with step-by-step itinerary panel.

**Independent Test**: Select any two connected lift stations, click Calculate, verify route polyline appears on map (yellow, weight 6), itinerary lists segments with distances, total distance shown. Test same-point and no-route edge cases show clear messages.

> **NOTE: Write tests T021-T022 FIRST and verify they FAIL before implementing T023-T028**

- [X] T021 [P] [US2] Write unit tests in `src/routing/dijkstra.rs` for edge cases: `dijkstra(start==end)` returns `Some(vec![])` or similar (not panic), `dijkstra` with no path between nodes returns `None`
- [X] T022 [P] [US2] Extend `tests/integration.rs` with `compute_route` tests: POST valid start/end -> 200 with non-empty `steps` and `highlight_coords`; POST same start/end -> 200 with `error` field set; POST disconnected nodes -> 200 with `error` field set
- [X] T023 [US2] Create `src/components/selector.rs` with `SelectorPanel` component: two `<select>` elements bound to `WriteSignal<String>` props (`set_start`, `set_end`), populated from `selectable_elements: Signal<Vec<SelectableElement>>` prop with `<option value=name>name</option>` per element
- [X] T024 [US2] Implement `compute_route()` server function in `src/server/api.rs`: validate start/end names exist in `route_elements`; resolve to `start_node`/`end_node`; build filtered adjacency (remove edges where segment difficulty is in `excluded_difficulties` or segment lift sub-type is in `excluded_lift_types`); call `dijkstra()`; map result segments to `RouteStep` + collect `highlight_coords`; return `RouteResponse` (with `error: Some(msg)` for same-point, no-route, invalid-name cases)
- [X] T025 [US2] Create `src/components/itinerary.rs` with `ItineraryPanel` component: takes `steps: Signal<Vec<RouteStep>>`, `total_distance_m: Signal<u32>`, `error: Signal<Option<String>>` props; renders each step as `<li>name - kind - distance_m m</li>`, total at bottom, or error message if present
- [X] T026 [US2] Wire route computation in `src/app.rs`: add Calculate `<button>` triggering `create_action(|req: RouteRequest| compute_route(req))`; on action value change, update `route_coords: RwSignal<Vec<Vec<[f64;2]>>>` and `steps: RwSignal<Vec<RouteStep>>`; render `<ItineraryPanel>` in sidebar
- [X] T027 [US2] Add route highlight overlay to `src/components/map.rs`: `<For each=route_coords>` rendering one `<Polyline>` per segment with `color="yellow" weight=6.0 opacity=1.0`, rendered on top of ski segments
- [X] T028 [US2] Add selector and itinerary CSS in `style/main.css`: dropdown full-width styling, Calculate button styling, itinerary list item layout, "no route" message styling

**Checkpoint**: US2 fully functional -- route computation works end-to-end. All T021-T022 tests pass.

---

## Phase 5: User Story 3 - Filter by Difficulty and Lift Type (Priority: P3)

**Goal**: Skier uses filter checkboxes to exclude run difficulties or lift types before computing a route; excluded segments are dimmed on the map and excluded from route computation.

**Independent Test**: Uncheck "advanced" (black), compute route, verify no advanced piste segments in result. Uncheck "chair_lift", compute route, verify no chair_lift segments. Check all by default on page load.

> **NOTE: Write test T029 FIRST and verify it FAILS before implementing T030-T033**

- [X] T029 [US3] Write unit tests for filter application in `src/routing/dijkstra.rs`: build test graph with mixed difficulties; call `dijkstra` with `excluded_difficulties=["advanced"]`; assert returned segment list contains no segment with `difficulty=="advanced"`; repeat for lift type exclusion
- [X] T030 [P] [US3] Create `src/components/filters.rs` with `FilterPanel` component: difficulty section with checkboxes for `novice` (Green), `easy` (Blue), `intermediate` (Red), `advanced` (Black) -- all checked by default via `create_rw_signal(true)`; lift type section with checkboxes for `chair_lift`, `gondola`, `drag_lift`, `cable_car` -- all checked by default; expose `excluded_difficulties: Signal<Vec<String>>` and `excluded_lift_types: Signal<Vec<String>>` derived from unchecked boxes
- [X] T031 [US3] Wire filter signals in `src/app.rs`: pass `excluded_difficulties` and `excluded_lift_types` from `FilterPanel` into `RouteRequest` when calling `compute_route()`; render `<FilterPanel>` in sidebar above selector
- [X] T032 [US3] Update `src/components/map.rs` `SkiMap` to accept `excluded_difficulties: Signal<Vec<String>>` and `excluded_lift_types: Signal<Vec<String>>` props; compute `opacity` per segment: 0.2 if segment's `difficulty` is in excluded set (or segment `kind=="lift"` and difficulty/aerialway in excluded lift types), 1.0 otherwise
- [X] T033 [US3] Add filter panel CSS in `style/main.css`: checkbox row layout, section labels ("Difficulty", "Lift type"), group spacing

**Checkpoint**: US3 fully functional -- filters applied to route computation and map dimming. T029 unit tests pass.

---

## Phase 6: User Story 4 - Mode Tabs (Priority: P4)

**Goal**: Interface shows Short (active), Sport (disabled), Safe (disabled) tabs. Only Short is functional.

**Independent Test**: Verify three tabs render; Short tab is styled as active; clicking Sport/Safe produces no change; clicking Short keeps it active.

- [X] T034 [US4] Create `src/components/mode_tabs.rs` with `ModeTabs` component: three tab buttons -- "Short" (active state, triggers no mode change since only mode), "Sport" (`disabled` attribute, greyed CSS class), "Safe" (`disabled` attribute, greyed CSS class); accept `active_mode: Signal<String>` prop (always "short" for now)
- [X] T035 [US4] Wire `<ModeTabs>` into `src/app.rs` layout, above the `<SelectorPanel>` in the sidebar; pass `active_mode` signal (initialized to "short")
- [X] T036 [US4] Add tab CSS in `style/main.css`: tab bar flex layout, active tab highlighted, disabled tab `opacity: 0.4 cursor: not-allowed`

**Checkpoint**: US4 visible -- three tabs rendered, Sport/Safe non-functional.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Verification, cleanup, and validation of the full system.

- [ ] T037 [P] Verify all integration tests pass: start server in background, run `cargo test --test integration`, confirm all assertions pass for `get_area` and `compute_route` cases
- [ ] T038 [P] Verify `cargo leptos build --release` succeeds with zero warnings (fix any unused imports, dead code warnings)
- [ ] T039 Run quickstart.md full validation: start server, open `http://localhost:3000`, complete full workflow (open app -> see map -> select points -> apply filters -> compute route -> read itinerary) within 30 seconds (SC-003)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies -- start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion -- BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational -- no other story dependency
- **US2 (Phase 4)**: Depends on Foundational + US1 (map must render route highlight)
- **US3 (Phase 5)**: Depends on US2 (filters modify route computation)
- **US4 (Phase 6)**: Depends on Foundational only (pure UI, no logic dependency)
- **Polish (Final)**: Depends on all desired stories complete

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational -- independent
- **US2 (P2)**: Starts after US1 -- route highlight needs SkiMap
- **US3 (P3)**: Starts after US2 -- filters extend route computation
- **US4 (P4)**: Starts after Foundational -- can be done in parallel with US1

### Within Each User Story

- Write test tasks FIRST -- verify they FAIL before any implementation
- Models (T008) before server functions
- Server functions before component wiring
- Component wiring before CSS polish

---

## Parallel Examples

### Phase 2 (Foundational)

```powershell
# Run in parallel (different files):
# T005: copy data.rs, graph.rs, chains.rs
# T006: extract dijkstra.rs
# T008: write models.rs
# T009: write lib.rs
# T010: write components/mod.rs
```

### Phase 3 (US1)

```powershell
# Write test first:
# T012: integration test for get_area

# Then implement in parallel where marked [P]:
# T013: AppState + JSON loading in main.rs
# T014: port resolution in main.rs  (sequential after T013)
# T015: tracing setup in main.rs    (sequential after T013)
# T016: get_area server function
# T018: SkiMap component             (parallel with T016)
```

### Phase 4 (US2)

```powershell
# Write tests first in parallel:
# T021: unit tests dijkstra.rs
# T022: integration tests integration.rs

# Then implement:
# T023: SelectorPanel component       (parallel with T024)
# T024: compute_route server function (parallel with T023)
# T025: ItineraryPanel component      (parallel with T024)
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: User Story 1 (map view)
4. **STOP and VALIDATE**: server runs, map visible, `cargo test --test integration` passes T012
5. Demo if ready

### Incremental Delivery

1. Setup + Foundational -> foundation ready
2. US1 -> map visible -> validate -> demo (MVP)
3. US2 -> route computation -> validate -> demo
4. US3 -> filters -> validate -> demo
5. US4 -> mode tabs -> validate -> final demo

---

## Notes

- `[P]` tasks touch different files with no incomplete-task dependencies
- `[USn]` label maps every task to its user story for traceability
- Constitution Principle I: routing module files (`src/routing/`) copied as-is -- no logic changes
- Constitution Principle III: test tasks MUST be written and FAIL before their implementation tasks start
- Constitution Principle VI: no `println!` -- use `tracing::info!` / `tracing::error!` throughout
- Verify `cargo test` (unit) passes after T005-T007 before moving to Phase 3
