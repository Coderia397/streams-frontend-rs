//! The season menu of Netflix's Android app, shared by the phone's title page and its player (recorded on a Pixel 8, note 116): the
//! seasons in a column in the middle over the darkened screen, 52 px apart, the one shown in bold and larger, a white round close button
//! 64 px wide under them, 20 px above the bottom. A tap on a season picks it and closes the menu; a tap beside it closes it.
use leptos::prelude::*;

#[component]
pub fn SeasonPicker(open: RwSignal<bool>, seasons: u32, season: RwSignal<u32>) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-(--z-sheet) bg-void/90 text-ink animate-in fade-in" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); open.set(false); }>
                <div class="absolute inset-x-0 top-0 bottom-[96px] flex flex-col items-center overflow-y-auto scrollbar-hide pt-[64px]">
                    {(1..=seasons).map(|n| view! {
                        <button
                            type="button"
                            on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); season.set(n); open.set(false); }
                            class=move || format!("flex h-[52px] w-[248px] shrink-0 items-center justify-center cursor-pointer {}", if season.get() == n { "text-title-md font-bold text-ink" } else { "text-title-sm text-ink-body" })
                        >
                            {crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(n as i64))])}
                        </button>
                    }).collect_view()}
                </div>
                <button
                    type="button"
                    on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); open.set(false); }
                    aria-label=crate::i18n::t("action-buttons-close")
                    class="absolute bottom-5 left-1/2 flex h-16 w-16 -translate-x-1/2 items-center justify-center rounded-full bg-ink text-void cursor-pointer"
                >
                    <i class="ph-bold ph-x text-heading"></i>
                </button>
            </div>
        </Show>
    }
}
