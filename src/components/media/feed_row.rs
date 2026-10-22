use leptos::prelude::*;
use crate::components::media::collection_row::CollectionRow;
use crate::components::media::top_ten_row::TopTenRow;
use crate::components::media::vibe_row::VibeRow;
use crate::services::ai_engine::FeedRow;

/// One row the engine sent, drawn by what it is: the Top 10 strip, a collection banner, or an ordinary row with its title in the language of
/// the page. `kind` is the page's media kind for the chart row ("all", "movie", "tv").
#[component]
pub fn FeedRowView(row: FeedRow, #[prop(into)] kind: String) -> impl IntoView {
    if row.row_type == "top_ten" {
        let title = row.display_title();
        let items = row.to_media_items();
        return view! { <TopTenRow title=title kind=kind items=Some(items) /> }.into_any();
    }
    if row.collection.is_some() {
        return view! { <CollectionRow row=row /> }.into_any();
    }
    let title = row.display_title();
    let tagline = row.tagline.clone();
    let mood_pills = row.mood_pills.clone();
    let items = row.to_media_items();
    let more = row.more_source();
    view! { <VibeRow title=title tagline=tagline mood_pills=mood_pills items=items more=more /> }.into_any()
}

/// The engine's Continue Watching row repeats the one the page draws from the watch records (with progress bars), so it is left out.
pub fn is_continue_row(row: &FeedRow) -> bool {
    row.title_key.as_deref() == Some("row.continue_watching")
}
