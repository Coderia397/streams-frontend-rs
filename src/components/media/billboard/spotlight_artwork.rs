use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use crate::components::trailer_player::TrailerPlayer;
use crate::store::use_ui_store;

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
    let paused = RwSignal::new(false);
    let seek = RwSignal::new(None::<f64>);

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
            paused.set(false);
        }
        key
    });

    let me_for_active = me.clone();
    let active = Signal::derive(move || visible.get() && ui_store.spotlight_playing.get().as_deref() == Some(me_for_active.as_str()));
    let playing = Signal::derive(move || active.get() && !paused.get());
    let has_trailer = move || video_key.get().is_some();

    view! {
        <div node_ref=root class="relative w-full aspect-video overflow-hidden bg-void shrink-0">
            {(!backdrop_src.is_empty()).then(|| view! {
                <img src=backdrop_src.clone() alt=title.clone() class=move || format!("absolute inset-0 w-full h-full object-cover transition-opacity duration-(--motion-slow) {}", if ready.get() && playing.get() { "opacity-0" } else { "opacity-100" }) loading="lazy" />
            })}
            {move || video_key.get().map(|_| view! {
                <TrailerPlayer
                    video_key=Signal::derive(move || video_key.get())
                    is_playing=playing
                    is_muted=ui_store.preview_muted
                    seek_time=Signal::derive(move || seek.get())
                    variant="card".to_string()
                    crop_factor=1.12f32
                    iframe_id=iframe_id.clone()
                    on_ready=Callback::new(move |_| set_ready.set(true))
                    on_ended=Callback::new(move |_| { paused.set(true); })
                    on_unplayable=Callback::new(|_| {})
                    on_time_update=Callback::new(move |t| set_time.set(t))
                    on_duration=Callback::new(move |d| set_duration.set(d))
                />
            })}

            // the app's controls (tap: pause, scrubber, time, sound); without a trailer the picture stays still
            <Show when=has_trailer>
                <crate::components::trailer_controls::TrailerControls
                    time=Signal::derive(move || time.get())
                    duration=Signal::derive(move || duration.get())
                    paused=paused
                    seek=seek
                    on_scrub=Callback::new(move |t| set_time.set(t))
                />
            </Show>
        </div>
    }
}
