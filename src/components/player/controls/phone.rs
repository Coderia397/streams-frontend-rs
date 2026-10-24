//! The phone's player controls, as Netflix's Android app draws them over a landscape video (view tree of its player on a Pixel 8,
//! 2026-10-24, note 116; positions in px of a landscape screen, measured from the edge of the area the page may draw in). A tap shows
//! them and four seconds without one hides them while the video plays. Top: back, the title, the three ratings, Report a problem and
//! Lock. Middle: back 10 s, play or pause and forward 10 s, 84 px each, 186 px apart from the middle. Bottom: the seek bar with the time
//! left, then Speed, Episodes, Audio & Subtitles and Next Ep. spread across the width. Lock leaves one button that unlocks.
//! Netflix's brightness slider is left out (a page cannot set the screen's brightness); in place of its Clip button, Full screen, which
//! brings full screen and landscape back after the browser left them (owner, 2026-10-24).
use leptos::prelude::*;
use super::{time_left, ControlsState};
use crate::components::media::movie_card_rating::{MovieRating, RatingIcon};

/// How long the controls stay after the last touch while the video plays, in milliseconds.
const SHOW_MS: u64 = 4_000;
/// The speeds of the speed menu, as Netflix's.
const SPEEDS: [f64; 5] = [0.5, 0.75, 1.0, 1.25, 1.5];

fn speed_label(v: f64) -> String {
    let s = format!("{v}");
    format!("{s}x")
}

