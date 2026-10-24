//! The landing page on tablets and computers (note 114): the picture wall, the form, the Top 10, the reasons, the questions.
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use super::wall;
use super::art::{FeatureIcon, RankNumber};
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

// ── Size constants ────────────────────────────────────────────────────────────

const CARD_CLASS: &str = "h-[110px] w-[75px] sm:h-[140px] sm:w-[95px] md:h-[170px] md:w-[115px] lg:h-[200px] lg:w-[135px]";


// ── LandingTopTenCard ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TopMovie {
    pub id: i64,
    pub title: String,
    pub poster_path: String,
}

#[component]
fn LandingTopTenCard(movie: TopMovie, index: usize) -> impl IntoView {
    let poster_src = if movie.poster_path.starts_with("http") {
        movie.poster_path.clone()
    } else {
        format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, movie.poster_path)
    };
    let alt = movie.title.clone();

    view! {
        <div class=format!(
            "relative flex-none {} ml-6 sm:ml-8 md:ml-10 lg:ml-12 mr-4 md:mr-6 flex items-end select-none z-10 cursor-default transition-transform duration-(--motion-base) hover:scale-105 hover:-translate-y-1",
            CARD_CLASS
        )>
            <RankNumber index=index />
            <div class="absolute right-0 bottom-0 w-full h-full z-10 rounded-card overflow-hidden shadow-soft">
                <img
                    src=poster_src
                    class="w-full h-full object-cover object-top"
                    alt=alt
                    loading="lazy"
                />
                <div class="absolute inset-0 bg-void/10 pointer-events-none" />
            </div>
        </div>
    }
}

// ── LandingTopTenRow ──────────────────────────────────────────────────────────

