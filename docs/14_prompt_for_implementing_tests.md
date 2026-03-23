Based on the test gap analysis (./docs/11_testing.md), write the missing tests.
Prefer integration tests for routing behavior and unit tests for pure logic.
Each test must have a descriptive name and a comment explaining what scenario it covers.

After all tests are written and passing:
- Run `cargo test` to confirm all pass
- Run `cargo clippy` to confirm no new warnings
- Update ./docs/11_testing.md to mark each gap as [IMPLEMENTED]
- git commit -m "test: add missing test coverage"