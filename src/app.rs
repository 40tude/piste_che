// Rust guideline compliant 2026-02-16
//
// Root Leptos application component.  Wires together all user stories:
//   US1 -- SkiMap with area data
//   US2 -- SelectorPanel + compute_route action + ItineraryPanel
//   US3 -- FilterPanel with excluded-difficulty/lift-type signals
//   US4 -- ModeTabs (Short active, Sport/Safe disabled)

use crate::components::{
    filters::FilterPanel,
    itinerary::ItineraryPanel,
    map::SkiMap,
    mode_tabs::ModeTabs,
    selector::SelectorPanel,
};
use crate::models::{HighlightSegment, RouteStep, SelectableElement};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::server::api::{compute_route, get_area};

/// Root component -- sets up routing and provides the app shell.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let fallback = || view! { <p class="not-found">"Page not found."</p> };

    view! {
        <Stylesheet id="leptos" href="/pkg/piste_che.css"/>
        <link rel="stylesheet" href="/leaflet.css"/>
        <Router>
            <main>
                <Routes fallback>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Home page -- full-screen layout with sidebar and map.
#[component]
fn HomePage() -> impl IntoView {
    // --- Shared state ---

    // US3: filter signals (empty = all included = all checked)
    let excluded_difficulties: RwSignal<Vec<String>> = RwSignal::new(Vec::new());
    let excluded_lift_types: RwSignal<Vec<String>> = RwSignal::new(Vec::new());

    // US2: start/end selector values
    let start = RwSignal::new(String::new());
    let end = RwSignal::new(String::new());

    // US2: computed route state
    let route_segments = RwSignal::new(Vec::<HighlightSegment>::new());
    let route_steps = RwSignal::new(Vec::<RouteStep>::new());
    let route_total = RwSignal::new(0u32);
    let route_error: RwSignal<Option<String>> = RwSignal::new(None);

    // US4: active mode (always "short" for MVP)
    let active_mode = RwSignal::new("short".to_string());

    // --- US1: fetch area data once on load ---
    let area = Resource::new(|| (), |_| async { get_area().await });

    // Extracted selectable elements signal (populated after area loads)
    let selectable_elements: RwSignal<Vec<SelectableElement>> = RwSignal::new(Vec::new());

    // --- US2: route computation action ---
    let compute_action = Action::new(move |_: &()| {
        let s = start.get_untracked();
        let e = end.get_untracked();
        let ed = excluded_difficulties.get_untracked();
        let el = excluded_lift_types.get_untracked();
        async move { compute_route(s, e, ed, el, "short".to_string()).await }
    });

    // React to action result
    Effect::new(move |_| {
        if let Some(result) = compute_action.value().get() {
            match result {
                Ok(resp) => {
                    route_segments.set(resp.highlight_segments);
                    route_steps.set(resp.steps);
                    route_total.set(resp.total_distance_m);
                    route_error.set(resp.error);
                }
                Err(e) => {
                    route_error.set(Some(e.to_string()));
                }
            }
        }
    });

    view! {
        <div class="layout">
            // ---- Sidebar ----
            <aside class="sidebar">
                // US4: mode tabs
                <ModeTabs active_mode=active_mode.read_only()/>

                // US3: filter panel
                <FilterPanel
                    excluded_difficulties=excluded_difficulties
                    excluded_lift_types=excluded_lift_types
                />

                // US2: start/end selector + calculate button
                <SelectorPanel
                    selectable_elements=selectable_elements.read_only()
                    start=start
                    end=end
                />

                <button
                    class="calculate-btn"
                    on:click=move |_| {
                        let _ = compute_action.dispatch(());
                    }
                >
                    "Calculate"
                </button>

                // US2: itinerary panel
                <ItineraryPanel
                    steps=route_steps.read_only()
                    total_distance_m=route_total.read_only()
                    error=route_error.read_only()
                />
            </aside>

            // ---- Map ----
            <div class="map-container">
                <Suspense fallback=|| view! { <div class="loading">"Loading map..."</div> }>
                    {move || {
                        area.get().map(|result| {
                            match result {
                                Ok(data) => {
                                    let segments = RwSignal::new(data.segments);
                                    selectable_elements.set(data.selectable_elements);
                                    view! {
                                        <SkiMap
                                            segments=segments.read_only()
                                            route_segments=route_segments.read_only()
                                            excluded_difficulties=excluded_difficulties
                                                .read_only()
                                            excluded_lift_types=excluded_lift_types.read_only()
                                        />
                                    }
                                    .into_any()
                                }
                                Err(e) => {
                                    view! {
                                        <div class="loading">
                                            "Failed to load area data: "
                                            {e.to_string()}
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
