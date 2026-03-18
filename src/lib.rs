// Rust guideline compliant 2026-02-16
pub mod app;
pub mod components;
pub mod models;

// `server` is NOT feature-gated: the `#[server]` macro generates client stubs
// that must be accessible from the hydrate (WASM) build as well.
pub mod server;

// Routing module is server-only; entire subtree gated behind `ssr`.
#[cfg(feature = "ssr")]
pub mod routing;

/// WASM hydration entry point -- called automatically by the browser after the
/// WASM module is instantiated.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn hydrate() {
    use app::App;
    leptos::mount::hydrate_body(App);
}
