// Rust guideline compliant 2026-02-16
//
// ItineraryPanel -- step-by-step route list with distances and total.

use crate::models::RouteStep;
use leptos::prelude::*;

/// Displays the computed route as a numbered list of steps.
///
/// Shows an error message when `error` is `Some`.  Shows nothing when
/// `steps` is empty and `error` is `None` (no route computed yet).
#[component]
pub fn ItineraryPanel(
    steps: ReadSignal<Vec<RouteStep>>,
    total_distance_m: ReadSignal<u32>,
    error: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="itinerary-panel">
            <Show
                when=move || error.get().is_some()
                fallback=|| ()
            >
                <p class="itinerary-error">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>

            <Show
                when=move || !steps.get().is_empty()
                fallback=|| ()
            >
                <ol class="itinerary-list">
                    // Reactive list -- re-renders when steps change.
                    // collect_view() avoids keyed-For attribute parsing issues.
                    {move || {
                        steps
                            .get()
                            .into_iter()
                            .map(|step| {
                                view! {
                                    <li class="itinerary-item">
                                        <span class="step-name">{step.name.clone()}</span>
                                        <span class="step-meta">
                                            {step.kind.clone()}
                                            " \u{00b7} "
                                            {step.distance_m}
                                            " m"
                                        </span>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ol>
                <p class="itinerary-total">
                    "Total: "
                    {move || total_distance_m.get()}
                    " m"
                </p>
            </Show>
        </div>
    }
}
