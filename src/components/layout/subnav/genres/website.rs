//! The genre header on tablets and computers (note 114): fixed under the top bar, the title and the Genres pill (website.rs of `genres`).
use leptos::prelude::*;
use leptos::html::Div;
use super::SubNavGenre;
use leptos_router::hooks::use_query_map;

/// The website's genre header: the page's title (or the strip and the genre picked), the Genres pill and its menu.
#[component]
pub fn CategorySubNavDesktop(
    title: String,
    genres: Vec<SubNavGenre>,
    #[prop(into)] selected_genre: Signal<Option<SubNavGenre>>,
    on_genre_select: Callback<Option<SubNavGenre>>,
    dropdown_label: Option<String>,
) -> impl IntoView {
    let query_map = use_query_map();
    let initial_open = query_map.with_untracked(|q| q.get("open_genres").is_some());
    let (genre_menu_open, set_genre_menu_open) = signal(initial_open);
    let container_ref = NodeRef::<Div>::new();

    // Close on outside click
    // a press anywhere else closes the menu
    crate::utils::listen::on_press_outside(
        move |target| crate::utils::listen::contains(container_ref, target),
        move || if genre_menu_open.get_untracked() { set_genre_menu_open.set(false) },
    );

    // 3-column transposed layout matching Netflix
    let mut sorted_genres = genres.clone();
    sorted_genres.sort_by_cached_key(|g| crate::i18n::genre_label(&g.name));
    let num_cols = 3;
    let num_rows = (sorted_genres.len() + num_cols - 1) / num_cols;
    let mut transposed_genres: Vec<Option<SubNavGenre>> = Vec::new();
    for r in 0..num_rows {
        for c in 0..num_cols {
            let idx = c * num_rows + r;
            if idx < sorted_genres.len() {
                transposed_genres.push(Some(sorted_genres[idx].clone()));
            } else {
                transposed_genres.push(None);
            }
        }
    }

    let (scroll_y, set_scroll_y) = signal(0.0);
    crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
        set_scroll_y.set(web_sys::window().and_then(|w| w.scroll_y().ok()).unwrap_or(0.0));
    });

    let scroll_bg_style = move || {
        let y = scroll_y.get();
        let op = if y <= 40.0 {
            y / 80.0
        } else {
            (0.5 + (y - 40.0) / 160.0).min(1.0)
        };
        format!("background-color: color-mix(in srgb, var(--color-page) {:.1}%, transparent);", op * 100.0)
    };

    let toggle_menu = move |e: leptos::ev::MouseEvent| {
        e.prevent_default();
        e.stop_propagation();
        set_genre_menu_open.update(|o| *o = !*o);
    };

    let title_clone = title.clone();
    let all_label = crate::i18n::t_args("category-all", &[("page", fluent_bundle::FluentValue::from(title.clone()))]);

    view! {
        <div
            class="fixed inset-x-0 top-(--nav-h) z-(--z-subnav) pointer-events-none transition-colors duration-(--motion-base)"
            style=scroll_bg_style
        >
            // the title and Genres sit 4 px above the bar's middle: Netflix centres them at 113 with the billboard at 154, as ours (measured at 1736 wide)
            <div class="pointer-events-none relative z-30 flex items-center justify-between px-(--app-x) pb-2 h-(--subnav-h) select-none w-full">
                <div node_ref=container_ref class="pointer-events-auto flex items-center gap-4">
                // A picked genre is a page of its own, as on Netflix: the genre's name after a short strip in the brand colour (Netflix: 4 x 28 at 70 %, 16 px before the
                // title). The Genres pill stays beside it, so another genre or the whole page is one click away (Netflix drops it and leaves only the nav tab).
                <h1 class="flex items-center text-[20px] font-medium leading-[31px] text-ink md:text-[24px]">
                    {move || selected_genre.get().map(|_| view! { <span aria-hidden="true" class="me-4 h-7 w-1 shrink-0 bg-brand/70" /> })}
                    {
                        let page_title = title_clone.clone();
                        move || selected_genre.get().map(|g| crate::i18n::genre_label(&g.name)).unwrap_or_else(|| page_title.clone())
                    }
                </h1>

                {
                    let s_g = sorted_genres.clone();
                    let t_g = transposed_genres.clone();
                    let d_l = dropdown_label.clone();
                    move || {
                        if !s_g.is_empty() {
                            let label = d_l.clone().unwrap_or_else(|| crate::i18n::t("nav-genres"));
                            let transposed = t_g.clone();
                            let on_select = on_genre_select;

                            view! {
                                <div class="relative">
                                    <button
                                        on:click=toggle_menu
                                        // Netflix's Genres button: the medium pill with a faint fill and a thin grey edge (note 111), 16 px after the title
                                        class="btn-outline btn-pill btn-pill-md"
                                        class=("!bg-glass", move || genre_menu_open.get())
                                        aria-haspopup="listbox"
                                        aria-expanded=move || genre_menu_open.get()
                                    >
                                        <span>{label}</span>
                                        <i
                                            class="ph-fill ph-caret-down shrink-0 transition-transform duration-(--motion-base)"
                                            class=("rotate-180", move || genre_menu_open.get())
                                        ></i>
                                    </button>

                                    // Dropdown popup
                                    <div
                                        class=move || if genre_menu_open.get() {
                                            "absolute top-[calc(100%+8px)] left-0 z-50 transition-all duration-(--motion-base) translate-x-0 opacity-100 translate-y-0 pointer-events-auto"
                                        } else {
                                            "absolute top-[calc(100%+8px)] left-0 z-50 transition-all duration-(--motion-base) translate-x-0 opacity-0 -translate-y-2 pointer-events-none"
                                        }
                                        role="listbox"
                                    >
                                        <div class="w-max max-w-[90vw] md:max-w-none max-h-[65vh] md:max-h-[460px] overflow-y-auto bg-page/95 backdrop-blur-2xl border border-ink/15 rounded-menu p-4 md:p-6 scrollbar-hide shadow-2xl">
                                            // on a genre's page the menu starts with the whole page (All Series, All Films)
                                            {
                                                let all_label = all_label.clone();
                                                move || selected_genre.get().is_some().then(|| view! {
                                                    <button
                                                        on:click=move |_| {
                                                            set_genre_menu_open.set(false);
                                                            on_select.run(None);
                                                        }
                                                        role="option"
                                                        aria-selected="false"
                                                        class="mb-2 flex w-full items-center gap-2 px-3 py-1.5 rounded-menu-item text-left text-body-sm md:text-body font-medium text-ink/80 hover:bg-ink/15 hover:text-ink whitespace-nowrap cursor-pointer"
                                                    >
                                                        <i class="ph-bold ph-arrow-left shrink-0 rtl:-scale-x-100" aria-hidden="true"></i>
                                                        {all_label.clone()}
                                                    </button>
                                                })
                                            }
                                            <div class="grid grid-cols-[repeat(3,max-content)] gap-x-6 md:gap-x-10 gap-y-1">
                                                {transposed.into_iter().enumerate().map(|(_idx, opt)| {
                                                    if let Some(genre) = opt {
                                                        let g_clone = genre.clone();
                                                        let id = genre.id;
                                                        let current = move || selected_genre.get().is_some_and(|s| s.id == id);
                                                        view! {
                                                            <button
                                                                on:click=move |_| {
                                                                    set_genre_menu_open.set(false);
                                                                    if !current() { on_select.run(Some(g_clone.clone())); }
                                                                }
                                                                role="option"
                                                                aria-selected=move || current().to_string()
                                                                class="text-left text-body-sm md:text-body font-medium transition-all px-3 py-1.5 rounded-menu-item hover:bg-ink/15 whitespace-nowrap text-ink/80 hover:text-ink cursor-pointer aria-selected:bg-ink/10 aria-selected:text-ink"
                                                            >
                                                                {crate::i18n::genre_label(&genre.name)}
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div class="h-6 pointer-events-none"></div> }.into_any()
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }
                }
            </div>
        </div>
    </div>
    }
}
