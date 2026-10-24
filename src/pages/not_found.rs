use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::components::layout::nav::Navbar;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimensionConfig {
    pub id: &'static str,
    pub title: &'static str,
    pub backdrop: &'static str,
    pub poster: &'static str,
    pub title_img: &'static str,
    pub subtitle: &'static str,
    pub synopsis: &'static str,
    pub error_code: &'static str,
}

pub const DIMENSIONS: [DimensionConfig; 6] = [
    DimensionConfig {
        id: "dim1",
        title: "nf-dim1-title",
        backdrop: "/assets/404_assets/dim1-backdrop.jpeg",
        poster: "/assets/404_assets/dim1-poster.jpeg",
        title_img: "/assets/404_assets/dim1-title.png",
        subtitle: "nf-original",
        synopsis: "nf-dim1-synopsis",
        error_code: "NSES-404-CUBE",
    },
    DimensionConfig {
        id: "dim2",
        title: "nf-dim2-title",
        backdrop: "/assets/404_assets/dim2-backdrop.jpeg",
        poster: "/assets/404_assets/dim2-poster.jpeg",
        title_img: "/assets/404_assets/dim2-title.png",
        subtitle: "nf-original",
        synopsis: "nf-dim2-synopsis",
        error_code: "NSES-404-SIGNAL",
    },
    DimensionConfig {
        id: "dim3",
        title: "nf-dim3-title",
        backdrop: "/assets/404_assets/dim3-backdrop.jpeg",
        poster: "/assets/404_assets/dim3-poster.jpeg",
        title_img: "/assets/404_assets/dim3-title.png",
        subtitle: "nf-original",
        synopsis: "nf-dim3-synopsis",
        error_code: "NSES-404-SHATTER",
    },
    DimensionConfig {
        id: "dim4",
        title: "nf-dim4-title",
        backdrop: "/assets/404_assets/dim4-backdrop.jpeg",
        poster: "/assets/404_assets/dim4-poster.jpeg",
        title_img: "/assets/404_assets/dim4-title.png",
        subtitle: "nf-original",
        synopsis: "nf-dim4-synopsis",
        error_code: "NSES-404-REEL",
    },
    DimensionConfig {
        id: "dim5",
        title: "nf-dim5-title",
        backdrop: "/assets/404_assets/dim5-backdrop.jpeg",
        poster: "/assets/404_assets/dim5-poster.jpeg",
        title_img: "/assets/404_assets/dim5-title.png",
        subtitle: "nf-original",
        synopsis: "nf-dim5-synopsis",
        error_code: "NSES-404-EYE",
    },
    DimensionConfig {
        id: "dim6",
        title: "nf-dim6-title",
        backdrop: "/assets/404_assets/dim6-backdrop.jpeg",
        poster: "/assets/404_assets/dim6-poster.jpeg",
        title_img: "/assets/404_assets/dim6-title.png",
        subtitle: "nf-original",
        synopsis: "nf-dim6-synopsis",
        error_code: "NSES-404-NEBULA",
    },
];

pub const GLITCH_ZONES: [&str; 6] = [
    "nf-zone-1",
    "nf-zone-2",
    "nf-zone-3",
    "nf-zone-4",
    "nf-zone-5",
    "nf-zone-6",
];

