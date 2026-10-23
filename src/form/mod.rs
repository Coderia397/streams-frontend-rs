//! Which form of the site this window shows, decided here once and asked everywhere (no component reads the width itself, note 114):
//! Phone below 600 px (the app's layouts, the width where the CSS tokens change too), Tablet from 600 (the website, compact: Browse
//! menu, no footer) and Desktop from 1024. The layout width decides, never the kind of device: a touch laptop is a computer, a phone in a
//! wide window is a tablet. The input in use (mouse, pen, finger) is a separate question: see `input`.
pub mod input;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Phone,
    Tablet,
    Desktop,
}

/// Where the website starts (below it the phone designs) and where the full desktop starts, in CSS px. The media queries below use them,
/// and style/input.css switches its tokens at the same widths.
pub const TABLET_FROM: u32 = 600;
pub const DESKTOP_FROM: u32 = 1024;

impl Form {
    pub fn of_width(width: f64) -> Form {
        if width >= DESKTOP_FROM as f64 {
            Form::Desktop
        } else if width >= TABLET_FROM as f64 {
            Form::Tablet
        } else {
            Form::Phone
        }
    }
    pub fn is_phone(self) -> bool {
        self == Form::Phone
    }
    pub fn is_desktop(self) -> bool {
        self == Form::Desktop
    }
}

thread_local! {
    static FORM: ArcRwSignal<Form> = ArcRwSignal::new(measure());
}

fn query(min: u32) -> Option<web_sys::MediaQueryList> {
    web_sys::window()?.match_media(&format!("(min-width: {min}px)")).ok().flatten()
}

/// The form by the same rule as CSS (media queries count the viewport as CSS does, scroll bar included).
fn measure() -> Form {
    let at = |min| query(min).is_some_and(|q| q.matches());
    if at(DESKTOP_FROM) {
        Form::Desktop
    } else if at(TABLET_FROM) {
        Form::Tablet
    } else {
        Form::Phone
    }
}

/// Follows the window from now on (called once when the app starts): a change of width across a boundary changes the form.
pub fn start() {
    for min in [TABLET_FROM, DESKTOP_FROM] {
        if let Some(q) = query(min) {
            let on_change = Closure::<dyn Fn(web_sys::Event)>::new(move |_| {
                let now = measure();
                FORM.with(|f| if f.get_untracked() != now { f.set(now) });
            });
            let _ = q.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
            // the app's own listener, for the life of the page
            on_change.forget();
        }
    }
    input::start();
}

/// The form now, for code outside a view (an event handler, a position computed once).
pub fn current() -> Form {
    FORM.with(|f| f.get_untracked())
}

/// The form, reactive: a view or effect that reads it follows the window.
pub fn use_form() -> Signal<Form> {
    let f = FORM.with(|f| f.clone());
    Signal::derive(move || f.get())
}

/// Whether this pointer opens a tile's hover card: a pointer that hovers (a mouse or a pen; a browser fires mouse events after a tap
/// too, so the pointer's own type decides), and not on the phone form, which has no hover cards. Every tile with a hover card asks this.
pub fn opens_hover_card(e: &web_sys::PointerEvent) -> bool {
    input::hovers(e) && !current().is_phone()
}

/// Mounts the view of the form in use only (the others are not built at all, so they run no requests or listeners); crossing a
/// boundary swaps it. `tablet` falls back to `desktop`.
#[component]
pub fn ByForm(
    #[prop(into)] phone: ViewFn,
    #[prop(into)] desktop: ViewFn,
    #[prop(optional, into)] tablet: Option<ViewFn>,
) -> impl IntoView {
    let form = use_form();
    let chosen = Memo::new(move |_| form.get());
    move || match chosen.get() {
        Form::Phone => phone.run(),
        Form::Tablet => tablet.as_ref().unwrap_or(&desktop).run(),
        Form::Desktop => desktop.run(),
    }
}

/// Mounts `desktop` on the desktop form and `compact` on phones and tablets, for pages whose compact design serves both (Settings: the
/// app's lists below 1024 px, the website's pages from there).
#[component]
pub fn DesktopOr(#[prop(into)] compact: ViewFn, #[prop(into)] desktop: ViewFn) -> impl IntoView {
    let form = use_form();
    let wide = Memo::new(move |_| form.get().is_desktop());
    move || if wide.get() { desktop.run() } else { compact.run() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_map_to_forms_at_the_css_boundaries() {
        assert_eq!(Form::of_width(375.0), Form::Phone);
        assert_eq!(Form::of_width(599.9), Form::Phone);
        assert_eq!(Form::of_width(600.0), Form::Tablet);
        assert_eq!(Form::of_width(1023.9), Form::Tablet);
        assert_eq!(Form::of_width(1024.0), Form::Desktop);
    }
}
