//! colour: art. The chips' small drawings; the Top 10 square takes --color-top10-badge.
//! The small coloured glyphs of the New & Hot pills (Coming Soon, Everyone's Watching, Games, Top 10), drawn as SVG so they look the same on every device (an emoji font draws
//! its own). Shapes and gradients follow the glyphs of the app: a striped popcorn bucket, a flame with an inner flame, a pink controller, a red TOP 10 label.
use leptos::prelude::*;

#[component]
pub fn Popcorn(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg width=size height=size viewBox="0 0 32 32" aria-hidden="true">
            <defs>
                <linearGradient id="pc-corn" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffd54a" /><stop offset="1" stop-color="#ffb11f" /></linearGradient>
                <linearGradient id="pc-red" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff7a1a" /><stop offset="1" stop-color="#f01616" /></linearGradient>
                <linearGradient id="pc-white" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff" /><stop offset="1" stop-color="#e3dcf5" /></linearGradient>
            </defs>
            // the popcorn: a cloud of bumps over the bucket
            <path d="M6 13.5c-1.2-1.7-.6-3.7 1.2-4.4 0-2.1 2-3.4 3.9-2.7 1-1.7 3.5-2 4.9-.6 1.5-1.4 4-.8 4.8 1 1.8-.3 3.4 1.2 3.2 3 1.6.7 2.3 2.7 1.2 4.1z" fill="url(#pc-corn)" />
            <path d="M12 10.8c.9-.8 2.2-.9 3.1-.2M18 11.6c.8-.7 2-.7 2.8 0M9.4 13c.7-.5 1.7-.5 2.3.1" stroke="#ffe27a" stroke-width="1.3" stroke-linecap="round" fill="none" />
            // the bucket: five stripes, outer ones orange to red, the two between white
            <path d="M5.2 13.2h21.6l-2.5 15a1.5 1.5 0 0 1-1.5 1.3H9.2a1.5 1.5 0 0 1-1.5-1.3z" fill="url(#pc-red)" />
            <path d="M10.4 13.2h3.4l.5 16.3h-3.6zM18.2 13.2h3.4l-.9 16.3h-3z" fill="url(#pc-white)" />
        </svg>
    }
}

#[component]
pub fn Flame(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg width=size height=size viewBox="0 0 32 32" aria-hidden="true">
            <defs>
                <linearGradient id="fl-out" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff8a1c" /><stop offset=".55" stop-color="#f23a14" /><stop offset="1" stop-color="#d9202e" /></linearGradient>
                <radialGradient id="fl-in" cx=".5" cy=".95" r=".9"><stop offset="0" stop-color="#ff5fb0" /><stop offset=".3" stop-color="#ffb21f" /><stop offset="1" stop-color="#ff7a1a" /></radialGradient>
            </defs>
            // the outer flame with its two flicks
            <path d="M16.2 2.2c1.6 3.2 6.1 4.7 7.6 8.9.4-1.2 1.9-3.3 3.3-4.3.5 3.6.9 5.2.9 8.9 0 6.2-4.6 10.9-11.9 10.9S4.2 21.9 4.2 16c0-4 1.4-6.7 3.6-8.7.2 2.4.7 3.6 1.9 4.7.3-4.4 2.9-6.6 6.5-9.8z" fill="url(#fl-out)" />
            <path d="M4.1 8.4c1.6.4 3.1 1.8 3.4 4 .1 1.8-1.5 2.9-3 2.3-1.6-.7-1.9-3.7-.4-6.3z" fill="#ff9a1f" />
            <path d="M27.4 7.4c1.4 1.8 1.1 4.4-.4 5.3-1.6.9-2.9-.7-2.4-2.4.4-1.4 1.4-2.3 2.8-2.9z" fill="#ff9a1f" />
            // the inner flame
            <path d="M16 12.1c3.7 3 6.4 5.8 6.4 9.2 0 3.3-2.7 5.5-6.4 5.5s-6.4-2.2-6.4-5.5c0-3.4 2.7-6.2 6.4-9.2z" fill="url(#fl-in)" />
        </svg>
    }
}

#[component]
pub fn Controller(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg width=size height=size viewBox="0 0 32 32" aria-hidden="true">
            <defs>
                <linearGradient id="gc-body" x1=".1" y1="0" x2=".9" y2="1"><stop offset="0" stop-color="#ff6fd8" /><stop offset=".55" stop-color="#e255e8" /><stop offset="1" stop-color="#b857ee" /></linearGradient>
                <linearGradient id="gc-shine" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#ffffff" stop-opacity=".55" /><stop offset=".45" stop-color="#ffffff" stop-opacity="0" /></linearGradient>
            </defs>
            <path d="M8.4 7.2h15.2c3 0 4.6 1.7 5.2 4.8l1.4 8.4c.4 2.7-1.2 5-3.9 5-2 0-3-.9-4.1-2.8-.6-1-1.3-1.5-2.4-1.5h-6.6c-1.1 0-1.8.5-2.4 1.5-1.1 1.9-2.1 2.8-4.1 2.8-2.7 0-4.3-2.3-3.9-5L3.2 12c.6-3.1 2.2-4.8 5.2-4.8z" fill="url(#gc-body)" />
            <path d="M8.4 7.2h15.2c3 0 4.6 1.7 5.2 4.8l.1.7-3.2-.2c-.5-2.1-1.3-3.5-3.6-3.5H9.7c-2.3 0-3.7 1.4-4.3 3.4l-2.2.3c.6-3.2 2.1-5.5 5.2-5.5z" fill="url(#gc-shine)" />
            // the cross and the four buttons, dark purple
            <path d="M8 13.6h1.9v-1.9h2v1.9h1.9v2h-1.9v1.9h-2v-1.9H8z" fill="#4a1561" />
            <circle cx="22.4" cy="12" r="1.15" fill="#4a1561" /><circle cx="19.9" cy="14.5" r="1.15" fill="#4a1561" /><circle cx="24.9" cy="14.5" r="1.15" fill="#4a1561" /><circle cx="22.4" cy="17" r="1.15" fill="#4a1561" />
        </svg>
    }
}

/// The red TOP 10 label: a rounded-control square with the word TOP small above a large 10.
#[component]
pub fn Top10(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! {
        <svg width=size height=size viewBox="0 0 32 32" aria-hidden="true">
            <rect x="1.5" y="1.5" width="29" height="29" rx="3.6" style="fill: var(--color-top10-badge)" />
            <text x="16" y="12.6" text-anchor="middle" font-family="Inter, Helvetica, Arial, sans-serif" font-weight="800" font-size="8.6" fill="#fff" letter-spacing=".3">TOP</text>
            <text x="16" y="26.4" text-anchor="middle" font-family="Inter, Helvetica, Arial, sans-serif" font-weight="900" font-size="15.6" fill="#fff">10</text>
        </svg>
    }
}
