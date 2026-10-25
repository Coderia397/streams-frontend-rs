pub mod brand;
pub mod components;
pub mod config;
pub mod consent;
pub mod data;
pub mod form;
pub mod i18n;
pub mod keys;
pub mod legacy_keys;
pub mod models;
pub mod pages;
pub mod services;
pub mod store;
pub mod utils;


use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use pages::browse::BrowseHome;

#[component]
pub fn App() -> impl IntoView {
    // an account's data left without its session (it ended while the browser was closed, or with its tab) is cleared before any store reads it
    if crate::store::orphaned_account_data() {
        crate::store::forget_account_data();
    }
    // the form of the site (phone, tablet, desktop) and the input in use follow the window from here on (note 114)
    crate::form::start();
    crate::store::provide_ui_store();
    crate::store::provide_profile_store();
    // Who's watching again after a while away (shorter on a computer than on a phone)
    crate::services::profile_gate::start(crate::store::use_profile_store());
    crate::store::provide_library_store();
    crate::store::provide_watch_store();
    crate::consent::provide_ui();
    crate::components::profiles::switch_overlay::provide_switch();
    crate::components::profiles::switch_overlay::resume_compact_switch();
    crate::pages::settings::provide_prefs();
    crate::components::share::provide_share();
    // a browser that starts already signed in tells the account it is still here
    crate::services::account::keep_session_fresh();
    crate::services::device::register_in_background();

    // On help.<domain> the front door is the help center
    let on_help_host = crate::config::on_help_host();

    let profile_store = crate::store::use_profile_store();
    let has_no_profile = move || profile_store.active_profile_id.get().is_none();

    // The data of a profile (viewing progress, My List, ratings) is read for the profile in use whenever it changes, and the account's changes are pulled for it.
    let library_store = crate::store::use_library_store();
    let watch_store = crate::store::watch_store::use_watch_store();
    crate::services::sync::init(profile_store, library_store, watch_store);
    crate::services::sync::init_prefs(crate::pages::settings::use_prefs());
    Effect::new(move |_| {
        // no profile in use (Who's watching is up): nothing to read, and picking one loads the page afresh. Reading then also woke the
        // views of the page being taken down, which panicked on their disposed signals (2026-10-23)
        if profile_store.active_profile_id.get().is_none() {
            return;
        }
        watch_store.reload();
        library_store.reload();
        crate::services::sync::sync_in_background();
    });
    // when the session ends while the page is open (signed out here or in another tab, signed out everywhere, refused by the account), the
    // account's data leaves this browser and the tab loads afresh (nothing stays in memory): the start page, or sign in (and back) when the
    // account ended it
    let signed_in = crate::services::account::signed_in();
    let was_signed_in = StoredValue::new(signed_in.get_untracked());
    Effect::new(move |_| {
        let now = signed_in.get();
        if was_signed_in.get_value() && !now {
            crate::store::forget_account_data();
            let here = web_sys::window().and_then(|w| Some(format!("{}{}", w.location().pathname().ok()?, w.location().search().ok()?))).unwrap_or_default();
            let target = match crate::services::account::take_session_end() {
                crate::services::account::SessionEnd::Everywhere => crate::config::routes::LOGIN.to_string(),
                crate::services::account::SessionEnd::Refused => format!("{}?next={}", crate::config::routes::LOGIN, js_sys::encode_uri_component(&here)),
                _ => crate::config::routes::HOME.to_string(),
            };
            if let Some(w) = web_sys::window() {
                let _ = w.location().replace(&target);
            }
        }
        was_signed_in.set_value(now);
    });
    // a browser that already holds an account takes the account's profiles at start, and looks for changes whenever the tab comes back
    if crate::services::account::has_account() {
        crate::services::sync::refresh_profiles(profile_store);
    }
    if let Some(win) = web_sys::window() {
        let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::wrap(Box::new(|| crate::services::sync::sync_in_background()));
        let _ = win.add_event_listener_with_callback("focus", wasm_bindgen::JsCast::unchecked_ref(cb.as_ref()));
        cb.forget();
        // another tab signed in or out (it changed the session in local storage, or cleared it): this tab follows, through the account gate
        let on_storage = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::StorageEvent)>::wrap(Box::new(|e: web_sys::StorageEvent| {
            if e.key().is_none_or(|k| k == crate::keys::SESSION) {
                crate::services::account::announce();
            }
        }));
        let _ = win.add_event_listener_with_callback("storage", wasm_bindgen::JsCast::unchecked_ref(on_storage.as_ref()));
        on_storage.forget();
    }

    view! {
        <crate::components::media::info_modal::InfoModal />
        <crate::components::share::ShareHost />
        <crate::components::profiles::switch_overlay::GlobalSwitchOverlay />
        <Show when=has_no_profile>
            <crate::components::profiles::whos_watching::WhosWatchingGate />
        </Show>
        <Router>
            <HostGuard />
            <crate::components::media::info_modal::SharedTitleOpener />
            // an address the site does not have goes to the site's root (the landing page, or Home for a returning visitor), with no page of its own
            <Routes fallback=|| view! { <UnknownAddress /> }>
                <Route path=path!("/browse") view=|| view! { <HomeRoute kids=false /> } />
                <Route path=path!("/kids") view=|| view! { <HomeRoute kids=true /> } />
                // the developer pages exist only when the site is opened from this machine or its network (note 67 E8); anywhere else they go to the root, as an address the site does not have
                <Route path=path!("/dev/badges") view=|| if is_local_host() { view! { <crate::pages::dev_badges::DevBadgesPage /> }.into_any() } else { view! { <leptos_router::components::Redirect path="/" /> }.into_any() } />
                <Route path=path!("/dev/logos") view=|| if is_local_host() { view! { <crate::components::media::collection_logo::LogoGalleryPage /> }.into_any() } else { view! { <leptos_router::components::Redirect path="/" /> }.into_any() } />
                <Route path=path!("/browse/genre/:id") view=crate::pages::category::GenreRoute />
                <Route path=path!("/title/:slug") view=crate::pages::title::TitlePage />
                <Route path=path!("/watch/:id") view=crate::pages::player::PlayerPage />
                <Route path=path!("/watch/:kind/:id") view=crate::pages::player::PlayerPage />
                <Route
                    path=path!("")
                    view=move || {
                        if on_help_host {
                            view! { <leptos_router::components::Redirect path="/help" /> }.into_any()
                        } else if crate::services::account::is_returning() {
                            view! { <leptos_router::components::Redirect path="/browse" /> }.into_any()
                        } else {
                            view! { <crate::pages::landing::LandingPage /> }.into_any()
                        }
                    }
                />
                <Route path=path!("/profiles/manage") view=crate::components::profiles::whos_watching::ManageProfilesPage />
                <Route path=path!("/help") view=crate::pages::help::HelpPage />
                <Route path=path!("/help/search") view=crate::pages::help::ResultsPage />
                <Route path=path!("/help/contact") view=crate::pages::help::ContactPage />
                <Route path=path!("/help/article/:slug") view=crate::pages::help::ArticlePage />
                <Route path=path!("/search") view=crate::pages::search::SearchPage />
                <Route path=path!("/browse/my-list") view=crate::pages::my_list::MyListPage />
                // the addresses of the app: Series is genre 83 and Films is genre 34399; the named addresses of before lead there
                <Route path=path!("/browse/movies") view=|| view! { <leptos_router::components::Redirect path="/browse/genre/34399" /> } />
                <Route path=path!("/browse/films") view=|| view! { <leptos_router::components::Redirect path="/browse/genre/34399" /> } />
                <Route path=path!("/browse/shows") view=|| view! { <leptos_router::components::Redirect path="/browse/genre/83" /> } />
                <Route path=path!("/browse/series") view=|| view! { <leptos_router::components::Redirect path="/browse/genre/83" /> } />
                <Route path=path!("/browse/games") view=crate::pages::games::GamesPage />
                <Route path=path!("/games") view=crate::pages::games::GamesPage />
                <Route path=path!("/tv") view=|| view! { <leptos_router::components::Redirect path="/browse/shows" /> } />
                <Route path=path!("/movies") view=|| view! { <leptos_router::components::Redirect path="/browse/movies" /> } />
                <Route path=path!("/new") view=|| view! { <leptos_router::components::Redirect path="/latest" /> } />
                <Route path=path!("/list") view=|| view! { <leptos_router::components::Redirect path="/browse/my-list" /> } />
                <Route path=path!("/browse/original-audio") view=crate::pages::language::BrowseLanguagePage />
                <Route path=path!("/browse/language") view=|| view! { <leptos_router::components::Redirect path="/browse/original-audio" /> } />
                <Route path=path!("/latest") view=crate::pages::new_popular::NewPopularPage />
                // Notifications have a page on the phone only (the website lists them under its bell): a computer goes Home (note 116)
                <Route path=path!("/notifications") view=|| view! {
                    <crate::form::ByForm
                        phone=|| view! { <crate::pages::notifications::NotificationsPage /> }
                        desktop=|| view! { <leptos_router::components::Redirect path="/browse" /> }
                    />
                } />
                // Clips is the app's alone (Netflix's website has none): at a computer or tablet width the address goes Home, also when a phone
                // is switched to a big screen while on it (note 116)
                <Route path=path!("/clips") view=|| view! {
                    <crate::form::ByForm
                        phone=|| view! { <crate::pages::clips::ClipsPage /> }
                        desktop=|| view! { <leptos_router::components::Redirect path="/browse" /> }
                    />
                } />
                <Route path=path!("/reads") view=crate::pages::reads::ReadsPage />
                <Route path=path!("/settings") view=crate::pages::settings::SettingsPage />
                <Route path=path!("/settings/*any") view=crate::pages::settings::SettingsPage />
                <Route path=path!("/login") view=crate::pages::auth::LoginPage />
                <Route path=path!("/signup") view=crate::pages::auth::SignupPage />
                <Route path=path!("/forgot") view=crate::pages::auth::ForgotPage />
                <Route path=path!("/auth/confirm") view=crate::pages::auth::ConfirmPage />
                <Route path=path!("/welcome") view=|| view! { <leptos_router::components::Redirect path="/" /> } />
                // the policies, at config::routes (their earlier addresses are in config::routes::MOVED, answered by UnknownAddress)
                <Route path=path!("/help/legal/privacy") view=|| view! { <crate::pages::help::LegalPage doc="privacy" /> } />
                <Route path=path!("/help/legal/terms") view=|| view! { <crate::pages::help::LegalPage doc="terms" /> } />
                <Route path=path!("/help/legal/cookies") view=|| view! { <crate::pages::help::LegalPage doc="cookie-policy" /> } />
                <Route path=path!("/help/legal/dmca") view=|| view! { <crate::pages::help::LegalPage doc="dmca" /> } />
                <Route path=path!("/help/legal/credits") view=|| view! { <crate::pages::help::LegalPage doc="credits" /> } />
                <Route path=path!("/help/legal/disclaimer") view=|| view! { <crate::pages::help::LegalPage doc="disclaimer" /> } />
                <Route path=path!("/contact") view=|| view! { <leptos_router::components::Redirect path="/help/contact" /> } />
            </Routes>
            <crate::components::layout::account_gate::AccountGate />
            <crate::components::consent::CookieManager />
        </Router>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_target_linkage() {
        // Confirms host rlib target linkage and basic mapping resolution
        let result = models::mapping::map_genre_key_to_tmdb("1365");
        assert!(result.is_some());
        let ctx = result.unwrap();
        assert_eq!(ctx.title, "Action & Adventure");
        assert_eq!(ctx.tmdb_genre_id, Some(28));
    }

    #[test]
    fn test_static_data_linkage() {
        assert_eq!(data::GENRES.len(), 27);
        assert_eq!(data::DISPLAY_LANGUAGES.len(), 20);
        assert_eq!(data::ALL_AVATARS.len(), data::AVATAR_CATEGORIES.iter().map(|c| c.avatars.len()).sum::<usize>());
        assert_eq!(data::MOVIE_GENRES.len(), 26);
        assert_eq!(data::TV_GENRES.len(), 24);
    }
}

