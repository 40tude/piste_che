# piste_che

> **Warning:** The `.cargo/` folder contains Windows-specific configuration (custom `target-dir` for OneDrive, CPU flags). Delete or rename before building:
> ```bash
> mv .cargo .cargo.bak
> ```
> More information on this [page](https://www.40tude.fr/docs/06_programmation/rust/005_my_rust_setup_win11/my_rust_setup_win11.html#onedrive).



## Description

* The “Piste Che” app creates itineraries for skiers in the Serre Chevalier ski area.
* Made with Claude Code and Spec Kit

<figure style="text-align: center;">
<img src="./docs/img00.webp" alt="" width="900" loading="lazy"/>
<figcaption>...</figcaption>
</figure>


## Prerequisites
- Rust stable 1.85+ (edition 2024 support)
- Make sure Perl is available (mandatory to compile leptos)
    - winget install StrawberryPerl.StrawberryPerl
- cargo-leptos: `cargo install cargo-leptos`
    - This takes several minute (enough for a green tea)
- Check with `rustup target list --installed | Select-String wasm`
    - If `wasm32-unknown-unknown` is **NOT** visible then type `rustup target add wasm32-unknown-unknown`
    - To explain what this is. Rust normally compiles for our PC (x86-64 Windows). wasm32-unknown-unknown is a different compilation target—it produces WebAssembly, the
  binary format that the browser can execute. cargo-leptos needs it to compile the client-side part of the app.



## Build

```powershell
# Development (watch mode with hot-reload)
cargo leptos watch

# Release build (single binary + WASM bundle)
cargo leptos build --release
```

## Run

```powershell
# Default port (from Cargo.toml site-addr)
cargo leptos watch

# Custom port via environment variable (takes precedence)
$env:PORT='3000'; cargo leptos watch

# Release binary with CLI flag
./target/release/piste_che --port 3000
```

Open browser at `http://localhost:3000`.

## Test

```powershell
# All tests (unit + integration)
cargo test

# Integration tests only (requires server running)
cargo test --test integration
```




## Deploy Heroku
Heroku does NOT run `cargo leptos build`. The `site/` folder and the release
binary must be committed and pushed.

**IMPORTANT:** always run `cargo leptos build --release` immediately before
committing for Heroku. `cargo leptos watch` (dev) and `cargo leptos build --release` produce different artifact names (`piste_che.wasm` vs
`piste_che_bg.wasm`). Committing dev artifacts while the JS expects release
artifacts causes a 404 and a blank page.

```powershell
cargo leptos build --release
git add site/ target/release/piste_che
git commit -m "deploy: rebuild assets"
git push heroku main
```

`cargo leptos build --release` generates (among others):

```txt
site/pkg/
  piste_che.js          <- references piste_che_bg.wasm
  piste_che_bg.wasm     <- must be committed (new file, easy to miss)
  piste_che_bg.wasm.d.ts
  piste_che.css
  piste_che.d.ts
```

`site-root = "site"` in `Cargo.toml` controls the output directory.
`site/` and `target/` are NOT in `.gitignore` so artifacts are versioned.





## License

MIT License - see [LICENSE](LICENSE) for details


## Contributing
This project is developed for personal and educational purposes. Feel free to explore and use it to enhance your own learning.

Given the nature of the project, external contributions are not actively sought nor encouraged. However, constructive feedback aimed at improving the project (in terms of speed, accuracy, comprehensiveness, etc.) is welcome. Please note that this project is being created as a hobby and is unlikely to be maintained once my initial goal has been achieved.
