use leptos::prelude::*;
use crate::services::tmdb::{fetch_trending};
use crate::components::media::row::Row;
use crate::components::media::top_ten_row::TopTenRow;
use crate::components::media::hero::HeroSection;
use crate::components::media::continue_watching_row::ContinueWatchingRow;
use crate::components::layout::subnav::genres::{CategorySubNav, SubNavGenre};
use crate::data::HOME_MOBILE_GENRES;

/// The Home page; opened for an event in season (`event`: genre id and name, for example Halloween) it is the same page for that event: films and series mixed, with the hero, rows and collections of the event.
#[component]
pub fn BrowseHome(#[prop(optional)] event: Option<(u32, &'static str)>, #[prop(optional)] collection: Option<String>) -> impl IntoView {
    use crate::components::layout::Layout;
    use crate::store::use_ui_store;

    let ui_store = use_ui_store();

    // Mobile genre picker: Home has genres on mobile only. The genre travels as its name (several pickers share an
    // id, for example the three documentary entries) and the engine resolves it for movies and series together.
    let genres: Vec<SubNavGenre> = HOME_MOBILE_GENRES
        .iter()
        .map(|g| SubNavGenre { id: g.id as u32, name: g.name.to_string() })
        .collect();
    let query_map = leptos_router::hooks::use_query_map();
    let genres_for_init = genres.clone();
    // the genre shown is the address's (`?genre=<name>`), so Back and Forward move between genres; an event page is its event's
    let selected_genre = Memo::new(move |_| {
        if let Some((id, name)) = event {
            return Some(SubNavGenre { id, name: name.to_string() });
        }
        let named = query_map.read().get("genre")?;
        genres_for_init.iter().find(|g| g.name.eq_ignore_ascii_case(&named)).cloned()
    });
    // the page of a collection: its header (instead of the hero) and its own rows; an event page has its header only
    let collection_page = LocalResource::new({
        let collection = collection.clone();
        move || {
            let collection = collection.clone();
            async move {
                match collection { Some(c) => crate::services::ai_engine::fetch_collection_page(&c, 40, &crate::services::ai_engine::personal_signals()).await, None => None }
            }
        }
    });
    let event_header = LocalResource::new(move || async move {
        match event { Some((_, name)) => crate::services::ai_engine::fetch_page_header(name).await, None => None }
    });
    let page_header = Signal::derive(move || collection_page.get().flatten().and_then(|p| p.header).or_else(|| event_header.get().flatten()));
    // picking a genre goes to its address through the router, so the browser's history and the page agree
    let navigate = StoredValue::new_local(leptos_router::hooks::use_navigate());
    let pathname = leptos_router::hooks::use_location().pathname;
    let on_genre_select_cb = Callback::new(move |opt_g: Option<SubNavGenre>| {
        let path = pathname.get_untracked();
        let url = match opt_g {
            Some(g) => format!("{path}?genre={}", String::from(js_sys::encode_uri_component(&g.name))),
            None => path,
        };
        navigate.with_value(|go| go(&url, Default::default()));
    });

    // Hero: pull AI-curated home feed with trending fallback
    let hero = LocalResource::new(move || {
        let g = selected_genre.get().map(|g| g.name);
        async move {
            if let Some(items) = crate::services::ai_engine::fetch_hero_feed("home", g.as_deref()).await {
                if !items.is_empty() {
                    return items;
                }
            }
            if g.is_some() {
                return Vec::new();
            }
            fetch_trending("all").await.unwrap_or_default()
        }
    });

    let ambient_bg_style = move || {
        let (r, g, b) = ui_store.ambient_color.get();
        // the colour of the hero's artwork at the top of the page (behind the header), fading to the page colour over the first screen, as in the app
        format!(
            "background: linear-gradient(to bottom, \
                 rgba({r}, {g}, {b}, 0.88) 0%, \
                 rgba({r}, {g}, {b}, 0.64) 20%, \
                 rgba({r}, {g}, {b}, 0.34) 42%, \
                 rgba({r}, {g}, {b}, 0.12) 62%, \
                 transparent 84%);",
            r = r, g = g, b = b
        )
    };

    // Dynamic Feed: AI-curated polymorphic rows from sovereign intelligence engine
    let feed_collection = collection.clone();
    let feed = LocalResource::new(move || {
        let g = selected_genre.get().map(|g| g.name);
        // the profile's own titles for the personal rows (read here, where the page's stores are in reach)
        let signals = if g.is_none() { crate::services::ai_engine::personal_signals() } else { String::new() };
        let collection = feed_collection.clone();
        async move {
            // the page of a collection lists the collection's own rows
            if let Some(c) = collection {
                return crate::services::ai_engine::fetch_collection_page(&c, 40, &signals).await.map(|p| p.rows).unwrap_or_default();
            }
            crate::services::ai_engine::fetch_feed_page("home", g.as_deref(), Some(24), &signals).await.map(|p| p.rows).unwrap_or_default()
        }
    });

    // Genre pages a limited profile has too few titles for are not offered
    let genre_names: Vec<String> = genres.iter().map(|g| g.name.clone()).collect();
    let all_genres = genres.clone();
    let hidden = LocalResource::new(move || {
        let names = genre_names.clone();
        async move { crate::services::ai_engine::hidden_genres("both", &names).await }
    });
    let offered = move || {
        let hide = hidden.get().unwrap_or_default();
        all_genres.iter().filter(|g| !hide.contains(&g.name)).cloned().collect::<Vec<SubNavGenre>>()
    };

    view! {
        <Layout>
            <div class="w-full pb-20 bg-void md:bg-page min-h-screen relative overflow-x-clip">
                // Ambient Atmospheric Glow behind navbar and hero billboard (fades after 50% of hero height)
                <div
                    class="absolute inset-x-0 top-0 h-[92vh] md:h-[760px] lg:h-[820px] pointer-events-none -z-0 transition-all duration-(--motion-slowest) ease-out"
                    style=ambient_bg_style
                />

                // Genre picker: phones only (the website has none on Home)
                {move || view! {
                    <CategorySubNav
                        title=crate::i18n::t("browse-home").to_string()
                        genres=offered()
                        selected_genre=selected_genre
                        on_genre_select=on_genre_select_cb
                        show_categories=true
                        phone_only=true
                    />
                }}

                // The page of a collection (Explore all on Halloween) opens with the collection's own header in place of the hero
                {move || page_header.get().map(|c| view! { <crate::components::media::collection_row::CollectionHero collection=c /> })}

                // Hero Carousel driven by real TMDB / AI feed
                <Suspense fallback=move || view! {
                    <crate::components::skeleton::HeroSkeleton />
                }>
                    {move || hero.get().map(|result| if page_header.get().is_some() { view! { <span /> }.into_any() } else {
                        let movies = result;
                        if movies.is_empty() {
                            view! { <div class="h-[500px] bg-void"></div> }.into_any()
                        } else {
                            view! {
                                <HeroSection movies=movies />
                            }.into_any()
                        }
                    })}
                </Suspense>

                <div class="relative z-30 rows-stack">
                    // Continue Watching belongs to Home, not to the page of a collection or an event
                    {(event.is_none() && collection.is_none()).then(|| view! { <ContinueWatchingRow /> })}

                    <Suspense fallback=move || view! { <crate::components::skeleton::RowsSkeleton /> }>
                        {move || feed.get().map(|rows| {
                            if rows.is_empty() {
                                view! {
                                    <TopTenRow title=crate::i18n::t("browse-top-10-in-the-uk") kind="all" />
                                    <Row title=crate::i18n::t("browse-action-adventure") genre_id="1365" kind="movie" />
                                    <Row title=crate::i18n::t("browse-comedies") genre_id="6548" kind="movie" />
                                    <Row title=crate::i18n::t("browse-scifi-fantasy") genre_id="1492" kind="movie" />
                                }.into_any()
                            } else {
                                rows.into_iter().filter(|r| !crate::components::media::feed_row::is_continue_row(r)).map(|r| {
                                    view! { <crate::components::media::feed_row::FeedRowView row=r kind="all" /> }
                                }).collect::<Vec<_>>().into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </Layout>
    }
}
