# Feature Specification: Map, Filters & Shortest Route (Local)

**Feature Branch**: `001-map-filters-route`
**Created**: 2026-03-18
**Status**: Draft
**Input**: User description: "Deliver the core loop of Piste Che running locally: interactive map, start/end selection, difficulty/lift-type filters, shortest route computation, route highlighted on map with step-by-step itinerary list."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Ski Area Map (Priority: P1)

A skier opens the Piste Che application in a browser and immediately sees the full Serre Chevalier ski area displayed on an interactive map. Runs are color-coded by difficulty (green, blue, red, black) and lifts are shown in a visually distinct style. The user can pan and zoom to explore the area.

**Why this priority**: Without the map, no other feature is usable. This is the visual foundation of the entire application.

**Independent Test**: Can be fully tested by opening the app in a browser and verifying the ski area is displayed with correct color-coding and interactivity.

**Acceptance Scenarios**:

1. **Given** the application is running locally, **When** the user navigates to the home page, **Then** an interactive map of Serre Chevalier is displayed with all runs and lifts visible.
2. **Given** the map is loaded, **When** the user inspects run colors, **Then** green runs appear green, blue runs appear blue, red runs appear red, and black runs appear black.
3. **Given** the map is loaded, **When** the user compares lifts to runs visually, **Then** lifts are clearly distinguishable from runs by style (color, dash pattern, or weight).

---

### User Story 2 - Compute Shortest Route (Priority: P2)

A skier selects a starting point and an ending point from drop-down menus, clicks "Calculate", and sees the shortest route highlighted on the map along with a step-by-step itinerary panel listing each segment (name, type, distance) and the total distance.

**Why this priority**: This is the core value proposition -- finding the shortest path between two points on the ski area.

**Independent Test**: Can be tested by selecting any two connected points, clicking Calculate, and verifying the route appears on the map and in the itinerary panel.

**Acceptance Scenarios**:

1. **Given** the map is loaded, **When** the user selects a start point and end point from the drop-downs and clicks "Calculate", **Then** the shortest route is highlighted on the map in a distinct color/weight.
2. **Given** a route has been computed, **When** the user views the itinerary panel, **Then** each segment shows its name, type (run difficulty or lift type), and distance, and the total distance is displayed.
3. **Given** the user selects two points with no possible path between them, **When** they click "Calculate", **Then** a clear message indicates no route was found.
4. **Given** the user selects the same point as start and end, **When** they click "Calculate", **Then** the system handles it gracefully (empty route or informative message).

---

### User Story 3 - Filter by Difficulty and Lift Type (Priority: P3)

A skier uses filter checkboxes to exclude certain run difficulties (e.g., uncheck "black") or certain lift types (e.g., uncheck "drag lift") before computing a route. The route computation only considers segments matching the active filters. Filtered-out segments are dimmed/greyed on the map (still visible for spatial context but visually de-emphasized).

**Why this priority**: Filtering personalizes the experience for different skill levels (beginners avoid black runs, etc.) but requires routing to work first.

**Independent Test**: Can be tested by unchecking a difficulty filter, computing a route, and verifying the result avoids excluded segment types.

**Acceptance Scenarios**:

1. **Given** the filter panel is visible, **When** the page loads, **Then** all difficulty checkboxes (green, blue, red, black) and all lift type checkboxes (chairlift, gondola, drag lift, cable car) are checked by default.
2. **Given** the user unchecks "black" difficulty, **When** they compute a route, **Then** the result contains no black run segments.
3. **Given** the user unchecks "drag lift", **When** they compute a route, **Then** the result contains no drag lift segments.
4. **Given** the user unchecks all run difficulties, **When** they compute a route, **Then** the system either finds a lift-only path or reports no route found.

---

### User Story 4 - Future Mode Tabs (Priority: P4)

The interface shows three routing mode tabs: "Short" (active and functional), "Sport" (disabled/greyed out), and "Safe" (disabled/greyed out). Only "Short" is usable in this version.

**Why this priority**: This is purely visual scaffolding for future features. No logic required beyond showing disabled tabs.

**Independent Test**: Can be tested by verifying the three tabs are visible, "Short" is selectable, and "Sport"/"Safe" are visually disabled and non-functional.

**Acceptance Scenarios**:

1. **Given** the app is loaded, **When** the user views the mode tabs, **Then** three tabs are visible: "Short", "Sport", "Safe".
2. **Given** the tabs are displayed, **When** the user clicks "Short", **Then** it is active and route computation uses shortest-distance mode.
3. **Given** the tabs are displayed, **When** the user clicks "Sport" or "Safe", **Then** nothing happens (tabs are disabled/greyed out).

---

### Edge Cases

