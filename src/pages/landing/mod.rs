//! The landing page (signed out, `/`), one form at a time (note 114): `phone` (the app's first screens, slides swiped through) or
//! `website` (the long page).
pub mod art;
pub mod phone;
pub mod website;

use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <crate::form::ByForm
            phone=|| view! { <phone::MobileWelcome /> }
            desktop=|| view! { <website::WebsiteLanding /> }
        />
    }
}

/// The wall of posters behind the landing page: real posters from the catalogue laid out in a few ways (`assets/landing/walls`, made by rendering the poster pages, so every title is right).
/// One is chosen at random for each visit; the rest of the set (noir, kids, fantasy) is kept for other pages.
pub(crate) fn wall(size: &str) -> String {
    const NAMES: [&str; 5] = ["diagonal-popular", "perspective-cinema", "diagonal-animation", "columns-scifi", "perspective-romcom"];
    let pick = ((js_sys::Math::random() * NAMES.len() as f64) as usize).min(NAMES.len() - 1);
    format!("/assets/landing/walls/{}-{size}.webp", NAMES[pick])
}
