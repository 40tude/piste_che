// Rust guideline compliant 2026-02-16
//
// ModeTabs -- Short (active) / Sport (disabled) / Safe (disabled) tabs.

use leptos::prelude::*;

/// Three routing-mode tabs.  Only "Short" is functional; the others are
/// rendered as disabled for future extensibility.
#[component]
pub fn ModeTabs(active_mode: ReadSignal<String>) -> impl IntoView {
    view! {
        <div class="mode-tabs">
            <button
                class=move || {
                    if active_mode.get() == "short" {
                        "tab tab-active"
                    } else {
                        "tab"
                    }
                }
            >
                "Short"
            </button>
            // Sport and Safe tabs disabled until implemented.
            <button class="tab tab-disabled" disabled=true>
                "Sport"
            </button>
            <button class="tab tab-disabled" disabled=true>
                "Safe"
            </button>
        </div>
    }
}
