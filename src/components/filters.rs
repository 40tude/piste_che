// Rust guideline compliant 2026-02-16
//
// FilterPanel -- difficulty and lift-type checkboxes.
// All boxes are checked by default (nothing excluded on load).

use leptos::prelude::*;

/// Checkbox groups for filtering piste difficulties and lift types.
///
/// When a box is unchecked, that value is added to the corresponding
/// `RwSignal<Vec<String>>`.  The signals are read by the route action
/// and by `SkiMap` for opacity dimming.
#[component]
pub fn FilterPanel(
    excluded_difficulties: RwSignal<Vec<String>>,
    excluded_lift_types: RwSignal<Vec<String>>,
) -> impl IntoView {
    // (osm_value, ui_label) -- order matches display spec.
    let difficulties: &'static [(&'static str, &'static str)] = &[
        ("novice", "Green"),
        ("easy", "Blue"),
        ("intermediate", "Red"),
        ("advanced", "Black"),
    ];

    let lift_types: &'static [(&'static str, &'static str)] = &[
        ("chair_lift", "Chairlift"),
        ("gondola", "Gondola"),
        ("drag_lift", "Drag lift"),
        ("cable_car", "Cable car"),
    ];

    // Build static item lists before entering the view! macro, avoiding
    // `move` keyword in attribute position (unparseable by the view! proc macro).
    let difficulty_items = difficulties
        .iter()
        .copied()
        .map(|(key, label)| {
            let is_checked =
                move || !excluded_difficulties.get().contains(&key.to_string());
            view! {
                <label class="filter-row">
                    <input
                        type="checkbox"
                        prop:checked=is_checked
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            excluded_difficulties.update(|excl| {
                                if checked {
                                    excl.retain(|d| d != key);
                                } else if !excl.iter().any(|d| d == key) {
                                    excl.push(key.to_string());
                                }
                            });
                        }
                    />
                    {label}
                </label>
            }
        })
        .collect_view();

    let lift_items = lift_types
        .iter()
        .copied()
        .map(|(key, label)| {
            let is_checked =
                move || !excluded_lift_types.get().contains(&key.to_string());
            view! {
                <label class="filter-row">
                    <input
                        type="checkbox"
                        prop:checked=is_checked
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            excluded_lift_types.update(|excl| {
                                if checked {
                                    excl.retain(|d| d != key);
                                } else if !excl.iter().any(|d| d == key) {
                                    excl.push(key.to_string());
                                }
                            });
                        }
                    />
                    {label}
                </label>
            }
        })
        .collect_view();

    view! {
        <div class="filter-panel">
            <div class="filter-section">
                <h4 class="filter-label">"Difficulty"</h4>
                {difficulty_items}
            </div>
            <div class="filter-section">
                <h4 class="filter-label">"Lift type"</h4>
                {lift_items}
            </div>
        </div>
    }
}
