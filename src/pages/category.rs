use leptos::prelude::*;
use crate::components::layout::Layout;
use crate::services::tmdb::fetch_trending;
use crate::components::media::row::Row;
use crate::components::media::top_ten_row::TopTenRow;
use crate::components::media::hero::HeroSection;
use crate::components::layout::subnav::genres::{CategorySubNav, SubNavGenre};
use crate::data::{MOVIE_GENRES, TV_GENRES};

/// The page behind `/browse/genre/<id>`: the Series page (83), the Films page (34399), or a genre page of one of them (`?bc=` says which; without it the genre decides). An id that
/// is not a genre keeps the older grid of a row.
#[component]
pub fn GenreRoute() -> impl IntoView {
    use crate::data::page_genres::{event_name, genre_url_on, is_event_page_id, is_kids_genre_id, page_kind_of_genre_route, FILMS_PAGE_ID, KIDS_FILMS_PAGE_ID, KIDS_SERIES_PAGE_ID, SERIES_PAGE_ID};
    let params = leptos_router::hooks::use_params_map();
    let query = leptos_router::hooks::use_query_map();
    let navigate = leptos_router::hooks::use_navigate();
    // A page the engine no longer offers (an event out of season, a genre without enough titles) is not shown: the address goes back to the page it belongs to, so no
    // reference to a page that is gone stays reachable.
    let listing = LocalResource::new(move || {
        let id = params.read().get("id").and_then(|i| i.parse::<u32>().ok());
        let bc = query.read().get("bc");
        async move {
            let (Some(id), Some(bc)) = (id, bc) else { return Some(true) };
            let kind = page_kind_of_genre_route(&format!("/browse/genre/{id}"), &format!("?bc={bc}")).unwrap_or("movie");
            let list = crate::services::ai_engine::fetch_genre_list(kind).await?;
            Some(list.iter().find(|g| g.id == id).is_none_or(|g| g.supported))
        }
    });
    let chips = LocalResource::new(|| async { crate::services::ai_engine::fetch_season_chips().await });
    Effect::new(move |_| {
        let id = params.read().get("id").and_then(|i| i.parse::<u32>().ok());
        let bc = query.read().get("bc");
        match (id, bc.as_deref()) {
            (Some(_), Some(bc)) if listing.get() == Some(Some(false)) => {
                // back to the page the address belongs to: the Series, Films or Kids page named by bc
                let page = bc.parse::<u32>().unwrap_or(FILMS_PAGE_ID);
                navigate(&genre_url_on(page, None), Default::default());
            }
            // the page of an event is only reachable while the event is on (the chips name the events in season)
            (Some(i), None) if is_event_page_id(i) => {
                if chips.get().is_some_and(|c| !c.iter().any(|x| x.genre_id == i)) { navigate("/browse", Default::default()); }
            }
            _ => {}
        }
    });
    // Which page the address shows. A genre of the Series or Films page is that page with another hero and other rows, so moving between its genres (the menu, Back,
    // Forward) keeps the page and its header, and only the address says which genre is shown.
    let shown = Memo::new(move |_| {
        let id = params.read().get("id").unwrap_or_default();
        let bc = query.read().get("bc").map(|b| format!("?bc={b}")).unwrap_or_default();
        let kind = page_kind_of_genre_route(&format!("/browse/genre/{id}"), &bc);
        let has_bc = !bc.is_empty();
        match (id.parse::<u32>(), kind) {
            // the page of a mood collection (Explore all): a number made from its id, and the id
            (Ok(n), _) if n >= 20_000 && query.read().get("c").is_some() => Shown::Collection(query.read().get("c").unwrap_or_default()),
            (Ok(n @ (SERIES_PAGE_ID | KIDS_SERIES_PAGE_ID)), _) => Shown::Category("tv", n),
            (Ok(n @ (FILMS_PAGE_ID | KIDS_FILMS_PAGE_ID)), _) => Shown::Category("movie", n),
            // the page of an event without a page to belong to (Halloween, Christmas) mixes films and series, like Home
            (Ok(n), _) if !has_bc && is_event_page_id(n) => Shown::Event(n),
            (Ok(n), Some(k)) if MOVIE_GENRES.iter().chain(TV_GENRES.iter()).any(|g| g.id == n) || is_kids_genre_id(n) || is_event_page_id(n) => {
                let kind = if k == "tv" { "tv" } else { "movie" };
                let page = query.read().get("bc").and_then(|b| b.parse().ok()).unwrap_or(if kind == "tv" { SERIES_PAGE_ID } else { FILMS_PAGE_ID });
                Shown::Category(kind, page)
            }
            _ => Shown::Grid,
        }
    });
    move || match shown.get() {
        Shown::Collection(id) => view! { <crate::pages::browse::BrowseHome collection=id /> }.into_any(),
        Shown::Category(kind, _) => view! { <CategoryPage kind=kind /> }.into_any(),
        Shown::Event(n) => view! { <crate::pages::browse::BrowseHome event=(n, event_name(n).unwrap_or("Event")) /> }.into_any(),
        Shown::Grid => view! { <crate::pages::browse_grid::BrowseGridPage /> }.into_any(),
    }
}

