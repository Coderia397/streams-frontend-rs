//! Opening the player, from every Play on the site: the address to play (where the profile left off), and on the phone form full screen
//! and landscape asked within the same tap (a browser allows them only during a tap; asked when the player page has opened, they were
//! refused, note 116). Links to the player (`<a href="/watch/...">`) are caught by one listener; buttons call `open_player`. The move
//! stays inside the app (a page load would leave full screen again).
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

thread_local! {
    /// The router's navigate, kept by `keep_navigate` from inside the router (the buttons that play are not all inside it)
    static NAVIGATE: RefCell<Option<Rc<dyn Fn(&str)>>> = const { RefCell::new(None) };
}

/// The address that plays a title: the episode the profile reached for a series, or the start.
pub fn watch_url(id: u32, is_tv: bool) -> String {
    let watch_store = crate::store::use_watch_store();
    match watch_store.get_record(id, is_tv) {
        Some(rec) if is_tv && rec.season.is_some() && rec.episode.is_some() => {
            format!("/watch/tv/{id}?season={}&episode={}", rec.season.unwrap_or(1), rec.episode.unwrap_or(1))
        }
        _ if is_tv => format!("/watch/tv/{id}"),
        _ => format!("/watch/movie/{id}"),
    }
}

/// Opens the player at `url` from a tap: full screen and landscape first on the phone form, then the move inside the app.
pub fn open_player(url: &str) {
    if crate::form::current().is_phone() {
        super::frame::enter_landscape();
    }
    let nav = NAVIGATE.with(|n| n.borrow().clone());
    match nav {
        Some(go) => go(url),
        None => {
            if let Some(w) = web_sys::window() {
                let _ = w.location().set_href(url);
            }
        }
    }
}

/// Keeps the router's navigate for `open_player` and catches taps on links to the player (called once, inside the router).
pub fn keep_navigate() {
    let navigate = leptos_router::hooks::use_navigate();
    NAVIGATE.with(|n| *n.borrow_mut() = Some(Rc::new(move |url: &str| navigate(url, Default::default()))));
    // a tap on a link to the player: full screen and landscape within the tap, before the router moves (capture, so it runs first)
    let on_click = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::MouseEvent)>::new(|e: web_sys::MouseEvent| {
        if !crate::form::current().is_phone() {
            return;
        }
        let to_player = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("a[href^='/watch/']").ok().flatten())
            .is_some();
        if to_player {
            super::frame::enter_landscape();
        }
    });
    if let Some(w) = web_sys::window() {
        let _ = w.add_event_listener_with_callback_and_bool("click", on_click.as_ref().unchecked_ref(), true);
    }
    // the app's own listener, for the life of the page
    on_click.forget();
}
