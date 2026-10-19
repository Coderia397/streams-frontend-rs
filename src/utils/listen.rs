//! A listener on the window that goes away with the component that made it. `window_event_listener` alone stays on the window after its component is gone, and the next event then reads
//! signals that were disposed with the component (a panic that also stops the click that caused it).
use leptos::ev::EventDescriptor;
use leptos::prelude::*;
use wasm_bindgen::convert::FromWasmAbi;

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
