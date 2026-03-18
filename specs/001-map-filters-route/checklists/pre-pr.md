# Pre-PR Author Checklist: Map, Filters & Shortest Route (Local)

**Purpose**: Author self-review -- validate spec/plan/tasks quality across all 4 clusters before submitting PR
**Created**: 2026-03-18
**Feature**: [spec.md](../spec.md)

## Filter & Routing Requirements

- [ ] CHK001 - Are difficulty filter labels ("green, blue, red, black" in spec) mapped to their OSM tag values (`novice`, `easy`, `intermediate`, `advanced` in plan) in the spec? [Ambiguity, Spec §FR-002, FR-005]
- [ ] CHK002 - Is "freeride" / off-piste difficulty explicitly addressed in requirements, or are exactly four difficulty levels in scope? (plan.md maps `advanced/#freeride` with a shared color but spec lists only four named levels) [Gap, Spec §FR-002]
- [ ] CHK003 - Are lift type filter labels ("chairlift, gondola, drag lift, cable car") mapped to their OSM aerialway tag values (`chair_lift`, `gondola`, `drag_lift`, `cable_car`) in the spec? [Ambiguity, Spec §FR-006]
- [ ] CHK004 - Is the behavior when all filters of a given type are unchecked specified consistently between Edge Cases and FR-008? (Edge Cases addresses all-difficulties-unchecked and all-lifts-unchecked separately, but not both simultaneously in FR-008) [Consistency, Spec §FR-008, Edge Cases]

## API Contract Requirements

- [ ] CHK005 - Are the enumerated error conditions and the `error` field format in `RouteResponse` specified in the spec requirements, not just in plan.md DTOs? [Gap, Spec §FR-016]
- [ ] CHK006 - Are the valid values and behavior of the `mode` parameter documented in the spec? (FR-020 mandates including it; only "short" is defined; no enumeration of accepted string values provided) [Completeness, Spec §FR-020]
- [ ] CHK007 - Is the `highlight_coords` field structure (one array per segment vs. flat coordinate list) specified in requirements? [Gap, Spec §FR-009]
- [ ] CHK008 - Are node identifiers used in route requests (names as strings vs. opaque IDs) specified unambiguously in the spec? (FR-004 says "lift base station names" in dropdowns; FR-015 says "start, end" params -- are these the same string values?) [Clarity, Spec §FR-004, FR-015]

## Test Requirements

- [ ] CHK009 - Does SC-004 ("100% of existing prototype routing module tests pass without modification") conflict with research.md R5 confirming no existing test files exist in the prototype? [Conflict, Spec §SC-004, FR-018]
- [ ] CHK010 - Are the specific edge cases that FR-018 mandates unit tests for enumerated in requirements, or are they only implicit in the tasks? [Completeness, Spec §FR-018]
- [ ] CHK011 - Is the TDD write-fail-implement sequence a formal requirement or an implementation note? (Tasks mandate it under Principle III, but the spec and plan do not capture it as a verifiable requirement) [Clarity, Spec §FR-018, tasks.md]

## Non-Functional & Success Criteria

- [ ] CHK012 - Are SC-001 (<3s map load) and SC-002 (<2s route compute) defined with specific measurement conditions (hardware baseline, data file size, network type)? [Measurability, Spec §SC-001, SC-002]
- [ ] CHK013 - Is the "ski area data fails to load" failure scenario from Edge Cases backed by a functional requirement (FR)? (Listed as an edge case with expected behavior but no corresponding FR entry) [Gap, Spec §Edge Cases]
- [ ] CHK014 - Is FR-009 ("distinct color and weight") specific enough to be verifiable, or does the route highlight color need to be a named, measurable requirement? [Ambiguity, Spec §FR-009]
- [ ] CHK015 - Are FR-009b dimming semantics quantified with a specific opacity or visual threshold, or left entirely to implementation? [Clarity, Spec §FR-009b]
