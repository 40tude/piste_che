# Tasks: Resort Data Generator CLI

**Input**: Design documents from `/specs/002-resort-generator/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md, quickstart.md

**Tests**: Included -- plan.md mandates `mockall` HTTP mocking; constitution principle III (Test-First) is non-negotiable.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1, US2, US3)
- Exact file paths in all descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Cargo workspace initialization and module stubs

- [x] T001 Add `[workspace]` section with `members = ["resort_generator"]` to root `Cargo.toml`
- [x] T002 Create `resort_generator/Cargo.toml` with deps: clap 4 (derive), reqwest 0.12 (json), serde/serde_json 1, chrono 0.4, anyhow 1, tokio 1 (full), tracing 0.1, tracing-subscriber 0.3, mockall 0.13 (dev)
- [x] T003 [P] Create empty module stubs: `resort_generator/src/{main.rs, types.rs, http.rs, overpass.rs, elevation.rs, merge.rs}`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure required by all user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Define `HttpClient` trait (`async fn get`, `async fn post`) and `ReqwestClient` production impl in `resort_generator/src/http.rs`
- [x] T005 [P] Define all shared types (`OverpassResponse`, `Element` enum with `Way`/`Node` variants, `ElevationResponse`, `ResortConfig`) in `resort_generator/src/types.rs`
- [x] T006 [P] Initialize `#[tokio::main]` entry point and `tracing_subscriber::fmt::init()` in `resort_generator/src/main.rs`

**Checkpoint**: Foundation ready -- all three user stories can begin

---

## Phase 3: User Story 1 - Generate merged resort data file (Priority: P1) MVP

**Goal**: Complete pipeline that fetches trail geometry and elevation data, merges them, and writes a timestamped JSON file to `data/`

**Independent Test**: Run `cargo run -p resort_generator -- --resort "Serre Chevalier"`; verify `data/serre_chevalier_YYYYMMDD_HHMMSS.json` exists and contains both trail geometry and elevation values for all nodes

### Tests for User Story 1

> **Write tests FIRST; confirm they FAIL before implementing**

- [x] T007 [P] [US1] Unit test: `build_query("Serre Chevalier")` returns QL string containing `"name"="Serre Chevalier"` and `landuse=winter_sports` -- add to `resort_generator/src/overpass.rs`
- [x] T008 [P] [US1] Unit test: batching 110 nodes produces 3 batches (50+50+10) and calls mock `HttpClient` 3 times with correct pipe-separated coords -- add to `resort_generator/src/elevation.rs`
- [x] T009 [US1] Unit test: `validate_elevation` returns `Err` listing missing node IDs when any way-referenced node has `ele = None` -- add to `resort_generator/src/merge.rs`

### Implementation for User Story 1

- [x] T010 [US1] Implement `overpass.rs`: `build_query(resort_name)` formats Overpass QL, `fetch_trails(client, query)` POSTs to `https://overpass-api.de/api/interpreter`, deserializes `OverpassResponse`, validates `>= 1 Way` and `>= 1 Node` -- `resort_generator/src/overpass.rs`
- [x] T011 [US1] Implement `elevation.rs`: `fetch_elevation(client, nodes)` chunks `Vec<(u64, f64, f64)>` into batches of 50, builds pipe-separated lat/lon params, GETs IGN endpoint (`https://data.geopf.fr/altimetrie/1.0/calcul/alti/rest/elevation.json`), applies 200 ms inter-batch delay, 3-retry exponential backoff (2s/4s/8s), respects `Retry-After` on HTTP 429 -- `resort_generator/src/elevation.rs`
- [x] T012 [US1] Implement `merge.rs`: `merge_elevation(response, elevations)` patches `ele` into each `Node` element; `validate_completeness(response)` collects all way-referenced node IDs and returns `Err` with missing IDs if any `Node` has `ele = None` -- `resort_generator/src/merge.rs`
- [x] T013 [US1] Implement orchestration in `resort_generator/src/main.rs`: call `fetch_trails` -> extract nodes -> `fetch_elevation` -> `merge_elevation` -> `validate_completeness` -> `fs::create_dir_all("data/")` -> serialize to `data/<slug>_YYYYMMDD_HHMMSS.json` -> print path to stdout

**Checkpoint**: US1 fully functional -- `cargo run -p resort_generator` writes a valid JSON file

---

## Phase 4: User Story 2 - Select resort by name at runtime (Priority: P2)

**Goal**: Replace hardcoded resort name with `--resort "<Name>"` CLI argument; output filename uses derived slug

**Independent Test**: Run with `--resort "Chamonix"`; verify output file is named `chamonix_*.json` and contains Chamonix trail data; run with no args and verify exit code 1 with usage message

### Tests for User Story 2

