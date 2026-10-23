//! The four pictures of the My page, drawn as vectors in the brand colour (blue): a clock, a treasure chest, a clapperboard and a thumbs up.
//! Each is 80 px and has a soft gradient and a light edge so it reads as a small solid object.
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShelfIcon {
    Clock,
    Chest,
    Clapper,
    ThumbsUp,
}

#[component]
pub fn ShelfPicture(icon: ShelfIcon) -> impl IntoView {
    let (id, body) = match icon {
        ShelfIcon::Clock => ("clock", view! {
            <circle cx="40" cy="40" r="34" fill="url(#shelf-clock)" />
            <circle cx="40" cy="40" r="27" fill="none" stroke="#ffffff" stroke-opacity="0.35" stroke-width="2" />
            <g stroke="#ffffff" stroke-linecap="round" stroke-opacity="0.8" stroke-width="2">
                <path d="M40 17 V21 M40 59 V63 M17 40 H21 M59 40 H63" />
            </g>
            <path d="M40 40 V25 M40 40 L52 46" stroke="#ffffff" stroke-linecap="round" stroke-width="3.5" fill="none" />
            <circle cx="40" cy="40" r="3.5" fill="#ffffff" />
        }.into_any()),
        ShelfIcon::Chest => ("chest", view! {
            <path d="M12 38 C12 22 24 14 40 14 C56 14 68 22 68 38 Z" fill="url(#shelf-chest)" />
            <rect x="12" y="38" width="56" height="30" rx="5" fill="url(#shelf-chest-dark)" />
            <rect x="12" y="34" width="56" height="8" fill="#ffffff" fill-opacity="0.35" />
            <rect x="33" y="40" width="14" height="16" rx="3" fill="#ffffff" fill-opacity="0.9" />
            <circle cx="40" cy="47" r="2.4" fill="#2358d6" />
            <g fill="#ffffff"><path d="M60 10 l2 5 5 2 -5 2 -2 5 -2 -5 -5 -2 5 -2z" /><path d="M20 26 l1.5 3.5 3.5 1.5 -3.5 1.5 -1.5 3.5 -1.5 -3.5 -3.5 -1.5 3.5 -1.5z" /></g>
        }.into_any()),
        ShelfIcon::Clapper => ("clapper", view! {
            <g transform="rotate(-12 40 40)">
                <rect x="12" y="32" width="56" height="34" rx="4" fill="url(#shelf-clap)" />
                <rect x="12" y="20" width="56" height="12" rx="3" fill="#ffffff" fill-opacity="0.92" />
                <path d="M20 20 l8 12 M34 20 l8 12 M48 20 l8 12 M62 20 l6 9" stroke="#2f6bff" stroke-width="6" />
            </g>
        }.into_any()),
        ShelfIcon::ThumbsUp => ("thumb", view! {
            <path d="M14 36 H26 V68 H14 Z" fill="url(#shelf-thumb)" />
            <path d="M30 36 C38 30 40 22 40 14 C47 14 52 20 50 30 L49 34 H64 C70 34 73 39 71 45 L67 62 C66 66 63 68 59 68 H30 Z" fill="url(#shelf-thumb)" />
            <path d="M30 40 V66" stroke="#ffffff" stroke-opacity="0.35" stroke-width="2" />
        }.into_any()),
    };
    let _ = id;
    view! {
        <svg width="80" height="80" viewBox="0 0 80 80" aria-hidden="true" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <radialGradient id="shelf-clock" cx="35%" cy="30%" r="80%"><stop offset="0" stop-color="#7fa6ff" /><stop offset="1" stop-color="#1d4bbd" /></radialGradient>
                <linearGradient id="shelf-chest" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#6e9bff" /><stop offset="1" stop-color="#2f6bff" /></linearGradient>
                <linearGradient id="shelf-chest-dark" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3f78ff" /><stop offset="1" stop-color="#1d4bbd" /></linearGradient>
                <linearGradient id="shelf-clap" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#6e9bff" /><stop offset="1" stop-color="#1d4bbd" /></linearGradient>
                <linearGradient id="shelf-thumb" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#7fa6ff" /><stop offset="1" stop-color="#2358d6" /></linearGradient>
            </defs>
            {body}
        </svg>
    }
}
