# Implementation Plan: Map, Filters & Shortest Route

**Branch**: `001-map-filters-route` | **Date**: 2026-03-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-map-filters-route/spec.md`

## Summary

Build the core Piste Che MVP: an interactive Leaflet map of Serre Chevalier, start/end point selection via dropdowns, difficulty/lift-type filter checkboxes, shortest-route Dijkstra computation, route highlight on map, and step-by-step itinerary panel. Integrates the existing prototype routing module from `020_serre_che_1/get_itinerary` as-is. Leptos SSR with Axum via cargo-leptos produces a single deployable binary.

## Technical Context

**Language/Version**: Rust stable (edition 2024, requires Rust 1.85+)
**Primary Dependencies**: Axum, Tokio, Leptos (SSR + hydrate), leptos_axum, leptos_router, leptos-leaflet, serde/serde_json, clap (derive), tracing/tracing-subscriber, wasm-bindgen (JS interop fallback), thiserror (library error types), anyhow (binary error handling)
**Storage**: In-memory directed graph built at startup from bundled JSON (no database)
**Testing**: cargo test (unit), reqwest (integration), mockall (mocking)
**Target Platform**: Local web server (Axum) + browser WASM (Leptos hydrate)
**Project Type**: Web application (full-stack SPA with SSR)
**Performance Goals**: Map load <3s, route compute <2s (SC-001, SC-002)
**Constraints**: Single user local, ~200 graph nodes, ~500 edges, 2.7 MB JSON data file
**Scale/Scope**: 6 UI components, 2 server functions, 1 routing module (4 source files)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Preserve Existing Code | PASS | Prototype routing module (data.rs, graph.rs, chains.rs, dijkstra) copied into `src/routing/` as-is. Existing tests preserved. |
| II | Graph as Single Source of Truth | PASS | JSON loaded at startup into in-memory graph. No database. |
| III | Test-First (NON-NEGOTIABLE) | PASS | TDD cycle planned. Existing tests preserved. New tests: filter logic, edge cases, API integration via reqwest. |
| IV | Simplicity and MVP Focus | PASS | CSR-heavy map app. No database, auth, i18n, saved itineraries. Desktop-first. |
| V | Clean Layering | PASS | Routing module behind `src/routing/mod.rs` public interface. Server functions call that interface. Leptos frontend communicates via server functions under `/api/`. |
| VI | Structured Observability | PASS | tracing + tracing-subscriber. Spans on every server function call. Route computation logs mode, start, end, segment count, total distance. No println! |
| VII | Mandated Tech Stack | PASS | All mandated crates used. Additions: leptos, leptos_axum, leptos_router, leptos-leaflet, wasm-bindgen (required for Leptos SSR + Leaflet interop). |

**Gate result**: PASS -- proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/001-map-filters-route/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── api.md           # REST endpoint contracts
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
Cargo.toml               # Single crate: leptos features (ssr, hydrate), all deps
Cargo.lock
Procfile                  # Heroku: web: ./target/release/piste_che

src/
├── main.rs               # Server entry (#[cfg(feature = "ssr")]): CLI, graph init, Axum setup
├── lib.rs                 # Shared module declarations, hydrate entry point
├── app.rs                 # Root Leptos component, layout shell
├── routing/               # Prototype routing module (server-only, #[cfg(feature = "ssr")])
│   ├── mod.rs             # Public interface: re-exports, query functions
│   ├── data.rs            # OsmData, JSON parsing, haversine (from prototype)
│   ├── graph.rs           # Node, Segment, RouteElement, build_graph (from prototype)
│   ├── chains.rs          # Chain building (from prototype)
│   └── dijkstra.rs        # Dijkstra algorithm, segment_length (extracted from prototype main.rs)
├── server/                # Server functions (#[cfg(feature = "ssr")])
│   ├── mod.rs
│   └── api.rs             # get_area(), compute_route() server functions
├── models.rs              # Shared DTOs (Serialize+Deserialize, used by server + client)
└── components/            # Leptos UI components
    ├── mod.rs
    ├── map.rs             # Leaflet map initialization + update (JS interop via wasm-bindgen)
    ├── filters.rs         # Difficulty + lift type checkboxes
    ├── selector.rs        # Start/end point dropdowns (abstracted for future map-click)
    ├── itinerary.rs       # Step-by-step route panel with distances
    └── mode_tabs.rs       # Short (active) / Sport (disabled) / Safe (disabled) tabs

style/
└── main.css               # Layout, map container, filter panel, itinerary styling

public/                    # Static assets served at /
├── leaflet.js             # Leaflet library (self-hosted, no CDN dependency)
├── leaflet.css
└── favicon.ico

data/
└── 20260315_164849_ele.json  # Serre Chevalier ski area data (OSM-derived)

tests/
└── integration.rs         # Reqwest tests: GET /api/area, POST /api/route
```

**Structure Decision**: Single crate with Leptos `ssr`/`hydrate` feature flags, built via cargo-leptos. Server-only code (`routing/`, `server/`, `main.rs`) gated behind `#[cfg(feature = "ssr")]`. Components and models are shared. This avoids workspace complexity while maintaining clean layering.

## Complexity Tracking

No constitution violations. All mandated technologies used without deviation. Additional crates (leptos, wasm-bindgen, js-sys, web-sys) are necessary infrastructure for the Leptos+Leaflet stack, not complexity additions.

## Post-Design Constitution Re-Check

| # | Principle | Status | Design Impact |
|---|-----------|--------|---------------|
| I | Preserve Existing Code | PASS | 4 prototype files copied to `src/routing/`. Dijkstra extracted from main.rs into dijkstra.rs. No logic changes. |
| II | Graph as Single Source of Truth | PASS | `AppState` holds the graph in `Arc`. Server functions read it. No secondary store. |
| III | Test-First | PASS | Prototype had no formal tests (CLI binary). New unit tests (filters, edge cases) and integration tests (reqwest) planned per TDD. |
| IV | Simplicity | PASS | Single crate, single binary. JS glue for Leaflet (6 functions). No over-engineering. |
| V | Clean Layering | PASS | `routing/mod.rs` public interface -> `server/api.rs` server functions -> `components/` UI. No layer bypasses. |
| VI | Structured Observability | PASS | tracing spans on server functions. Route computation logs start/end/mode/segments/distance. |
| VII | Mandated Tech Stack | PASS | All mandated crates present. Leptos stack additions are necessary, not deviations. |

**Post-design gate**: PASS

## Agent Context Update

The `update-agent-context.ps1` script failed due to a PowerShell compatibility issue (`New-TemporaryFile` cmdlet not found). No CLAUDE.md was generated. This is a tooling bug, not a plan issue. Technology context was parsed correctly:
- Language: Rust stable (edition 2024)
- Framework: Axum, Tokio, Leptos SSR, wasm-bindgen
- Storage: In-memory graph from JSON
- Project type: Full-stack SPA with SSR
