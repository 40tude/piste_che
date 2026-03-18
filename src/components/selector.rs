// Rust guideline compliant 2026-02-16
//
// SelectorPanel -- start/end point dropdowns populated from selectable lift elements.

use crate::models::SelectableElement;
use leptos::prelude::*;

/// Dropdown pair for selecting the route start and end points.
///
/// Both dropdowns are populated with the same `selectable_elements` list.
/// Changes are written back to the provided `RwSignal`s.
#[component]
pub fn SelectorPanel(
    selectable_elements: ReadSignal<Vec<SelectableElement>>,
    start: RwSignal<String>,
    end: RwSignal<String>,
) -> impl IntoView {
    // Build option lists outside the view! macro to avoid move/borrow conflicts
    // that arise when the same String is used as both an attribute value and child.
    let start_options = move || {
        selectable_elements
            .get()
            .into_iter()
            .map(|e| {
                let value = e.name.clone();
                let label = e.name;
                view! { <option value=value>{label}</option> }
            })
            .collect_view()
    };

    let end_options = move || {
        selectable_elements
            .get()
            .into_iter()
            .map(|e| {
                let value = e.name.clone();
                let label = e.name;
                view! { <option value=value>{label}</option> }
            })
            .collect_view()
    };

    view! {
        <div class="selector-panel">
            <div class="selector-row">
                <label class="selector-label">"Start"</label>
                <select
                    class="selector-select"
                    on:change=move |ev| start.set(event_target_value(&ev))
                >
                    <option value="">"-- select --"</option>
                    {start_options}
                </select>
            </div>

            <div class="selector-row">
                <label class="selector-label">"End"</label>
                <select
                    class="selector-select"
                    on:change=move |ev| end.set(event_target_value(&ev))
                >
                    <option value="">"-- select --"</option>
                    {end_options}
                </select>
            </div>
        </div>
    }
}
