use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use crate::components::trailer_player::TrailerPlayer;
use crate::store::use_ui_store;

fn mmss(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

/// The picture area of a card on the New & Hot page: the artwork, and over it the trailer, which starts by itself (muted) while the card is mostly on screen and stops when another
/// card takes over or the card scrolls away. A red line under it shows how far the trailer is, the round button at the bottom right mutes it, and a tap shows the pause button, the
/// scrubber and the time left for three seconds. A title without a trailer shows a round play button that opens its details.
#[component]
pub fn SpotlightArtwork(movie_id: u32, is_tv: bool, #[prop(into)] backdrop_src: String, #[prop(into)] title: String) -> impl IntoView {
    let ui_store = use_ui_store();
    let me = format!("{}-{}", if is_tv { "tv" } else { "movie" }, movie_id);
    let iframe_id = format!("spot-{me}");
    let root = NodeRef::<leptos::html::Div>::new();
    let (visible, set_visible) = signal(false);
    let (videos, set_videos) = signal(Vec::<crate::services::tmdb::VideoResult>::new());
    // the best trailer YouTube has not refused here, chosen again when it refuses the one playing (services/unplayable.rs)
    let video_key = Memo::new(move |_| {
        crate::services::unplayable::track();
        crate::services::tmdb::select_best_tmdb_trailer(&videos.get()).map(|(k, _)| k)
    });
    let (looked_up, set_looked_up) = signal(false);
    let (ready, set_ready) = signal(false);
    let (time, set_time) = signal(0.0f64);
    let (duration, set_duration) = signal(0.0f64);
    let (paused, set_paused) = signal(false);
    let (overlay, set_overlay) = signal(false);
    let (seek, set_seek) = signal(None::<f64>);
    let overlay_timer = StoredValue::new_local(None::<i32>);

    // the card is "visible" when 60 percent of it is on screen
    let me_for_observer = me.clone();
    Effect::new(move |_| {
        let Some(el) = root.get() else { return };
        let me = me_for_observer.clone();
        let cb = Closure::<dyn Fn(js_sys::Array)>::wrap(Box::new(move |entries: js_sys::Array| {
            for e in entries.iter() {
                if let Ok(entry) = e.dyn_into::<web_sys::IntersectionObserverEntry>() {
                    let on = entry.is_intersecting() && entry.intersection_ratio() >= 0.6;
                    set_visible.try_set(on);
                    if on && !ui_store.spotlight_hold.get_untracked() { ui_store.spotlight_playing.set(Some(me.clone())); }
                }
            }
        }));
        let init = web_sys::IntersectionObserverInit::new();
        init.set_threshold(&js_sys::Array::of2(&JsValue::from_f64(0.0), &JsValue::from_f64(0.6)));
        if let Ok(obs) = web_sys::IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &init) {
            obs.observe(&el);
            // the observer and its callback go with the card (they ran on for good before, writing to a card that was gone)
            let kept = StoredValue::new_local(Some((obs, cb)));
            on_cleanup(move || {
                kept.try_update_value(|k| {
                    if let Some((obs, _cb)) = k.take() {
                        obs.disconnect();
                    }
                });
            });
        }
    });

    // when the page has arrived at a tab, the card on screen takes the trailer
    let me_for_hold = me.clone();
    Effect::new(move |_| {
        if !ui_store.spotlight_hold.get() && visible.get_untracked() && ui_store.spotlight_playing.get_untracked().is_none() {
            ui_store.spotlight_playing.set(Some(me_for_hold.clone()));
        }
    });

    // the trailer is looked up the first time the card is on screen
    Effect::new(move |_| {
        if visible.get() && !looked_up.get_untracked() {
            set_looked_up.set(true);
            leptos::task::spawn_local(async move {
                if let Ok(found) = crate::services::tmdb::fetch_videos(movie_id, is_tv).await {
                    set_videos.set(found);
                }
            });
        }
    });

    // another trailer starts from the picture, at its own beginning
    Effect::new(move |previous: Option<Option<String>>| {
        let key = video_key.get();
        if previous.is_some_and(|p| p != key) {
            set_ready.set(false);
            set_time.set(0.0);
            set_duration.set(0.0);
            set_paused.set(false);
        }
        key
    });

    let me_for_active = me.clone();
    let active = Signal::derive(move || visible.get() && ui_store.spotlight_playing.get().as_deref() == Some(me_for_active.as_str()));
    let playing = Signal::derive(move || active.get() && !paused.get());
    let has_trailer = move || video_key.get().is_some();

    let show_overlay = move || {
        set_overlay.set(true);
        if let Some(win) = web_sys::window() {
            if let Some(t) = overlay_timer.get_value() { win.clear_timeout_with_handle(t); }
            let cb = Closure::once_into_js(move || set_overlay.set(false));
            overlay_timer.set_value(win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 3000).ok());
        }
    };
    let on_tap = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        if has_trailer() { show_overlay(); }
    };
    let fraction = move || if duration.get() > 0.0 { (time.get() / duration.get()).clamp(0.0, 1.0) } else { 0.0 };
    let on_scrub = move |e: leptos::ev::MouseEvent| {
        e.stop_propagation();
        let Some(target) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let rect = target.get_bounding_client_rect();
        if rect.width() > 0.0 && duration.get_untracked() > 0.0 {
            let f = ((e.client_x() as f64 - rect.left()) / rect.width()).clamp(0.0, 1.0);
            set_seek.set(Some(f * duration.get_untracked()));
            set_time.set(f * duration.get_untracked());
            show_overlay();
        }
    };

    view! {
        <div node_ref=root on:click=on_tap class="relative w-full aspect-video overflow-hidden bg-void shrink-0">
            {(!backdrop_src.is_empty()).then(|| view! {
                <img src=backdrop_src.clone() alt=title.clone() class=move || format!("absolute inset-0 w-full h-full object-cover transition-opacity duration-(--motion-slow) {}", if ready.get() && playing.get() { "opacity-0" } else { "opacity-100" }) loading="lazy" />
            })}
            {move || video_key.get().map(|_| view! {
                <TrailerPlayer
                    video_key=Signal::derive(move || video_key.get())
                    is_playing=playing
                    is_muted=ui_store.preview_muted
                    seek_time=seek
                    variant="card".to_string()
                    crop_factor=1.12f32
                    iframe_id=iframe_id.clone()
                    on_ready=Callback::new(move |_| set_ready.set(true))
                    on_ended=Callback::new(move |_| { set_paused.set(true); })
                    on_unplayable=Callback::new(|_| {})
                    on_time_update=Callback::new(move |t| set_time.set(t))
                    on_duration=Callback::new(move |d| set_duration.set(d))
                />
            })}

            // over the player, so that a tap is ours (the YouTube frame would swallow it)
            <Show when=has_trailer><div class="absolute inset-0 z-10"></div></Show>

            // without a trailer: nothing to play, the picture stays still
            // a tap on a playing card: pause or play in the middle, the time left and the scrubber
            <Show when=move || overlay.get() && has_trailer()>
                <div class="absolute inset-0 z-20 bg-void/25 flex items-center justify-center">
                    <button on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); set_paused.update(|p| *p = !*p); show_overlay(); } aria-label=crate::i18n::t("media-play") class="text-ink">
                        <i class=move || if playing.get() { "ph-fill ph-pause text-icon-xl" } else { "ph-fill ph-play text-icon-xl" }></i>
                    </button>
                    <div class="absolute left-4 right-4 bottom-3 flex items-center gap-3">
                        <div on:click=on_scrub class="relative flex-1 h-9 flex items-center cursor-pointer">
                            <div class="w-full h-[3px] bg-ink/35 rounded-full"></div>
                            <div class="absolute left-0 h-[3px] bg-brand rounded-full" style=move || format!("width:{}%", fraction() * 100.0)></div>
                            <div class="absolute w-[22px] h-[22px] -ml-[11px] rounded-full bg-brand" style=move || format!("left:{}%", fraction() * 100.0)></div>
                        </div>
                        <span class="text-ink text-body-lg tabular-nums w-[58px] text-right">{move || mmss(duration.get() - time.get())}</span>
                    </div>
                </div>
            </Show>

            // the mute button and the red line under the picture
            <Show when=has_trailer>
                <button
                    on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); ui_store.set_preview_muted(!ui_store.preview_muted.get_untracked()); }
                    aria-label=move || if ui_store.preview_muted.get() { crate::i18n::t("media-unmute") } else { crate::i18n::t("media-mute") }
                    class="absolute right-2.5 bottom-2.5 z-30 btn-circle btn-circle-sm"
                >
                    <i class=move || if ui_store.preview_muted.get() { "ph-fill ph-speaker-slash text-body" } else { "ph-fill ph-speaker-high text-body" }></i>
                </button>
                <div class="absolute left-0 bottom-0 z-30 h-[3px] bg-brand" style=move || format!("width:{}%", fraction() * 100.0)></div>
            </Show>
        </div>
    }
}
