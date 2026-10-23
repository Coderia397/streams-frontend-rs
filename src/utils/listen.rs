//! Listeners that go away with the component that made them. A listener added straight to the window or the document stays there after its
//! component is gone: every later event then runs it again (more work on every scroll, resize or YouTube message the longer the site is
//! used) and it reads signals that were disposed with the component (a panic that also stops the event that caused it). Every listener on
//! the window or the document goes through here; only the app's own, which lives as long as the page, is added directly (lib.rs).
use leptos::ev::EventDescriptor;
use leptos::prelude::*;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// A listener on the window, removed with the component.
pub fn listen_window<E>(event: E, cb: impl Fn(E::EventType) + 'static)
where
    E: EventDescriptor + 'static,
    E::EventType: FromWasmAbi + wasm_bindgen::JsCast,
{
    let handle = StoredValue::new_local(Some(window_event_listener(event, cb)));
    on_cleanup(move || {
        handle.try_update_value(|h| {
            if let Some(h) = h.take() {
                h.remove();
            }
        });
    });
}

/// A listener on the document, removed with the component.
pub fn listen_document(event: &'static str, cb: impl Fn(web_sys::Event) + 'static) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let closure = Closure::<dyn Fn(web_sys::Event)>::new(cb);
    let _ = doc.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
    let kept = StoredValue::new_local(Some(closure));
    on_cleanup(move || {
        kept.try_update_value(|c| {
            if let (Some(c), Some(doc)) = (c.take(), web_sys::window().and_then(|w| w.document())) {
                let _ = doc.remove_event_listener_with_callback(event, c.as_ref().unchecked_ref());
            }
        });
    });
}

/// Calls `on_outside` when a press (mouse, pen or finger) lands outside what `inside` recognises: closing a menu or a field when the page is
/// pressed elsewhere. One listener on the document for the component's life; pointer events, so a touch screen behaves as a mouse does.
pub fn on_press_outside(inside: impl Fn(&web_sys::Node) -> bool + 'static, on_outside: impl Fn() + 'static) {
    listen_document("pointerdown", move |e: web_sys::Event| {
        if let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) {
            if !inside(&target) {
                on_outside();
            }
        }
    });
}

/// True when `target` is inside the element of `node` (false while the element is not on the page).
pub fn contains<E>(node: NodeRef<E>, target: &web_sys::Node) -> bool
where
    E: leptos::html::ElementType,
    E::Output: JsCast + Clone + 'static,
{
    node.get_untracked().is_some_and(|el| el.unchecked_ref::<web_sys::Node>().contains(Some(target)))
}
