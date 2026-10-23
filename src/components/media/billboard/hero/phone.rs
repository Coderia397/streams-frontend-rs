use leptos::prelude::*;
use crate::models::movie::Movie;
use crate::store::{use_library_store, use_ui_store, LibraryEntry};
use crate::services::tmdb::{fetch_movie_logo, MediaItem};

#[component]
pub fn MobileHero(
    movie: Movie,
    #[prop(optional)] on_play: Option<Callback<Movie>>,
    #[prop(optional)] on_select: Option<Callback<Movie>>,
) -> impl IntoView {
    let library_store = use_library_store();
    let ui_store = use_ui_store();
    let movie_id = movie.id_u32();
    let is_tv = movie.is_tv();
    let title = movie.display_title();

    let is_added = Signal::derive(move || {
        library_store.my_list.get().contains_key(&movie_id)
    });

    let logo_resource = LocalResource::new(move || {
        let mid = movie_id;
        let tv = is_tv;
        async move {
            if mid > 0 {
                fetch_movie_logo(mid, tv).await.ok().flatten()
            } else {
                None
            }
        }
    });

    // The backdrop, cropped to the portrait card: backdrops carry no title, so the logo drawn over it (or the title text) is the only one. A poster has its own title baked in
    // and showed it twice; and one kind of image for every hero means no swap from one to the other while the page loads.
    let poster_url = movie.backdrop_path.as_ref().or(movie.poster_path.as_ref()).map(|p| {
        if p.starts_with("http") {
            p.clone()
        } else {
            format!("{}/w1280{}", crate::config::TMDB_IMAGE_URL, p)
        }
    }).unwrap_or_default();

    // up to three tags, as many as fit on one line next to the kind (about 38 characters)
    let tags: Vec<String> = {
        let mut used = if is_tv { 6 } else { 4 };
        movie.vibe_pills.clone().unwrap_or_default().into_iter().take(3).take_while(|t| { used += t.chars().count() + 3; used <= 40 }).collect()
    };
    let movie_c1 = movie.clone();
    let movie_c2 = movie.clone();
    let movie_c3 = movie.clone();

    let toggle_my_list = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        let mut list = library_store.my_list.get();
        if list.contains_key(&movie_id) {
            list.remove(&movie_id);
        } else {
            list.insert(movie_id, LibraryEntry {
                media: MediaItem {
                    id: movie_id,
                    title: movie_c1.title.clone(),
                    name: movie_c1.name.clone(),
                    overview: movie_c1.overview.clone(),
                    poster_path: movie_c1.poster_path.clone(),
                    backdrop_path: movie_c1.backdrop_path.clone(),
                    vote_average: movie_c1.vote_average,
                    release_date: movie_c1.release_date.clone(),
                    first_air_date: movie_c1.first_air_date.clone(),
                    media_type: movie_c1.media_type.clone(),
                    ..Default::default()
                },
                added_at: js_sys::Date::now() as u64,
            });
        }
        library_store.my_list.set(list);
    };

    let handle_card_click = move |_| {
        if let Some(cb) = on_select {
            cb.run(movie_c2.clone());
        }
    };

    let handle_play = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        if let Some(cb) = on_play {
            cb.run(movie_c3.clone());
        } else if let Some(w) = web_sys::window() {
            let watch_store = crate::store::use_watch_store();
            let target = if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
                if is_tv && rec.season.is_some() && rec.episode.is_some() {
                    format!("/watch/tv/{}?season={}&episode={}", movie_id, rec.season.unwrap(), rec.episode.unwrap())
                } else if is_tv {
                    format!("/watch/tv/{}", movie_id)
                } else {
                    format!("/watch/movie/{}", movie_id)
                }
            } else if is_tv {
                format!("/watch/tv/{}", movie_id)
            } else {
                format!("/watch/movie/{}", movie_id)
            };
            let _ = w.location().set_href(&target);
        }
    };

    let mobile_ambient_style = move || {
        let (r, g, b) = ui_store.ambient_color.get();
        format!(
            "background: linear-gradient(to bottom, \
                rgba({r},{g},{b},0.22) 0%, \
                rgba({r},{g},{b},0.14) 25%, \
                rgba({r},{g},{b},0.06) 48%, \
                rgba({r},{g},{b},0.01) 65%, \
                rgba(0,0,0,0) 78%);",
            r = r, g = g, b = b
        )
    };

    view! {
        <div class="relative z-0 overflow-visible w-full px-5 pt-0 pb-5 flex flex-col items-center justify-center transition-all duration-(--motion-slowest) ease-in-out md:hidden">
            // Ambient gradient glow (fades after 50% of hero height)
            <div
                class="absolute inset-x-0 top-0 h-[520px] pointer-events-none -z-10 transition-all duration-(--motion-slowest) ease-out"
                style=mobile_ambient_style
            />

            // Floating Centered Card: Netflix's app measures 978 x 1374 px on a Pixel 8 (381.7 x 536.2 dp) with a 1 dp edge of about 35 % white (note 116)
            <div
                on:click=handle_card_click
                class="w-full max-w-[560px] aspect-[978/1374] relative rounded-poster overflow-hidden border-[1.2px] border-ink/35 cursor-pointer active:scale-[0.98] transition-all duration-(--motion-base)"
            >
                <img
                    src=poster_url
                    alt=title.clone()
                    class="absolute inset-0 w-full h-full object-cover object-[50%_35%] select-none pointer-events-none"
                    loading="eager"
                />

                // Shade under the logo, tags and buttons only, as Netflix's app: the art stays as it is down to about half the card and shows
                // through behind the buttons (note 116); no shade from the side
                <div class="absolute inset-0 bg-linear-to-t from-void/60 from-0% via-void/25 via-30% to-transparent to-60% pointer-events-none" />

                // Details Container
                <div class="absolute inset-x-0 bottom-0 px-4 pb-3.5 pt-12 flex flex-col items-center text-center z-10 w-full">
                    // Logo or Title
                    <div class="relative inline-flex items-end mb-4 max-w-[71%] max-h-[108px] w-full justify-center">
                        <Suspense fallback=move || view! { <crate::components::skeleton::Bone class="h-10 w-32 rounded-control" /> }>
                            {
                                let tc = title.clone();
                                move || {
                                    let t = tc.clone();
                                    logo_resource.get().map(|res| match res {
                                        Some(url) if !url.is_empty() => view! {
                                            <img
                                                src=url
                                                alt=t.clone()
                                                class="object-contain object-bottom max-h-[108px] w-auto drop-shadow-xl"
                                            />
                                        }.into_any(),
                                        _ => view! {
                                            <h2 class="text-xl font-black font-brand drop-shadow-xl leading-tight text-ink tracking-wide uppercase line-clamp-2">
                                                {t.clone()}
                                            </h2>
                                        }.into_any()
                                    })
                                }
                            }
                        </Suspense>
                    </div>

                    // Kind and tags of the title (the engine's tone words and genres), as on the app: Series · Romantic · Social Experiment · Dating
                    // one grey for all, the kind in bold and the tags in regular weight, dots at half white (Netflix's app, note 116)
                    <div class="flex items-center justify-center flex-wrap gap-x-1.5 gap-y-1 mb-3.5 text-body-sm font-normal text-ink-tag select-none">
                        <span class="font-bold">{if is_tv { crate::i18n::t("media-series") } else { crate::i18n::t("media-film") }}</span>
                        {tags.iter().map(|t| view! {
                            <span class="text-ink/50">"•"</span>
                            <span>{t.clone()}</span>
                        }).collect::<Vec<_>>()}
                    </div>

                    // Buttons Row
                    <div class="flex items-center justify-center w-full gap-4">
                        // Play button
                        <button
                            type="button"
                            on:click=handle_play
                            class="btn-ink flex-1 flex items-center justify-center h-11 text-lg gap-2 transition-all active:scale-95"
                        >
                            <i class="ph-fill ph-play text-2xl"></i>
                            <span>{move || {
                                let watch_store = crate::store::use_watch_store();
                                if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
                                    if rec.percentage > 0.0 {
                                        return crate::i18n::t("media-resume");
                                    }
                                }
                                crate::i18n::t(if is_tv { "media-play-series" } else { "media-play-film" })
                            }}</span>
                        </button>

                        // My List button
                        <button
                            type="button"
                            on:click=toggle_my_list
                            class="btn-glass flex-1 flex items-center justify-center h-11 text-lg gap-2 transition-all active:scale-95"
                        >
                            <i class=move || if is_added.get() { "ph-bold ph-check text-2xl" } else { "ph-bold ph-plus text-2xl" }></i>
                            <span>{crate::i18n::t("my-list")}</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
