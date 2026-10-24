use leptos::prelude::*;
use crate::services::ai_engine::{engine_text, FeedRow};
use crate::components::media::movie_card::MovieCard;
use crate::components::media::collection_logo::{design_of, CollectionLogo, LogoFilters};
use crate::components::media::row_scroll::{page_target, Edge, EdgeButton};
use crate::components::media::kids_banner::{KidsBackdrop, KidsCollectionLogo};

/// The page that lists a whole collection: the page of the genre the engine names (Halloween), or for a mood the numbered page of the collection (`/browse/genre/<n>?c=<id>`, the number
/// is made from the id, as Netflix numbers its clusters).
pub fn collection_link(target: &serde_json::Value) -> Option<String> {
    if let Some(id) = target.get("collection").and_then(|v| v.as_str()) {
        let n = 20_000 + id.bytes().fold(7u32, |h, b| h.wrapping_mul(131).wrapping_add(b as u32)) % 9_000;
        return Some(format!("/browse/genre/{n}?c={id}"));
    }
    let name = target.get("genre")?.as_str()?;
    let genre = crate::data::page_genres::MOVIE_GENRES.iter().find(|g| g.name.eq_ignore_ascii_case(name))?;
    Some(crate::data::page_genres::genre_url("movie", Some(genre.id as u32)))
}

