use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_location};
use super::SubNavGenre;

/// The pills under the header of the phone layout, as in the app: on Home the pills Series, Films, Games, the events in season (Halloween while it is on) and New & Hot;
/// then Categories, as in the app; on the Series and Films pages one outlined pill, All Categories, that opens the list of genres; on the page of an event nothing.
#[component]
pub fn CategorySubNavMobile(
    #[prop(optional)] _title: Option<String>,
    genres: Vec<SubNavGenre>,
    #[prop(into)] selected_genre: Signal<Option<SubNavGenre>>,
    on_genre_select: Callback<Option<SubNavGenre>>,
    #[prop(default = None)] dropdown_label: Option<String>,
    /// Home: the last pill, Categories, that opens the list of genres (the pill of the Series and Films pages shows whatever this says)
    #[prop(optional, default = true)] show_categories: bool,
) -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();
    let (genre_menu_open, set_genre_menu_open) = signal(false);
    let stored_genres = StoredValue::new(genres);

    let path = move || location.pathname.get();
    let on_home = move || { let p = path(); p == "/browse" || p == "/" };
    let on_kind_page = move || {
        let p = path();
        let search = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
        p.starts_with("/browse/series") || p.starts_with("/browse/films") || p.starts_with("/browse/shows") || p.starts_with("/browse/movies")
            || crate::data::page_genres::page_kind_of_genre_route(&p, &search).is_some() && (search.contains("bc=") || p.ends_with("/83") || p.ends_with("/34399"))
    };
    let is_new_active = move || path() == "/latest";
    let is_games_active = move || path().starts_with("/browse/games");
    // events in season with a page of their own (Halloween); the list is empty out of season, so the pill goes away by itself
    let features = LocalResource::new(|| async { crate::services::ai_engine::fetch_features().await });
    let games_on = move || features.get().is_some_and(|f| f.get("games") == Some(&true));
    let season_chips = LocalResource::new(|| async { crate::services::ai_engine::fetch_season_chips().await });

    let to = |url: &'static str| { let nav = navigate.clone(); move |_| nav(url, Default::default()) };
    let on_series = to("/browse/genre/83");
    let on_films = to("/browse/genre/34399");
    let on_latest = to("/latest");
    let navigate_event = StoredValue::new_local(navigate.clone());

    let active_genre_label = Signal::derive(move || {
        selected_genre.get().map(|g| crate::i18n::genre_label(&g.name)).unwrap_or_else(|| dropdown_label.clone().unwrap_or_else(|| crate::i18n::t("nav-all-categories")))
    });

    // one look for every chip: filled when it is the page in use, glass otherwise. Netflix's Android app (note 116): 15 px padding, 10 %
    // white with a 10 % white edge; height and corners come from `chip-group` on the row
    let pill = |active: bool| if active {
        "flex items-center justify-center px-[15px] text-body-lg font-medium whitespace-nowrap active:scale-95 transition-all leading-none shrink-0 cursor-pointer bg-ink text-void border border-ink"
    } else {
        "flex items-center justify-center px-[15px] text-body-lg font-medium whitespace-nowrap active:scale-95 transition-all leading-none shrink-0 cursor-pointer bg-ink/10 text-ink-dim border border-ink/10"
    };

    view! {
        <div class="relative w-full select-none">
            // Home: Series, Films, Games, events in season, New & Hot
            <Show when=on_home>
                <div class="pt-2 pb-4 px-4 flex items-center justify-start overflow-x-auto scrollbar-hide max-w-full">
                    <div class="chip-group flex items-center gap-[6px] shrink-0">
                        <button on:click=on_series.clone() class=move || pill(false)>{crate::i18n::t("category-sub-nav-mobile-series")}</button>
                        <button on:click=on_films.clone() class=move || pill(false)>{crate::i18n::t("category-sub-nav-mobile-films")}</button>
                        <Show when=games_on><button on:click=move |_| navigate_event.with_value(|n| n("/browse/games", Default::default())) class=move || pill(is_games_active())>{crate::i18n::t("nav-games")}</button></Show>
                        {move || season_chips.get().unwrap_or_default().into_iter().map(|chip| {
                            let label = crate::services::ai_engine::engine_text(&chip.label_key, None).unwrap_or_else(|| chip.label.clone());
                            let id = chip.genre_id;
                            view! { <button on:click=move |_| navigate_event.with_value(|nav| nav(&format!("/browse/genre/{id}"), Default::default())) class=move || pill(false)>{label}</button> }
                        }).collect::<Vec<_>>()}
                        <button on:click=on_latest.clone() class=move || pill(is_new_active())>{crate::i18n::t("category-sub-nav-mobile-new-hot")}</button>
                        <Show when=move || show_categories>
                            <button on:click=move |_| set_genre_menu_open.update(|o| *o = !*o) aria-haspopup="listbox" class=move || format!("{} gap-2", pill(false))>
                                <span>{crate::i18n::t("nav-categories")}</span>
                                <i class="ph-fill ph-caret-down text-sm opacity-80"></i>
                            </button>
                        </Show>
                    </div>
                </div>
            </Show>

            // Series and Films: one pill that opens the genres
            <Show when=on_kind_page>
                <div class="pt-2 pb-4 px-4">
                    <button
                        on:click=move |_| set_genre_menu_open.update(|o| *o = !*o)
                        aria-haspopup="listbox"
                        // radius: a dropdown chip, 12 px corners as in Netflix's Android app (43 px high, 40 % white edge; note 116)
                        class="flex items-center gap-3 h-[43px] ps-5 pe-4 rounded-chip-inner border border-ink/40 bg-transparent text-title font-medium leading-none whitespace-nowrap active:scale-95 transition-transform cursor-pointer text-ink"
                    >
                        <span>{move || active_genre_label.get()}</span>
                        <i class="ph-fill ph-caret-down text-base opacity-80"></i>
                    </button>
                </div>
            </Show>

            // The list of genres: a card over a dimmed page, on the side of the pill that opened it (right for Categories on Home, left for the pill of the Series and Films pages)
            <Show when=move || genre_menu_open.get()>
                <div class="fixed inset-0 z-(--z-drawer)">
                    <div on:click=move |_| set_genre_menu_open.set(false) class="absolute inset-0 bg-void/60"></div>
                    <div class=move || format!("absolute {} top-[calc(env(safe-area-inset-top)+8px)] w-[min(66vw,320px)] max-h-[calc(100dvh-24px)] overflow-y-auto rounded-menu bg-raised py-2 shadow-2xl scrollbar-hide", if on_home() { "right-2" } else { "left-2" }) role="listbox">
                        <button
                            on:click=move |_| { set_genre_menu_open.set(false); on_genre_select.run(None); }
                            class=move || if selected_genre.get().is_none() { "block w-full text-left px-4 py-3 text-title font-bold text-ink cursor-pointer" } else { "block w-full text-left px-4 py-3 text-title text-ink/85 cursor-pointer" }
                        >
                            {crate::i18n::t("nav-all-categories")}
                        </button>
                        {move || stored_genres.with_value(|g_list| g_list.iter().map(|genre| {
                            let g = genre.clone();
                            let name = genre.name.clone();
                            let is_sel = move || selected_genre.get().is_some_and(|s| s.name == name);
                            view! {
                                <button
                                    on:click=move |_| { set_genre_menu_open.set(false); on_genre_select.run(Some(g.clone())); }
                                    class=move || if is_sel() { "block w-full text-left px-4 py-3 text-title font-bold text-ink cursor-pointer" } else { "block w-full text-left px-4 py-3 text-title text-ink/85 cursor-pointer" }
                                >
                                    {crate::i18n::genre_label(&genre.name)}
                                </button>
                            }
                        }).collect::<Vec<_>>())}
                    </div>
                </div>
            </Show>
        </div>
    }
}
