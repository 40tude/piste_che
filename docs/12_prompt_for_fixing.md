Before starting any fixes, create a git branch:
git checkout -b audit/code-review

Based on the audit findings marked [CRITICAL] and [WARN],
Fix the issues one by one.

For each fix:
- Run `cargo clippy` and `cargo test` after each fix to confirm nothing is broken
- Update ./docs/09_review.md to mark the finding as [FIXED] with a brief explanation of what was changed and why
- git commit -m "fix(audit): <short description of the specific finding>"