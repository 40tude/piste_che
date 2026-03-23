/review
You are auditing a working Rust project.
Read the following files to understand the project intent and current implementation:
- README.md
- ARCHITECTURE.md
- SPECS.md or any *.spec.md files if present
- Cargo.toml
- All source files under src/

Your goal is a thorough code audit covering:
- Correctness and logic errors
- Error handling (unwrap/expect usage, propagation strategy)
- Rust idioms and best practices
- Security concerns
- Performance issues
- Dead code or unused dependencies
- Gaps between the declared architecture/specs and the actual implementation

Rules:
- Do NOT fix anything
- Do NOT suggest refactors beyond what is needed to fix findings
- Output a prioritized findings list using [CRITICAL] / [WARN] / [INFO] tags
- For each finding: file + line reference, explanation, suggested fix (brief)

Write the full report to docs/09_review.md and as is ARCHITECTURE tag the file with "Version XXX | Commit YYY | YYYY-MM-DD"

















## Stategy for the review/audit

Approche avec un seul contexte cohérent qui lit tout le code, fait l'audit, puis les docs. Plus fiable car pas de perte de contexte entre agents.

### Étape 1 — Audit de code
```
/review
Voir 08_prompt_for_review.md
```

### Étape 2 — Audit des tests
```
Voir 10_prompt_for_testing.md
```


### Étape 3 — Implémentation des fixes prioritaires

* ATTENTION: on créée une branche
* Voir 12_prompt_for_fixing.md



### Étape 4 — Écriture des tests manquants

* Il a fallu faire un clear => Voir 13_prompt_after_clear
* Voir 14_prompt_for_implementing_tests.md






### Étape 5 — Mise à jour README.md
* Il a fallu faire un clear => Voir 13_prompt_after_clear

```
Read the current README.md and the full source code.
Update the README so it reflects the actual current state of the project:
- accurate setup instructions
- correct feature list
- updated architecture overview (brief)
- how to run tests

Keep it concise and developer-focused.
git commit -m "docs: update README"
```

### Étape 6 — Mise à jour ARCHITECTURE.md
```
Read ARCHITECTURE.md, all src/ files, and the routing module.
Update ARCHITECTURE.md to reflect the current implementation:
- module structure
- data flow (request → routing → response)
- key design decisions and why they were made
- what is intentionally out of scope for now

git commit -m "docs: update ARCHITECTURE"
```

### Étape 7 — Pull Request, merge de branch

Puis une PR audit/code-review → main à la fin pour merger proprement.




### Étape 8 — Lancer les test, jouer avec l'application, merge la branche

```powershell

# 1. Switch to main
git checkout main

# 2. Merge (no fast-forward = keeps the branch history visible)
git merge --no-ff audit/code-review -m "chore: merge audit/code-review into main"

# 3. Verify everything still passes on main
cargo test

# 4. Delete the local branch
git branch -d audit/code-review

# 5. Push
git push origin main
git push origin --delete audit/code-review  # only if we pushed the branch remotely
```


## Tips pratiques
- Utiliser `/memory` pour ancrer les règles du projet (PowerShell, US English, etc.) si ce n'est pas déjà dans ton `CLAUDE.md`.
- Pour l'audit, ajoute `--profile` à tes commandes cargo si tu veux des infos perf.
