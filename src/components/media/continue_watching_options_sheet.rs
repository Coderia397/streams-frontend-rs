use leptos::prelude::*;
use leptos::portal::Portal;
use crate::models::movie::Movie;

#[component]
pub fn ContinueWatchingOptionsSheet(
    movie: Option<Movie>,
    on_close: Callback<()>,
    on_remove: Callback<Movie>,
) -> impl IntoView {
    let has_movie = movie.is_some();

    // Body scroll lock effect
    Effect::new(move |_| {
        if let Some(w) = web_sys::window() {
            if let Some(doc) = w.document() {
                if let Some(body) = doc.body() {
                    if has_movie {
                        let _ = body.style().set_property("overflow", "hidden");
                    } else {
                        let _ = body.style().set_property("overflow", "");
                    }
                }
            }
        }
    });

    let movie_stored = StoredValue::new(movie);
    let on_close_stored = StoredValue::new(on_close);
    let on_remove_stored = StoredValue::new(on_remove);

    view! {
        <Portal>
            {move || if let Some(m) = movie_stored.get_value() {
                let title = m.display_title();
                let m_for_remove = m.clone();

                let handle_remove = move |_| {
                    on_remove_stored.get_value().run(m_for_remove.clone());
                    on_close_stored.get_value().run(());
                };

                let handle_close = move |_| {
                    on_close_stored.get_value().run(());
                };

                view! {
                    <div class="fixed inset-0 z-(--z-sheet-backdrop) flex flex-col justify-end">
                        // Backdrop
                        <div
                            class="fixed inset-0 bg-void/60 transition-opacity duration-(--motion-base)"
                            on:click=handle_close
                        />

                        // Sheet content
                        <div class="relative z-(--z-sheet) bg-sheet rounded-t-sheet px-4 pt-2 pb-8 max-w-lg mx-auto w-full shadow-2xl transition-transform duration-(--motion-slow)">
                            <div class="w-10 h-1 rounded-full bg-ink/30 mx-auto mb-4" />

                            <div class="px-1 mb-4">
                                <span class="text-ink text-body-lg font-bold truncate block">
                                    {title}
                                </span>
                            </div>

                            <button
                                type="button"
                                on:click=handle_remove
                                // radius: a row of the options sheet, a card in looks
                                class="w-full flex items-center gap-4 bg-menu rounded-modal px-4 py-4 active:scale-[0.99] active:bg-line transition-all cursor-pointer"
                            >
                                <i class="ph-bold ph-x-circle text-ink text-2xl shrink-0"></i>
                                <span class="text-ink text-body-md font-medium">
                                    {crate::i18n::t("continue-watching-options-sheet-remove-from-continue-watching")}
                                </span>
                            </button>

                            <button
                                type="button"
                                on:click=handle_close
                                class="w-full text-center text-ink/60 text-body font-medium py-4 mt-1 active:text-ink/90 cursor-pointer"
                            >
                                {crate::i18n::t("continue-watching-options-sheet-cancel")}
                            </button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <span /> }.into_any()
            }}
        </Portal>
    }
}
