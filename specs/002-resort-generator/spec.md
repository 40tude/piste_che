# Feature Specification: Resort Data Generator CLI

**Feature Branch**: `002-resort-generator`
**Created**: 2026-03-19
**Status**: Draft
**Input**: User description: "Add a new Cargo workspace member resort_generator: a single CLI binary that fetches trail and elevation data for a ski resort, merges the results, and writes a timestamped JSON file to data/ folder."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate merged resort data file (Priority: P1)

A developer or data operator needs to refresh the ski resort dataset. They run the tool with a resort name and it fetches trail geometry and elevation data from external sources, merges them, and writes a single timestamped file to the `data/` folder. The main application can then serve the updated data on next startup.

**Why this priority**: Core deliverable -- without this, no other story is possible. Produces the data file the application depends on.

**Independent Test**: Run the tool for Serre Chevalier; verify a timestamped JSON file appears in `data/` containing both trail geometry and elevation values for all segments.

**Acceptance Scenarios**:

1. **Given** valid network access to data sources, **When** operator runs the tool for "serre_chevalier", **Then** a file named `serre_chevalier_YYYYMMDD_HHMMSS.json` appears in `data/` within 5 minutes.
2. **Given** a generated output file, **When** the main application starts, **Then** it can load and parse the file without transformation or post-processing.
3. **Given** the tool is run twice in sequence, **When** both runs succeed, **Then** two separate timestamped files exist in `data/` (no overwrite).

---

### User Story 2 - Select resort by name at runtime (Priority: P2)

A developer wants to generate data for a different resort (e.g., Chamonix or Montgenevre) without modifying code. They pass the resort name as an argument and the tool fetches and outputs data scoped to that resort.

**Why this priority**: Future-proofing the tool for multi-resort use is a stated requirement; the resort name must be a runtime parameter, not a compile-time constant.

**Independent Test**: Run the tool with "chamonix" as the resort name; verify the output filename starts with "chamonix_" and contains trail data for the Chamonix area.

**Acceptance Scenarios**:

1. **Given** a valid resort name argument, **When** operator runs the tool, **Then** the output file is named `<resort_name>_YYYYMMDD_HHMMSS.json` in `data/`.
2. **Given** no resort name argument, **When** operator runs the tool, **Then** the tool exits with a clear error message listing required arguments.

---

### User Story 3 - Informative progress and error reporting (Priority: P3)

An operator runs the tool and needs to know whether it succeeded, what file was written, and -- if something went wrong -- what step failed and why.

**Why this priority**: Operators run this manually and infrequently; clear feedback reduces guesswork and re-runs.

**Independent Test**: Disconnect network mid-run; verify the tool prints a clear error identifying which data source failed, then exits with a non-zero status code.

**Acceptance Scenarios**:

1. **Given** a successful run, **When** the tool finishes, **Then** it prints the full path of the written file and exits with status code 0.
2. **Given** a data source is unreachable, **When** the tool fails, **Then** it prints an error message identifying which step failed and exits with a non-zero status code.
3. **Given** partial data (trails found, elevation unavailable), **When** the tool detects incomplete data, **Then** it exits with an error rather than writing an incomplete file.

---

### Edge Cases

