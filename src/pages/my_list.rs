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
        <section class="row-block">
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

    let profiles = crate::store::use_profile_store();
    let sheet = RwSignal::new(false);
    let active = Memo::new(move |_| profiles.profiles.with(|l| profiles.active_profile_id.with(|id| l.iter().find(|p| Some(&p.id) == id.as_ref()).or(l.first()).cloned())));

    view! {
        <Layout>
            // Netflix's first row title sits 20 px under the nav (100 at 1736 wide)
            <div class="relative min-h-screen pb-16 pt-28 md:pt-[calc(var(--nav-h)+20px)]">
                // On a phone the page opens with the profile in use and a caret; it opens the Profile sheet
                <button type="button" on:click=move |_| sheet.set(true) class="mb-6 flex items-center gap-3 px-4 sm:hidden" aria-haspopup="dialog">
                    <span class="h-8 w-8 overflow-hidden rounded-control">
                        {move || match active.get() {
                            Some(p) if p.is_kids && p.avatar_url.is_none() => view! { <crate::components::profiles::kids_avatar::KidsAvatar size=32.0 /> }.into_any(),
                            Some(p) => view! { <img src=p.avatar_url.unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string()) alt="" class="h-full w-full object-cover" /> }.into_any(),
                            None => view! { <span /> }.into_any(),
                        }}
                    </span>
                    <span class="text-heading font-bold">{move || active.get().map(|p| p.name).unwrap_or_default()}</span>
                    <i class="ph-fill ph-caret-down text-body-md text-ink-silver" aria-hidden="true"></i>
                </button>
                <crate::pages::settings::hub::ProfileSheet open=sheet />
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
