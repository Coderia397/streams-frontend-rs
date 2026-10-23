//! colour: art. The lock's drawing.
use leptos::prelude::*;

#[component]
pub fn LockShield(
    #[prop(optional, default = 120)] width: u32,
    #[prop(optional, default = 140)] height: u32,
) -> impl IntoView {
    view! {
        <svg
            width=width
            height=height
            viewBox="0 0 120 140"
            class="drop-shadow-brand-lg select-none"
        >
            <defs>
                <linearGradient id="pl-shield-fill" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#f5495c" />
                    <stop offset="45%" stop-color="#d92438" />
                    <stop offset="100%" stop-color="#8e2b86" />
                </linearGradient>
                <linearGradient id="pl-shield-rim" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#ff8a5c" />
                    <stop offset="50%" stop-color="#ff5d73" />
                    <stop offset="100%" stop-color="#b44ad1" />
                </linearGradient>
                <linearGradient id="pl-shield-gloss" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="rgba(255,255,255,0.55)" />
                    <stop offset="60%" stop-color="rgba(255,255,255,0)" />
                </linearGradient>
            </defs>
            <path d="M60 6 L106 22 V64 C106 98 88 120 60 134 C32 120 14 98 14 64 V22 Z" fill="url(#pl-shield-rim)" />
            <path d="M60 14 L98 27 V63 C98 92 83 111 60 124 C37 111 22 92 22 63 V27 Z" fill="url(#pl-shield-fill)" />
            <path d="M60 14 L98 27 V50 L27 88 C23 80 22 72 22 63 V27 Z" fill="url(#pl-shield-gloss)" opacity="0.35" />
            <rect x="43" y="62" width="34" height="26" rx="5" fill="none" stroke="white" stroke-width="4.5" />
            <path d="M49 62 V52 a11 11 0 0 1 22 0 V62" fill="none" stroke="white" stroke-width="4.5" stroke-linecap="round" />
            <line x1="60" y1="71" x2="60" y2="79" stroke="white" stroke-width="4.5" stroke-linecap="round" />
        </svg>
    }
}
