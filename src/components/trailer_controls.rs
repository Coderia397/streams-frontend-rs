//! The controls of a trailer playing inside a page on the phone form (New & Hot cards, the title page), as Netflix's Android app draws
//! them (recorded on a Pixel 8, 2026-10-24, note 116). One progress bar: at rest a 2 px line along the frame's bottom edge; a tap on the
//! frame dims the picture (half black), shows a pause or play mark in the middle and slides the same bar up into a scrubber 22 px above
//! the bottom (a 1.5 px track from 12 px in to 62 px from the right, the played part in the brand colour, the rest at a quarter, a 21 px
//! knob) with the time played at the right; the sound button rises with it. Three seconds after the last touch everything slides back.
//! Laid over a `relative` frame that holds the TrailerPlayer; the frame's owner keeps the time, the length and the pause state.
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// How long the controls stay after the last touch, in milliseconds.
const SHOW_MS: u64 = 3_000;

fn mm_ss(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

#[component]
pub fn TrailerControls(
    /// The time played, in seconds
    #[prop(into)] time: Signal<f64>,
    /// The trailer's length, in seconds (0 until the player has told it)
    #[prop(into)] duration: Signal<f64>,
    /// Paused by the viewer
    paused: RwSignal<bool>,
    /// A time to jump to (the player follows it)
    seek: RwSignal<Option<f64>>,
    /// Called with the new time as soon as the viewer moves the knob, so the bar follows the finger
    #[prop(optional)] on_scrub: Option<Callback<f64>>,
) -> impl IntoView {
    let ui_store = crate::store::use_ui_store();
    let shown = RwSignal::new(false);
    let hide_timer = StoredValue::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    let show = move || {
        shown.set(true);
        if let Some(h) = hide_timer.get_value() {
            h.clear();
        }
        hide_timer.set_value(
            leptos::leptos_dom::helpers::set_timeout_with_handle(move || { shown.try_set(false); }, std::time::Duration::from_millis(SHOW_MS)).ok(),
        );
    };
    on_cleanup(move || {
        if let Some(h) = hide_timer.get_value() {
            h.clear();
        }
    });
    let fraction = move || if duration.get() > 0.0 { (time.get() / duration.get()).clamp(0.0, 1.0) } else { 0.0 };

    // a touch on the track moves the knob there
    let scrub = move |e: leptos::ev::PointerEvent| {
        e.stop_propagation();
        let Some(track) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let rect = track.get_bounding_client_rect();
        if rect.width() > 0.0 && duration.get_untracked() > 0.0 {
            let to = ((e.client_x() as f64 - rect.left()) / rect.width()).clamp(0.0, 1.0) * duration.get_untracked();
            seek.set(Some(to));
            if let Some(cb) = on_scrub {
                cb.run(to);
            }
            show();
        }
    };

    view! {
        // the tap catcher over the player (the YouTube frame would take the tap); a tap shows the controls, a tap while they show hides them
        <div class="absolute inset-0 z-10" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); if shown.get_untracked() { shown.set(false) } else { show() } }></div>

        // the dimmed picture and the pause or play mark
        <div class="pointer-events-none absolute inset-0 z-20 bg-void/50 transition-opacity duration-(--motion-fast)" class=("opacity-0", move || !shown.get())></div>
        <Show when=move || shown.get()>
            <button
                type="button"
                on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); paused.update(|p| *p = !*p); show(); }
                class="absolute left-1/2 top-1/2 z-30 flex h-16 w-16 -translate-x-1/2 -translate-y-1/2 items-center justify-center text-ink cursor-pointer"
                aria-label=move || if paused.get() { crate::i18n::t("media-play") } else { crate::i18n::t("title-pause") }
            >
                {move || if paused.get() {
                    view! { <i class="ph-fill ph-play text-icon-md"></i> }.into_any()
                } else {
                    // Netflix's mark: two square bars, 6.6 x 30 px, 7 px apart
                    view! { <span class="flex h-[30px] items-stretch gap-[7px]" aria-hidden="true"><span class="w-[6.6px] bg-ink"></span><span class="w-[6.6px] bg-ink"></span></span> }.into_any()
                }}
            </button>
        </Show>

        // the one bar: a line on the bottom edge at rest, a scrubber 22 px up while the controls show
        <div
            class="absolute z-30 flex items-center transition-all duration-(--motion-base) ease-out"
            style=move || if shown.get() { "left:12px; right:62px; bottom:11px; height:22px;" } else { "left:0; right:0; bottom:0; height:2px;" }
            on:pointerdown=scrub
        >
            <div class="relative w-full transition-all duration-(--motion-base)" class=("h-[1.5px]", move || shown.get()) class=("h-[2px]", move || !shown.get()) class=("bg-brand/25", move || shown.get())>
                <div class="absolute inset-y-0 left-0 bg-brand" style=move || format!("width:{:.2}%", fraction() * 100.0)></div>
                <Show when=move || shown.get()>
                    <div class="absolute top-1/2 h-[21px] w-[21px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-brand" style=move || format!("left:{:.2}%", fraction() * 100.0)></div>
                </Show>
            </div>
        </div>
        <Show when=move || shown.get()>
            <span class="pointer-events-none absolute bottom-[14px] right-[5px] z-30 text-label tabular-nums text-ink">{move || mm_ss(time.get())}</span>
        </Show>

        // the sound: centred 22 px from the right and 28 px above the bottom at rest, 26.5 px higher while the controls show
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); ui_store.set_preview_muted(!ui_store.preview_muted.get_untracked()); if shown.get_untracked() { show() } }
            aria-label=move || if ui_store.preview_muted.get() { crate::i18n::t("media-unmute") } else { crate::i18n::t("media-mute") }
            class="absolute right-[6px] z-30 btn-circle btn-circle-sm transition-[bottom] duration-(--motion-base) ease-out"
            style=move || if shown.get() { "bottom:38.5px" } else { "bottom:12px" }
        >
            <i class=move || if ui_store.preview_muted.get() { "ph-fill ph-speaker-slash text-body" } else { "ph-fill ph-speaker-high text-body" }></i>
        </button>
    }
}
