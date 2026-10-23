//! colour: art. The landing page's drawings: the big outlined numbers of its Top 10 and the reasons' illustrations, in their own palette.
use leptos::prelude::*;

// ── RankNumber SVG ────────────────────────────────────────────────────────────

#[component]
pub fn RankNumber(index: usize) -> impl IntoView {
    let is_ten = index == 9;
    let viewbox = if is_ten { "0 0 280 210" } else { "0 0 200 210" };
    let transform = if is_ten { "scale(1.0, 0.98)" } else { "scale(1.1, 1.0)" };
    let letter_spacing = if is_ten { "-22" } else { "-6" };
    let width_style = if is_ten { "width:60%" } else { "width:50%" };
    let left_class = "absolute bottom-[8%] h-[48%] z-20 pointer-events-none overflow-visible drop-shadow-halo left-[-20px] sm:left-[-26px] md:left-[-34px] lg:left-[-42px]";
    let num = (index + 1).to_string();

    view! {
        <div class=left_class style=width_style>
            <svg
                viewBox=viewbox
                class="h-full w-auto"
                preserve-aspect-ratio="xMinYMid meet"
                style="overflow:visible"
            >
                <g transform=transform style="transform-origin:0px 205px">
                    <text x="8" y="195" text-anchor="start" dominant-baseline="auto"
                        fill="none" stroke="#737373" stroke-width="12" stroke-linejoin="round"
                        font-size="180" font-weight="900"
                        font-family="'Inter', sans-serif" letter-spacing=letter_spacing>
                        {num.clone()}
                    </text>
                    <text x="8" y="195" text-anchor="start" dominant-baseline="auto"
                        fill="#090909" stroke="#090909" stroke-width="4" stroke-linejoin="round"
                        font-size="180" font-weight="900"
                        font-family="'Inter', sans-serif" letter-spacing=letter_spacing>
                        {num}
                    </text>
                </g>
            </svg>
        </div>
    }
}

// ── Feature Icons ─────────────────────────────────────────────────────────────