#[component]
fn LandingTopTenRow(#[prop(into)] title: String) -> impl IntoView {
    let movies = LocalResource::new(move || async move {
        crate::services::tmdb::fetch_trending("all").await.unwrap_or_default()
    });

    let scroll_ref = NodeRef::<leptos::html::Div>::new();
    let (scroll_left, set_scroll_left) = signal(false);
    let (scroll_right, set_scroll_right) = signal(true);

    let update_scroll_state = move || {
        if let Some(el) = scroll_ref.get() {
            let scroll_left_val = el.scroll_left();
            let scroll_width = el.scroll_width();
            let client_width = el.client_width();
            set_scroll_left.set(scroll_left_val > 0);
            set_scroll_right.set(scroll_left_val < scroll_width - client_width - 4);
        }
    };

    let scroll = move |dir: &'static str| {
        if let Some(el) = scroll_ref.get() {
            let client_width = el.client_width() as f64;
            let amount = client_width * 0.8;
            let delta = if dir == "right" { amount } else { -amount };
            el.scroll_by_with_x_and_y(delta, 0.0);
            update_scroll_state();
        }
    };

    view! {
        <div class="relative group/row">
            <h2 class="text-ink text-xl md:text-2xl font-black mb-4 px-6 md:px-20 lg:px-32 xl:px-44 2xl:px-56">
                {title}
            </h2>
            <div class="relative mx-6 md:mx-20 lg:mx-32 xl:mx-44 2xl:mx-56">
                // Left arrow — only when scrolled right
                {move || if scroll_left.get() {
                    view! {
                        <button
                            on:click=move |_| scroll("left")
                            class="absolute left-0 top-0 bottom-0 z-20 w-12 md:w-16 flex items-center justify-center bg-gradient-to-r from-void/90 to-transparent opacity-0 group-hover/row:opacity-100 transition-opacity"
                        >
                            <i class="ph-bold ph-caret-left text-4xl text-ink drop-shadow-lg" />
                        </button>
                    }.into_any()
                } else { view! {<></>}.into_any() }}
                // Right arrow
                {move || if scroll_right.get() {
                    view! {
                        <button
                            on:click=move |_| scroll("right")
                            class="absolute right-0 top-0 bottom-0 z-20 w-12 md:w-16 flex items-center justify-center bg-gradient-to-l from-void/90 to-transparent opacity-0 group-hover/row:opacity-100 transition-opacity"
                        >
                            <i class="ph-bold ph-caret-right text-4xl text-ink drop-shadow-lg" />
                        </button>
                    }.into_any()
                } else { view! {<></>}.into_any() }}

                <Suspense fallback=move || view! {
                    <div class="flex gap-2 overflow-x-auto overflow-y-hidden py-4 -my-4" style="scrollbar-width:none">
                        {(0..6).map(|_| view! {
                            <div class=format!("{} skeleton flex-none rounded-control", CARD_CLASS) aria-hidden="true" />
                        }).collect::<Vec<_>>()}
                    </div>
                }>
                    {move || {
                        movies.get().map(|data| {
                            let items: Vec<_> = data.into_iter().take(10).enumerate()
                                .map(|(i, m)| {
                                    let movie = TopMovie {
                                        id: m.id as i64,
                                        title: m.display_title().to_string(),
                                        poster_path: m.poster_path.clone().unwrap_or_default(),
                                    };
                                    view! { <LandingTopTenCard movie=movie index=i /> }
                                }).collect();
                            view! {
                                <div
                                    node_ref=scroll_ref
                                    on:scroll=move |_| update_scroll_state()
                                    class="flex gap-0 overflow-x-auto overflow-y-hidden scroll-smooth py-4 -my-4"
                                    style="scrollbar-width:none;-ms-overflow-style:none"
                                >
                                    {items}
                                </div>
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}


// ── FeatureCard ───────────────────────────────────────────────────────────────

#[component]
fn FeatureCard(id: &'static str, #[prop(into)] title: String, #[prop(into)] desc: String) -> impl IntoView {
    view! {
        // Netflix's reason card: 16 px corners and padding, a 149 degree navy to plum gradient, 24 px title, 16 px text at 70 % white
        <div class="relative flex min-h-[175px] flex-col justify-between overflow-hidden rounded-2xl bg-[linear-gradient(149deg,var(--color-reason-from)_0%,var(--color-reason-to)_96.86%)] p-4 group transition-all duration-(--motion-slow) md:min-h-[210px] lg:min-h-[318px]">
            <div>
                <h3 class="mb-4 text-title font-medium leading-tight text-ink md:text-heading">{title}</h3>
                <p class="text-body-md leading-normal text-ink/70">{desc}</p>
            </div>
            <div class="mt-4 self-end opacity-90 transition-transform duration-(--motion-slow) group-hover:scale-110">
                <FeatureIcon id=id />
            </div>
        </div>
    }
}

// ── AuthModal ─────────────────────────────────────────────────────────────────


/// The landing page on tablets and computers (note 114): the long page, as Netflix's signed out home.
#[component]
pub fn WebsiteLanding() -> impl IntoView {
    let (hero_email, set_hero_email) = signal(String::new());

    let navigate = use_navigate();
    let nav_signup = navigate.clone();
    let handle_get_started = Callback::new(move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        nav_signup(&format!("/signup?email={}", js_sys::encode_uri_component(&hero_email.get_untracked())), Default::default());
    });
    let nav_login = navigate.clone();

    let faq: [&'static str; 6] = ["what-is", "free-to-use", "need-account", "watch-on-tv", "sources", "kids-profile"];
    view! {
        <div class="min-h-screen bg-void font-sans text-ink">

            // Header over the picture wall
            <nav class="fixed top-0 z-(--z-nav) flex h-[88px] w-full items-center justify-between bg-linear-to-b from-void/80 to-transparent px-6 md:px-12 xl:px-[148px]">
                <a href="/" aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="h-7 w-auto md:h-9" /></a>
                <button
                    on:click=move |_| nav_login("/login", Default::default())
                    class="btn-brand btn-pill btn-pill-sm"
                >
                    {crate::i18n::t("login-sign-in")}
                </button>
            </nav>

            // Hero
            <section class="relative flex min-h-[70svh] flex-col items-center justify-center overflow-hidden px-6 pb-24 pt-28 text-center md:min-h-[78svh] md:pb-32 md:pt-40">
                <div class="absolute inset-0 z-0 overflow-hidden" aria-hidden="true">
                    <img src=wall("wide") class="h-full w-full scale-105 object-cover opacity-50" alt="" />
                    <div class="absolute inset-0 bg-linear-to-b from-void/60 via-void/20 to-void" />
                </div>
                <div class="relative z-10 mx-auto w-full max-w-[588px]">
                    <h1 class="text-heading-xl font-bold leading-tight drop-shadow-text md:text-display-lg md:leading-[70px]">
                        {crate::i18n::t("login-unlimited-films-series-and-more")}
                    </h1>
                    <p class="mt-4 text-title-sm font-medium md:text-title">{crate::i18n::t("auth-tick-free")}</p>
                    <p class="mt-8 text-body-md leading-6">{crate::i18n::t("login-ready-to-watch-enter-your")}</p>
                    <div class="mt-4"><LandingForm email=hero_email set_email=set_hero_email on_submit=handle_get_started /></div>
                </div>
                // curved glow at the bottom of the picture wall
                <div class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-16 overflow-hidden md:h-24" aria-hidden="true">
                    <div
                        class="absolute -inset-x-[15%] top-8 h-[400px] rounded-[50%] border-t-2 border-brand/80 bg-void"
                        style="box-shadow: 0 -12px 60px color-mix(in srgb, var(--color-brand) 45%, transparent)"
                    ></div>
                </div>
            </section>

            // Promo band
            <section class="mx-auto -mt-1 flex max-w-[1100px] flex-col gap-4 px-6 py-8 sm:flex-row sm:items-center md:px-12">
                <i class="ph-fill ph-gift text-display-lg text-brand-glow" aria-hidden="true"></i>
                <div class="flex flex-1 flex-col gap-4 rounded-card bg-linear-to-r from-brand-deep/60 to-void/60 p-5 sm:flex-row sm:items-center sm:justify-between">
                    <div>
                        <p class="text-body-md font-bold">{crate::pages::help::tr("landing-promo-title")}</p>
                        <p class="text-body-sm text-ink-body">{crate::i18n::t("landing-promo-body")}</p>
                    </div>
                    <a href="/help/article/free-to-use" class="btn-glass btn-pill btn-pill-md w-fit shrink-0">{crate::i18n::t("landing-learn-more")}</a>
                </div>
            </section>

            // Trending
            <section class="bg-void pb-6 pt-8">
                <LandingTopTenRow title=crate::i18n::t("media-badge-trending-now") />
            </section>

            // More reasons to join
            <section class="bg-void px-6 py-10 md:px-12 md:py-16 xl:px-[148px]">
                <h2 class="mb-6 text-title font-medium md:text-heading">{crate::i18n::t("login-more-reasons")}</h2>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    <FeatureCard id="tv" title=crate::i18n::t("login-reason-screens-title") desc=crate::i18n::t_args("login-reason-screens-body", &[("brand", fluent_bundle::FluentValue::from(crate::brand::NAME))]) />
                    <FeatureCard id="stream" title=crate::i18n::t("login-reason-hd-title") desc=crate::i18n::t("login-reason-hd-body") />
                    <FeatureCard id="subtitles" title=crate::i18n::t("login-reason-subs-title") desc=crate::i18n::t("login-reason-subs-body") />
                    <FeatureCard id="place" title=crate::i18n::t("login-reason-place-title") desc=crate::i18n::t("login-reason-place-body") />
                </div>
            </section>

            // Frequently asked questions, answered from the help articles
            <section class="px-6 py-8 md:px-12 xl:px-[148px]">
                <h2 class="mb-6 text-title font-medium md:text-heading">{crate::i18n::t("landing-faq")}</h2>
                <div class="grid items-start gap-2 md:grid-cols-2 md:gap-x-8">
                    {faq.into_iter().map(|slug| view! {
                        <details class="group bg-faq-item">
                            <summary class="flex min-h-[84px] cursor-pointer list-none items-center justify-between gap-4 p-6 text-title-sm hover:bg-faq-item-hover md:text-heading">
                                <span>{crate::pages::help::art(slug, "title")}</span>
                                <AppIcon glyph=Glyph::Plus class="text-heading transition-transform duration-(--motion-base) group-open:rotate-45" />
                            </summary>
                            <div class="border-t border-void p-6 text-title-sm leading-7 text-ink-soft md:text-heading md:leading-normal">
                                <p>{crate::pages::help::art(slug, "summary")}</p>
                                <a href=format!("/help/article/{slug}") class="mt-3 inline-block font-semibold underline underline-offset-2">{crate::i18n::t("landing-read-more")}</a>
                            </div>
                        </details>
                    }).collect_view()}
                </div>
            </section>

            // Bottom prompt
            <section class="flex w-full flex-col items-center bg-void px-6 py-14 text-center md:py-16">
                <p class="mx-auto mb-4 max-w-[783px] text-body-md leading-6">{crate::i18n::t("login-ready")}</p>
                <div class="w-full max-w-[783px]"><LandingForm email=hero_email set_email=set_hero_email on_submit=handle_get_started /></div>
            </section>

            <crate::pages::auth::AuthFooter />
        </div>
    }
}

/// Email field and Get Started button, stacked on a phone and side by side from a small tablet up.
#[component]
fn LandingForm(email: ReadSignal<String>, set_email: WriteSignal<String>, on_submit: Callback<leptos::ev::SubmitEvent>) -> impl IntoView {
    view! {
        <form on:submit=move |e| on_submit.run(e) class="flex w-full flex-col justify-center gap-2 sm:flex-row">
            <input
                type="email"
                autocomplete="email"
                prop:value=email
                on:input=move |e| set_email.set(event_target_value(&e))
                placeholder=crate::i18n::t("login-email-address")
                aria-label=crate::i18n::t("auth-email")
                class="h-14 w-full min-w-0 border border-scrim-edge bg-scrim-field px-4 text-body-md text-ink transition-colors placeholder:text-ink/70 focus:border-ink focus:outline-none sm:flex-1"
            />
            <button
                type="submit"
                class="btn-brand btn-pill btn-pill-xl text-title sm:text-heading"
            >
                {crate::i18n::t("login-get-started")}
                <i class="ph-bold ph-caret-right" aria-hidden="true"></i>
            </button>
        </form>
    }
}
