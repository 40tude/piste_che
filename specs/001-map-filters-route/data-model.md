# Data Model: Map, Filters & Shortest Route

## Internal Types (from prototype routing module)

### Node

Graph vertex representing a unique geographic position in the ski domain.

| Field | Type | Description |
|-------|------|-------------|
| id | usize | Sequential index (0..n) |
| coord | [f64; 3] | [latitude, longitude, elevation_m] |

Clustered from polyline endpoints + split points. Two candidates within 25 m merge into one node.

### Segment

Directed arc connecting two nodes. One segment per consecutive node pair along a polyline.

| Field | Type | Description |
|-------|------|-------------|
| id | usize | Sequential index |
| from | usize | Source node ID |
| to | usize | Target node ID |
| name | String | Group key (lift: "Name [type Np]", piste: name) |
| kind | String | "piste", "lift", "traverse", "ski-out", "ski-in" |
| difficulty | String | Piste: "novice"/"easy"/"intermediate"/"advanced"/"freeride". Lift: aerialway sub-type. Synthetic: "-" |
| coords | Vec<[f64; 3]> | Ordered [lat, lon, ele] points along the arc |

### RouteElement

Named ski domain element with entry/exit node IDs. Used to populate start/end dropdowns.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Group key (display name) |
| kind | String | "piste" or "lift" |
| difficulty | String | Same encoding as Segment |
| start_node | usize | Entry node (top of piste, base of lift) |
| end_node | usize | Exit node (bottom of piste, top of lift) |

## API DTOs (new, Serialize + Deserialize)

### AreaNode

Simplified node for frontend consumption.

| Field | Type | JSON key |
|-------|------|----------|
| id | usize | `id` |
| lat | f64 | `lat` |
| lon | f64 | `lon` |
| alt | f64 | `alt` |

### AreaSegment

Segment projected for map rendering.

| Field | Type | JSON key |
|-------|------|----------|
| id | usize | `id` |
| name | String | `name` |
| kind | String | `kind` |
| difficulty | String | `difficulty` |
| coords | Vec<[f64; 2]> | `coords` (lat/lon pairs, no elevation) |

### SelectableElement

Dropdown entry for start/end selection. Only lift-type RouteElements are selectable (FR-004: "populated with lift base station names").

| Field | Type | JSON key |
|-------|------|----------|
| name | String | `name` |
| kind | String | `kind` |
| difficulty | String | `difficulty` |

### RouteRequest

Client-to-server route computation request.

| Field | Type | JSON key | Validation |
|-------|------|----------|------------|
| start | String | `start` | Must match a SelectableElement name |
| end | String | `end` | Must match a SelectableElement name |
| excluded_difficulties | Vec\<String\> | `excluded_difficulties` | Valid: "novice", "easy", "intermediate", "advanced", "freeride" |
| excluded_lift_types | Vec\<String\> | `excluded_lift_types` | Valid: aerialway sub-types from data |
| mode | String | `mode` | "short" (only value implemented) |

### RouteStep

One step in the itinerary panel.

| Field | Type | JSON key |
|-------|------|----------|
| name | String | `name` |
| kind | String | `kind` |
| difficulty | String | `difficulty` |
| distance_m | u32 | `distance_m` |

### RouteResponse

Server-to-client route computation result.

| Field | Type | JSON key |
|-------|------|----------|
| steps | Vec\<RouteStep\> | `steps` |
| total_distance_m | u32 | `total_distance_m` |
| highlight_coords | Vec\<Vec\<[f64; 2]\>\> | `highlight_coords` (segment polylines for map overlay) |

### RouteError

Returned when routing fails.

| Field | Type | JSON key |
|-------|------|----------|
| message | String | `message` |

## Difficulty Mapping (OSM -> UI)

| OSM value | UI color | UI label |
|-----------|----------|----------|
| novice | green | Green |
| easy | blue | Blue |
| intermediate | red | Red |
| advanced | black | Black |
| freeride | black (dashed) | Freeride |

## Lift Type Mapping (OSM -> UI)

| OSM aerialway value | UI label | Filter category |
|---------------------|----------|-----------------|
| chair_lift | Chairlift | chairlift |
| gondola | Gondola | gondola |
| platter, drag_lift | Drag lift | drag_lift |
| cable_car | Cable car | cable_car |
| magic_carpet | Magic carpet | (not filterable, always included) |

## State Transitions

No entity state machines. The graph is immutable after startup. Filters are applied per-request by excluding edges before Dijkstra traversal (not post-filtering).

## Relationships

```
Node 1---* Segment (from/to)
Segment *---1 RouteElement (by name grouping)
RouteElement 1---1 Node (start_node)
RouteElement 1---1 Node (end_node)
Route = ordered Vec<Segment> with total distance
```
