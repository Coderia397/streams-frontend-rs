//! The input in use: the last pointer that touched the page (a mouse, a pen or a finger). Hover effects and the hover card follow it, so a
//! touch laptop behaves as a computer while its mouse is used and as a touch screen while a finger is, and an iPad gets hover with a mouse.
//! Kept on the root element as `data-input` (style/input.css gates every hover style on it) and in a signal.
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Mouse,
    Pen,
    Touch,
}

impl Input {
    /// From a pointer event's `pointerType` ("mouse", "pen", "touch"; anything else counts as a mouse).
    pub fn of(pointer_type: &str) -> Input {
        match pointer_type {
            "touch" => Input::Touch,
            "pen" => Input::Pen,
            _ => Input::Mouse,
        }
    }
    /// A pointer that rests over things (a mouse or a pen): hover styles and the hover card are for it.
    pub fn hovers(self) -> bool {
        self != Input::Touch
    }
    fn name(self) -> &'static str {
        match self {
            Input::Mouse => "mouse",
            Input::Pen => "pen",
            Input::Touch => "touch",
        }
    }
}

thread_local! {
    static INPUT: ArcRwSignal<Input> = ArcRwSignal::new(guess());
    /// Where and when a finger last touched (time in ms, x, y), to tell a real mouse from the mouse a browser imitates after a tap.
    static LAST_TOUCH: std::cell::Cell<(f64, f64, f64)> = const { std::cell::Cell::new((f64::NEG_INFINITY, 0.0, 0.0)) };
}

/// A browser imitates a mouse after a tap (on a touch laptop above all): mouse events, and in some browsers pointer events of the type
/// mouse, at the tap's point. One that comes this soon and this close after a touch is the tap's echo, not a mouse.
const ECHO_MS: f64 = 800.0;
const ECHO_PX: f64 = 12.0;

/// The input this pointer event really comes from: a "mouse" event that echoes the last touch counts as the touch.
pub fn of_event(e: &web_sys::PointerEvent) -> Input {
    let said = Input::of(&e.pointer_type());
    if said == Input::Touch {
        LAST_TOUCH.with(|t| t.set((js_sys::Date::now(), e.client_x() as f64, e.client_y() as f64)));
        return Input::Touch;
    }
    let (at, x, y) = LAST_TOUCH.with(|t| t.get());
    let echo = js_sys::Date::now() - at < ECHO_MS && (e.client_x() as f64 - x).abs() < ECHO_PX && (e.client_y() as f64 - y).abs() < ECHO_PX;
    if echo { Input::Touch } else { said }
}

/// Before any pointer has moved: a screen whose main pointer hovers is taken as used with a mouse, any other as touched.
fn guess() -> Input {
    let hovers = web_sys::window()
        .and_then(|w| w.match_media("(hover: hover) and (pointer: fine)").ok().flatten())
        .is_some_and(|q| q.matches());
    if hovers { Input::Mouse } else { Input::Touch }
}

fn mark(input: Input) {
    if let Some(root) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
        let _ = root.set_attribute("data-input", input.name());
    }
}

/// Follows the pointer from now on (called once by `form::start`).
pub(super) fn start() {
    let first = INPUT.with(|i| i.get_untracked());
    mark(first);
    let Some(win) = web_sys::window() else { return };
    let on_pointer = Closure::<dyn Fn(web_sys::PointerEvent)>::new(move |e: web_sys::PointerEvent| {
        let now = of_event(&e);
        INPUT.with(|i| {
            if i.get_untracked() != now {
                i.set(now);
                mark(now);
            }
        });
    });
    let opts = web_sys::AddEventListenerOptions::new();
    opts.set_capture(true);
    opts.set_passive(true);
    for event in ["pointerdown", "pointermove"] {
        let _ = win.add_event_listener_with_callback_and_add_event_listener_options(event, on_pointer.as_ref().unchecked_ref(), &opts);
    }
    // the app's own listener, for the life of the page
    on_pointer.forget();
}

/// The input now, for an event handler.
pub fn current() -> Input {
    INPUT.with(|i| i.get_untracked())
}

/// The input, reactive.
pub fn use_input() -> Signal<Input> {
    let i = INPUT.with(|i| i.clone());
    Signal::derive(move || i.get())
}

/// True when this pointer event comes from a pointer that hovers (a mouse or a pen, not a tap's echo), the only ones that open hover things.
pub fn hovers(e: &web_sys::PointerEvent) -> bool {
    of_event(e).hovers()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_types_map_to_inputs() {
        assert_eq!(Input::of("touch"), Input::Touch);
        assert_eq!(Input::of("pen"), Input::Pen);
        assert_eq!(Input::of("mouse"), Input::Mouse);
        assert!(!Input::Touch.hovers());
        assert!(Input::Pen.hovers() && Input::Mouse.hovers());
    }
}
