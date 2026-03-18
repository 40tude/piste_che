# Specify Prompt — Spec 01: Map, Filters & Shortest Route (Local)

## Goal

Deliver the core loop of Piste Che running locally: a user opens the app in a browser, sees the Serre Chevalier ski area on an interactive map, selects start and end points from drop-down menus, applies difficulty and lift-type filters, computes the shortest route, and sees it highlighted on the map with a step-by-step itinerary list.

---

## Scope

### Backend
- Integrate the existing routing module from the prototype repo (https://github.com/40tude/serre_che_proto):
  - The workspace to look at is : https://github.com/40tude/serre_che_proto/tree/main/get_itinerary
  - Reuse its JSON data loader and graph construction as-is
  - Reuse its Dijkstra shortest-distance algorithm as-is
  - Do NOT rewrite or restructure the existing code
- `GET /` — serve the main HTML page
- `GET /api/area` — return the full ski area data (nodes with altitude, runs with difficulty, lifts with type and coordinates) as JSON
- `POST /api/route` — accept `{ start, end, difficulty_filter, lift_type_filter, mode: "short" }` and return the computed route as JSON
  - The `mode` field is present in the API from day one (only `"short"` is implemented now) so that Sport/Safe can be added later without changing the API contract
- Serve static assets (JS, CSS) from `/static`
- Port configurable via `--port` CLI flag (clap) or `PORT` env var (PORT takes precedence — Heroku convention, ready for future deployment)
- Filter logic: when the user excludes certain difficulty levels or lift types, those edges are removed from the graph before routing

### Frontend
- Leaflet.js map (OpenStreetMap tiles, no API key needed) displaying the full ski area:
  - Runs color-coded by difficulty (green, blue, red, black)
  - Lifts displayed with a distinct style (different from runs)
- Start/end point selection via **two drop-down menus** populated from `/api/area`
  - Architect the selection mechanism so that map-click selection can be added later alongside the drop-downs without restructuring
- Filter panel:
  - Run difficulty checkboxes (green, blue, red, black — all checked by default)
  - Lift type checkboxes (chairlift, gondola, drag lift, cable car — all checked by default)
- Three tabs visible: **Short** (active and functional), **Sport** (disabled/greyed out), **Safe** (disabled/greyed out)
  - Tab structure is in place so enabling Sport/Safe later only requires wiring them to new backend logic
- "Calculate" button that calls `POST /api/route` with current selections and filters
- Route display:
  - Route highlighted on the map with a distinct color/weight
  - Step-by-step itinerary list panel next to the map showing: segment name, type (run difficulty or lift type), distance per segment, and total distance

### Testing
- Preserve all existing tests from the prototype routing module
- Unit tests for filter logic (removing edges from graph based on difficulty/lift type selections)
- Integration tests for all three API endpoints (valid requests, invalid inputs, missing fields, no route found)
- All tests pass via `cargo test`

---

## Architecture notes for extensibility

The following features are NOT in scope for this spec but the code must be structured so they can be added cleanly in later specs without refactoring:
- **Map-click selection** (spec 02): the frontend selection logic should be behind an abstraction (e.g., a `setStartPoint(nodeId)` / `setEndPoint(nodeId)` function) so a map click handler can call the same interface
- **Sport and Safe routing modes** (spec 02 or 03): the `mode` parameter already flows through the API; the routing module should accept a weighting strategy so new modes plug in without modifying the core Dijkstra logic
- **Heroku deployment** (spec 02): PORT env var support is already in place; only Procfile and buildpack config are needed later

---

## Out of scope for this spec
- Map-click selection for start/end points
- Sport and Safe routing logic (tabs are visible but disabled)
- Heroku deployment (local only)
- User accounts, saved itineraries
- Real-time lift/run status
- 3D map or elevation profile
