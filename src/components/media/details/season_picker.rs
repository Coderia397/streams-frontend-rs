//! The season menu of Netflix's Android app, shared by the phone's title page and its player (its layout and theme read from the app on
//! a Pixel 8, note 116): over the screen in the menu black, at once with no fade; the seasons in a column 52 px apart, the one shown in
//! bold and larger, centred between 90 px from the top and 110 px from the bottom, or from 90 px down and scrolling when they do not fit;
//! the lowest 170 px fade to solid black, and a white round close button 64 px wide sits 20 px above the bottom. A tap on a season picks
//! it and closes the menu; a tap beside it closes it.
use leptos::prelude::*;

#[component]
pub fn SeasonPicker(open: RwSignal<bool>, seasons: u32, season: RwSignal<u32>) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-(--z-sheet) bg-scrim-menu text-ink" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); open.set(false); }>
                <div class="absolute inset-0 overflow-y-auto overscroll-contain scrollbar-hide">
                    <div class="flex min-h-full flex-col items-center justify-center pt-[90px] pb-[110px]">
                        {(1..=seasons).map(|n| view! {
                            <button
                                type="button"
                                on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); season.set(n); open.set(false); }
                                class=move || format!("flex h-[52px] w-[248px] shrink-0 items-center justify-center cursor-pointer {}", if season.get() == n { "text-heading font-bold text-ink-soft" } else { "text-title text-ink-body" })
                            >
                                {crate::i18n::t_args("media-season-n", &[("n", fluent_bundle::FluentValue::from(n as i64))])}
                            </button>
                        }).collect_view()}
                    </div>
                </div>
                <div class="pointer-events-none absolute inset-x-0 bottom-0 h-[170px] bg-linear-to-b from-transparent to-void"></div>
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
