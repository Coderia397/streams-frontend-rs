use leptos::prelude::*;
use leptos::html::Div;
use crate::components::layout::Layout;
use crate::components::media::movie_card::MovieCard;
use crate::services::tmdb::fetch_by_language;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LangItem {
    pub code: &'static str,
    pub label: &'static str,
}

pub const BROWSE_LANGUAGES: &[LangItem] = &[
    LangItem { code: "en", label: "English" },
    LangItem { code: "es", label: "Spanish" },
    LangItem { code: "fr", label: "French" },
    LangItem { code: "de", label: "German" },
    LangItem { code: "it", label: "Italian" },
    LangItem { code: "pt", label: "Portuguese" },
    LangItem { code: "ru", label: "Russian" },
    LangItem { code: "ja", label: "Japanese" },
    LangItem { code: "ko", label: "Korean" },
    LangItem { code: "zh", label: "Chinese" },
    LangItem { code: "ar", label: "Arabic" },
    LangItem { code: "hi", label: "Hindi" },
    LangItem { code: "tr", label: "Turkish" },
    LangItem { code: "nl", label: "Dutch" },
    LangItem { code: "pl", label: "Polish" },
    LangItem { code: "sv", label: "Swedish" },
    LangItem { code: "da", label: "Danish" },
    LangItem { code: "no", label: "Norwegian" },
    LangItem { code: "th", label: "Thai" },
    LangItem { code: "id", label: "Indonesian" },
];

