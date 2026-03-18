# Constitution Prompt — Piste Che

Write the constitution for **Piste Che**, a web application that creates ski itineraries for the Serre Chevalier ski area.

---

## Product Overview

Piste Che helps skiers plan their route across the Serre Chevalier ski domain. Users select a starting point and a destination, apply filters (run difficulty, lift types), and the app calculates and displays an optimized itinerary on an interactive map.

---

## Tech Stack

- **Language:** Rust
- **Web framework:** Axum
- **Async runtime:** Tokio
- **Frontend:** Server delivers static HTML + Bootstrap 5 + vanilla JavaScript (no SPA framework)
- **Map:** Leaflet.js with OpenStreetMap tiles
- **Serialization:** serde + serde_json
- **CLI:** clap (derive feature) — for the `--port` flag
- **Error handling:** thiserror for library-style modules, anyhow for the binary crate
- **Logging:** tracing + tracing-subscriber (structured, async-aware)
- **Testing:** cargo test, reqwest for integration tests, mockall for mocking
- **Utilities:** derive_more, itertools, regex, rand (as needed)

---

## Existing Assets (do NOT rewrite — integrate as-is)

**Prototype repository:** https://github.com/40tude/serre_che_proto

The project includes **pre-existing, tested Rust code** that must be reused. This code solves the hardest domain problems and must not be rebuilt from scratch.

### 1. Ski Area JSON Data File
- A **custom-format JSON file** describing the entire Serre Chevalier ski area
- Contains: runs (with difficulty level), lifts (with type), and nodes/points (with **altitude**)
- Altitude is used to determine direction: runs go downhill (higher → lower altitude), lifts go uphill (lower → higher altitude)
- This file is the **single source of truth** for the ski area graph

### 2. Routing Module (Rust crate/module)
- A **tested Rust module** that:
  - **Loads and parses** the JSON data file into an in-memory graph
  - Implements **Dijkstra's shortest-distance algorithm** on the ski area graph
  - Already handles real-world complexity:
    - Two runs can cross geographically without having a shared junction node
    - The end of a run does not always coincide with the start of a lift
    - All such edge cases are already solved in the existing code
- Currently supports **shortest distance mode only**
- **Sport and Safe modes must be added** on top of the existing algorithm as new weighting strategies (not a rewrite)

**Integration approach:** Copy the existing module code into the project. Wrap it behind a clean interface that the Axum handlers call. Extend it with Sport/Safe weighting — do not restructure the existing graph construction or parsing logic.

---

## Architecture

- REST API backend (Axum) serving JSON endpoints
- Static frontend assets (HTML, CSS, JS) served by Axum
- Ski area data loaded at startup by the **existing routing module** from its JSON file (custom format)
- The ski area is modeled internally (by the existing module) as a **weighted directed graph**:
  - **Nodes** = points of interest (top/bottom of lifts, junctions between runs), each with **altitude**
  - **Edges** = runs (downhill, with difficulty: green/blue/red/black) and lifts (uphill, with type: chairlift, gondola, drag lift, cable car)
  - **Edge weights** = distance in meters (extended with multipliers for Sport/Safe modes)
- No database for MVP — all data is in-memory after startup

---

## Core Features (MVP)

### 1. Interactive Map
- Display the full Serre Chevalier ski area on a Leaflet.js map
- Runs are color-coded by difficulty (green, blue, red, black)
- Lifts are displayed distinctly from runs
- Users can select start and end points by:
  - Choosing from **drop-down menus**, OR
  - **Clicking directly on the map**
- The calculated route is **highlighted on the map**

### 2. Filters
- **Run difficulty filter:** checkboxes to include/exclude green, blue, red, black runs
- **Lift type filter:** checkboxes to include/exclude chairlift, gondola, drag lift, cable car
- Filtered-out segments are **removed from the graph** before routing

### 3. Routing — Three Modes (tabs in the UI)
All three modes use **Dijkstra's algorithm** on the directed graph, with different edge weighting strategies:

- **Short:** minimize total distance (standard Dijkstra on distance weights)
- **Sport:** prefer harder runs — apply a discount multiplier to red/black run edges so they are favored even if physically longer
- **Safe:** prefer easier runs — apply a discount multiplier to green/blue run edges so they are favored; penalize red/black edges

The UI presents the three modes as **tabs**. Each tab shows:
- The route highlighted on the map
- A **step-by-step itinerary list** next to the map (run name, difficulty, lift name, lift type, distance per segment, total distance)

### 4. Route Display
- Route shown as an ordered list of segments (run or lift) with:
  - Segment name
  - Type (run difficulty or lift type)
  - Distance
- Total distance displayed
- Route highlighted on the map with a distinct color/weight

---

## API Endpoints

- `GET /` — serves the main HTML page (map + controls)
- `GET /api/area` — returns the full ski area data (nodes, runs, lifts) as JSON
- `POST /api/route` — accepts JSON body with `{ start, end, difficulty_filter, lift_type_filter, mode }` and returns the computed route as JSON
- Static assets served from a `/static` path

---

## Deployment

- Run locally: `cargo run -- --port 3000` (port configurable via `--port` CLI flag or `PORT` env var)
- `PORT` env var takes precedence over `--port` CLI flag (Heroku convention)
- Deploy on Heroku using Rust buildpack
- Procfile included in the repository

---

## Quality & Testing (TDD)

- **Existing routing module** already has its own unit tests — preserve them
- **New unit tests** for:
  - Sport/Safe weighting strategies (correct mode selection, weight adjustments)
  - Filter logic (difficulty, lift types — removing edges from graph)
  - Edge cases: no route found, start == end, all segments filtered out
- **Integration tests** for API endpoints using reqwest:
  - Valid route requests (all three modes)
  - Invalid inputs (unknown nodes, malformed JSON, missing fields)
  - Area data endpoint
- All tests runnable via `cargo test`

---

## Non-Goals (out of scope for MVP)

- No API versioning
- No authentication or user accounts
- No saved itineraries or favorites
- No real-time lift/run status (open/closed)
- No multilingual support (English only)
- No database
- No mobile-specific responsive design (desktop-first, basic responsiveness via Bootstrap is fine)

---

## UI Language

English only.

---

## Future Considerations (post-MVP, do not implement now)

- Real-time lift/run status integration
- User accounts and saved itineraries
- French language support
- Click-to-select improvements (snapping to nearest node)
- Elevation profile for routes
- Estimated time per route (factoring lift wait times)
- 3D map visualization (altitude data is already available in the dataset)
