//! The frame of the browsing pages and its parts, grouped: `nav` (the bars, a folder per form) and `subnav` (the headers under them).
pub mod nav;
pub mod subnav;
pub mod footer;
pub mod tour;

pub use subnav::{category_sub_nav, category_sub_nav_mobile, new_popular_sub_nav_mobile};

use leptos::prelude::*;
use nav::Navbar;
use footer::Footer;

/// The page frame of the browsing pages, by form (note 114): on a phone the bottom bar is the navigation, so the content keeps room above it;
/// a tablet has the top bar and no footer; a computer has the footer. Help, Settings, legal and sign-in pages have their own frames.
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let form = crate::form::use_form();
    let room = move || match form.get() {
        crate::form::Form::Phone => "pb-[calc(var(--bottom-nav-h)+env(safe-area-inset-bottom))]",
        crate::form::Form::Tablet => "pb-8",
        crate::form::Form::Desktop => "pb-0",
    };
    view! {
        <div class="bg-void md:bg-page min-h-screen flex flex-col font-sans text-ink selection:bg-brand selection:text-ink">
            <Navbar />
            <tour::TourHost />

            <div class=move || format!("flex-1 transition-all duration-(--motion-slow) {}", room())>
                {children()}
            </div>

            <Show when=move || form.get().is_desktop()>
                <Footer />
            </Show>
        </div>
    }
}
pub mod account_gate;
