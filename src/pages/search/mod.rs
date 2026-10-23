//! The Search page, one form at a time (note 114): `phone` is the app's search screen (its own bar with back, field, voice and clear;
//! the recommendations as a list; Top Results and More like as rows); `website` uses the top bar's field and shows grids under Netflix's
//! "More to explore" line. This file owns what both show: the query, the results and the recommendations.
pub mod phone;
pub mod website;

use crate::components::layout::Layout;
use crate::services::tmdb::{search_media, MediaItem};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn SearchPage() -> impl IntoView {
    let query_map = use_query_map();
    let q = Signal::derive(move || query_map.read().get("q").unwrap_or_default());

    // what is shown: the search, or the recommendations before anything is typed
    let results = LocalResource::new(move || {
        let current_q = q.get();
        async move {
            if current_q.trim().is_empty() {
                vec![]
            } else {
                match crate::services::ai_engine::search_titles(&current_q).await {
                    Some(items) => items,
                    None => search_media(&current_q).await.unwrap_or_default(),
                }
            }
        }
    });
    // before anything is typed: the engine's suggestions for this session, with the Home feed's picks row as the fallback
    let recommended = LocalResource::new(move || {
        let signals = crate::services::ai_engine::personal_signals();
        async move {
            if let Some(items) = crate::services::ai_engine::fetch_search_suggestions(12).await.filter(|v| !v.is_empty()) {
                return Some(items);
            }
            let page = crate::services::ai_engine::fetch_feed_page("home", None, Some(6), &signals).await?;
            let row = page.rows.into_iter().find(|r| r.title_key.as_deref() == Some("row.todays_top_picks"))?;
            Some(row.to_media_items())
        }
    });
    view! {
        <Layout>
            <crate::form::ByForm
                phone=move || view! { <phone::PhoneSearch q=q results=results recommended=recommended /> }
                desktop=move || view! { <website::WebsiteSearch q=q results=results recommended=recommended /> }
            />
        </Layout>
    }
}

/// What the search found, and the recommendations before anything is typed (none when nothing could be fetched).
pub type Found = LocalResource<Vec<MediaItem>>;
pub type Recommended = LocalResource<Option<Vec<MediaItem>>>;
