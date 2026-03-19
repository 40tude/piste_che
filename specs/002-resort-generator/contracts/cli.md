# CLI Contract: resort_generator

## Command

```text
resort_generator --resort "<Name>"
```

## Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `--resort` | Yes | String | Human-readable resort name (e.g., "Serre Chevalier") |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success: file written, path printed to stdout |
| 1 | Error: failure message printed to stderr |

## Output

### Success (exit 0)

Prints full path of written file to stdout:

```text
data/serre_chevalier_20260319_143022.json
```

### Error (exit non-zero)

Prints human-readable error to stderr identifying the failed step:

```text
Error: IGN elevation fetch failed for batch 12/45: HTTP 503 (3 retries exhausted)
```

```text
Error: Overpass API returned no trails for resort "Chamonix"
```

```text
Error: Post-merge validation failed: 3 nodes missing elevation [id: 12345, 67890, 11111]
```

## Output File Format

Standard Overpass API JSON response with `ele` field added to all nodes.

**Filename**: `<slug>_YYYYMMDD_HHMMSS.json`
- Slug: resort name lowercased, spaces replaced with underscores
- Timestamp: local system time at moment of write

**Schema**:

```json
{
  "version": 0.6,
  "generator": "Overpass API ...",
  "osm3s": { "timestamp_osm_base": "...", "copyright": "..." },
  "elements": [
    {
      "type": "way",
      "id": 19652498,
      "nodes": [203954539, 11505420240],
      "tags": { "aerialway": "chair_lift", "name": "Yret" }
    },
    {
      "type": "node",
      "id": 203954539,
      "lat": 44.8987,
      "lon": 6.6192,
      "ele": 1476.5
    }
  ]
}
```

## Compatibility

Output file MUST be loadable by `OsmData::load()` in the main `piste_che` application without modification. The JSON structure matches existing `*_ele.json` files in `data/`.
