//! colour: art. The Kids profile's face.
use leptos::prelude::*;

#[component]
pub fn KidsBadge(
    #[prop(optional, default = 15.0)] size: f64,
    #[prop(optional, default = "".to_string())] class: String,
) -> impl IntoView {
    let bg_stroke_width = size * 0.42;
    let fg_stroke_width = size * 0.04;
    let tracking = -(size * 0.035);
    let width = size * 2.8;
    let height = size * 1.5;

    view! {
        <span
            class=format!("inline-flex items-center justify-center shrink-0 select-none {class}")
            style=format!("width: {width}px; height: {height}px;")
        >
            <svg
                width=format!("{width}")
                height=format!("{height}")
                class="overflow-visible drop-shadow-sm"
                style=format!("font-family: 'Arial Rounded MT Bold', ui-rounded, 'SF Pro Rounded', 'Nunito', sans-serif; letter-spacing: {tracking}px;")
            >
                // Outer shell
                <text
                    x="50%"
                    y="52%"
                    text-anchor="middle"
                    dominant-baseline="central"
                    fill="white"
                    stroke="white"
                    stroke-width=format!("{bg_stroke_width}")
                    stroke-linejoin="round"
                    stroke-linecap="round"
                    font-weight="900"
                    font-size=format!("{size}")
                >
                    "kids"
                </text>
                // Foreground text
                <text
                    x="50%"
                    y="52%"
                    text-anchor="middle"
                    dominant-baseline="central"
                    style="fill: var(--color-brand); stroke: var(--color-brand)"
                    stroke-width=format!("{fg_stroke_width}")
                    stroke-linejoin="round"
                    stroke-linecap="round"
                    font-weight="900"
                    font-size=format!("{size}")
                >
                    "kids"
                </text>
            </svg>
        </span>
    }
}

#[component]

pub fn KidsAvatar(
    #[prop(optional, default = 100.0)] size: f64,
    #[prop(optional, default = "".to_string())] class: String,
) -> impl IntoView {
    let _ = size; // the picture scales with its box
    // the Kids picture: the pink face (as on Netflix, drawn here in our own lines) with the small "kids" mark at the bottom right
    view! {
        <div class=format!("relative h-full w-full overflow-hidden {class}")>
            <svg class="h-full w-full" viewBox="0 0 160 160" aria-hidden="true" xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="kids-face" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#e7099b" /><stop offset="0.35" stop-color="#e7099b" /><stop offset="1" stop-color="#a9548f" /></linearGradient>
                </defs>
                <rect width="160" height="160" fill="url(#kids-face)" />
                <circle cx="31.5" cy="52.5" r="9.3" fill="#fff" />
                <circle cx="128" cy="52.5" r="9.3" fill="#fff" />
                <path d="M67 88 C88 100 118 100 137 87" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="6.2" />
            </svg>
            <crate::components::profiles::kids_splash::KidsWordmark class="absolute bottom-[6%] right-[6%] h-[18%] w-auto" />
        </div>
    }
}
