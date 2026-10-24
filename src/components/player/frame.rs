//! The box the player page draws in, one per form (note 114). The website: the whole window. The phone: Netflix's app plays in
//! landscape, full screen (note 116), so the phone's frame asks the browser for full screen and a landscape lock when the player opens
//! (a browser allows it only right after a tap: the Play that opened the page). When the browser leaves full screen (the phone locked,
//! another app), the player stays as it is and its Full screen button brings it back (owner's choice, 2026-10-24: no turning by itself).
//! Leaving the player gives both back.
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

/// Full screen and a landscape lock; either may be refused (no tap just before, or a browser without them).
pub fn enter_landscape() {
    let Some(win) = web_sys::window() else { return };
    let Some(doc) = win.document() else { return };
    if doc.fullscreen_element().is_none() {
        if let Some(root) = doc.document_element() {
            // the standard call, or Safari's older prefixed one (iPad, Mac before 16.4); an iPhone's Safari has neither for a page
            let standard = js_sys::Reflect::get(&root, &"requestFullscreen".into()).ok().filter(|f| f.is_function());
            let prefixed = js_sys::Reflect::get(&root, &"webkitRequestFullscreen".into()).ok().filter(|f| f.is_function());
            if let Some(f) = standard.or(prefixed).and_then(|f| f.dyn_into::<js_sys::Function>().ok()) {
                if let Ok(p) = f.call0(&root).and_then(|p| p.dyn_into::<js_sys::Promise>()) {
                    let ignore = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(|_: JsValue| {});
                    let _ = p.catch(&ignore);
                    ignore.forget();
                }
            }
        }
    }
    // screen.orientation.lock("landscape"), read by name (its web-sys type needs features the app does not use elsewhere)
    let Ok(orientation) = js_sys::Reflect::get(&win.screen().map(JsValue::from).unwrap_or(JsValue::NULL), &"orientation".into()) else { return };
    let Ok(lock) = js_sys::Reflect::get(&orientation, &"lock".into()).and_then(|f| f.dyn_into::<js_sys::Function>()) else { return };
    if let Ok(promise) = lock.call1(&orientation, &"landscape".into()).and_then(|p| p.dyn_into::<js_sys::Promise>()) {
        // a refusal is expected outside full screen; it is not an error of the page
        let ignore = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(|_: JsValue| {});
        let _ = promise.catch(&ignore);
        ignore.forget();
    }
}

pub fn leave_landscape() {
    let Some(win) = web_sys::window() else { return };
    if let Ok(orientation) = js_sys::Reflect::get(&win.screen().map(JsValue::from).unwrap_or(JsValue::NULL), &"orientation".into()) {
        if let Ok(unlock) = js_sys::Reflect::get(&orientation, &"unlock".into()).and_then(|f| f.dyn_into::<js_sys::Function>()) {
            let _ = unlock.call0(&orientation);
        }
    }
    if let Some(doc) = win.document() {
        if doc.fullscreen_element().is_some() {
            doc.exit_fullscreen();
        }
    }
}

/// The player page's box for the form in use.
#[component]
pub fn PlayerFrame(children: ChildrenFn) -> impl IntoView {
    let children = StoredValue::new(children);
    view! {
        <crate::form::ByForm
            phone=move || view! { <PhoneFrame>{children.with_value(|c| c())}</PhoneFrame> }
            desktop=move || view! { <div class="fixed inset-0 bg-void">{children.with_value(|c| c())}</div> }
        />
    }
}

/// The page may draw under the camera's cutout while the player is open (`viewport-fit=cover`), as Netflix's player does: its menus
/// start 36 px from the screen's edge, its controls keep clear of the cutout (env(safe-area-inset-*)). Other pages keep the default,
/// so nothing of theirs goes under a notch.
fn cover_cutout(on: bool) {
    let Some(meta) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.query_selector("meta[name=viewport]").ok().flatten()) else { return };
    let content = meta.get_attribute("content").unwrap_or_default();
    let base: Vec<&str> = content.split(',').map(str::trim).filter(|p| !p.is_empty() && !p.starts_with("viewport-fit")).collect();
    let mut parts = base.join(", ");
    if on {
        parts.push_str(", viewport-fit=cover");
    }
    let _ = meta.set_attribute("content", &parts);
}

#[component]
fn PhoneFrame(children: Children) -> impl IntoView {
    enter_landscape();
    cover_cutout(true);
    on_cleanup(|| {
        leave_landscape();
        cover_cutout(false);
    });
    // the phone locked or another app opened: the video pauses, as Netflix's app (the browser has left full screen; the Full screen
    // button brings it back)
    let on_hidden = wasm_bindgen::closure::Closure::<dyn Fn()>::new(|| {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
        if doc.hidden() {
            if let Some(video) = doc.get_element_by_id(crate::keys::VIDEO_ELEMENT_ID).and_then(|e| e.dyn_into::<web_sys::HtmlMediaElement>().ok()) {
                let _ = video.pause();
            }
        }
    });
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let _ = doc.add_event_listener_with_callback("visibilitychange", on_hidden.as_ref().unchecked_ref());
    }
    let kept = StoredValue::new_local(Some(on_hidden));
    on_cleanup(move || {
        kept.try_update_value(|k| {
            if let (Some(cb), Some(doc)) = (k.take(), web_sys::window().and_then(|w| w.document())) {
                let _ = doc.remove_event_listener_with_callback("visibilitychange", cb.as_ref().unchecked_ref());
            }
        });
    });
    view! {
        <div class="fixed inset-0 bg-void">{children()}</div>
    }
}
