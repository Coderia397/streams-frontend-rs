//! colour: art. The two small drawings beside the billboard's lines (a megaphone, a clapperboard), in warm and violet gradients over the
//! brand's accent.
use leptos::prelude::*;

/// The megaphone of the billboard's "new" lines.
#[component]
pub fn MegaphoneBadgeIcon() -> impl IntoView {
    view! {
        <svg
            class="w-[15px] h-[15px] flex-shrink-0 drop-shadow-accent-pink"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <defs>
                <linearGradient id="megaGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color: var(--color-accent-alt)" />
                    <stop offset="60%" stop-color="#ff1a40" />
                    <stop offset="100%" stop-color="#ff6b00" />
                </linearGradient>
            </defs>
            <path
                d="M4 11.5V8.5C4 7.95 4.45 7.5 5 7.5H7L13.2 3.4C13.8 3.0 14.6 3.4 14.6 4.1V15.9C14.6 16.6 13.8 17.0 13.2 16.6L7 12.5H5C4.45 12.5 4 12.05 4 11.5Z"
                fill="url(#megaGrad)"
            />
            <path
                d="M6 12.5V15.5C6 16.05 6.45 16.5 7 16.5H7.5C8.05 16.5 8.5 16.05 8.5 15.5V12.5"
                fill="url(#megaGrad)"
            />
            <path
                d="M16 7.5C16.8 8.8 16.8 11.2 16 12.5"
                stroke="url(#megaGrad)"
                stroke-width="1.8"
                stroke-linecap="round"
            />
        </svg>
    }
}

/// The clapperboard of the billboard's credit and acclaim lines.
#[component]
pub fn ClapperboardBadgeIcon() -> impl IntoView {
    view! {
        <svg
            class="w-[15px] h-[15px] flex-shrink-0 drop-shadow-accent-violet"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <defs>
                <linearGradient id="clapGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" style="stop-color: var(--color-accent-alt)" />
                    <stop offset="50%" stop-color="#9b5de5" />
                    <stop offset="100%" stop-color="#7209b7" />
                </linearGradient>
            </defs>
            <rect x="2.5" y="4" width="15" height="3.5" rx="1" fill="url(#clapGrad)" />
            <path d="M5.5 4L4 7.5M9.5 4L8 7.5M13.5 4L12 7.5" stroke="#ffffff" stroke-width="1.2" stroke-linecap="round" />
            <rect x="2.5" y="8" width="15" height="8" rx="1.2" fill="url(#clapGrad)" />
            <line x1="5" y1="11" x2="11" y2="11" stroke="#ffffff" stroke-opacity="0.8" stroke-width="1.2" stroke-linecap="round" />
            <line x1="5" y1="13.5" x2="9" y2="13.5" stroke="#ffffff" stroke-opacity="0.5" stroke-width="1.2" stroke-linecap="round" />
            <circle cx="14" cy="12.2" r="1.2" fill="#ffffff" fill-opacity="0.8" />
        </svg>
    }
}
