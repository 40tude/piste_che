# Pre-Implementation Checklist: Resort Data Generator CLI

**Purpose**: Author self-review of requirement quality before starting implementation. Tests whether the spec, plan, and tasks are clear, complete, and consistent -- not whether the implementation works.
**Created**: 2026-03-19
**Feature**: specs/002-resort-generator/spec.md
**Audience**: Author (pre-implementation)
**Depth**: Standard
**Scope**: Full feature -- CLI contract, pipeline, error handling, testing strategy

---

## Requirement Completeness

- [ ] CHK001 Are the exact Overpass QL query parameters (endpoint URL, timeout, output format) fully documented in the spec or research? [Completeness, research.md R1]
- [ ] CHK002 Are all required IGN API request parameters (`resource`, `zonly`, coordinate format) explicitly specified? [Completeness, Spec §FR-003]
- [ ] CHK003 Is the behavior when `data/` directory already exists (vs. needs creation) distinguished from the auto-create case? [Completeness, Spec §edge cases]
- [ ] CHK004 Are requirements defined for the case where Overpass returns Ways referencing Node IDs not present in the response (dangling references)? [Completeness, Gap]
- [ ] CHK005 Is the output JSON encoding (UTF-8, pretty-print vs. compact) specified in contracts/cli.md or spec? [Completeness, contracts/cli.md]
- [ ] CHK006 Are requirements defined for what constitutes a "valid" slug when the resort name contains non-ASCII characters (e.g., accents)? [Completeness, Spec §FR-001]

---

## Requirement Clarity

- [ ] CHK007 Is "human-readable string used directly in the Overpass API area name query" precise enough to uniquely identify resort areas, or does it require quoting/escaping rules? [Clarity, Spec §FR-001]
- [ ] CHK008 Is "exponential backoff (2s/4s/8s)" specified as wall-clock delays or as base factors, and is jitter excluded? [Clarity, research.md R2]
- [ ] CHK009 Is "IGN sentinel value -99999 accepted as-is" specified as a deliberate valid elevation or as an implicit pass-through? Should the spec define whether -99999 values propagate to the output file? [Clarity, Spec §edge cases]
- [ ] CHK010 Is the phrase "print the full path of the written file" in FR-009 / contracts/cli.md defined as absolute path or relative path? [Clarity, Spec §FR-009, contracts/cli.md]
- [ ] CHK011 Is "local system time" in FR-006 sufficiently precise -- does it mean the system's configured timezone, UTC, or requires no timezone normalization? [Clarity, Spec §FR-006]
- [ ] CHK012 Is "descriptive message" in FR-008 defined with enough specificity that two different authors would produce equivalent messages? [Clarity, Spec §FR-008]

---

## Requirement Consistency

- [ ] CHK013 Does FR-009 ("print path to stdout") conflict with constitution VI ("no `println!` in production code paths"), and is this resolved in plan.md? [Consistency, Spec §FR-009, constitution VI]
- [ ] CHK014 Are the validation rules in data-model.md ("abort with 'no trails found'") consistent with the error message format examples in contracts/cli.md? [Consistency, data-model.md, contracts/cli.md]
- [ ] CHK015 Does the phrase "Non-numeric elevation values (null) cause batch failure" in spec edge cases align with the `Vec<f64>` type in data-model.md `ElevationResponse`? Is this behavior emergent (serde deserialization error) or an explicit requirement? [Consistency, Spec §edge cases, data-model.md]
- [ ] CHK016 Is the ResortConfig `output_dir` field (hardcoded `data/`) consistent with the "auto-create `data/`" requirement -- i.e., is the path relative to the working directory or the binary location? [Consistency, data-model.md, Spec §assumptions]

---

## Acceptance Criteria Quality

- [ ] CHK017 Can SC-002 ("100% of trail coordinate points include an elevation value") be objectively measured -- is "trail coordinate point" defined as nodes referenced by Ways only, or ALL nodes in the response? [Measurability, Spec §SC-002, data-model.md]
- [ ] CHK018 Is SC-001 ("< 5 minutes per resort") measurable from a fixed starting point (cold start, no cached HTTP) -- or does it allow warm-up? [Measurability, Spec §SC-001]
- [ ] CHK019 Is US1's independent test ("verify a timestamped JSON file appears") sufficient to confirm SC-004 (main app loads the file)? Or does the acceptance test need to explicitly reference `OsmData::load()`? [Measurability, Spec §US1, SC-004]
- [ ] CHK020 Is FR-007 ("MUST NOT overwrite existing files") expressed as a measurable acceptance criterion, or only as a constraint? Is the expected exit behavior on collision defined? [Measurability, Spec §FR-007]