/// The page an address under `/browse/genre/` shows; `Category` carries the kind and the page the genres belong to (Series, Films or the Kids profile's own).
#[derive(Clone, PartialEq)]
enum Shown {
    Collection(String),
    Category(&'static str, u32),
    Event(u32),
    Grid,
}

/// The Series or Films page and its genre pages. The genre shown is the address's (`/browse/genre/<genre id>?bc=<page id>`, none on the page's own address), so Back
/// and Forward move between genres as between pages.
#[component]
pub fn CategoryPage(
    kind: &'static str, // "movie" or "tv"
) -> impl IntoView {
    use crate::data::page_genres::{FILMS_PAGE_ID, KIDS_FILMS_PAGE_ID, KIDS_SERIES_PAGE_ID, SERIES_PAGE_ID};
    let genres: Vec<SubNavGenre> = if kind == "tv" {
        TV_GENRES.iter().map(|g| SubNavGenre { id: g.id as u32, name: g.name.to_string() }).collect()
    } else {
        MOVIE_GENRES.iter().map(|g| SubNavGenre { id: g.id as u32, name: g.name.to_string() }).collect()
    };

    let params = leptos_router::hooks::use_params_map();
    let query_map = leptos_router::hooks::use_query_map();
    let genre_id = Memo::new(move |_| {
        params.read().get("id").and_then(|i| i.parse::<u32>().ok()).filter(|i| !matches!(*i, SERIES_PAGE_ID | FILMS_PAGE_ID | KIDS_SERIES_PAGE_ID | KIDS_FILMS_PAGE_ID))
    });

    // The genre pages come from the engine (ids, order, surface, the page their addresses hang under, and whether this profile is offered the page); the page's own list is
    // the fallback when the engine cannot be reached
    let menu = LocalResource::new(move || async move { crate::services::ai_engine::fetch_genre_menu(if kind == "tv" { "tv" } else { "movie" }).await });
    // the page the genre addresses hang under: the engine's (the Kids profile has pages of its own), else the Series or Films page
    let page_id = move || menu.get().flatten().map(|m| m.page_id).filter(|p| *p != 0).unwrap_or(if kind == "tv" { crate::data::page_genres::SERIES_PAGE_ID } else { crate::data::page_genres::FILMS_PAGE_ID });
    // the surface a genre page is fetched for: the engine's ("both" for a page that mixes films and series, such as the Kids For Little Ones or an event), else the page's
    let surface_of = move |id: u32| -> &'static str {
        match menu.get().flatten().and_then(|m| m.genres.into_iter().find(|g| g.id == id)).map(|g| g.surface).as_deref() {
            Some("both") => "home",
            Some("tv") => "tv",
            Some("movie") => "movie",
            _ => kind,
        }
    };
    // the genre shown: the address's, from the page's own list, else (a Kids genre, an event) from the engine's menu once it arrives; older addresses carry it as
    // `?genre=` (an id or a name)
    let own_genres = genres.clone();
    let selected_genre = Memo::new(move |_| {
        let by_id = |gid: u32| {
            own_genres.iter().find(|g| g.id == gid).cloned().or_else(|| {
                menu.get().flatten().and_then(|m| m.genres.into_iter().find(|g| g.id == gid)).map(|g| SubNavGenre { id: g.id, name: g.name })
            })
        };
        if let Some(gid) = genre_id.get() {
            return by_id(gid);
        }
        let named = query_map.read().get("genre")?;
        match named.parse::<u32>() {
            Ok(gid) => by_id(gid),
            Err(_) => own_genres.iter().find(|g| g.name.eq_ignore_ascii_case(&named)).cloned(),
        }
    });

