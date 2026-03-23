Review all test files (unit tests, integration tests).
For each module in src/, assess:
- what is tested vs what is not
- missing edge cases
- tests that are redundant or test implementation details instead of behavior
- whether the happy path, error paths, and boundaries are covered

Produce a gap analysis: what tests are MISSING, what tests should be REMOVED or SIMPLIFIED.
Do not write the tests yet.
Write the full report to docs/11_test_audit.md and as is ARCHITECTURE tag the file with "Version XXX | Commit YYY | YYYY-MM-DD"