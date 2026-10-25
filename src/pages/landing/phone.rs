//! The landing page on phones (note 114): the app's first screens.
use super::wall;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// The phone version of the landing page: four full screen slides (a wall of posters, then three illustrations) that the person
/// swipes through, dots, and one big button, like the first screens of the Netflix app. From a small tablet width up the long
/// page is shown instead, so the layout follows the width of the window and a touch laptop gets the page that fits its width.
#[component]
pub fn MobileWelcome() -> impl IntoView {
    let navigate = use_navigate();
    let nav_login = navigate.clone();
    let nav_start = navigate.clone();
    let track = NodeRef::<leptos::html::Div>::new();
    let (index, set_index) = signal(0usize);
    // art for each slide: None is the wall of posters
    let slides: [(Option<&'static str>, &'static str, &'static str); 4] = [
        (None, "login-unlimited-films-series-and-more", "auth-tick-free"),
        (Some("/assets/landing/slide-2.svg"), "landing-slide-everyone", "auth-tick-profiles"),
        (Some("/assets/landing/slide-3.svg"), "login-reason-screens-title", "auth-tick-devices"),
        (Some("/assets/landing/slide-4.svg"), "login-reason-place-title", "landing-slide-place"),
    ];
    let on_scroll = move |_| {
        if let Some(el) = track.get_untracked() {
            let width = el.client_width().max(1);
            set_index.set(((el.scroll_left() as f64) / (width as f64)).round() as usize);
        }
    };

    view! {
        <div class="relative isolate flex h-svh min-h-[560px] flex-col overflow-hidden bg-linear-to-b from-brand-deep/35 via-page to-void font-sans text-ink">
            // the wall of posters fills the whole screen behind the header and fades into the page, so nothing shows an edge; the other slides keep the plain page
            <div class="pointer-events-none absolute inset-0 -z-10 transition-opacity duration-(--motion-slow)" class=("opacity-0", move || index.get() != 0) aria-hidden="true">
                <img src=wall("phone") class="h-full w-full object-cover object-top" alt="" />
                <div class="absolute inset-0" style="background: linear-gradient(to bottom, rgba(0,0,0,.88) 0%, rgba(0,0,0,.35) 14%, rgba(0,0,0,0) 30%, rgba(0,0,0,.25) 46%, rgba(0,0,0,.82) 62%, #000 76%)"></div>
            </div>
            <header class="relative z-10 flex items-center justify-between px-4 py-3">
                <a href="/" aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="h-7 w-auto" /></a>
                <div class="flex items-center gap-1 text-fine font-bold uppercase tracking-wide">
                    <a href=crate::config::routes::PRIVACY class="px-3 py-3">{crate::i18n::t("help-ui-legal-privacy")}</a>
                    <button on:click=move |_| nav_login("/login", Default::default()) class="px-3 py-3">{crate::i18n::t("login-sign-in")}</button>
                </div>
            </header>
            <div
                node_ref=track
                on:scroll=on_scroll
                class="relative flex min-h-0 flex-1 snap-x snap-mandatory scroll-smooth overflow-x-auto overscroll-x-contain [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
            >
                {slides
                    .into_iter()
                    .map(|(art, title, body)| view! {
                        <section class="relative flex h-full w-full shrink-0 snap-center flex-col">
                            <div class="relative min-h-0 flex-1">
                                {match art {
                                    None => view! { <div></div> }.into_any(),
                                    Some(src) => view! {
                                        <img
                                            src=src
                                            class="mx-auto h-full w-full object-contain"
                                            style="-webkit-mask-image: radial-gradient(closest-side, #000 62%, transparent 100%); mask-image: radial-gradient(closest-side, #000 62%, transparent 100%)"
                                            alt=""
                                        />
                                    }
                                    .into_any(),
                                }}
                            </div>
                            <div class="px-8 pb-4 text-center">
                                <h1 class="text-display-sm font-black leading-tight drop-shadow-text">{crate::pages::help::tr(title)}</h1>
                                <p class="mt-3 text-body-lg font-semibold text-ink-soft">{crate::pages::help::tr(body)}</p>
                            </div>
                        </section>
                    })
                    .collect_view()}
            </div>
            <div class="flex items-center justify-center gap-3 pb-5 pt-2" aria-hidden="true">
                {(0..4usize).map(|i| view! {
                    <span
                        class="h-2.5 w-2.5 rounded-full transition-colors duration-(--motion-base)"
                        class=("bg-ink", move || index.get() == i)
                        class=("bg-ink/30", move || index.get() != i)
                    ></span>
                }).collect_view()}
            </div>
            <div class="px-4 pb-[max(1.25rem,env(safe-area-inset-bottom))]">
                <button
                    on:click=move |_| nav_start("/login", Default::default())
                    class="flex h-12 w-full items-center justify-center rounded-button bg-brand text-body-lg font-bold text-ink active:scale-[0.98]"
                >
                    {crate::i18n::t("login-get-started")}
                </button>
            </div>
        </div>
    }
}
