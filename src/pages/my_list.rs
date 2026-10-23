//! The My page ("My Streams"): four shelves, each a heading and either a card with a picture and one line (nothing there yet) or a row of titles.
//! Continue watching, the saved list, the trailers watched and the titles given a thumbs up.
use crate::components::layout::Layout;
use crate::components::media::continue_watching_row::ContinueWatchingRow;
use crate::components::media::shelf_icons::{ShelfIcon, ShelfPicture};
use crate::components::media::title_strip::TitleStrip;
use crate::i18n::t;
use crate::services::tmdb::{fetch_details, MediaItem};
use crate::store::{use_library_store, use_watch_store};
use leptos::prelude::*;

/// One shelf: the heading and, below, the content or the empty card. Laid out as every other row (Netflix's My Netflix rows are its Home rows).
#[component]
fn Shelf(
    #[prop(into)] title: String,
    icon: ShelfIcon,
    #[prop(into)] empty: String,
    #[prop(into)] has_items: Signal<bool>,
    /// True when the content carries its own heading (the continue watching row), so ours shows only while the shelf is empty.
    #[prop(default = false)] own_heading: bool,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        // a row that brings its own heading is a whole row already (its spacing would count twice)
        <section class=move || if own_heading && has_items.get() { "" } else { "row-block" }>
            <h2 class="row-title px-[var(--app-x,56px)]" class=("hidden", move || own_heading && has_items.get())>{title}</h2>
            <Show
                when=move || has_items.get()
                fallback=move || view! {
                    <div class="mx-[var(--app-x,56px)] flex h-[161px] flex-col items-center justify-center gap-4 rounded-modal bg-menu">
                        <ShelfPicture icon=icon />
                        <p class="text-body-md text-ink/70">{empty.clone()}</p>
                    </div>
                }
            >
                {children()}
            </Show>
        </section>
    }
}

/// Loads the details of titles known only by id, newest first, as cards.
async fn load_titles(keys: Vec<(u32, bool)>) -> Vec<MediaItem> {
    let mut out = Vec::new();
    for (id, is_tv) in keys.into_iter().take(24) {
        if let Ok(d) = fetch_details(id, is_tv).await {
            out.push(MediaItem {
                id: d.id,
                title: d.title.clone(),
                name: d.name.clone(),
                overview: d.overview.clone(),
                poster_path: d.poster_path.clone(),
                backdrop_path: d.backdrop_path.clone(),
                vote_average: d.vote_average,
                release_date: d.release_date.clone(),
                first_air_date: d.first_air_date.clone(),
                media_type: Some(if is_tv { "tv" } else { "movie" }.to_string()),
                genre_ids: d.genres.clone().unwrap_or_default().iter().map(|g| g.id).collect(),
                ..Default::default()
            });
        }
    }
    out
}

#[component]
pub fn MyListPage() -> impl IntoView {
    let library = use_library_store();
    let watch = use_watch_store();

    let saved = Signal::derive(move || {
        let mut items: Vec<_> = library.my_list.get().into_values().collect();
        items.sort_by(|a, b| b.added_at.cmp(&a.added_at)); // newest first
        items.into_iter().map(|e| e.media).collect::<Vec<MediaItem>>()
    });
    let continuing = Memo::new(move |_| !watch.get_continue_watching_list().is_empty());
    let trailers = LocalResource::new(|| async { load_titles(crate::store::trailers::all()).await });
    let liked = LocalResource::new(|| async {
        let keys = crate::store::ratings::liked_keys(24)
            .into_iter()
            .filter_map(|k| { let (kind, id) = k.split_once(':')?; Some((id.parse::<u32>().ok()?, kind == "tv")) })
            .collect();
        load_titles(keys).await
    });
    let trailers_list = Signal::derive(move || trailers.get().unwrap_or_default());
    let liked_list = Signal::derive(move || liked.get().unwrap_or_default());


    view! {
        <Layout>
            // Netflix's first row title sits 20 px under the nav (100 at 1736 wide); on a phone the first part starts 10 px under the bar, which
            // carries the profile and its sheet (layout/nav/phone)
            <div class="relative min-h-screen pb-16 pt-[calc(var(--nav-h-mobile)+10px+env(safe-area-inset-top))] sm:pt-[calc(var(--nav-h)+20px)]">
                <div class="rows-stack">
                <Shelf title=t("my-continue") icon=ShelfIcon::Clock empty=t("my-continue-empty") has_items=continuing own_heading=true>
                    <ContinueWatchingRow title=t("my-continue") />
                </Shelf>
                <Shelf title=t("my-list") icon=ShelfIcon::Chest empty=t("my-list-empty") has_items=Signal::derive(move || !saved.get().is_empty())>
                    {move || view! { <TitleStrip items=saved.get() /> }}
                </Shelf>
                <Shelf title=t("my-trailers") icon=ShelfIcon::Clapper empty=t("my-trailers-empty") has_items=Signal::derive(move || !trailers_list.get().is_empty())>
                    {move || view! { <TitleStrip items=trailers_list.get() /> }}
                </Shelf>
                <Shelf title=t("my-liked") icon=ShelfIcon::ThumbsUp empty=t("my-liked-empty") has_items=Signal::derive(move || !liked_list.get().is_empty())>
                    {move || view! { <TitleStrip items=liked_list.get() /> }}
                </Shelf>
                </div>
            </div>
        </Layout>
    }
}
