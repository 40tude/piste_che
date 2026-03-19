# Data Model: Resort Data Generator CLI

## Entities

### OverpassResponse

Root structure of the Overpass API JSON response.

| Field | Type | Description |
|-------|------|-------------|
| version | f64 | API version (0.6) |
| generator | String | Generator identifier |
| osm3s | Osm3s | Metadata (timestamp, copyright) |
| elements | Vec\<Element\> | All returned OSM elements |

### Element (tagged enum via `"type"` field)

| Variant | Discriminator | Key fields |
|---------|---------------|------------|
| Way | `"type": "way"` | id, nodes, tags |
| Node | `"type": "node"` | id, lat, lon, ele? |

#### Way

| Field | Type | Description |
|-------|------|-------------|
| id | u64 | OSM way ID |
| nodes | Vec\<u64\> | Ordered node ID references |
| tags | HashMap\<String, String\> | OSM tags (name, piste:type, aerialway, difficulty, etc.) |

#### Node

| Field | Type | Description |
|-------|------|-------------|
| id | u64 | OSM node ID |
| lat | f64 | Latitude (WGS84) |
| lon | f64 | Longitude (WGS84) |
| ele | Option\<f32\> | Elevation in meters; absent before enrichment, present after |

### ElevationResponse

IGN Altimetrie API response.

| Field | Type | Description |
|-------|------|-------------|
| elevations | Vec\<f64\> | Elevation values in same order as request coordinates |

### ResortConfig

Runtime configuration derived from CLI arguments.

| Field | Type | Source |
|-------|------|--------|
| resort_name | String | `--resort` argument, human-readable |
| filename_slug | String | Derived: lowercase, spaces to underscores |
| output_dir | PathBuf | Hardcoded: `data/` |

## Relationships

```text
OverpassResponse.elements
  ├── Way { nodes: [node_id, ...] }
  └── Node { id, lat, lon, ele? }

Way.nodes[i]  ──references──>  Node.id
ElevationResponse.elevations[i]  ──maps to──>  batch[i] coordinate pair
```

## Pipeline State Transitions

```text
1. FETCH_TRAILS   -> OverpassResponse (nodes without ele)
2. EXTRACT_NODES  -> Vec<(node_id, lat, lon)> from all Node elements
3. FETCH_ELEVATION -> ElevationResponse per batch of 50 nodes
4. MERGE          -> Patch ele into each Node in OverpassResponse
5. VALIDATE       -> Verify 100% of way-referenced nodes have ele
6. WRITE          -> Serialize full OverpassResponse to timestamped JSON
```

## Validation Rules

| Rule | When | Action on failure |
|------|------|-------------------|
| Resort name non-empty | CLI parse | clap rejects; exit 1 |
| Overpass response has >= 1 Way | After fetch | Abort with "no trails found" |
| Overpass response has >= 1 Node | After fetch | Abort with "no nodes found" |
| Elevation batch returns expected count | Each batch | Retry up to 3x, then abort |
| All way-referenced nodes have ele | Post-merge | Abort with missing node list |
| Output directory writable | Before write | Abort with IO error |
