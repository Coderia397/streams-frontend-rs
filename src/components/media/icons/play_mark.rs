//! The play mark on an episode's still, drawn as Netflix's Android app draws it (its vector, 56 units square, read from the app on a
//! Pixel 8, note 116): a black disc at half strength, a white ring 2 units wide and a white triangle.
use leptos::prelude::*;

/// `size` is the disc's width in px (36 on the player's Episodes cards).
#[component]
pub fn PlayMark(size: f64) -> impl IntoView {
    view! {
        <svg width=size height=size viewBox="0 0 56 56" aria-hidden="true" class="pointer-events-none">
            <circle cx="28" cy="28" r="28" class="fill-void" fill-opacity="0.5" />
            <circle cx="28" cy="28" r="27" fill="none" class="stroke-ink" stroke-width="2" />
            <path d="M21,18l18.5,10l-18.5,10z" class="fill-ink" />
        </svg>
    }
}