/// Message keys of the ways to pick a language. Only the original language of a title is in our data (TMDB lists no dubbing or subtitle
/// tracks), so it is the only way offered; the choice appears again when a source for the others exists.
pub const PREF_TYPES: &[&'static str] = &["lang-original"];

#[component]
pub fn BrowseLanguagePage() -> impl IntoView {
    let (selected_pref_type, set_selected_pref_type) = signal("lang-original");
    let (pref_dropdown_open, set_pref_dropdown_open) = signal(false);

    let (selected_lang, set_selected_lang) = signal("en");
    let (lang_dropdown_open, set_lang_dropdown_open) = signal(false);

    let pref_container_ref = NodeRef::<Div>::new();
    let lang_container_ref = NodeRef::<Div>::new();

    // a press outside a dropdown closes it
    crate::utils::listen::on_press_outside(
        move |target| crate::utils::listen::contains(pref_container_ref, target),
        move || if pref_dropdown_open.get_untracked() { set_pref_dropdown_open.set(false) },
    );
    crate::utils::listen::on_press_outside(
        move |target| crate::utils::listen::contains(lang_container_ref, target),
        move || if lang_dropdown_open.get_untracked() { set_lang_dropdown_open.set(false) },
    );

    let active_label = move || {
        let code = selected_lang.get();
        BROWSE_LANGUAGES
            .iter()
            .find(|l| l.code == code)
            .map(|l| crate::i18n::language_name(l.code))
            .unwrap_or_else(|| crate::i18n::language_name("en"))
    };

    // Languages a limited profile has too few titles for are not offered
    let hidden_langs = LocalResource::new(move || {
        let codes: Vec<String> = BROWSE_LANGUAGES.iter().map(|l| l.code.to_string()).collect();
        async move { crate::services::ai_engine::hidden_languages(&codes).await }
    });

    let items_resource = LocalResource::new(move || {
        let lang = selected_lang.get();
        async move {
            if let Some(items) = crate::services::ai_engine::fetch_language_titles(lang).await {
                return items;
            }
            let mut all = fetch_by_language(lang, 1).await.unwrap_or_default();
            if let Ok(mut more) = fetch_by_language(lang, 2).await {
                all.append(&mut more);
            }
            all
        }
    });

    let toggle_pref_dropdown = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        set_lang_dropdown_open.set(false);
        set_pref_dropdown_open.update(|o| *o = !*o);
    };

    let toggle_lang_dropdown = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        set_pref_dropdown_open.set(false);
        set_lang_dropdown_open.update(|o| *o = !*o);
    };

    view! {
        <Layout>
            // the header as the Series and Films pages' (Netflix's own Browse by Language is still its old page with square, outlined menus): title 24 px
            // weight 500 at 98, the menus as the Genres pill, centred at 113
            <div class="bg-void md:bg-page min-h-screen pb-20 pt-[calc(4rem+env(safe-area-inset-top))] md:pt-[93px]">
                <div class="px-6 md:px-[var(--app-x,56px)] pt-2 md:pt-0">
                    // Header with title and Netflix-matching preference dropdowns
                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-8">
                        <h1 class="select-none text-heading-lg font-bold leading-tight tracking-tight text-ink md:text-[24px] md:font-medium md:leading-[31px] md:tracking-normal">
                            {crate::i18n::t("language-browse-by-language")}
                        </h1>

                        <div class="flex items-center gap-3">
                            <span class="hidden select-none whitespace-nowrap text-body-sm text-ink md:block">
                                {crate::i18n::t("language-select-your-preferences")}
                            </span>

                            // Dropdown 1: Preference Type (only shown when there is more than one way to pick)
                            <div node_ref=pref_container_ref class="relative" >
                                <button
                                    on:click=toggle_pref_dropdown
                                    class="btn-outline btn-pill btn-pill-md select-none"
                                    class=("!bg-glass", move || pref_dropdown_open.get())
                                >
                                    <span>{move || crate::i18n::t(selected_pref_type.get())}</span>
                                    <i class="ph-fill ph-caret-down shrink-0 transition-transform duration-(--motion-base)"
                                       class=("rotate-180", move || pref_dropdown_open.get())></i>
                                </button>

                                {move || {
                                    if pref_dropdown_open.get() {
                                        view! {
                                            <div class="absolute top-[calc(100%+8px)] left-0 w-full min-w-[170px] bg-page/95 backdrop-blur-2xl border border-ink/15 shadow-2xl z-50 p-4 md:p-6 scrollbar-hide rounded-sheet">
                                                {PREF_TYPES.iter().map(|&t| {
                                                    let is_selected = selected_pref_type.get() == t;
                                                    view! {
                                                        <button
                                                            on:click=move |_| {
                                                                set_selected_pref_type.set(t);
                                                                set_pref_dropdown_open.set(false);
                                                            }
                                                            class=if is_selected {
                                                                "w-full text-left px-3.5 py-1.5 text-label md:text-body-sm text-ink font-bold bg-ink/10 rounded-menu-item cursor-pointer"
                                                            } else {
                                                                "w-full text-left px-3.5 py-1.5 text-label md:text-body-sm text-ink-soft hover:text-ink hover:bg-ink/15 rounded-menu-item transition-colors cursor-pointer"
                                                            }
                                                        >
                                                            {crate::i18n::t(t)}
                                                        </button>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div /> }.into_any()
                                    }
                                }}
                            </div>

                            // Dropdown 2: Language Selector
                            <div node_ref=lang_container_ref class="relative">
                                <button
                                    on:click=toggle_lang_dropdown
                                    class="btn-outline btn-pill btn-pill-md select-none"
                                    class=("!bg-glass", move || lang_dropdown_open.get())
                                >
                                    <span>{active_label}</span>
                                    <i class="ph-fill ph-caret-down shrink-0 transition-transform duration-(--motion-base)"
                                       class=("rotate-180", move || lang_dropdown_open.get())></i>
                                </button>

                                {move || {
                                    if lang_dropdown_open.get() {
                                        view! {
                                            <div class="absolute top-[calc(100%+8px)] left-0 w-full min-w-[150px] max-h-64 overflow-y-auto bg-page/95 backdrop-blur-2xl border border-ink/15 shadow-2xl z-50 p-4 md:p-6 scrollbar-hide rounded-sheet">
                                                {BROWSE_LANGUAGES.iter().filter(|lang| !hidden_langs.get().unwrap_or_default().contains(lang.code) || selected_lang.get_untracked() == lang.code).map(|lang| {
                                                    let code = lang.code;
                                                    let label = crate::i18n::language_name(lang.code);
                                                    let is_selected = selected_lang.get() == code;
                                                    view! {
                                                        <button
                                                            on:click=move |_| {
                                                                set_selected_lang.set(code);
                                                                set_lang_dropdown_open.set(false);
                                                            }
                                                            class=if is_selected {
                                                                "w-full text-left px-3.5 py-1.5 text-label md:text-body-sm text-ink font-bold bg-ink/10 rounded-menu-item cursor-pointer"
                                                            } else {
                                                                "w-full text-left px-3.5 py-1.5 text-label md:text-body-sm text-ink-soft hover:text-ink hover:bg-ink/15 rounded-menu-item transition-colors cursor-pointer"
                                                            }
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div /> }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // Content grid — exact 6-column Netflix desktop grid with tight horizontal gap and full aspect-video cards
                    <Suspense fallback=move || view! {
                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-1.5 md:gap-x-2 gap-y-6 md:gap-y-5 animate-pulse">
                            {(0..18).map(|_| view! {
                                <div class="aspect-video bg-card rounded-control border border-ink/[0.04]"></div>
                            }).collect::<Vec<_>>()}
                        </div>
                    }>
                        {move || items_resource.get().map(|items| {
                            if items.is_empty() {
                                view! {
                                    <div class="flex flex-col items-center justify-center mt-24 text-center">
                                        <i class="ph ph-globe text-5xl text-ink/30 mb-3"></i>
                                        <p class="text-ink/50 text-lg">{crate::i18n::t("lang-no-titles")}</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-1.5 md:gap-x-2 gap-y-6 md:gap-y-5">
                                        {items.into_iter().map(|item| {
                                            let movie_id = item.id;
                                            let is_tv = item.is_tv();
                                            let title = item.display_title().to_string();
                                            let backdrop = item.backdrop_url("w780").unwrap_or_default();
                                            let poster = item.poster_url("w342").unwrap_or_default();
                                            let vote_avg = item.vote_average;
                                            view! {
                                                <MovieCard
                                                    movie_id=movie_id
                                                    is_tv=is_tv
                                                    title=title
                                                    backdrop_path=backdrop
                                                    poster_path=poster
                                                    vote_average=vote_avg
                                                    overview=item.overview.clone()
                                                    genre_ids=item.genre_ids.clone()
                                                    vibe_pills=item.vibe_pills.clone()
                                                    is_grid=true
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </Layout>
    }
}