/// True when the page is opened from this machine or a private network address (localhost, 127.0.0.1, 10.x, 192.168.x, 172.16 to 31.x, *.local): where the developer pages may show.
fn is_local_host() -> bool {
    let host = web_sys::window().and_then(|w| w.location().hostname().ok()).unwrap_or_default();
    let private_172 = host.strip_prefix("172.").and_then(|r| r.split('.').next()).and_then(|n| n.parse::<u8>().ok()).is_some_and(|n| (16..=31).contains(&n));
    host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host.starts_with("10.") || host.starts_with("192.168.") || private_172 || host.ends_with(".local")
}

/// An address the app does not have: a policy's earlier or common address goes to the policy, anything else to the root.
#[component]
fn UnknownAddress() -> impl IntoView {
    let to = crate::config::routes::moved(&leptos_router::hooks::use_location().pathname.get_untracked()).unwrap_or(crate::config::routes::HOME);
    let navigate = leptos_router::hooks::use_navigate();
    // after the fallback is drawn: navigating while the router draws it writes the address the router is reading, which aborted the app on
    // every unknown address (RwLock<Url>::write inside NestedRoutesView::build, 2026-10-25)
    Effect::new(move |_| navigate(to, leptos_router::NavigateOptions { replace: true, ..Default::default() }));
}