#[component]
pub fn PhoneControls(state: ControlsState) -> impl IntoView {
    let ControlsState {
        title, episode, episode_name, media_id, is_tv, is_playing, is_buffering, current_time, duration, speed, on_set_speed,
        on_toggle_play, on_seek, on_skip_by, on_toggle_episodes, on_toggle_subtitles, on_next_episode, on_back, ..
    } = state;
    let shown = RwSignal::new(true);
    let locked = RwSignal::new(false);
    let speed_open = RwSignal::new(false);
    // whether the browser is in full screen (it leaves it by itself when the phone locks or another app opens)
    let is_full = || web_sys::window().and_then(|w| w.document()).is_some_and(|d| d.fullscreen_element().is_some());
    let full_screen = RwSignal::new(is_full());
    crate::utils::listen::listen_window(leptos::ev::resize, move |_| full_screen.set(is_full()));
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || { full_screen.try_set(is_full()); });
        let _ = doc.add_event_listener_with_callback("fullscreenchange", wasm_bindgen::JsCast::unchecked_ref(cb.as_ref()));
        let kept = StoredValue::new_local(Some(cb));
        on_cleanup(move || {
            kept.try_update_value(|k| {
                if let (Some(cb), Some(doc)) = (k.take(), web_sys::window().and_then(|w| w.document())) {
                    let _ = doc.remove_event_listener_with_callback("fullscreenchange", wasm_bindgen::JsCast::unchecked_ref(cb.as_ref()));
                }
            });
        });
    }
    let hide_timer = StoredValue::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    // shows the controls and hides them again after a while (not while paused, as the app)
    let touch = move || {
        shown.set(true);
        if let Some(h) = hide_timer.get_value() {
            h.clear();
        }
        hide_timer.set_value(
            leptos::leptos_dom::helpers::set_timeout_with_handle(
                move || { if is_playing.try_get_untracked() == Some(true) { shown.try_set(false); } },
                std::time::Duration::from_millis(SHOW_MS),
            )
            .ok(),
        );
    };
    touch();
    on_cleanup(move || {
        if let Some(h) = hide_timer.get_value() {
            h.clear();
        }
    });
    // the app's line: `S1:E23 "Red John's Footsteps"` for an episode, the title for a film
    let heading = move || match episode {
        Some((s, e)) => match episode_name.get() {
            Some(name) => format!("S{s}:E{e} \"{name}\""),
            None => format!("S{s}:E{e}"),
        },
        None => title.get_value(),
    };
    let rating = RwSignal::new(crate::store::ratings::get(media_id, is_tv));
    let rate = move |r: MovieRating| {
        let next = if rating.get_untracked() == Some(r) { None } else { Some(r) };
        rating.set(next);
        crate::store::ratings::set(media_id, is_tv, next);
        touch();
    };
    // every button: the control's own action, then the controls stay a while longer
    let act = move |f: Callback<()>| move |e: leptos::ev::MouseEvent| { e.stop_propagation(); f.run(()); touch(); };

    view! {
        // the tap catcher over the video: a tap shows the controls, a tap while they show hides them
        <div class="absolute inset-0 z-20 select-none" on:click=move |_| if locked.get_untracked() || !shown.get_untracked() { touch() } else { shown.set(false) }></div>

        // the buffering ring, whether the controls show or not
        <Show when=move || is_buffering.get()>
            <div class="pointer-events-none absolute inset-0 z-30 flex items-center justify-center">
                <div class="h-16 w-16 animate-spin rounded-full border-4 border-brand border-t-transparent"></div>
            </div>
        </Show>

        // locked: one button at the bottom, which unlocks
        <Show when=move || locked.get() && shown.get()>
            <div class="absolute inset-x-0 bottom-[24px] z-30 flex flex-col items-center gap-2">
                <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); locked.set(false); touch(); } class="btn-circle btn-circle-lg bg-ink/15" aria-label=crate::i18n::t("player-unlock")>
                    <i class="ph-fill ph-lock-simple text-heading"></i>
                </button>
                <span class="text-body-sm text-ink">{crate::i18n::t("player-screen-locked")}</span>
            </div>
        </Show>

        // the controls: one layer that fades; a tap on its empty space hides it
        <div
            class="absolute inset-0 z-30 bg-void/40 text-ink transition-opacity duration-(--motion-base)"
            class=("opacity-0", move || !shown.get() || locked.get())
            class=("pointer-events-none", move || !shown.get() || locked.get())
            on:click=move |_| shown.set(false)
        >
            <div>
                // the top: back, the title, the three ratings after it, Report a problem and Lock at the right
                <div class="absolute inset-x-0 top-0 flex h-14 items-center pe-[env(safe-area-inset-right)] ps-[env(safe-area-inset-left)]">
                    <button type="button" on:click=act(on_back) class="flex h-14 w-14 shrink-0 items-center justify-center" aria-label=crate::i18n::t("player-back")>
                        <i class="ph ph-arrow-left text-heading"></i>
                    </button>
                    <span class="ms-2 min-w-0 truncate text-title-sm font-bold">{heading}</span>
                    <div class="ms-[29px] flex shrink-0 items-center gap-4">
                        {[(MovieRating::Dislike, "rating-not-for-me"), (MovieRating::Like, "rating-like"), (MovieRating::Love, "rating-love")].into_iter().map(|(r, key)| view! {
                            <button
                                type="button"
                                on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); rate(r); }
                                aria-label=crate::i18n::t(key)
                                class=move || format!("btn-circle btn-circle-lg {}", if rating.get() == Some(r) { "bg-ink text-void" } else { "bg-ink/15 text-ink" })
                            >
                                {move || view! { <RatingIcon rating=Some(r) size=24 mask_color=if rating.get() == Some(r) { "var(--color-ink)".to_string() } else { "var(--color-lift)".to_string() } /> }}
                            </button>
                        }).collect_view()}
                    </div>
                    <span class="flex-1"></span>
                    <a href="/help/contact" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation() class="flex h-14 w-14 shrink-0 items-center justify-center" aria-label=crate::i18n::t("controls-overlay-report-problem")>
                        <i class="ph ph-flag text-heading"></i>
                    </a>
                    <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); locked.set(true); touch(); } class="flex h-14 w-14 shrink-0 items-center justify-center" aria-label=crate::i18n::t("player-lock")>
                        <i class="ph ph-lock-simple-open text-heading"></i>
                    </button>
                </div>

                // the middle: back 10, play or pause, forward 10
                <div class="absolute left-1/2 top-1/2 flex -translate-x-1/2 -translate-y-1/2 items-center gap-[102px]">
                    <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); on_skip_by.run(-10.0); touch(); } class="relative flex h-[84px] w-[84px] items-center justify-center" aria-label=crate::i18n::t("controls-overlay-rewind-10-seconds-left-arrow")>
                        <i class="ph ph-arrow-counter-clockwise text-icon-lg"></i>
                        <span class="pointer-events-none absolute pt-[3px] text-label font-bold">"10"</span>
                    </button>
                    <button type="button" on:click=act(on_toggle_play) class="flex h-[84px] w-[84px] items-center justify-center" aria-label=move || if is_playing.get() { crate::i18n::t("player-pause") } else { crate::i18n::t("player-play") }>
                        {move || if is_playing.get() {
                            // Netflix's mark: two square bars
                            view! { <span class="flex h-[42px] items-stretch gap-[8.2px]" aria-hidden="true"><span class="w-[13.7px] bg-ink"></span><span class="w-[13.7px] bg-ink"></span></span> }.into_any()
                        } else {
                            view! { <i class="ph-fill ph-play text-icon-lg"></i> }.into_any()
                        }}
                    </button>
                    <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); on_skip_by.run(10.0); touch(); } class="relative flex h-[84px] w-[84px] items-center justify-center" aria-label=crate::i18n::t("controls-overlay-forward-10-seconds-right-arrow")>
                        <i class="ph ph-arrow-clockwise text-icon-lg"></i>
                        <span class="pointer-events-none absolute pt-[3px] text-label font-bold">"10"</span>
                    </button>
                </div>

                // the seek bar, its middle 82 px above the bottom, with the time left 25 px from the right
                // the visible line runs from 24.5 px in to 89 px from the right (Netflix's frames; its touch area is wider)
                <div class="absolute bottom-[71px] left-[calc(env(safe-area-inset-left)+24.5px)] right-[calc(env(safe-area-inset-right)+89px)] h-[22px]" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()>
                    <SeekBar current_time=current_time duration=duration on_seek=Callback::new(move |t| { on_seek.run(t); touch(); }) />
                </div>
                <span class="absolute bottom-[72px] right-[calc(env(safe-area-inset-right)+25px)] text-body-lg leading-[20px] tabular-nums">{move || time_left(current_time.get(), duration.get())}</span>

                // the buttons along the bottom, their middle 35 px above it
                <div class="absolute inset-x-0 bottom-[11px] flex h-12 items-center justify-evenly ps-[env(safe-area-inset-left)] pe-[env(safe-area-inset-right)]">
                    <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); if full_screen.get_untracked() { crate::components::player::frame::leave_landscape() } else { crate::components::player::frame::enter_landscape() }; touch(); } class="flex h-12 items-center gap-3 text-body-lg font-semibold">
                        <i class=move || if full_screen.get() { "ph ph-corners-in text-heading" } else { "ph ph-corners-out text-heading" }></i>
                        <span class="phone-player-label">{move || if full_screen.get() { crate::i18n::t("player-exit-full-screen") } else { crate::i18n::t("player-full-screen") }}</span>
                    </button>
                    <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); speed_open.set(true); touch(); } class="flex h-12 items-center gap-3 text-body-lg font-semibold">
                        <i class="ph ph-gauge text-heading"></i>
                        <span class="phone-player-label">{move || crate::i18n::t_args("player-speed", &[("speed", fluent_bundle::FluentValue::from(speed_label(speed.get())))])}</span>
                    </button>
                    {is_tv.then(|| view! {
                        <button type="button" on:click=act(on_toggle_episodes) class="flex h-12 items-center gap-3 text-body-lg font-semibold">
                            <i class="ph ph-cards text-heading"></i>
                            <span class="phone-player-label">{crate::i18n::t("media-episodes")}</span>
                        </button>
                    })}
                    <button type="button" on:click=act(on_toggle_subtitles) class="flex h-12 items-center gap-3 text-body-lg font-semibold">
                        <i class="ph ph-subtitles text-heading"></i>
                        <span class="phone-player-label">{crate::i18n::t("controls-overlay-audio-subtitles")}</span>
                    </button>
                    {on_next_episode.filter(|_| is_tv).map(|next| view! {
                        <button type="button" on:click=act(next) class="flex h-12 items-center gap-3 text-body-lg font-semibold">
                            <i class="ph ph-skip-forward text-heading"></i>
                            <span class="phone-player-label">{crate::i18n::t("player-next-ep")}</span>
                        </button>
                    })}
                </div>
            </div>
        </div>

        // the speed menu: the five speeds on a line over the darkened video, the one in use marked, a close button at the top right
        <Show when=move || speed_open.get()>
            <div class="absolute inset-0 z-40 flex flex-col items-center justify-center bg-void/85 text-ink" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); speed_open.set(false); touch(); }>
                <button type="button" class="absolute right-[calc(env(safe-area-inset-right)+16px)] top-4 btn-circle btn-circle-md" aria-label=crate::i18n::t("action-buttons-close")>
                    <i class="ph-bold ph-x text-title"></i>
                </button>
                <p class="mb-10 text-title-md font-bold">{crate::i18n::t("player-speed-title")}</p>
                <div class="relative flex w-[min(640px,80%)] items-start justify-between">
                    <span class="absolute left-[12px] right-[12px] top-[11px] h-[2px] bg-ink/40" aria-hidden="true"></span>
                    {SPEEDS.into_iter().map(|v| view! {
                        <button
                            type="button"
                            on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); on_set_speed.run(v); speed_open.set(false); touch(); }
                            class="relative flex w-6 flex-col items-center gap-4"
                        >
                            <span class=move || format!("block rounded-full bg-ink {}", if (speed.get() - v).abs() < 0.01 { "h-6 w-6" } else { "mt-[6px] h-3 w-3" })></span>
                            <span class=move || format!("whitespace-nowrap text-body-md {}", if (speed.get() - v).abs() < 0.01 { "font-bold" } else { "text-ink-body" })>
                                {if (v - 1.0).abs() < 0.01 { crate::i18n::t("player-speed-normal") } else { speed_label(v) }}
                            </span>
                        </button>
                    }).collect_view()}
                </div>
            </div>
        </Show>
    }
}