#[component]
pub fn NotFoundPage() -> impl IntoView {
    let navigate = use_navigate();
    let (active_index, set_active_index) = signal(0usize);
    let (is_drawer_open, set_is_drawer_open) = signal(false);
    let (dropdown_open, set_dropdown_open) = signal(false);

    let active = move || DIMENSIONS[active_index.get()];

    let current_path = move || {
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_else(|| "/404".to_string())
    };

    let user_agent = move || {
        web_sys::window()
            .and_then(|w| w.navigator().user_agent().ok())
            .unwrap_or_else(|| crate::i18n::t("nf-unknown-agent"))
    };

    let iso_now = move || {
        js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default()
    };

    let scroll_to_top = move || {
        if let Some(win) = web_sys::window() {
            win.scroll_to_with_x_and_y(0.0, 0.0);
        }
    };

    view! {
        <div class="relative min-h-screen bg-page text-ink overflow-x-hidden select-none pb-16">
            // Primary Navbar
            <Navbar />

            // CRT scanlines overlay
            // colour: the error page's scan lines (decoration in red, green and blue)
            <div class="fixed inset-0 pointer-events-none z-(--z-grain) opacity-[0.25] mix-blend-overlay bg-[linear-gradient(rgba(18,16,16,0)_50%,rgba(0,0,0,0.3)_50%),linear-gradient(90deg,rgba(255,0,0,0.06),rgba(0,255,0,0.02),rgba(0,0,255,0.06))] bg-[size:100%_4px,6px_100%]" />

            // SubNav Header: "Alternate Dimensions" + "Glitch Zones ▾"
            <div class="pt-20 px-6 md:px-14 lg:px-16 flex items-center gap-6 z-30 relative">
                <h1 class="text-3xl md:text-4xl font-bold tracking-tight text-ink">
                    {crate::i18n::t("not-found-alternate-dimensions")}
                </h1>

                // Glitch Zones dropdown
                <div class="relative">
                    <button
                        on:click=move |_| set_dropdown_open.update(|o| *o = !*o)
                        class="flex items-center gap-2 bg-void/80 hover:bg-void border border-ink/70 px-4 py-1.5 text-sm font-bold text-ink rounded-button transition-colors"
                    >
                        <span>{move || crate::i18n::t(GLITCH_ZONES[active_index.get()])}</span>
                        <i class="ph-bold ph-caret-down text-xs" />
                    </button>

                    {move || if dropdown_open.get() {
                        view! {
                            <div class="absolute left-0 top-full mt-2 w-48 bg-void/95 border border-ink/20 rounded-control shadow-2xl py-1 z-50">
                                {GLITCH_ZONES.iter().enumerate().map(|(idx, &name)| {
                                    let is_active = move || active_index.get() == idx;
                                    view! {
                                        <button
                                            on:click=move |_| {
                                                set_active_index.set(idx);
                                                set_dropdown_open.set(false);
                                                scroll_to_top();
                                            }
                                            class=move || {
                                                if is_active() {
                                                    "w-full text-left px-4 py-2 text-xs font-bold text-ink bg-ink/20 hover:bg-ink/30 transition-colors"
                                                } else {
                                                    "w-full text-left px-4 py-2 text-xs font-medium text-ink/80 hover:bg-ink/10 hover:text-ink transition-colors"
                                                }
                                            }
                                        >
                                            {crate::i18n::t(name)}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                </div>
            </div>

            // Dynamic Banner Backdrop
            <div class="relative h-[60vh] sm:h-[70vh] md:h-[80vh] w-full overflow-hidden bg-void mt-4">
                <div class="absolute inset-0 z-0 pointer-events-none transition-all duration-(--motion-slowest)">
                    <img
                        src=move || active().backdrop
                        class="w-full h-full object-cover opacity-70 scale-105 transition-all duration-(--motion-glacial) ease-out"
                        alt=crate::i18n::t("not-found-cinematic-background")
                    />
                    <div class="absolute inset-0 bg-gradient-to-b from-void/80 via-void/15 to-transparent" />
                    <div class="absolute inset-0 bg-gradient-to-t from-page via-page/30 to-transparent" />
                    <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_20%,#000000_90%)] opacity-65" />
                </div>

                // Hero Content
                <div class="absolute top-0 left-0 w-full h-full flex flex-col justify-end z-20 pl-[calc(1.5rem+env(safe-area-inset-left,0px))] md:pl-14 lg:pl-16 pr-4 md:pr-12 pointer-events-none pb-[12%] sm:pb-[9%] md:pb-[6%]">
                    <div class="max-w-[95%] sm:max-w-lg md:max-w-xl lg:max-w-2xl space-y-4 md:space-y-5 pointer-events-auto">
                        <div class="text-micro sm:text-xs font-bold tracking-[0.25em] text-danger drop-shadow-brand-sm">
                            {move || crate::i18n::t(active().subtitle)}
                        </div>

                        // Neon 404 Title Image with layers
                        <div class="relative flex items-end mb-2 md:mb-4">
                            <div class="relative inline-flex items-end">
                                <img
                                    src=move || active().title_img
                                    aria-hidden="true"
                                    class="absolute object-contain object-bottom select-none opacity-45 blur-[20px] scale-[1.06]"
                                    style="filter: brightness(1.5) sepia(1) hue-rotate(-50deg); width: 100%; height: 100%; inset: 0; mix-blend-mode: screen;"
                                />
                                <img
                                    src=move || active().title_img
                                    aria-hidden="true"
                                    class="absolute object-contain object-bottom select-none opacity-60 blur-[4px] scale-[1.01]"
                                    style="filter: brightness(1.2) sepia(1) hue-rotate(-50deg); width: 100%; height: 100%; inset: 0; mix-blend-mode: screen;"
                                />
                                <img
                                    src=move || active().title_img
                                    alt=move || crate::i18n::t(active().title)
                                    class="relative object-contain object-bottom select-none animate-pulse duration-[3000ms]"
                                    style="max-height: clamp(80px, 18vw, 190px); max-width: clamp(250px, 55vw, 550px); mix-blend-mode: screen;"
                                />
                            </div>
                        </div>

                        <p class="text-fine sm:text-label md:text-body font-medium text-ink/90 line-clamp-3 leading-relaxed max-w-[90%] sm:max-w-lg transition-all duration-(--motion-slowest) drop-shadow-md">
                            {move || crate::i18n::t(active().synopsis)}
                        </p>

                        <div class="flex items-center flex-wrap gap-2 md:gap-3 pt-2">
                            <button
                                on:click={
                                    let nav = navigate.clone();
                                    move |_| nav("/browse", Default::default())
                                }
                                class="btn-ink flex items-center justify-center px-6 sm:px-8 h-[36px] md:h-[42px] text-body-sm md:text-body-lg gap-2 active:scale-95 shadow-lg duration-(--motion-base)"
                            >
                                <AppIcon glyph=Glyph::HomeFilled class="text-xl text-void" />
                                <span>{crate::i18n::t("nf-return-home")}</span>
                            </button>

                            <button
                                on:click=move |_| set_is_drawer_open.set(true)
                                class="btn-glass flex items-center justify-center px-5 sm:px-8 h-[36px] md:h-[42px] transition-all text-body-sm md:text-body-lg gap-2 active:scale-95 shadow-lg duration-(--motion-base)"
                            >
                                <span>{crate::i18n::t("nf-more-info")}</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            // Alternate Dimensions Row (Horizontal 16:9 cards)
            <main class="relative z-10 -mt-8 sm:-mt-12 md:-mt-16 space-y-4 md:space-y-6 px-6 md:px-14 lg:px-16">
                <h2 class="text-ink text-xl md:text-2xl font-black mb-4">
                    {crate::i18n::t("not-found-alternate-dimensions-more-like-this")}
                </h2>
                <div class="flex gap-3 overflow-x-auto pb-4 scrollbar-hide py-2">
                    {DIMENSIONS.iter().enumerate().map(|(idx, dim)| {
                        let is_sel = move || active_index.get() == idx;
                        view! {
                            <div
                                on:click=move |_| {
                                    set_active_index.set(idx);
                                    scroll_to_top();
                                }
                                class=move || {
                                    format!(
                                        "relative flex-none w-[200px] sm:w-[240px] md:w-[280px] aspect-video rounded-field overflow-hidden cursor-pointer transition-all duration-(--motion-slow) transform hover:scale-105 border-2 bg-raised {}",
                                        if is_sel() { "border-brand shadow-brand-glow" } else { "border-transparent hover:border-ink/30" }
                                    )
                                }
                            >
                                <img
                                    src=dim.backdrop
                                    alt=crate::i18n::t(dim.title)
                                    class="w-full h-full object-cover"
                                    loading="lazy"
                                />
                                <div class="absolute inset-0 bg-void/20 hover:bg-void/0 transition-colors flex items-center justify-center p-4">
                                    <img
                                        src=dim.title_img
                                        alt=crate::i18n::t(dim.title)
                                        class="max-h-[70%] max-w-[85%] object-contain select-none filter drop-shadow-text"
                                        style="mix-blend-mode: screen;"
                                    />
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </main>

            // Technical Details Drawer
            {move || if is_drawer_open.get() {
                view! {
                    <div class="fixed inset-0 z-(--z-dialog) flex items-center justify-end bg-void/60 backdrop-blur-sm">
                        <div
                            class="absolute inset-0"
                            on:click=move |_| set_is_drawer_open.set(false)
                        />
                        <div class="relative w-full max-w-md sm:max-w-lg h-full bg-raised/95 border-l border-ink/10 p-6 sm:p-8 flex flex-col justify-between shadow-vignette z-10">
                            <div class="space-y-6">
                                <div class="flex items-center justify-between border-b border-ink/10 pb-4">
                                    <div class="flex items-center gap-3">
                                        <i class="ph-fill ph-warning text-danger text-2xl" />
                                        <h3 class="text-xl sm:text-2xl font-bold tracking-tight text-ink font-sans">
                                            {crate::i18n::t("not-found-diagnostic-report")}
                                        </h3>
                                    </div>
                                    <button
                                        on:click=move |_| set_is_drawer_open.set(false)
                                        class="btn-circle btn-circle-clear btn-circle-sm text-ink/60 hover:text-ink"
                                    >
                                        <i class="ph-bold ph-x text-2xl" />
                                    </button>
                                </div>

                                <div class="space-y-4 text-sm font-sans text-ink/80">
                                    <div class="space-y-1">
                                        <span class="text-xs text-ink/40 uppercase tracking-wider">{crate::i18n::t("nf-system-code")}</span>
                                        <p class="text-base font-mono text-danger font-semibold">{move || active().error_code}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-ink/40 uppercase tracking-wider">{crate::i18n::t("nf-endpoint")}</span>
                                        <p class="text-base font-mono bg-void/40 px-3 py-1.5 rounded-control border border-ink/5 whitespace-nowrap overflow-x-auto select-text">
                                            {current_path()}
                                        </p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-ink/40 uppercase tracking-wider">{crate::i18n::t("nf-timestamp")}</span>
                                        <p class="text-base font-mono">{iso_now()}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-ink/40 uppercase tracking-wider">{crate::i18n::t("nf-agent")}</span>
                                        <p class="text-xs font-mono text-ink/50 bg-void/20 p-2 rounded-control leading-normal border border-ink/5 max-h-24 overflow-y-auto select-text">
                                            {user_agent()}
                                        </p>
                                    </div>
                                </div>

                                <div class="bg-danger/10 border border-danger/20 rounded-control p-4 text-label leading-relaxed text-danger-mid font-sans">
                                    <strong>{crate::i18n::t("nf-system-notice")}" "</strong>
                                    {crate::i18n::t("not-found-the-targeted-router-endpoint-has")}
                                </div>
                            </div>

                            <button
                                on:click={
                                    let nav = navigate.clone();
                                    move |_| {
                                        set_is_drawer_open.set(false);
                                        nav("/browse", Default::default());
                                    }
                                }
                                class="w-full py-3.5 bg-brand hover:bg-danger-strong text-ink font-bold rounded-button flex items-center justify-center gap-2 transition-colors duration-(--motion-base) active:scale-[0.98] shadow-lg font-sans text-sm sm:text-base mt-8"
                            >
                                <i class="ph-bold ph-caret-left text-xl" />
                                <span>{crate::i18n::t("nf-return-home")}</span>
                            </button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </div>
    }
}
