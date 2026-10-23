//! "Who's watching" again after a while away, as Netflix does: a browser left alone for longer than `idle_limit` asks for the profile
//! when it is used next (at start, on coming back to the tab, or at the first touch), so the next person does not land in someone else's
//! profile. Time away is counted from the last touch, key, scroll or playing video, and from the moment the page was left or closed; a
//! film that plays for two hours is not time away. An account with one profile is not asked (owner's request, 2026-10-23, note 116).
use leptos::prelude::*;
use crate::form::Form;
use crate::store::ProfileStore;

/// How long a browser may be left before it asks again: shorter on a computer or tablet, which more people share; longer on a phone,
/// which is someone's own (the owner's suggestion, 2026-10-23; Netflix does not publish its own).
pub fn idle_limit(form: Form) -> f64 {
    const MINUTE: f64 = 60_000.0;
    match form {
        Form::Phone => 120.0 * MINUTE,
        Form::Tablet | Form::Desktop => 40.0 * MINUTE,
    }
}

/// Activity is written at most this often (a scroll fires many events).
const WRITE_EVERY_MS: f64 = 30_000.0;

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn last_seen() -> Option<f64> {
    storage()?.get_item(crate::keys::LAST_SEEN).ok().flatten()?.parse().ok()
}

fn mark_seen(now: f64) {
    if let Some(s) = storage() {
        let _ = s.set_item(crate::keys::LAST_SEEN, &format!("{}", now as u64));
    }
}

/// True when the browser has been left longer than the limit for this form.
fn away_too_long(now: f64) -> bool {
    last_seen().is_some_and(|t| now - t > idle_limit(crate::form::current()))
}

/// Shows Who's watching instead of the profile in use, when there is a choice to make.
fn ask(profiles: &ProfileStore) {
    if profiles.profiles.with_untracked(|l| l.len() < 2) || profiles.active_profile_id.get_untracked().is_none() {
        return;
    }
    if let Some(s) = storage() {
        let _ = s.remove_item(crate::keys::ACTIVE_PROFILE);
    }
    profiles.active_profile_id.set(None);
}

/// Checks the time away at start and whenever the browser is used again, and keeps the time of the last activity.
pub fn start(profiles: ProfileStore) {
    let now = js_sys::Date::now();
    if away_too_long(now) {
        ask(&profiles);
    }
    mark_seen(now);
    let written = StoredValue::new(now);

    // anything the person does: ask first if they were away too long, then note the activity
    let on_activity = move || {
        let now = js_sys::Date::now();
        if away_too_long(now) {
            ask(&profiles);
        }
        if now - written.get_value() > WRITE_EVERY_MS {
            written.set_value(now);
            mark_seen(now);
        }
    };
    for event in ["pointerdown", "keydown", "wheel", "touchstart"] {
        let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::new(on_activity);
        if let Some(w) = web_sys::window() {
            let _ = w.add_event_listener_with_callback_and_bool(event, wasm_bindgen::JsCast::unchecked_ref(cb.as_ref()), true);
        }
        cb.forget();
    }

    // leaving the page counts from now; coming back checks
    let on_visibility = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
        let hidden = web_sys::window().and_then(|w| w.document()).is_some_and(|d| d.hidden());
        let now = js_sys::Date::now();
        if hidden {
            written.set_value(now);
            mark_seen(now);
        } else {
            on_activity();
        }
    });
    if let Some(d) = web_sys::window().and_then(|w| w.document()) {
        let _ = d.add_event_listener_with_callback("visibilitychange", wasm_bindgen::JsCast::unchecked_ref(on_visibility.as_ref()));
    }
    on_visibility.forget();

    // a video playing on a page in view is not time away (a film, a trailer the person is watching)
    let _ = leptos::leptos_dom::helpers::set_interval_with_handle(
        move || {
            let Some(d) = web_sys::window().and_then(|w| w.document()) else { return };
            let playing = !d.hidden()
                && d.query_selector_all("video").ok().is_some_and(|list| {
                    (0..list.length()).filter_map(|i| list.item(i)).any(|n| {
                        wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlMediaElement>(n).is_ok_and(|v| !v.paused())
                    })
                });
            if playing {
                let now = js_sys::Date::now();
                written.set_value(now);
                mark_seen(now);
            }
        },
        std::time::Duration::from_millis(WRITE_EVERY_MS as u64),
    );
}
