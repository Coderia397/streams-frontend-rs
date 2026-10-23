//! The frame of the browsing pages and its parts, grouped: `nav` (the bars), `subnav` (the headers under them). Their modules are re-exported
//! here, so `layout::navbar` and `layout::nav::navbar` name the same module and the paths used across the app keep working.
pub mod nav;
pub mod subnav;
pub mod footer;
pub mod tour;

pub use nav::{bottom_nav_mobile, navbar, navbar_mobile, notifications_dropdown, search_bar};
pub use subnav::{category_sub_nav, category_sub_nav_mobile, new_popular_sub_nav_mobile};

use leptos::prelude::*;
use navbar::Navbar;
use footer::Footer;

/// The page frame of the browsing pages. On phones and tablets (under 1024 px) the bottom bar is the navigation and there is no footer, so the
/// content keeps room above the bar (under 640 px; tablets use the top bar); from 1024 px the website layout shows the footer. Help, Settings, legal and sign-in pages have their own frames.
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="bg-void md:bg-page min-h-screen flex flex-col font-sans text-ink selection:bg-brand selection:text-ink">
            <Navbar />
            <tour::TourHost />

            <div class="flex-1 transition-all duration-(--motion-slow) pb-[calc(var(--bottom-nav-h)+env(safe-area-inset-bottom))] sm:pb-8 lg:pb-0">
                {children()}
            </div>

            <div class="hidden lg:block">
                <Footer />
            </div>
        </div>
    }
}
pub mod account_gate;
