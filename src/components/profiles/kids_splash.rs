//! The screen shown while a Kids profile opens: four bands of colour (green, orange to red, pink, blue to lavender) behind the word "kids" in rounded-control red letters
//! with a white edge. Everything is drawn here as vectors (no picture file), in a 2000 x 1120 box that fills the page.
use leptos::prelude::*;

/// The letters, as strokes with round ends: one description used twice, wide and white for the edge, then narrower and red.
const LETTERS: &str = "M640 432 V685 M672 600 L765 514 M700 622 L772 690 M868 522 V690 M1132 432 V690 M1340 540 C1325 515 1235 505 1233 553 C1231 592 1362 580 1362 640 C1362 690 1262 695 1236 648";
/// The bowl of the d.
const BOWL: (f64, f64, f64, f64) = (1066.0, 601.0, 97.0, 87.0);

#[component]
pub fn KidsSplash(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 2000 1120" preserveAspectRatio="xMidYMid slice" role="img" aria-label=crate::i18n::t("nav-kids") xmlns="http://www.w3.org/2000/svg">
            <defs>
                <linearGradient id="kids-green" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#50a65e" /><stop offset="1" stop-color="#2d5c3a" /></linearGradient>
                <linearGradient id="kids-orange" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f7c862" /><stop offset="0.55" stop-color="#ee7b4e" /><stop offset="1" stop-color="#ea4a42" /></linearGradient>
                <linearGradient id="kids-blue" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2962e0" /><stop offset="0.55" stop-color="#9aa0f2" /><stop offset="0.8" stop-color="#cfc0fa" /><stop offset="1" stop-color="#d3c3fb" /></linearGradient>
            </defs>
            <rect x="0" y="0" width="446" height="1120" fill="url(#kids-green)" />
            <rect x="446" y="0" width="428" height="1120" fill="url(#kids-orange)" />
            <rect x="874" y="0" width="632" height="1120" fill="#ed77c5" />
            <rect x="1506" y="0" width="494" height="1120" fill="url(#kids-blue)" />
            // the white edge, then the red letters
            <g fill="none" stroke="#ffffff" stroke-linecap="round" stroke-linejoin="round" stroke-width="170">
                <path d=LETTERS />
                <ellipse cx=BOWL.0 cy=BOWL.1 rx=BOWL.2 ry=BOWL.3 />
                <circle cx="866" cy="428" r="89" fill="#ffffff" stroke="none" />
            </g>
            <g fill="none" stroke="#d22f26" stroke-linecap="round" stroke-linejoin="round" stroke-width="68">
                <path d=LETTERS />
                <ellipse cx=BOWL.0 cy=BOWL.1 rx=BOWL.2 ry=BOWL.3 />
            </g>
            <circle cx="866" cy="428" r="38" fill="#d22f26" />
        </svg>
    }
}

/// Just the word, for the small mark that stays in the corner of every page of a Kids profile.
#[component]
pub fn KidsWordmark(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="552 342 896 448" role="img" aria-label=crate::i18n::t("nav-kids") xmlns="http://www.w3.org/2000/svg">
            <g fill="none" stroke="#ffffff" stroke-linecap="round" stroke-linejoin="round" stroke-width="170">
                <path d=LETTERS />
                <ellipse cx=BOWL.0 cy=BOWL.1 rx=BOWL.2 ry=BOWL.3 />
                <circle cx="866" cy="428" r="89" fill="#ffffff" stroke="none" />
            </g>
            <g fill="none" stroke="#d22f26" stroke-linecap="round" stroke-linejoin="round" stroke-width="68">
                <path d=LETTERS />
                <ellipse cx=BOWL.0 cy=BOWL.1 rx=BOWL.2 ry=BOWL.3 />
            </g>
            <circle cx="866" cy="428" r="38" fill="#d22f26" />
        </svg>
    }
}