/// A themed row, as in the app: the full width of the page, the artwork of the collection (larger than the band, so it can fade into the page above and below) behind a centred panel
/// (the title drawn as a logo, the label between two thin lines, a sentence, Explore all) and the titles as ordinary tiles that run off the edge of the page, with the same page buttons
/// as every other row. Everything it shows comes from the row the engine sent (`collection` object).
#[component]
pub fn CollectionRow(row: FeedRow) -> impl IntoView {
    let Some(c) = row.collection.clone() else { return view! { <span /> }.into_any() };
    let eyebrow = engine_text(&c.label_key, None).unwrap_or_else(|| c.label.clone());
    let is_event = c.label_key.starts_with("event.");
    let heading = if is_event {
        engine_text("collection.title", Some(&serde_json::json!({ "label": eyebrow }))).unwrap_or_else(|| row.title.clone())
    } else {
        row.title.clone()
    };
    let link = collection_link(&c.target);
    let eyebrow_text = if is_event { crate::i18n::t("collection-eyebrow-event") } else { eyebrow.clone() };
    let artwork = c.images.first().map(|p| format!("{}/w1280{}", crate::config::TMDB_IMAGE_URL, p)).unwrap_or_default();
    let items = row.to_media_items();
    let description = c.description.clone();
    let aria = heading.clone();
    let h = c.hue;
    let design = design_of(&c.id);
    // a Kids collection has its own look (Kids lettering, colour, confetti)
    let is_kids = c.label_key == "collection.kids";
    // a Kids collection has a designed backdrop (colour scheme and flat shapes) in place of its dimmed artwork
    let background = if is_kids { "none".to_string() } else { format!("linear-gradient(hsl({h} 55% 14% / 0.7), hsl({h} 55% 14% / 0.7)), url('{artwork}')") };
    let logo_text = heading.clone();

    // the row of titles pages like every other row
    let strip = NodeRef::<leptos::html::Div>::new();
    let (can_left, set_can_left) = signal(false);
    let (can_right, set_can_right) = signal(true);
    // the first page turn starts the loop (see row_scroll.rs): from then on the row also pages back from its start
    let (looping, set_looping) = signal(false);
    let update = Callback::new(move |_: ()| {
        if let Some(el) = strip.get() {
            set_can_left.set(el.scroll_left() > 10);
            set_can_right.set(el.scroll_width() > el.client_width() + 10 && el.scroll_width() - (el.scroll_left() + el.client_width()) > 10);
        }
    });
    let page = Callback::new(move |forward: bool| {
        if let Some(el) = strip.get() {
            set_looping.set(true);
            // from tablet width up the row loops round (a copy of its first cards behind and of its last cards in front); a phone pages once to the end
            if crate::components::media::row_scroll::turn_page(&el, forward, 0.0) {
                leptos::leptos_dom::helpers::set_timeout(move || update.run(()), std::time::Duration::from_millis(600));
                return;
            }
            let (client, cur, total) = (el.client_width() as f64, el.scroll_left() as f64, el.scroll_width() as f64);
            let target = page_target(forward, cur, client, total, (client * 0.9).max(100.0));
            el.scroll_to_with_x_and_y(target, 0.0);
            leptos::leptos_dom::helpers::set_timeout(move || update.run(()), std::time::Duration::from_millis(600));
        }
    });

    let card = move |item: crate::services::tmdb::MediaItem, role: u8| {
        let is_tv = item.media_type.as_deref() == Some("tv");
        let backdrop = item.backdrop_path.as_ref().map(|p| format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, p)).unwrap_or_default();
        let poster = item.poster_path.as_ref().map(|p| format!("{}/w342{}", crate::config::TMDB_IMAGE_URL, p));
        let is_new = item.badges.iter().any(|b| b == "new");
        view! {
            <div
                class=format!("relative flex-none tile-width aspect-video max-[599px]:w-[118px] max-[599px]:aspect-[5/7] overflow-visible rounded-tile{}", if role >= 2 { " max-md:hidden" } else { "" })
                data-loop-first=(role == 1).then_some("1")
                data-loop-clone=(role == 2).then_some("1")
            >
                <MovieCard
                    movie_id=item.id
                    is_tv=is_tv
                    title=item.display_title().to_string()
                    backdrop_path=backdrop
                    poster_path=poster.unwrap_or_default()
                    vote_average=item.vote_average
                    overview=item.overview.clone()
                    genre_ids=item.genre_ids.clone()
                    match_percentage=item.match_percentage
                    vibe_pills=item.vibe_pills.clone()
                    is_new=is_new
                />
            </div>
        }
    };
    // the titles, with a copy of the last ones in front and of the first ones behind, so that the row can be paged round
    let n = items.len();
    let k = if n > 7 { n.min(12) } else { 0 };
    let lead_views: Vec<_> = items[n - k..].iter().cloned().map(|it| card(it, 3)).collect();
    let copies: Vec<_> = items[..k].to_vec();
    let mut card_views: Vec<_> = items.into_iter().enumerate().map(|(i, it)| card(it, if i == 0 { 1 } else { 0 })).collect();
    card_views.extend(copies.into_iter().enumerate().map(|(i, it)| card(it, if i == 0 { 2 } else { 3 })));

    view! {
        <section class="collection-row group/row relative isolate w-full my-4 md:my-8" aria-label=aria style="container-type: inline-size;">
            <LogoFilters />
            // the artwork: taller than the band and stretched past its sides, so that its edges fade into the page
            <div
                class="absolute -inset-x-[6%] -inset-y-[18%] -z-10 bg-cover bg-center"
                style=format!("background-image: {background}; -webkit-mask-image: linear-gradient(to bottom, transparent 0%, rgba(0,0,0,0.55) 14%, #000 32%, #000 68%, rgba(0,0,0,0.55) 86%, transparent 100%); mask-image: linear-gradient(to bottom, transparent 0%, rgba(0,0,0,0.55) 14%, #000 32%, #000 68%, rgba(0,0,0,0.55) 86%, transparent 100%);")
                aria-hidden="true"
            >{is_kids.then(|| view! { <KidsBackdrop seed=h /> })}</div>
            <div class="relative flex flex-col md:flex-row md:items-center gap-5 md:gap-6 py-8 md:py-14 md:min-h-[430px]">
                <div class="px-[var(--app-x,56px)] md:pe-0 md:w-[38%] md:flex-none flex flex-col items-center text-center">
                    {if is_kids {
                        view! { <KidsCollectionLogo text=logo_text hue=h /> }.into_any()
                    } else {
                        view! { <CollectionLogo text=heading hue=h design=design size=46.0 /> }.into_any()
                    }}
                    {if is_kids {
                        view! { <p class="mt-3 rounded-full bg-ink px-4 py-1 text-caption md:text-xs font-bold uppercase tracking-[0.2em] text-kids-red shadow-md">{eyebrow_text}</p> }.into_any()
                    } else {
                        view! { <div class="mt-5 w-[min(260px,70%)] border-y border-ink/40 py-1.5"><p class="text-caption md:text-xs font-semibold uppercase tracking-[0.3em] text-ink/85">{eyebrow_text}</p></div> }.into_any()
                    }}
                    {(!description.is_empty()).then(|| view! { <p class="mt-5 text-body md:text-title-sm leading-snug text-ink max-w-[30ch]" style=if is_kids { "font-family: 'Logo Kids','Fredoka',ui-rounded,sans-serif; font-weight: 500;" } else { "" }>{description}</p> })}
                    {link.map(|href| view! {
                        <a href=href class=if is_kids { "mt-5 inline-flex h-11 items-center rounded-button bg-ink px-7 text-body-md font-bold text-kids-red shadow-lg hover:scale-105 transition-transform" } else { "mt-5 inline-flex h-11 items-center rounded-button bg-chrome-ink px-6 text-body-md font-semibold text-chrome hover:bg-chrome-ink/85 transition-colors" }>
                            {crate::i18n::t("collection-explore-all")}
                        </a>
                    })}
                </div>
                <div class="relative flex-1 min-w-0">
                    <div node_ref=strip on:scroll=move |_| update.run(()) class="flex gap-[var(--tile-gap)] overflow-x-auto scrollbar-hide py-2 ps-[var(--app-x,56px)] md:ps-0" style="scroll-behavior: smooth;">
                        <div data-loop-lead="1" class="flex-none flex gap-[var(--tile-gap)]" style="display: none;">{lead_views}</div>
                        <div data-loop-spacer="1" class="flex-none w-0"></div>
                        {card_views}
                        <div class="flex-none w-[var(--app-x,56px)]"></div>
                    </div>
                    <EdgeButton edge=Edge::Left visible=Signal::derive(move || can_left.get() || looping.get()) on_click=Callback::new(move |_| page.run(false)) label=crate::i18n::t("vibe-row-scroll-left") inset="-top-2 -bottom-2" />
                    <EdgeButton edge=Edge::Right visible=can_right on_click=Callback::new(move |_| page.run(true)) label=crate::i18n::t("vibe-row-scroll-right") inset="-top-2 -bottom-2" />
                </div>
            </div>
        </section>
    }.into_any()
}