- If the `data/` folder does not exist, the tool auto-creates it before writing the output file.
- How does the tool handle a resort name with no results from the data source?
- What happens if a previously written file for the same resort and timestamp already exists (collision)?
- Elevation failure handling (matches existing `get_elevation` behavior): if any batch fails after 3 retries, the tool aborts with no output written. Non-numeric elevation values (null) cause batch failure. IGN sentinel values (-99999) are accepted as-is. After all batches, the tool SHOULD verify that every collected node received an elevation value before writing (improvement over prototype which had a silent zip-mismatch gap).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Tool MUST accept a resort name via `--resort "<Name>"` (e.g., `--resort "Serre Chevalier"`). The name is a human-readable string used directly in the Overpass API area name query. The filename slug is derived by lowercasing and replacing spaces with underscores.
- **FR-002**: Tool MUST fetch trail and geographic boundary data for the specified resort from an external source.
- **FR-003**: Tool MUST fetch elevation data from the IGN Altimetrie API (`https://data.geopf.fr/altimetrie/1.0/calcul/alti/rest/elevation.json`, resource `ign_rge_alti_wld`, ~5 m precision). Coordinates are sent in batches of 50 with pipe-separated lat/lon values, 200 ms inter-batch delay, and exponential-backoff retry on transient errors (including HTTP 429 with `Retry-After` support).
- **FR-004**: Tool MUST merge trail geometry and elevation data into a single unified dataset before writing output.
- **FR-005**: Tool MUST write the merged dataset to the `data/` folder as a single JSON file.
- **FR-006**: Output filename MUST follow the pattern `<resort_name>_YYYYMMDD_HHMMSS.json` using the local system time at the moment of writing.
- **FR-007**: Tool MUST NOT overwrite existing files; each run produces a new timestamped file.
- **FR-008**: Tool MUST exit with a non-zero status code and a descriptive message on any failure (network, parsing, or I/O).
- **FR-009**: Tool MUST exit with status code 0 and print the path of the written file on success.
- **FR-010**: Tool MUST be runnable on-demand without any scheduler or daemon dependency.

### Key Entities

- **Resort**: A ski area identified by its human-readable name (e.g., "Serre Chevalier") passed via `--resort`. The Overpass API is queried using an area name match. A filename slug is derived by lowercasing and replacing spaces with underscores (e.g., "serre_chevalier").
- **Trail record**: Geographic element representing a piste or lift, including name, type, difficulty, and coordinate sequence.
- **Elevation point**: Altitude value (in meters) associated with a specific geographic coordinate.
- **Merged dataset**: Combined structure pairing each trail record's coordinates with their elevation values.
- **Output file**: A timestamped JSON file in `data/` that the main application can load directly.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operator generates a complete, application-ready data file in under 5 minutes for any supported resort.
- **SC-002**: 100% of trail coordinate points in the output file include an elevation value; no segment is written without elevation data.
- **SC-003**: Generating data for a new resort requires only changing the resort name argument -- zero code changes.
- **SC-004**: The output file is accepted by the main application on first load with no manual editing or transformation.
- **SC-005**: Every failed run exits with a non-zero code and a human-readable error message sufficient to diagnose the failure without reading source code.

## Clarifications

### Session 2026-03-19

- Q: How does the tool resolve a resort name to a geographic Overpass API query? → A: Use `--resort "Serre Chevalier"` (human-readable name); query Overpass via area name match; derive filename slug by lowercasing + replacing spaces with underscores.
- Q: Which API for elevation data? → A: IGN Altimetrie API (`data.geopf.fr/altimetrie/...`, resource `ign_rge_alti_wld`), as used in existing `get_elevation` prototype. Batch size 50, 200 ms delay, exponential backoff + 429 retry.
- Q: What if `data/` folder does not exist? → A: Auto-create it.
- Q: How to handle partial/failed elevation data? → A: Follow existing `get_elevation` behavior: abort on batch failure after 3 retries (no partial output). Add post-fetch validation that all nodes received elevation (improvement over prototype).
- Q: Should spec reference existing prototypes as implementation source? → A: Yes. Port and merge `get_data` + `get_elevation` from `serre_che_proto` repo.

## Assumptions

- The tool auto-creates the `data/` folder at the project root if it does not exist.
- Trail data: Overpass API (public, no auth). Elevation data: IGN Altimetrie API at `data.geopf.fr` (public, no auth, ~5 m precision).
- Resort names are human-readable strings passed via `--resort` (e.g., "Serre Chevalier", "Chamonix"). Filename slugs are derived automatically.
- The output JSON schema matches the format the main application already consumes (same as the existing `_ele.json` files in `data/`).
- Runs are infrequent (operator-triggered), so there is no requirement to optimize for high throughput or concurrent execution.
- The tool runs on the developer's local machine, not in a CI/CD or server environment.
- Implementation is a port/merge of two existing Rust CLI prototypes: [`get_data`](https://github.com/40tude/serre_che_proto/tree/main/get_data) (Overpass fetch) and [`get_elevation`](https://github.com/40tude/serre_che_proto/tree/main/get_elevation) (IGN elevation enrichment) from the `serre_che_proto` repository.