---

## Scenario Coverage

- [ ] CHK021 Are requirements defined for a resort name that resolves to multiple Overpass areas (ambiguous area match)? [Coverage, Gap]
- [ ] CHK022 Are requirements for the empty-Ways scenario (Overpass returns nodes but zero ways) distinct from the zero-nodes scenario? [Coverage, data-model.md validation rules]
- [ ] CHK023 Are requirements defined for the case where all IGN batches succeed but the final node count does not match the Overpass node count (e.g., coordinate deduplication)? [Coverage, Spec §edge cases, data-model.md R6]
- [ ] CHK024 Are requirements specified for the intermediate state if the tool is interrupted mid-run (e.g., partial file written or empty file created before crash)? [Coverage, Gap]
- [ ] CHK025 Is the behavior defined for running the tool with no network connectivity at all (vs. a transient network error mid-run)? [Coverage, Spec §US3]

---

## Edge Case Coverage

- [ ] CHK026 Is the timestamp collision edge case resolved with a defined behavior? The spec poses it as an open question; tasks.md T024 adds a file-exists abort -- is this decision captured back in spec.md? [Edge Case, Spec §edge cases, tasks.md T024]
- [ ] CHK027 Is the maximum resort name length or character set constrained? Could a very long resort name produce a filename exceeding Windows MAX_PATH (260 chars)? [Edge Case, Gap]
- [ ] CHK028 Is the behavior specified when `data/` exists as a file rather than a directory? [Edge Case, Gap]
- [ ] CHK029 Is the -99999 IGN sentinel explicitly scoped to French Alps coverage, or is it treated as a universal valid value regardless of resort location? [Edge Case, Spec §edge cases]
- [ ] CHK030 Are requirements defined for a resort where some nodes appear in multiple Ways -- does each node get one elevation fetch or one per Way occurrence? [Edge Case, data-model.md pipeline, Gap]

---

## Non-Functional Requirements

- [ ] CHK031 Is the "< 5 minutes" performance budget (SC-001) allocated across pipeline stages (Overpass fetch, IGN batching, merge) -- or is it only an end-to-end constraint? [NFR, Spec §SC-001]
- [ ] CHK032 Is there a defined memory constraint given that the full OverpassResponse is held in memory during processing? [NFR, Gap]
- [ ] CHK033 Are logging verbosity levels specified (e.g., default INFO vs. DEBUG)? Is RUST_LOG documented as the control mechanism? [NFR, research.md R5, constitution VI]
- [ ] CHK034 Is the CLI required to produce machine-parseable output (JSON stdout) for automation, or is human-readable text sufficient? [NFR, contracts/cli.md]

---

## Dependencies & Assumptions

- [ ] CHK035 Is the assumption "Overpass API is public, no auth required" validated -- is the specific endpoint (`overpass-api.de`) stable enough for a production tool? [Assumption, research.md R1]
- [ ] CHK036 Is the assumption "output JSON schema matches existing `*_ele.json` files" validated by referencing a specific file in `data/` -- or is it declared without a concrete schema comparison? [Assumption, Spec §assumptions, contracts/cli.md]
- [ ] CHK037 Is the dependency on `mockall` (dev-only) explicitly scoped to avoid leaking into the production binary? [Dependency, plan.md, research.md R5]
- [ ] CHK038 Is the assumption that IGN `data.geopf.fr` remains accessible from a Windows local workstation (no VPN, no proxy) documented? [Assumption, Spec §assumptions]

---

## Testing Strategy Requirements

- [ ] CHK039 Is the `HttpClient` trait contract (method signatures, error type) specified precisely enough that a mock implementation can be written without reading the production code? [Completeness, research.md R5]
- [ ] CHK040 Are the canned JSON fixtures needed for unit tests (Overpass response, IGN response) described in spec or research, or left entirely to implementer discretion? [Completeness, Gap]
- [ ] CHK041 Is the boundary between unit tests (mock HTTP) and integration tests (live APIs, feature flag) explicitly defined in requirements -- i.e., which behaviors MUST have unit coverage vs. integration-only? [Clarity, research.md R5, constitution III]
- [ ] CHK042 Is there a requirement for what percentage of code paths must be covered by unit tests, or is coverage left undefined? [Measurability, constitution III, Gap]

---

## Notes

- Check items off as completed: `[x]`
- Add inline findings: `[x] CHK007 -- clarified: spaces are passed literally, no encoding needed`
- Advisory items (F1-F8 from speckit.analyze): CHK013, CHK020, CHK026 are the most actionable pre-implementation
- Items marked `[Gap]` require a spec update before they can be resolved; others may be answerable by re-reading existing docs
