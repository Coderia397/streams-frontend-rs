//! colour: art. The Kids screen's bands and lettering (the lettering in --color-kids-red).
//! The screen shown while a Kids profile opens: four bands of colour (green, orange to red, pink, blue to lavender) behind the word "kids" in rounded red letters with a white
//! edge. The bands are vectors in a 2000 x 1120 box that fills the page; the word is set in a real font (Fredoka Bold, see `style/input.css`), not drawn by hand.
use leptos::prelude::*;

/// The word "kids" as lettering: red letters, a white sticker edge around them (the stroke is painted under the fill). `cx` and `cy` give the middle of the baseline box, `size` the font size.
#[component]
fn Lettering(cx: f64, base: f64, size: f64, edge: f64) -> impl IntoView {
    let font = "'Logo Kids', 'Fredoka', ui-rounded, 'Arial Rounded MT Bold', sans-serif";
    view! {
        // the edge first (white letters grown by a round stroke), then the red letters on top
        <text x=cx y=base text-anchor="middle" font-family=font font-weight="500" font-size=size fill="#ffffff" stroke="#ffffff" stroke-width=edge stroke-linejoin="round" stroke-linecap="round" letter-spacing=size * 0.03>"kids"</text>
        <text x=cx y=base text-anchor="middle" font-family=font font-weight="500" font-size=size style="fill: var(--color-kids-red); stroke: var(--color-kids-red)" stroke-width=size * 0.03 stroke-linejoin="round" letter-spacing=size * 0.03>"kids"</text>
    }
}

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
            <Lettering cx=1000.0 base=690.0 size=460.0 edge=120.0 />
        </svg>
    }
}

/// Just the word, for the small mark that stays in the corner of every page of a Kids profile.
#[component]
pub fn KidsWordmark(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 900 470" role="img" aria-label=crate::i18n::t("nav-kids") xmlns="http://www.w3.org/2000/svg">
            <Lettering cx=450.0 base=345.0 size=430.0 edge=112.0 />
        </svg>
    }
}
