//! A row of title cards with the page buttons of the other rows, for lists the app already holds (My List, trailers watched, liked titles).
use crate::components::media::movie_card::MovieCard;
use crate::components::media::row_scroll::{Edge, EdgeButton};
use crate::services::tmdb::MediaItem;
use leptos::prelude::*;

#[component]
pub fn TitleStrip(items: Vec<MediaItem>) -> impl IntoView {
    let strip = NodeRef::<leptos::html::Div>::new();
    let (can_left, set_can_left) = signal(false);
    let (can_right, set_can_right) = signal(true);
    let update = Callback::new(move |_: ()| {
        if let Some(el) = strip.get() {
            set_can_left.set(el.scroll_left() > 10);
            set_can_right.set(el.scroll_width() > el.client_width() + 10 && el.scroll_width() - (el.scroll_left() + el.client_width()) > 10);
        }
    });
    let page = Callback::new(move |forward: bool| {
        if let Some(el) = strip.get() {
            let (client, cur, total) = (el.client_width() as f64, el.scroll_left() as f64, el.scroll_width() as f64);
            el.scroll_to_with_x_and_y(crate::components::media::row_scroll::page_target(forward, cur, client, total, (client * 0.9).max(100.0)), 0.0);
            leptos::leptos_dom::helpers::set_timeout(move || update.run(()), std::time::Duration::from_millis(600));
        }
    });
    Effect::new(move |_| { update.run(()); });
    view! {
        <div class="group/row relative" style="container-type: inline-size;">
            <div node_ref=strip on:scroll=move |_| update.run(()) class="scrollbar-hide flex gap-[var(--tile-gap)] overflow-x-auto py-[2px] ps-[var(--app-x,56px)]" style="scroll-behavior: smooth;">
                {items.into_iter().map(|item| {
                    let backdrop = item.backdrop_url("w780").unwrap_or_default();
                    let poster = item.poster_url("w342").unwrap_or_default();
                    view! {
                        <div class="tile-width relative aspect-video flex-none overflow-visible rounded-tile">
                            <MovieCard
                                movie_id=item.id
                                is_tv=item.is_tv()
                                title=item.display_title().to_string()
                                backdrop_path=backdrop
                                poster_path=poster
                                vote_average=item.vote_average
                                overview=item.overview.clone()
                                genre_ids=item.genre_ids.clone()
                                vibe_pills=item.vibe_pills.clone()
                            />
                        </div>
                    }
                }).collect_view()}
                <div class="w-[var(--app-x,56px)] flex-none"></div>
            </div>
            <EdgeButton edge=Edge::Left visible=can_left on_click=Callback::new(move |_| page.run(false)) label=crate::i18n::t("vibe-row-scroll-left") />
            <EdgeButton edge=Edge::Right visible=can_right on_click=Callback::new(move |_| page.run(true)) label=crate::i18n::t("vibe-row-scroll-right") />
        </div>
    }
}
