//! The navigation of the browsing pages, one form at a time (note 114): the website's top bar (`website`: tabs or the Browse menu,
//! search, notifications, Kids, profile) on tablets and computers, the app's header and bottom bar (`phone`) on phones. Only the form in
//! use is mounted; this file owns what both read (how far the page is scrolled).
pub mod phone;
pub mod website;

use leptos::prelude::*;

#[component]
pub fn Navbar(
    /// A fixed scroll position (a page drawn without scrolling, such as the error page); otherwise the window's
    #[prop(optional)] scroll_y: Option<f64>,
) -> impl IntoView {
    let (scroll, set_scroll) = signal(scroll_y.unwrap_or(0.0));
    if scroll_y.is_none() {
        crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
            set_scroll.set(web_sys::window().and_then(|w| w.scroll_y().ok()).unwrap_or(0.0));
        });
    }
    let scroll: Signal<f64> = scroll.into();
    view! {
        <crate::form::ByForm
            phone=move || view! { <phone::PhoneNav scroll_y=scroll /> }
            desktop=move || view! { <website::WebsiteNav scroll_y=scroll /> }
        />
    }
}