/// The top of the page that Explore all opens, in place of the hero: the artwork of the collection over the width of the page, larger than the section and fading into the page
/// below, with the logo and the sentence at the bottom left, as in the app (`header` of the engine's answer).
#[component]
pub fn CollectionHero(collection: crate::services::ai_engine::Collection) -> impl IntoView {
    let eyebrow = engine_text(&collection.label_key, None).unwrap_or_else(|| collection.label.clone());
    let is_event = collection.label_key.starts_with("event.");
    // an event is "<label> Collection"; a mood has a name of its own and the shared label ("For every feeling") above it
    let heading = if !collection.title.is_empty() {
        collection.title.clone()
    } else {
        engine_text("collection.title", Some(&serde_json::json!({ "label": eyebrow }))).unwrap_or_else(|| eyebrow.clone())
    };
    let eyebrow_text = if is_event { crate::i18n::t("collection-eyebrow-event") } else { eyebrow.clone() };
    let heading_label = heading.clone();
    // 1280 px, as the row's own artwork: a CSS background cannot pick a size, and TMDB's original (3840 px) cost hundreds of KB under a dark tint
    let artwork = collection.images.first().map(|p| format!("{}/w1280{}", crate::config::TMDB_IMAGE_URL, p)).unwrap_or_default();
    let h = collection.hue;
    let design = design_of(&collection.id);
    let description = collection.description.clone();
    let is_kids = collection.label_key == "collection.kids";
    let background = if is_kids { "none".to_string() } else { format!("linear-gradient(hsl({h} 55% 14% / 0.5), hsl({h} 55% 14% / 0.5)), url('{artwork}')") };
    let frames: Vec<String> = collection.images.iter().take(3).map(|p| format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, p)).collect();
    let logo_text = heading.clone();
    view! {
        <section class="collection-hero relative isolate w-full h-[78vh] md:h-[680px] -mb-24 md:-mb-32" aria-label=heading_label>
            <LogoFilters />
            <div
                class="absolute -inset-x-[4%] -top-[6%] -bottom-[6%] -z-10 bg-cover bg-center"
                style=format!("background-image: {background}; -webkit-mask-image: linear-gradient(to bottom, #000 0%, #000 58%, rgba(0,0,0,0.5) 80%, transparent 100%); mask-image: linear-gradient(to bottom, #000 0%, #000 58%, rgba(0,0,0,0.5) 80%, transparent 100%);")
                aria-hidden="true"
            >{is_kids.then(|| view! { <KidsBackdrop seed=h images=frames.clone() /> })}</div>
            <div class="absolute inset-x-0 bottom-24 md:bottom-32 px-[var(--app-x,56px)] max-w-[820px]">
                {if is_kids {
                    view! { <div class="inline-block w-[min(460px,90%)]"><KidsCollectionLogo text=logo_text hue=h /></div><p class="mt-4 inline-block rounded-full bg-ink px-4 py-1 text-caption md:text-xs font-bold uppercase tracking-[0.2em] text-kids-red shadow-md">{eyebrow_text}</p> }.into_any()
                } else {
                    view! { <div class="inline-block"><CollectionLogo text=heading hue=h design=design size=62.0 /></div><p class="mt-5 text-caption md:text-xs font-semibold uppercase tracking-[0.3em] text-ink/85">{eyebrow_text}</p> }.into_any()
                }}
                {(!description.is_empty()).then(|| view! { <p class="mt-3 text-body-lg md:text-title-md leading-snug text-ink max-w-[34ch]">{description}</p> })}
            </div>
        </section>
    }
}
