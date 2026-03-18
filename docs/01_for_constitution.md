
## Description

The “Piste Che” app creates itineraries for skiers in the Serre Chevalier ski area.



## Core Functionality
* The app is a web server
* It can run locally (testing etc.)
* The app is deployed on Heroku
* The app is written in Rust
* Skiers select their starting point (from a drop-down menu [by clicking on the map?]), their destination, filter the difficulty level of the runs (red, green, etc.), and choose the types of lifts they wish to use.
* The app takes constraints into account and calculates the route
* It displays the route as a list next to the map
* By default, the app calculates the shortest route
* Eventually, the app will offer three routes: Short, Sport, and Safe (three tabs)
* The route is highlighted on the map



## Deployment
- Run and test locally first — port configurable via --port CLI flag or PORT env var (Heroku convention)
- Deploy on Heroku using Rust buildpack
- PORT env var takes precedence over CLI flag when set
- Procfile included

## Non-goals
- No API versioning
- No authentication



## Quality & Testing (TDD)

* Unit tests for domain logic (calculation accuracy, category boundaries, edge cases like zero/negative inputs)
* Integration tests for API endpoints using Reqwest (valid requests, invalid inputs, missing fields)
* All tests runnable via `cargo test`





## Crates to be used preferably (if needed)

### Web UI
- `Bootstrap`

### Error Handling
- `thiserror` for library crates
- `anyhow` for binary crates

### CLI
- `clap` (with derive feature)

### Serialization
- `serde` + `serde_json`

### Async & Web
- `tokio` — async runtime
- `axum` — web framework
- `reqwest` — HTTP client

### Date & Time
- `chrono` or `time`

### Utilities
- `derive_more` — derive common traits (Display, From, Constructor, etc.)
- `rand` — random number generation
- `SQLx` — database access
- `itertools` — extended iterator methods
- `regex` — regular expressions

### Logging Strategy
**Use `tracing` + `tracing-subscriber` if:**
- Using tokio
- Writing a server / service
- Need request IDs, spans, timing
- Want future-proof observability
- Care about performance diagnostics

**Use `log` + `env_logger` if:**
- Small CLI tool
- No async complexity


### Testing
- `assert_cmd` + `predicates` for CLI testing
- `mockall` for mocking