/// Keeps each host to its own pages: `/help` (with the policies under `/help/legal`) on the main address moves to the Help Centre's host, and anything else on that host
/// (the app, sign in) moves back to the main one. The server does the same for crawlers (`functions/_middleware.js`); this covers a person moving between pages without a new page load.
#[component]
fn HostGuard() -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    Effect::new(move |_| {
        let path = location.pathname.get();
        let Some(win) = web_sys::window() else { return };
        let search = win.location().search().unwrap_or_default();
        let is_help_path = path == "/help" || path.starts_with("/help/");
        if crate::config::on_site_host() && is_help_path {
            let _ = win.location().replace(&format!("{}{path}{search}", crate::config::HELP_URL));
        } else if crate::config::on_help_host() && !is_help_path && path != "/" {
            let _ = win.location().replace(&format!("{}{path}{search}", crate::config::SITE_URL));
        }
    });
}

/// The two homes. A Kids profile lives at `/kids` and the others at `/browse`; each address sends the other kind of profile where it belongs, and `/kids` opened by a
/// profile that is not a Kids profile switches to the account's Kids profile (with the loading screen), as netflix.com/Kids does.
#[component]
fn HomeRoute(kids: bool) -> impl IntoView {
    let kids_active = crate::pages::settings::prefs::active_is_kids();
    let has_active = web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten()).is_some();
    if kids && !kids_active {
        let store = crate::store::use_profile_store();
        return match store.profiles.get_untracked().into_iter().find(|p| p.is_kids) {
            Some(k) if has_active => {
                crate::components::profiles::switch_overlay::begin_switch(k);
                view! { <span /> }.into_any()
            }
            _ => view! { <leptos_router::components::Redirect path="/browse" /> }.into_any(),
        };
    }
    if !kids && kids_active {
        return view! { <leptos_router::components::Redirect path="/kids" /> }.into_any();
    }
    // while Who's watching or the switch screen covers the page, the home page is not there at all: its banner video would go on playing (with sound) behind them
    let profile_store = crate::store::use_profile_store();
    let switching = use_context::<crate::components::profiles::switch_overlay::SwitchState>();
    view! {
        {move || if profile_store.active_profile_id.get().is_none() || switching.is_some_and(|s| s.0.get().is_some()) {
            view! { <span /> }.into_any()
        } else {
            view! { <BrowseHome /> }.into_any()
        }}
    }
    .into_any()
}