- [x] T014 [P] [US2] Unit test: `ResortConfig::from_name("Serre Chevalier")` sets `filename_slug = "serre_chevalier"` and `resort_name = "Serre Chevalier"` -- add to `resort_generator/src/types.rs`
- [x] T015 [P] [US2] Unit test: `ResortConfig::from_name("Mont Blanc 2000")` sets `filename_slug = "mont_blanc_2000"` -- add to `resort_generator/src/types.rs`

### Implementation for User Story 2

- [x] T016 [US2] Add `--resort <NAME>` argument to clap derive struct in `resort_generator/src/main.rs`; construct `ResortConfig` from parsed argument (clap exits with error if `--resort` is missing)
- [x] T017 [US2] Implement `ResortConfig::from_name(name)` in `resort_generator/src/types.rs`: lowercase + replace spaces with underscores to produce `filename_slug`
- [x] T018 [US2] Pass `config.resort_name` to `build_query()` in `resort_generator/src/overpass.rs`; apply `config.filename_slug` to output filename pattern in `resort_generator/src/main.rs`

**Checkpoint**: US1 + US2 functional -- any resort name produces correctly named output file

---

## Phase 5: User Story 3 - Informative progress and error reporting (Priority: P3)

**Goal**: Operators see per-stage progress via tracing; failures identify the failing step and source; exit codes are reliable

**Independent Test**: Run with network disabled; verify exit code `!= 0` and stderr message identifies which source (Overpass or IGN) failed

### Implementation for User Story 3

- [x] T019 [P] [US3] Add `tracing::info!` at each pipeline stage (fetch trails, extract nodes, fetch elevation, merge, validate, write) in `resort_generator/src/main.rs`
- [x] T020 [P] [US3] Add `tracing::info!` per elevation batch (batch N/total, node count) and `tracing::warn!` on retry in `resort_generator/src/elevation.rs`
- [x] T021 [US3] Wrap each `?` propagation with `.context("stage description")` using `anyhow` throughout `resort_generator/src/{main,overpass,elevation,merge}.rs`
- [x] T022 [US3] Print final error to stderr and call `std::process::exit(1)` on any pipeline failure in `resort_generator/src/main.rs`

**Checkpoint**: All three user stories functional and observable

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T023 [P] Add `#[cfg(feature = "integration")]` gate to any test that calls real external APIs; add `[features] integration = []` to `resort_generator/Cargo.toml`
- [x] T024 Implement collision guard before write in `resort_generator/src/main.rs`: if `output_path.exists()` abort with `anyhow::bail!("output file already exists: {output_path}")` and exit 1 (satisfies FR-007)
- [ ] T025 Run quickstart.md validation: `cargo build -p resort_generator`, then `cargo run -p resort_generator -- --resort "Serre Chevalier"`; verify output file in `data/`, exit code 0, and run completes within 5 minutes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies -- start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 -- BLOCKS all user stories
- **Phase 3 (US1 P1)**: Depends on Phase 2 -- MVP increment
- **Phase 4 (US2 P2)**: Depends on Phase 3 -- parameterizes the working pipeline
- **Phase 5 (US3 P3)**: Depends on Phase 2 -- can start alongside Phase 3 for tracing setup; full integration after Phase 4
- **Phase 6 (Polish)**: Depends on Phases 3-5

### User Story Dependencies

- **US1 (P1)**: Start after Foundational (Phase 2)
- **US2 (P2)**: Depends on US1 -- lifts hardcoded resort name to `--resort` arg
- **US3 (P3)**: Mostly independent; tracing setup (T019-T020) can run in parallel with US1; error propagation (T021-T022) needs US1 complete

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Types before services (`types.rs` before `overpass.rs`, `elevation.rs`)
- Modules before orchestration (`main.rs` wiring is always last)

---

## Parallel Opportunities

```text
Phase 1:  T001 -> (T002, T003 in parallel)
Phase 2:  T004, T005, T006 in parallel
Phase 3 tests: T007, T008 in parallel; then T009
Phase 3 impl:  T010, T011, T012 in parallel; then T013
Phase 4 tests: T014, T015 in parallel
Phase 4 impl:  T016 -> T017 -> T018
Phase 5:  T019, T020 in parallel; then T021 -> T022
Phase 6:  T023 parallel; then T024
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup workspace
2. Phase 2: Foundational infrastructure
3. Phase 3: US1 complete pipeline
4. **STOP and VALIDATE**: `cargo run -p resort_generator` writes valid JSON

### Incremental Delivery

1. Setup + Foundational -> foundation ready
2. US1 complete -> data file generated (MVP)
3. US2 complete -> any resort via `--resort`
4. US3 complete -> operator-friendly output
5. Polish -> integration test gate + quickstart verified

---

## Notes

- `[P]` = different files, no incomplete dependencies
- `[Story]` maps each task to its user story for traceability
- Commit after each phase checkpoint
- No `println!` anywhere -- all output via `tracing::` macros except the final success path print (`println!("{}", output_path)`)
- IGN sentinel value `-99999` is valid -- do not treat as missing elevation
