<!--
Sync Impact Report
===================
Version change: 1.0.0 -> 1.1.0
Modified principles:
  - IV (Simplicity): removed "No SPA framework" constraint, allow Leptos (WASM)
  - VII (Tech Stack): replaced vanilla JS frontend with Leptos (Rust/WASM)
Added sections: N/A
Removed sections: N/A
Templates requiring updates:
  - .specify/templates/plan-template.md: OK (Constitution Check section is generic)
  - .specify/templates/spec-template.md: OK (user stories + requirements align)
  - .specify/templates/tasks-template.md: OK (phase structure compatible)
  - .specify/templates/agent-file-template.md: OK (generic template)
Follow-up TODOs: None
-->

# Piste Che Constitution

## Core Principles

### I. Preserve Existing Code

The prototype routing module (from `serre_che_proto`) and the ski area
JSON data file MUST be integrated as-is into the project. Do NOT rewrite
graph construction, JSON parsing, or the Dijkstra implementation. New
functionality (Sport/Safe weighting strategies) MUST be added on top of
the existing algorithm. Existing unit tests MUST be preserved and pass.

### II. Graph as Single Source of Truth

The custom-format JSON file describing Serre Chevalier is the sole
authority for the ski area topology. All nodes, edges, altitudes, run
difficulties, and lift types derive from this file. No database or
secondary data store is permitted for MVP. The in-memory weighted
directed graph built at startup is the only runtime representation.

### III. Test-First (NON-NEGOTIABLE)

TDD is mandatory: tests MUST be written and fail before implementation.
Existing routing module tests MUST be preserved unchanged. New unit
tests MUST cover: Sport/Safe weighting strategies, filter logic
(difficulty + lift type), edge cases (no route, start == end, all
segments filtered). Integration tests MUST cover all API endpoints via
reqwest. All tests MUST pass via `cargo test`.

### IV. Simplicity and MVP Focus

Build only what the MVP requires. No database, no auth, no user
accounts, no i18n, no saved itineraries, no real-time status. YAGNI
applies: do not implement future considerations listed in the spec.
Desktop-first; basic responsiveness is sufficient.

### V. Clean Layering

The existing routing module MUST be wrapped behind a clean Rust
interface (trait or module boundary). Axum handlers call that interface;
they do NOT access graph internals directly. The Leptos frontend
communicates with the backend exclusively via server functions or REST
JSON endpoints. No tight coupling between layers.

### VI. Structured Observability

Use `tracing` + `tracing-subscriber` for all logging. Every API request
MUST produce a trace span. Routing computations SHOULD log mode, start,
end, segment count, and total distance. Errors MUST be logged with
structured context. No `println!` in production code paths.

### VII. Mandated Tech Stack

The following stack is mandatory for MVP. Deviations require explicit
justification documented in the plan.

- **Language:** Rust (stable)
- **Web framework:** Axum
- **Async runtime:** Tokio
- **Frontend:** Leptos (Rust/WASM) served by Axum
- **Map:** Leaflet.js with OpenStreetMap tiles (via JS interop from WASM)
- **Serialization:** serde + serde_json
- **CLI:** clap (derive feature) for `--port` flag
- **Error handling:** thiserror (library modules), anyhow (binary crate)
- **Logging:** tracing + tracing-subscriber
- **Testing:** cargo test, reqwest (integration), mockall (mocking)
- **Utilities:** derive_more, itertools, regex, rand (as needed)

## Tech Stack Constraints

- `PORT` env var takes precedence over `--port` CLI flag (Heroku convention)
- A `Procfile` MUST be present for Heroku deployment
- Static assets served from `/static` path by Axum
- API prefix: `/api/` for all JSON endpoints
- No external databases -- all data loaded in-memory from JSON at startup
- Edge weights are distance in meters; Sport/Safe modes apply multipliers
- Runs are color-coded: green, blue, red, black
- Lifts have types: chairlift, gondola, drag lift, cable car
- Filters remove edges from the graph before routing (not post-filtering)

## Development Workflow

1. **Integrate first:** copy existing routing module code into the
   project, verify all existing tests pass before writing new code.
2. **Branch per feature:** each feature gets its own branch.
3. **TDD cycle:** write failing test, implement, refactor. No code
   without a covering test for new functionality.
4. **Commit often:** commit after each task or logical group of changes.
   Commit messages follow `<action>: <what changed>` format, max 50
   chars, US English.
5. **Integration tests last:** API integration tests run after unit
   tests pass for the underlying module.
6. **Local validation:** `cargo test` MUST pass before any push.
7. **Heroku deploy:** validate locally with `cargo run -- --port 3000`,
   then deploy via Heroku Rust buildpack.

## Governance

This constitution supersedes all other development practices for the
Piste Che project. All code changes MUST comply with these principles.

- **Amendments** require updating this document, incrementing the
  version, and recording the amendment date.
- **Versioning** follows semantic versioning: MAJOR for principle
  removals/redefinitions, MINOR for new principles or expanded guidance,
  PATCH for wording clarifications.
- **Compliance** is verified during plan review (Constitution Check gate
  in plan.md) and before merging any feature branch.
- **Complexity justification:** any deviation from Simplicity (Principle
  IV) or Tech Stack (Principle VII) MUST be documented in the plan's
  Complexity Tracking table.

**Version**: 1.1.0 | **Ratified**: 2026-03-18 | **Last Amended**: 2026-03-18
