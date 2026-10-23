//! New & Popular, one form at a time (note 114): `phone` (the app's tabbed page of large cards) or `website` (rows).
pub mod phone;
pub mod website;

use crate::components::layout::Layout;
use leptos::prelude::*;

#[component]
pub fn NewPopularPage() -> impl IntoView {
    view! {
        <Layout>
            <crate::form::ByForm
                phone=|| view! { <phone::PhoneNewPopular /> }
                desktop=|| view! { <website::WebsiteNewPopular /> }
            />
        </Layout>
    }
}
