use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::store::{use_library_store, use_ui_store, LibraryEntry};
use crate::services::tmdb::MediaItem;
use crate::components::media::movie_card_badges::RatingBadge;

#[component]
pub fn SpotlightCard(
    movie_id: u32,
    is_tv: bool,
    title: String,
    backdrop_path: String,
    overview: String,
    #[prop(default = None)] rank: Option<usize>,
    #[prop(default = None)] release_date: Option<String>,
    #[prop(default = false)] is_coming_soon: bool,
    #[prop(default = false)] hide_play: bool,
) -> impl IntoView {
    let navigate = use_navigate();
    let library = use_library_store();
    let ui_store = use_ui_store();

    let is_in_my_list = move || library.my_list.with(|m| m.contains_key(&movie_id));

    let title_stored = title.clone();
    let backdrop_stored = backdrop_path.clone();
    let overview_stored = overview.clone();
    let rel_date_stored = release_date.clone();

    let on_toggle_list = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        if is_in_my_list() {
            library.my_list.update(|m| { m.remove(&movie_id); });
        } else {
            let item = MediaItem {
                id: movie_id,
                title: if is_tv { None } else { Some(title_stored.clone()) },
                name: if is_tv { Some(title_stored.clone()) } else { None },
                overview: overview_stored.clone(),
                poster_path: None,
                backdrop_path: Some(backdrop_stored.clone()),
                vote_average: 0.0,
                release_date: rel_date_stored.clone(),
                first_air_date: if is_tv { rel_date_stored.clone() } else { None },
                media_type: Some(if is_tv { "tv".to_string() } else { "movie".to_string() }),
                ..Default::default()
            };
            library.my_list.update(|m| {
                m.insert(movie_id, LibraryEntry {
                    media: item,
                    added_at: js_sys::Date::now() as u64,
                });
            });
        }
    };

    let on_play_click = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
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
        navigate(&target, Default::default());
    };

    let on_card_click = move |_: ()| {
        ui_store.hero_paused_by_modal.set(false);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(movie_id));
        ui_store.info_modal_is_tv.set(is_tv);
        ui_store.info_modal_open.set(true);
    };

    let logo_alt = title.clone();
    let logo = LocalResource::new(move || async move { crate::services::tmdb::fetch_movie_logo(movie_id, is_tv).await.ok() });
    let formatted_rank = rank.map(|r| if r < 10 { format!("0{}", r) } else { format!("{}", r) });

    let coming_soon_label = release_date.as_deref().map(crate::i18n::short_date).unwrap_or_default();

    let backdrop_src = if backdrop_path.starts_with("http") {
        backdrop_path.clone()
    } else if !backdrop_path.is_empty() {
        format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, backdrop_path)
    } else {
        String::new()
    };

    // the colour of the artwork, found once (cached by the hero's helper), turned into the gradient of the strip with plain arithmetic:
    //   left  = colour * 1.0                                   (its own brightness and colour)
    //   right = (grey + (colour - grey) * 0.15) * 0.88          (grey = luma of the colour: 15 percent of the colour kept, 12 percent less bright)
    let strip_style = RwSignal::new(String::from("background: linear-gradient(to right, rgb(40,40,40), rgb(30,30,30))"));
    if formatted_rank.is_some() {
        let make = move |(r, g, b): (u8, u8, u8)| {
            let (r, g, b) = (r as f32, g as f32, b as f32);
            let grey = 0.299 * r + 0.587 * g + 0.114 * b;
            let right = |c: f32| ((grey + (c - grey) * 0.15) * 0.88).clamp(0.0, 255.0).round() as u8;
            strip_style.set(format!("background: linear-gradient(to right, rgb({},{},{}), rgb({},{},{}))", r as u8, g as u8, b as u8, right(r), right(g), right(b)));
        };
        match crate::utils::ambient::get_cached_ambient_color(&backdrop_src) {
            Some(c) => make(c),
            None => crate::utils::ambient::extract_ambient_color(&backdrop_src, move |c| if let Some(c) = c { make(c) }),
        }
    }

    view! {
        <div class="spot-card relative overflow-visible py-3 group/spotlight z-10 w-full sm:max-w-[85vw] sm:mx-auto">
            // the shade the page lays over a card that is leaving or arriving (see new_popular.rs dim_cards)
            <div class="absolute inset-0 z-30 pointer-events-none rounded-pill bg-void transition-opacity duration-(--motion-fast)" style="opacity: var(--dim, 0)"></div>
            // Top 10: a strip peeks out above the card with the numeral on it. It is a gradient of the colour of the artwork (the one the hero uses): left the colour at its own brightness,
            // right a little less bright and almost grey (`strip_style`). The numeral is outlined thin and its foot is hidden behind the card, as in the app.
            {formatted_rank.clone().map(|r_text| {
                view! {
                    <div class="pointer-events-none select-none h-[62px]">
                        <div class="absolute left-0 right-0 top-[47px] h-[40px] rounded-t-pill border border-b-0 border-ink/25" style=move || strip_style.get()></div>
                        <svg viewBox="0 0 280 218" class="absolute left-[24px] top-[12px] w-[108px] h-[84px] overflow-visible">
                            <text x="0" y="215" fill="#000" stroke="#fff" stroke-width="3.4" stroke-linejoin="round" font-size="290" font-weight="900" font-family="'Inter', sans-serif" textLength="276" lengthAdjust="spacingAndGlyphs">
                                {r_text}
                            </text>
                        </svg>
                    </div>
                }
            })}

            // Spotlight card container
            <div
                class="relative z-10 w-full min-h-[440px] flex flex-col bg-void border border-ink/25 rounded-pill overflow-hidden select-none"
            >
                // Picture area: the trailer plays by itself while the card is on screen (see SpotlightArtwork)
                <div class="relative">
                    <crate::components::media::spotlight_artwork::SpotlightArtwork
                        movie_id=movie_id
                        is_tv=is_tv
                        backdrop_src=backdrop_src.clone()
                        title=title.clone()
                    />
                    // Maturity badge, the smaller preset
                    <div class="absolute top-3 right-3 z-20 pointer-events-none drop-shadow-md origin-top-right scale-75">
                        <RatingBadge id=movie_id is_tv=is_tv size="xs".to_string() />
                    </div>
                </div>

                // Below-media content area: the only part that opens the details (the picture area plays, pauses and mutes the trailer)
                <div on:click=move |_| on_card_click(()) class="px-5 pb-6 pt-4 flex flex-col flex-grow gap-3 cursor-pointer">
                    // The title as its logo (a title without one shows its name), then what is happening: coming on a day, or the title's own name when it is out
                    {move || logo.get().flatten().map(|url| view! {
                        <img src=url alt=logo_alt.clone() class="max-h-[74px] max-w-[70%] w-auto object-contain object-left" decoding="async" />
                    })}
                    {if is_coming_soon && !coming_soon_label.is_empty() {
                        view! {
                            <p class="text-ink font-bold text-title leading-tight">
                                {crate::i18n::t_args("media-coming", &[("when", fluent_bundle::FluentValue::from(coming_soon_label.clone()))])}
                            </p>
                        }.into_any()
                    } else {
                        view! { <h3 class="text-ink font-bold text-title leading-tight">{crate::i18n::t(if is_tv { "new-hot-watch-series" } else { "new-hot-watch-film" })}</h3> }.into_any()
                    }}

                    // Overview
                    <p class="text-body-lg text-ink-soft font-normal leading-relaxed line-clamp-3 flex-1">
                        {overview}
                    </p>

                    // CTA Buttons
                    <div class="mt-auto pt-3">
                        {if is_coming_soon || hide_play {
                            view! {
                                <button
                                    on:click=on_toggle_list
                                    class="flex items-center justify-center h-[51px] px-5 rounded-button bg-ink-soft text-chrome text-title font-semibold gap-2.5 transition-all active:scale-95 w-fit"
                                >
                                    {move || if is_in_my_list() {
                                        view! {
                                            <i class="ph-bold ph-check text-xl text-chrome"></i>
                                            <span>{crate::i18n::t("media-in-list")}</span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <i class="ph-bold ph-plus text-xl text-chrome"></i>
                                            <span>{crate::i18n::t("media-add-to-list")}</span>
                                        }.into_any()
                                    }}
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex items-center gap-2">
                                    <button
                                        on:click=on_play_click
                                        class="flex items-center justify-center h-[54px] px-5 rounded-button bg-ink-soft text-chrome text-title font-semibold gap-2 transition-all active:scale-95"
                                    >
                                        <i class="ph-fill ph-play text-xl text-chrome"></i>
                                        <span>{move || {
                                            let watch_store = crate::store::use_watch_store();
                                            if let Some(rec) = watch_store.get_record(movie_id, is_tv) {
                                                if rec.percentage > 0.0 {
                                                    return crate::i18n::t("media-resume");
                                                }
                                            }
                                            crate::i18n::t("media-play")
                                        }}</span>
                                    </button>
                                    <button
                                        on:click=on_toggle_list
                                        class="flex items-center justify-center h-[54px] px-5 rounded-button bg-line text-ink text-title font-semibold gap-2 transition-all active:scale-95"
                                    >
                                        {move || if is_in_my_list() {
                                            view! {
                                                <i class="ph-bold ph-check text-xl text-ink"></i>
                                                <span>{crate::i18n::t("media-in-list")}</span>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <i class="ph-bold ph-plus text-xl text-ink"></i>
                                                <span>{crate::i18n::t("my-list")}</span>
                                            }.into_any()
                                        }}
                                    </button>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
