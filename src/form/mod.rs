//! Which form of the site this window shows, decided here once and asked everywhere (no component reads the width itself, note 114):
//! Phone below 600 px wide (the app's layouts); and a phone itself in any position: a screen whose short side is below 600 px that is
//! used by touch (a coarse pointer, or the browser saying it is a mobile one). A phone held sideways keeps the phone designs, as Netflix's
//! app does (it was a tablet by width before, note 116); a desktop window made short or narrow is judged by its width only, since the
//! screen is the monitor's; a phone on a big monitor (Samsung DeX) takes the monitor's screen and becomes a tablet or computer. Tablet
//! from 600 (the website, compact: Browse menu, no footer) and Desktop from 1024. The form is written on the root element as
//! `data-form`, and the styles follow it (style/input.css: the `sm`, `md` and `lg` variants and the token blocks), so CSS and code never
//! disagree; index.html sets it before the first paint by the same rule. The input in use (mouse, pen, finger) is a separate question:
//! see `input`.
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
    /// The rule `measure` applies: `width` the layout width, `screen_short` the short side of the device's screen, `touch_device` a
    /// coarse primary pointer or a browser that says it is a mobile one.
    pub fn of_view(width: f64, screen_short: f64, touch_device: bool) -> Form {
        if width < TABLET_FROM as f64 || (touch_device && screen_short < TABLET_FROM as f64) {
            Form::Phone
        } else if width >= DESKTOP_FROM as f64 {
            Form::Desktop
        } else {
            Form::Tablet
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

fn media(q: &str) -> Option<web_sys::MediaQueryList> {
    web_sys::window()?.match_media(q).ok().flatten()
}

/// The queries the form is decided by (index.html's early script uses the same ones).
const NARROW: &str = "(max-width: 599.98px)";
const DESKTOP: &str = "(min-width: 1024px)";
const COARSE: &str = "(pointer: coarse)";

/// The browser says it runs on a mobile device: its client hints where it has them, its signature's "Mobi" otherwise (both phones'
/// browsers; tablets' leave it out).
fn browser_is_mobile() -> bool {
    let Some(nav) = web_sys::window().map(|w| w.navigator()) else { return false };
    let hint = js_sys::Reflect::get(&nav, &"userAgentData".into()).ok().filter(|v| v.is_object());
    if let Some(mobile) = hint.and_then(|d| js_sys::Reflect::get(&d, &"mobile".into()).ok()).and_then(|m| m.as_bool()) {
        return mobile;
    }
    nav.user_agent().is_ok_and(|ua| ua.contains("Mobi"))
}

/// The short side of the device's screen, in CSS px (the monitor's for a desktop window, the phone's for a phone whichever way it is held).
fn screen_short_side() -> f64 {
    web_sys::window()
        .and_then(|w| w.screen().ok())
        .map(|s| (s.width().unwrap_or(0).min(s.height().unwrap_or(0))) as f64)
        .filter(|v| *v > 0.0)
        .unwrap_or(f64::MAX)
}

/// The form by the same rule as CSS (media queries count the viewport as CSS does, scroll bar included).
fn measure() -> Form {
    let at = |q| media(q).is_some_and(|m| m.matches());
    let width = if at(NARROW) { 0.0 } else if at(DESKTOP) { f64::from(DESKTOP_FROM) } else { f64::from(TABLET_FROM) };
    Form::of_view(width, screen_short_side(), at(COARSE) || browser_is_mobile())
}

/// Writes the form on the root element for the styles.
fn mark(form: Form) {
    if let Some(root) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
        let _ = root.set_attribute("data-form", match form { Form::Phone => "phone", Form::Tablet => "tablet", Form::Desktop => "desktop" });
    }
}

/// Follows the window from now on (called once when the app starts): crossing a boundary, or turning a phone, changes the form.
pub fn start() {
    mark(current());
    let on_change = Closure::<dyn Fn(web_sys::Event)>::new(move |_| {
        let now = measure();
        FORM.with(|f| if f.get_untracked() != now { f.set(now) });
        mark(now);
    });
    // the layout width crossing a boundary, and the window or screen changing (a phone onto a monitor)
    for q in [NARROW, DESKTOP, COARSE] {
        if let Some(m) = media(q) {
            let _ = m.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
        }
    }
    if let Some(w) = web_sys::window() {
        let _ = w.add_event_listener_with_callback("resize", on_change.as_ref().unchecked_ref());
    }
    // the app's own listener, for the life of the page
    on_change.forget();
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
    fn views_map_to_forms_at_the_css_boundaries() {
        assert_eq!(Form::of_view(375.0, 800.0, true), Form::Phone);
        assert_eq!(Form::of_view(599.9, 900.0, false), Form::Phone);
        assert_eq!(Form::of_view(600.0, 900.0, false), Form::Tablet);
        assert_eq!(Form::of_view(1023.9, 800.0, false), Form::Tablet);
        assert_eq!(Form::of_view(1024.0, 800.0, false), Form::Desktop);
    }

    #[test]
    fn a_phone_on_its_side_stays_a_phone_and_a_short_laptop_window_does_not() {
        // a Pixel 8 held sideways (937 wide, screen 421 on its short side), a short laptop window on a 900 px monitor, an iPad either
        // way, a phone on a DeX monitor (1080 short side)
        assert_eq!(Form::of_view(937.0, 421.0, true), Form::Phone);
        assert_eq!(Form::of_view(1280.0, 900.0, false), Form::Desktop);
        assert_eq!(Form::of_view(768.0, 768.0, true), Form::Tablet);
        assert_eq!(Form::of_view(1180.0, 820.0, true), Form::Desktop);
        assert_eq!(Form::of_view(1600.0, 1080.0, true), Form::Desktop);
    }
}