#[component]
pub fn FeatureIcon(id: &'static str) -> impl IntoView {
    match id {
        "tv" => view! {
            <div class="relative w-20 h-16 flex flex-col items-center justify-end">
                <div class="absolute top-0 w-16 h-12 rounded-control bg-gradient-to-br from-[#ff1f75]/20 to-[#8000ff]/20 blur-md pointer-events-none" />
                <div class="w-16 h-11 bg-[#101018] rounded-t-field border-2 border-[#2b2b3d] p-[1.5px] relative overflow-hidden flex items-center justify-center shadow-lg">
                    <div class="w-full h-full rounded-hairline bg-gradient-to-tr from-[#9900ff] via-[#ff007c] to-[#ff8c00] opacity-80 relative">
                        <div class="absolute inset-0 bg-gradient-to-b from-ink/20 via-transparent to-transparent rotate-12 scale-150 transform -translate-y-4" />
                    </div>
                </div>
                <div class="w-2.5 h-2 bg-gradient-to-b from-[#2b2b3d] to-[#12121c] border-x border-[#3f3f5a]" />
                <div class="w-10 h-[3px] bg-gradient-to-r from-accent via-[#ff7700] to-accent rounded-full shadow-md" />
            </div>
        }.into_any(),

        "stream" => view! {
            <div class="relative w-16 h-16 flex items-center justify-center">
                <div class="absolute inset-0 rounded-full bg-gradient-to-br from-accent-alt/30 to-[#7b2cbf]/30 blur-md pointer-events-none" />
                <div class="w-14 h-14 rounded-full bg-gradient-to-tr from-[#7b2cbf] via-[#d90429] to-accent-alt p-[1.5px] shadow-raise relative flex items-center justify-center overflow-hidden">
                    <div class="absolute inset-0 bg-[radial-gradient(circle_at_30%_30%,rgba(255,255,255,0.4)_0%,transparent_60%)]" />
                    <svg class="w-6 h-6 text-ink relative z-10 drop-shadow-text-faint" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path d="M12 4V16M12 16L6 10M12 16L18 10" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
                        <path d="M4 20H20" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
                    </svg>
                </div>
            </div>
        }.into_any(),

        "subtitles" => view! {
            <div class="relative w-16 h-16 flex items-center justify-center">
                <div class="absolute -top-1 right-2 text-accent text-xs animate-pulse font-bold">"✦"</div>
                <div class="absolute bottom-2 -left-1 text-[#ff7700] text-micro animate-pulse font-bold">"✦"</div>
                <div class="absolute top-4 -left-2 text-accent text-pico animate-pulse font-bold">"✦"</div>
                <div class="absolute inset-2 rounded-full bg-accent-alt/10 blur-lg pointer-events-none" />
                <svg class="w-14 h-14 relative z-10" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <defs>
                        <linearGradient id="scopeGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                            <stop offset="0%" style="stop-color: var(--color-accent-alt)" />
                            <stop offset="50%" stop-color="#7e0fff" />
                            <stop offset="100%" stop-color="#ff5500" />
                        </linearGradient>
                        <linearGradient id="standGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                            <stop offset="0%" stop-color="#555577" />
                            <stop offset="100%" stop-color="#222233" />
                        </linearGradient>
                    </defs>
                    <line x1="32" y1="36" x2="20" y2="58" stroke="url(#standGrad)" stroke-width="3" stroke-linecap="round" />
                    <line x1="32" y1="36" x2="44" y2="58" stroke="url(#standGrad)" stroke-width="3" stroke-linecap="round" />
                    <line x1="32" y1="36" x2="32" y2="55" stroke="url(#standGrad)" stroke-width="3.5" stroke-linecap="round" />
                    <circle cx="32" cy="36" r="4.5" fill="#3f3f5a" stroke="#fff" stroke-width="1" />
                    <g transform="rotate(-30 32 32)">
                        <rect x="42" y="24" width="6" height="16" rx="1" fill="#fff" />
                        <path d="M16 26H42V38H16C15 38 14 37 14 36V28C14 27 15 26 16 26Z" fill="url(#scopeGrad)" stroke="#fff" stroke-width="1" />
                        <rect x="22" y="25" width="3" height="14" fill="#ffea00" />
                        <rect x="6" y="29" width="8" height="6" fill="#222" stroke="#fff" stroke-width="1" />
                        <path d="M2 28H6V36H2C1 36 0 35 0 34V30C0 29 1 28 2 28Z" fill="#555" />
                    </g>
                </svg>
            </div>
        }.into_any(),

        _ => view! { // "place"
            <div class="relative w-16 h-16 flex items-center justify-center">
                <div class="absolute inset-0 rounded-full bg-gradient-to-br from-accent/10 to-[#ff500f]/10 blur-md pointer-events-none" />
                // Back card (beige)
                <div class="absolute top-2 left-2 w-10 h-10 rounded-card bg-gradient-to-br from-[#ffd2a0] to-[#ff9f68] border border-ink/20 shadow-md flex flex-col items-center justify-center transform -rotate-6">
                    <div class="flex gap-2.5 mb-1">
                        <div class="w-1.5 h-1.5 rounded-full bg-[#3d1e03]" />
                        <div class="w-1.5 h-1.5 rounded-full bg-[#3d1e03]" />
                    </div>
                    <svg class="w-3.5 h-1.5 text-[#3d1e03]" viewBox="0 0 24 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path d="M2 2C6 9 18 9 22 2" stroke="currentColor" stroke-width="4" stroke-linecap="round" />
                    </svg>
                </div>
                // Front card (red/magenta)
                <div class="absolute bottom-2 right-2 w-11 h-11 rounded-card bg-gradient-to-br from-accent to-[#ff500f] border border-ink/30 shadow-lg flex flex-col items-center justify-center transform rotate-6">
                    <div class="flex gap-3 mb-1">
                        <div class="w-1.5 h-1.5 rounded-full bg-ink" />
                        <div class="w-1.5 h-1.5 rounded-full bg-ink" />
                    </div>
                    <svg class="w-4 h-2 text-ink" viewBox="0 0 24 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path d="M2 2C6 9 18 9 22 2" stroke="currentColor" stroke-width="4" stroke-linecap="round" />
                    </svg>
                </div>
            </div>
        }.into_any(),
    }
}