- What happens when no route exists between two points after applying filters? The system displays a clear "no route found" message.
- What happens when the user selects the same node as start and end? The system handles it gracefully without crashing.
- What happens when all difficulty filters are unchecked? Only lift-only paths are considered; if none exist, "no route found" is shown.
- What happens when all lift type filters are unchecked? Only downhill run paths are considered; if none connect start to end, "no route found" is shown.
- What happens when both all difficulties and all lift types are unchecked? "No route found" is displayed.
- What happens if the ski area data fails to load? The user sees an error message instead of a broken map.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display the full Serre Chevalier ski area on an interactive, pannable, zoomable Leaflet map using OpenStreetMap tiles.
- **FR-002**: System MUST color-code ski runs by difficulty: green, blue, red, black.
- **FR-003**: System MUST display lifts with a visually distinct style from runs.
- **FR-004**: System MUST provide two drop-down menus for start and end point selection, populated with lift base station names.
- **FR-005**: System MUST provide difficulty filter checkboxes (green, blue, red, black), all checked by default.
- **FR-006**: System MUST provide lift type filter checkboxes (chairlift, gondola, drag lift, cable car), all checked by default.
- **FR-007**: System MUST compute the shortest-distance route between selected points, respecting active filters.
- **FR-008**: System MUST exclude filtered-out run difficulties and lift types from route computation (edges removed from graph before routing).
- **FR-009**: System MUST highlight the computed route on the map with a distinct color and weight.
- **FR-009b**: System MUST dim/grey filtered-out segments on the map (visible but de-emphasized) when any filter is unchecked.
- **FR-010**: System MUST display a step-by-step itinerary panel showing: segment name, type (run difficulty or lift type), distance per segment in meters, and total distance in meters.
- **FR-011**: System MUST display three mode tabs: "Short" (active), "Sport" (disabled), "Safe" (disabled).
- **FR-012**: System MUST allow port configuration via command-line flag or environment variable (environment variable takes precedence).
- **FR-013**: System MUST serve the application locally as a single-page web application built with Leptos (Rust/WASM) and accessible via browser.
- **FR-014**: System MUST provide ski area data (nodes with altitude, runs with difficulty, lifts with type and coordinates) via a data endpoint.
- **FR-015**: System MUST accept route requests with start, end, difficulty filter, lift type filter, and mode parameters.
- **FR-016**: System MUST return a clear error message when no route is found.
- **FR-017**: System MUST reuse the existing prototype routing module (data loader, graph construction, Dijkstra algorithm) without rewriting or restructuring.
- **FR-018**: System MUST write and pass new unit tests covering routing edge cases (no route, start == end, all segments filtered) as mandated by Principle III. Note: the prototype has no existing test files (confirmed in research.md R5); this requirement targets new test coverage, not preservation of prior tests.
- **FR-019**: System MUST architect the point selection mechanism so that map-click selection can be added alongside drop-downs in a future version without restructuring.
- **FR-020**: System MUST accept a routing mode parameter from the start so that Sport/Safe modes can be wired in later without changing the interface contract.

### Key Entities

- **Node**: A lift base station in the ski area with a name and altitude value. Displayed by station name in drop-downs. Serves as start/end point for route selection.
- **Run**: A ski run connecting two nodes. Has a difficulty level (green, blue, red, or black), a name, and a distance. Represents downhill travel.
- **Lift**: A ski lift connecting two nodes. Has a type (chairlift, gondola, drag lift, or cable car), a name, and a distance. Represents uphill travel.
- **Route**: An ordered sequence of segments (runs and lifts) connecting a start node to an end node, with a total distance.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users see the full ski area map within 3 seconds of opening the application.
- **SC-002**: Route computation and display completes within 2 seconds of clicking "Calculate".
- **SC-003**: Users can complete the full workflow (open app, select points, apply filters, compute route, read itinerary) in under 30 seconds on first use.
- **SC-004**: 100% of existing prototype routing module tests pass without modification.
- **SC-005**: All four run difficulty levels and all four lift types are visually distinguishable on the map.
- **SC-006**: Filtered routes never contain segments of excluded types.
- **SC-007**: The system correctly reports "no route found" for all disconnected start/end pairs after filtering.

## Clarifications

### Session 2026-03-18

- Q: What frontend technology will serve the SPA UI? → A: Leptos (Rust WASM framework)
- Q: Which JS map library for the interactive map? → A: Leaflet + OpenStreetMap tiles (free, no API key)
- Q: What happens to filtered-out segments on the map? → A: Dim/grey them (visible but de-emphasized)
- Q: What distance unit for the itinerary panel? → A: Meters (e.g., "1200 m")
- Q: How should nodes be displayed in start/end drop-downs? → A: Lift base station name

## Assumptions

- The existing prototype routing module (JSON data loader, graph construction, Dijkstra algorithm) is stable and correct; it will be integrated as-is.
- Ski area data is loaded from static JSON files bundled with the application.
- The frontend is built with Leptos (Rust/WASM), keeping the full stack in Rust with no separate JS build pipeline.
- The interactive map uses Leaflet (via JS interop from WASM) with free OpenStreetMap tiles; no API key required.
- The application runs locally only; no remote deployment in this version.
- No user authentication or session management is needed.
- The point selection mechanism uses an abstraction layer (e.g., shared setter functions) so map-click selection can be added in a future spec without restructuring.
- The routing interface includes a mode parameter from day one; only "short" (shortest distance) is implemented.
- Port defaults to a sensible value if neither CLI flag nor environment variable is set.

## Out of Scope

- Map-click selection for start/end points (future spec).
- Sport and Safe routing logic (tabs visible but disabled).
- Remote/cloud deployment (local only).
- User accounts, saved itineraries, or personalization.
- Real-time lift/run status or conditions.
- 3D map view or elevation profile.
