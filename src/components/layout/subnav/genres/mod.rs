//! The genre header of the Series and Films pages (and Home's genre picker on phones), one form at a time (note 114): `website` (the
//! page's title, the Genres pill and its menu, fixed under the top bar) and `phone` (the app's pills under the header).
pub mod phone;
pub mod website;

use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubNavGenre {
    pub id: u32,
    pub name: String,
}

#[component]
pub fn CategorySubNav(
    #[prop(into)] title: String,
    genres: Vec<SubNavGenre>,
    #[prop(into)] selected_genre: Signal<Option<SubNavGenre>>,
    on_genre_select: Callback<Option<SubNavGenre>>,
    #[prop(optional)] dropdown_label: Option<String>,
    /// Home: the last pill of the phone header, Categories, that opens the list of genres
    #[prop(optional, default = true)] show_categories: bool,
    /// Home has the genre picker on phones only: the website form shows nothing
    #[prop(optional, default = false)] phone_only: bool,
) -> impl IntoView {
    let phone_genres = genres.clone();
    let phone_label = dropdown_label.clone();
    view! {
        <crate::form::ByForm
            phone=move || view! {
                <div class="mobile-subnav pt-[calc(var(--nav-h-mobile)+env(safe-area-inset-top))]">
                    <phone::CategorySubNavMobile
                        genres=phone_genres.clone()
                        selected_genre=selected_genre
                        on_genre_select=on_genre_select
                        dropdown_label=phone_label.clone()
                        show_categories=show_categories
                    />
                </div>
            }
            desktop=move || (!phone_only).then(|| view! {
                <website::CategorySubNavDesktop
                    title=title.clone()
                    genres=genres.clone()
                    selected_genre=selected_genre
                    on_genre_select=on_genre_select
                    dropdown_label=dropdown_label.clone()
                />
            })
        />
    }
}