/// The app's seek bar: a 4 px track, the part played in the brand colour and the rest grey, a 22 px knob; a touch or a drag anywhere on
/// it moves the knob, and the video goes there when the finger lifts.
#[component]
fn SeekBar(current_time: ReadSignal<f64>, duration: ReadSignal<f64>, on_seek: Callback<f64>) -> impl IntoView {
    use wasm_bindgen::JsCast;
    // where the finger holds the knob, as a share of the bar (none when not dragging)
    let held = RwSignal::new(None::<f64>);
    let share = move |e: &web_sys::PointerEvent| -> Option<f64> {
        let el = e.current_target()?.dyn_into::<web_sys::Element>().ok()?;
        let r = el.get_bounding_client_rect();
        (r.width() > 0.0).then(|| ((e.client_x() as f64 - r.left()) / r.width()).clamp(0.0, 1.0))
    };
    let fraction = move || held.get().unwrap_or_else(|| if duration.get() > 0.0 { (current_time.get() / duration.get()).clamp(0.0, 1.0) } else { 0.0 });
    view! {
        <div
            class="relative h-full w-full touch-none cursor-pointer"
            on:pointerdown=move |e: web_sys::PointerEvent| {
                if let Some(t) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) { let _ = t.set_pointer_capture(e.pointer_id()); }
                held.set(share(&e));
            }
            on:pointermove=move |e: web_sys::PointerEvent| { if held.get_untracked().is_some() { held.set(share(&e)); } }
            on:pointerup=move |e: web_sys::PointerEvent| {
                if let (Some(f), d) = (share(&e).or(held.get_untracked()), duration.get_untracked()) { if d > 0.0 { on_seek.run(f * d); } }
                held.set(None);
            }
            on:pointercancel=move |_| held.set(None)
        >
            <div class="absolute inset-x-0 top-1/2 h-1 -translate-y-1/2 bg-ink/40">
                <div class="h-full bg-brand" style=move || format!("width:{:.3}%", fraction() * 100.0)></div>
            </div>
            <div class="absolute top-1/2 h-[22px] w-[22px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-brand" style=move || format!("left:{:.3}%", fraction() * 100.0)></div>
        </div>
    }
}
