# Research: Resort Data Generator CLI

## R1: Overpass API Parameterization

**Decision**: Build Overpass QL query at runtime with resort name as variable.

**Rationale**: Prototype hardcodes "Serre Chevalier" in `data/request.json`. For multi-resort support, format the resort name into the area filter at runtime. The `landuse=winter_sports` tag scopes the query to ski areas.

**Query template**:

```overpassql
[out:json][timeout:25];
area["name"="{resort_name}"]["landuse"="winter_sports"]->.a;
(
  way["piste:type"="downhill"](area.a);
  way["aerialway"](area.a);
);
out body;
>;
out skel qt;
```

**Alternatives considered**:
- External query file with placeholder: rejected (unnecessary indirection)
- Bounding box instead of area name: rejected (requires geocoding, less precise)

## R2: IGN Altimetrie API Strategy

**Decision**: Port existing batch strategy from `get_elevation` prototype.

**Rationale**: Proven approach, already tuned for IGN rate limits:
- Batch size: 50 coordinates (URL length constraint)
- Inter-batch delay: 200 ms
- Retry: 3 attempts, exponential backoff (2 s, 4 s, 8 s)
- HTTP 429: respect `Retry-After` header (default 60 s)
- Client timeout: 60 s per request

**API endpoint**: `https://data.geopf.fr/altimetrie/1.0/calcul/alti/rest/elevation.json`
**Parameters**: `lon`, `lat` (pipe-separated), `resource=ign_rge_alti_wld`, `zonly=true`
**Response**: `{"elevations": [1234.5, ...]}`

**Alternatives considered**:
- Parallel batch requests: rejected (triggers 429s)
- Alternative elevation API: rejected (IGN is authoritative for French Alps)

## R3: Workspace Integration

**Decision**: Add `[workspace]` section to root `Cargo.toml`; `resort_generator` as a member.

**Rationale**: Root stays the `piste_che` Leptos package. The workspace section just declares additional members. `cargo-leptos` reads `[package.metadata.leptos]` from the root package, unaffected by the workspace declaration.

```toml
[workspace]
members = ["resort_generator"]
```

**Alternatives considered**:
- Separate repository: rejected (spec says workspace member)
- Shared type crate: rejected (JSON format is the contract; avoids coupling CLI to Leptos dependency tree)

## R4: Async vs Blocking

**Decision**: Async tokio + reqwest.

**Rationale**: Constitution mandates Tokio. `tokio::time::sleep` for inter-batch delays is cleaner than `std::thread::sleep`. Prototype uses blocking reqwest; conversion to async is mechanical.

**Alternatives considered**:
- Blocking reqwest: rejected (constitution mandates Tokio)

## R5: Testing Strategy

**Decision**: Trait-based HTTP abstraction for unit tests; live integration tests behind feature flag.

**Rationale**: Constitution III (Test-First) is non-negotiable. External API calls must be mockable.

**Approach**:
- `HttpClient` trait with `async fn get(url) -> Result<String>` and `async fn post(url, body) -> Result<String>`
- Production: `ReqwestClient` struct
- Tests: mock implementation returning canned Overpass/IGN JSON
- Integration tests: `#[cfg(feature = "integration")]` for live API tests

**Alternatives considered**:
- Record/replay (wiremock): overkill for this tool
- No mocking (real APIs only): rejected (flaky, slow)

## R6: Output Validation

**Decision**: Post-merge validation that 100% of way-referenced nodes have elevation before writing.

**Rationale**: Spec edge cases note a gap in the prototype where zip mismatch could silently drop elevation for some nodes. New tool validates completeness: count nodes with `ele` vs total way-referenced nodes; abort on mismatch.

**Approach**:
- After elevation merge, collect all node IDs referenced by ways
- Verify each has a non-None `ele` value
- Abort with descriptive error if any are missing
- Write file only after validation passes

**Alternatives considered**:
- Write partial file with warnings: rejected (spec says no partial output)
