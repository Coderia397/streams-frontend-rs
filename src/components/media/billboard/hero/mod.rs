//! The top of Home, Series and Films, one form at a time (note 114): `website` (the billboard: titles in turn with their trailers) on
//! tablets and computers, `phone` (the app's hero card) on phones. Only the form in use is mounted, so a phone fetches no trailers.
pub mod phone;
pub mod website;

use crate::services::tmdb::MediaItem;
use leptos::prelude::*;

#[component]
pub fn HeroSection(
    #[prop(optional)] movies: Option<Vec<MediaItem>>,
    #[prop(optional, default = 0)] movie_id: u32,
    #[prop(optional, default = false)] is_tv: bool,
    #[prop(optional, default = String::new())] title: String,
    #[prop(optional, default = String::new())] backdrop: String,
    #[prop(optional, default = String::new())] overview: String,
) -> impl IntoView {
    // the titles given, or one made of the single title's fields
    let items = match movies.filter(|m| !m.is_empty()) {
        Some(m) => m,
        None => vec![MediaItem {
            id: movie_id,
            title: (!is_tv).then(|| title.clone()),
            name: is_tv.then(|| title.clone()),
            overview: overview.clone(),
            backdrop_path: Some(backdrop.clone()),
            media_type: Some(if is_tv { "tv" } else { "movie" }.to_string()),
            ..Default::default()
        }],
    };
    let first = items.first().cloned().unwrap_or_default();
    let ui_store = crate::store::use_ui_store();
    view! {
        <crate::form::ByForm
            phone=move || {
                let movie = crate::models::movie::Movie::from(first.clone());
                let (id, tv) = (first.id, first.is_tv());
                // the hero card opens the title's details (no trailer to hand over on a phone)
                let open = Callback::new(move |_| {
                    ui_store.modal_video_key.set(None);
                    ui_store.modal_initial_time.set(0.0);
                    ui_store.info_modal_movie_id.set(Some(id));
                    ui_store.info_modal_is_tv.set(tv);
                    ui_store.info_modal_open.set(true);
                });
                view! { <phone::MobileHero movie=movie on_select=open /> }
            }
            desktop=move || view! { <website::WebsiteHero items=items.clone() /> }
        />
    }
}
