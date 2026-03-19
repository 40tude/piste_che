Feature: resort_generator workspace member

Add a new Cargo workspace member `resort_generator`: a single CLI binary that
fetches trail and elevation data for a ski resort, merges the results, and
writes a timestamped JSON file to `data/`.

Source code to port and merge (both are existing Rust CLIs):
- https://github.com/40tude/serre_che_proto/tree/main/get_data
- https://github.com/40tude/serre_che_proto/tree/main/get_elevation

## Functional requirements

- Single CLI: `cargo run -p resort_generator -- --resort "Serre Chevalier"`
- `--resort` value is the human-readable resort name used as-is in the
  Overpass API query (https://overpass-api.de/api/interpreter)
- Each resort has its own config (additional parameters, endpoints) defined in
  a dedicated module or config file; adding Chamonix or Montgenèvre later
  requires no changes to core logic
- Steps: fetch trail data, fetch elevation data, merge into unified structure,
  serialize to JSON
- Output filename derived from `--resort` value: lowercase, remove accents,
  replace spaces with underscores, append UTC timestamp
  Example: "Serre Chevalier" -> `data/serre_chevalier_20260319_143000.json`
- No overwrite risk thanks to timestamp (`_{YYYYMMDD_HHMMSS}.json`)

## Non-functional requirements

- Add `resort_generator` to root Cargo.toml `[workspace]` members
- No GUI, no scheduler -- manual invocation only
- Extensibility: adding a new resort = adding one config, zero core changes