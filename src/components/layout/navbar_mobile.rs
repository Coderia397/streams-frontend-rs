use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use crate::components::layout::bottom_nav_mobile::{BottomNavMobile, BottomNavItem};
use crate::store::use_profile_store;

#[component]
pub fn NavbarMobile(
    /// How far the page is scrolled; the header goes from transparent to solid with it
    #[prop(into)] scroll_y: Signal<f64>,
    #[prop(optional)] _search_query: Option<ReadSignal<String>>,
    #[prop(optional)] _set_search_query: Option<WriteSignal<String>>,
) -> impl IntoView {
    let profile_store = use_profile_store();
    let navigate = use_navigate();
    
    let (active_bottom_nav, set_active_bottom_nav) = signal("home");
    // the tab follows the page, also on a direct visit or Back
    let loc_tab = use_location();
    Effect::new(move |_| {
        let p = loc_tab.pathname.get();
        let tab = if p.starts_with("/search") { "search" } else if p.starts_with("/clips") || p.starts_with("/latest") { "clips" } else if p.starts_with("/browse/my-list") { "settings" } else { "home" };
        set_active_bottom_nav.set(tab);
    });

    // The header and the pills under it leave when the page is scrolled down and come back at the first swipe up (not only at the very top), as in the app. The state is on the root element
    // as `--nav-shown` (1 or 0) and `--hdr-solid` (0 to 1, solid black by 60px) so the pills under the header follow it.
    Effect::new(move |prev: Option<f64>| {
        let y = scroll_y.get();
        let last = prev.unwrap_or(y);
        let root = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()).and_then(|e| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(e).ok());
        if let Some(root) = root {
            let style = root.style();
            let shown = if y <= 80.0 || y < last - 6.0 { Some("1") } else if y > last + 6.0 { Some("0") } else { None };
            if let Some(v) = shown { let _ = style.set_property("--nav-shown", v); }
            let _ = style.set_property("--hdr-solid", &format!("{:.3}", (y / 60.0).clamp(0.0, 1.0)));
        }
        if (y - last).abs() > 6.0 { y } else { last }
    });

    // As in the app: a soft shade over the picture at the top, solid black (the colour of the pill row under it) by about 60px of scrolling
    let scroll_style = move || {
        let solid = (scroll_y.get() / 60.0).clamp(0.0, 1.0);
        format!(
            "background: linear-gradient(to bottom, rgba(0,0,0,{:.3}) 0%, rgba(0,0,0,{:.3}) 60%, rgba(0,0,0,0) 100%), rgba(0,0,0,{:.3});",
            0.45 * (1.0 - solid), 0.12 * (1.0 - solid), solid
        )
    };

    // Page title next to the logo, like the app header ("Home", "Series", ...).
    let location = use_location();
    let page_title = move || {
        let p = location.pathname.get();
        if p.starts_with("/browse/my-list") { crate::i18n::t("nav-my-list") }
        else if p.starts_with("/browse/series") || p.starts_with("/browse/shows") { crate::i18n::t("nav-series") }
        else if p.starts_with("/browse/films") || p.starts_with("/browse/movies") { crate::i18n::t("nav-films") }
        else if p.starts_with("/browse/games") || p.starts_with("/games") { crate::i18n::t("nav-games") }
        else if p.starts_with("/latest") { crate::i18n::t("nav-new-hot") }
        else if let Some(id) = p.strip_prefix("/browse/genre/").and_then(|r| r.split('/').next()).and_then(|r| r.parse::<u32>().ok()) {
            match id {
                crate::data::page_genres::SERIES_PAGE_ID => crate::i18n::t("nav-series"),
                crate::data::page_genres::FILMS_PAGE_ID => crate::i18n::t("nav-films"),
                n => crate::data::page_genres::genre_name(n).map(crate::i18n::genre_label).unwrap_or_default(),
            }
        }
        else if p == "/browse" || p == "/" || p.is_empty() { crate::i18n::t("nav-home") }
        else { String::new() }
    };

    // The pages under Home (Series, Films, a genre) have a back arrow instead of the logo and no icons on the right
    let on_sub_page = move || {
        let p = location.pathname.get();
        p.starts_with("/browse/series") || p.starts_with("/browse/shows") || p.starts_with("/browse/films") || p.starts_with("/browse/movies") || p.starts_with("/browse/genre/") || p.starts_with("/latest")
    };
    let on_search = move || location.pathname.get().starts_with("/search");
    let go_back = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            let can_go_back = web_sys::window().and_then(|w| w.history().ok()).and_then(|h| h.length().ok()).is_some_and(|n| n > 1);
            if can_go_back { if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) { let _ = h.back(); } } else { navigate("/browse", Default::default()); }
        })
    };

    // Avatar of the active profile (first profile until the profile store knows better), used in the profile tab and the header
    let active_profile = move || {
        let id = profile_store.active_profile_id.get();
        profile_store.profiles.with(|l| l.iter().find(|p| Some(&p.id) == id.as_ref()).or(l.first()).cloned())
    };
    let current_avatar = move || active_profile().and_then(|p| p.avatar_url).unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
    let is_kids = Signal::derive(move || active_profile().is_some_and(|p| p.is_kids));

    let nav_to_browse = navigate.clone();
    let nav_to_latest = navigate.clone();
    let nav_to_search = navigate.clone();
    let nav_to_settings = navigate.clone();

    let bottom_nav_items = vec![
        BottomNavItem {
            id: "home",
            icon: view! { <i class="ph-bold ph-house text-heading"></i> }.into_any(),
            label: "nav-home",
            on_click: Callback::new(move |_| {
                set_active_bottom_nav.set("home");
                nav_to_browse("/browse", Default::default());
            }),
            shown: Signal::derive(|| true),
        },
        BottomNavItem {
            id: "clips",
            icon: view! { <i class="ph-bold ph-play-circle text-heading"></i> }.into_any(),
            label: "nav-clips",
            on_click: Callback::new(move |_| {
                set_active_bottom_nav.set("clips");
                nav_to_latest("/latest", Default::default());
            }),
            shown: Signal::derive(move || !is_kids.get()),
        },
        BottomNavItem {
            id: "search",
            icon: view! { <i class="ph-bold ph-magnifying-glass text-heading"></i> }.into_any(),
            label: "nav-search",
            on_click: Callback::new(move |_| {
                set_active_bottom_nav.set("search");
                nav_to_search("/search", Default::default());
            }),
            shown: Signal::derive(|| true),
        },
        BottomNavItem {
            id: "settings",
            icon: view! { 
                <div class="w-6 h-6 rounded-control overflow-hidden flex items-center justify-center bg-brand text-ink font-bold text-micro">
                    {move || if active_profile().is_some_and(|p| p.is_kids && p.avatar_url.is_none()) {
                        view! { <crate::components::profiles::kids_avatar::KidsAvatar size=24.0 /> }.into_any()
                    } else {
                        view! { <img src=current_avatar() alt=crate::i18n::t("navbar-mobile-profile") class="w-full h-full object-cover" /> }.into_any()
                    }}
                </div>
            }.into_any(),
            label: "nav-my-list",
            on_click: Callback::new(move |_| {
                set_active_bottom_nav.set("settings");
                nav_to_settings("/browse/my-list", Default::default());
            }),
            shown: Signal::derive(|| true),
        },
    ];


    view! {
        <div class="sm:hidden select-none">
            // ── Top Brand Header ───────────────────────────────────────────────
            <header
                class:hidden=on_search
                class="mobile-header fixed top-0 left-0 right-0 z-(--z-nav) pl-1 pr-1 pt-[env(safe-area-inset-top)] h-[calc(var(--nav-h-mobile)+env(safe-area-inset-top))] box-content flex items-center justify-between pointer-events-auto transition-colors duration-(--motion-base)"
                style=scroll_style
            >
                // Logo mark and page title on Home, a back arrow and the page title on the pages under it, as in the app header
                <div class="flex items-center min-w-0">
                    {move || if on_sub_page() {
                        view! {
                            <button
                                on:click=move |_| go_back.run(())
                                class="flex h-11 w-11 flex-shrink-0 items-center justify-center ms-1 text-ink active:scale-95 transition-transform cursor-pointer"
                                aria-label=crate::i18n::t("navbar-mobile-back")
                            >
                                <i class="ph-bold ph-arrow-left text-heading"></i>
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <a href="/browse" class="flex h-11 w-11 flex-shrink-0 items-center justify-center ms-1">
                                <img
                                    src=crate::brand::LOGO_EMBLEM
                                    alt=crate::brand::NAME
                                    class="h-7 w-7 cursor-pointer select-none object-contain transition-transform active:scale-95"
                                />
                            </a>
                        }.into_any()
                    }}
                    <span class="ms-1 truncate text-heading font-bold leading-none tracking-tight text-ink">{page_title}</span>
                </div>
            </header>

            // ── Bottom Nav ───────────────────────────────────────────────────
            <BottomNavMobile items=bottom_nav_items active_id=active_bottom_nav />

        </div>
    }
}
