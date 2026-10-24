//! Back (Android's, or the browser's) closes what is open over the page before it leaves the page, as in Netflix's app. An overlay that
//! opens adds a history entry with the same address; Back takes that entry away and closes the overlay; an overlay closed by its own
//! button takes its entry away itself, so the next Back leaves the page as before. The address never changes, so the router and the
//! title page (which read the address on Back) see nothing.
use leptos::prelude::*;
use wasm_bindgen::JsValue;

/// Ties `open` to an entry in the history: Back runs `close`. For overlays whose component lives as long as they can open.
pub fn close_on_back(open: Signal<bool>, close: Callback<()>) {
    // whether the entry this overlay added is still in the history
    let pushed = StoredValue::new(false);
    Effect::new(move |was: Option<bool>| {
        let now = open.get();
        let history = web_sys::window().and_then(|w| w.history().ok());
        if now && was != Some(true) {
            if let Some(h) = &history {
                pushed.set_value(h.push_state(&JsValue::NULL, "").is_ok());
            }
        } else if !now && was == Some(true) && pushed.get_value() {
            pushed.set_value(false);
            if let Some(h) = &history {
                let _ = h.back();
            }
        }
        now
    });
    crate::utils::listen::listen_window(leptos::ev::popstate, move |_| {
        if pushed.get_value() && open.get_untracked() {
            pushed.set_value(false);
            close.run(());
        }
    });
}