    // picking a genre goes to its address through the router, so the browser's history and the page agree
    let navigate = StoredValue::new_local(leptos_router::hooks::use_navigate());
    let on_genre_select_cb = Callback::new(move |opt_g: Option<SubNavGenre>| {
        let url = crate::data::page_genres::genre_url_on(page_id(), opt_g.map(|g| g.id));
        navigate.with_value(|go| go(&url, Default::default()));
    });

    let title = if kind == "tv" { crate::i18n::t("category-series") } else { crate::i18n::t("category-films") };

    let ui_store = crate::store::use_ui_store();

    // Hero: pull featured item for selected genre or surface-specific AI hero feed
    let hero = LocalResource::new(move || {
        let sel = selected_genre.get();
        let k = kind;
        let surface = match sel.as_ref().map(|g| surface_of(g.id)).unwrap_or(k) { "tv" => "tv", "home" => "home", _ => "movies" };
        async move {
            let gid_opt = sel.map(|g| g.id.to_string());
            if let Some(items) = crate::services::ai_engine::fetch_hero_feed(surface, gid_opt.as_deref()).await {
                if !items.is_empty() {
                    return items;
                }
            }
            // a genre page never borrows the trending titles of the whole site: no title fits better than a wrong one
            if gid_opt.is_some() {
                return Vec::new();
            }
            fetch_trending(k).await.unwrap_or_default()
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

    // Dynamic Category Feed: AI-curated sub-vibe rows & category Top 10
    let feed = LocalResource::new(move || {
        let sel = selected_genre.get();
        // a genre page opened by an address the page's own list does not know waits for the engine's menu rather than showing the whole surface first
        let waiting = genre_id.get().is_some() && sel.is_none() && menu.get().is_none();
        let surface = sel.as_ref().map(|g| surface_of(g.id)).unwrap_or(kind).to_string();
        let g_opt = sel.map(|g| g.name);
        async move {
            if waiting {
                return Vec::new();
            }
            crate::services::ai_engine::fetch_dynamic_feed(&surface, g_opt.as_deref(), Some(12)).await.unwrap_or_default()
        }
    });

    let all_genres = genres.clone();
    let offered = move || match menu.get() {
        None => Vec::new(), // still asking
        Some(Some(m)) => m.genres.into_iter().filter(|g| g.supported).map(|g| SubNavGenre { id: g.id, name: g.name }).collect::<Vec<SubNavGenre>>(),
        Some(None) => all_genres.clone(),
    };

    view! {
        <Layout>
            <div class="with-subnav w-full pb-20 bg-void md:bg-page min-h-screen relative overflow-x-clip">
                // Ambient Atmospheric Glow behind navbar, category sub-nav, and hero billboard (fades after 50% of hero height)
                <div
                    class="absolute inset-x-0 top-0 h-[92vh] md:h-[760px] lg:h-[820px] pointer-events-none -z-0 transition-all duration-(--motion-slowest) ease-out"
                    style=ambient_bg_style
                />
                // Category Sub-Navigation bar (teleports to fixed header below navbar on desktop)
                {
                    let nav_title = title.to_string();
                    move || view! {
                        <CategorySubNav
                            title=nav_title.clone()
                            genres=offered()
                            selected_genre=selected_genre
                            on_genre_select=on_genre_select_cb
                        />
                    }
                }

                <Suspense fallback=move || view! {
                    <crate::components::skeleton::HeroSkeleton />
                }>
                    {move || hero.get().map(|result| {
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
                    <Suspense fallback=move || view! { <crate::components::skeleton::RowsSkeleton /> }>
                        {move || feed.get().map(|rows| {
                            if rows.is_empty() {
                                let default_top10_title = if kind == "movie" { crate::i18n::t("category-top10-movies") } else { crate::i18n::t("category-top10-series") };
                                view! {
                                    <TopTenRow title=default_top10_title.to_string() kind=kind.to_string() />
                                    <Row title=crate::i18n::t("browse-action-adventure") genre_id="1365" kind=kind.to_string() />
                                    <Row title=crate::i18n::t("browse-comedies") genre_id="6548" kind=kind.to_string() />
                                    <Row title=crate::i18n::t("browse-scifi-fantasy") genre_id="1492" kind=kind.to_string() />
                                }.into_any()
                            } else {
                                rows.into_iter().filter(|r| !crate::components::media::feed_row::is_continue_row(r)).map(|r| {
                                    view! { <crate::components::media::feed_row::FeedRowView row=r kind=kind.to_string() /> }
                                }).collect::<Vec<_>>().into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </Layout>
    }
}
