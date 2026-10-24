use leptos::prelude::*;
use wasm_bindgen::prelude::*;

fn parse_yt_number(s: &str, key: &str) -> Option<f64> {
    let pattern = format!(r#""{}""#, key);
    let idx = s.find(&pattern)?;
    let after_key = &s[idx + pattern.len()..];
    let colon_idx = after_key.find(':')?;
    let val_str = after_key[colon_idx + 1..].trim_start();
    let end_idx = val_str
        .find(|c: char| !c.is_numeric() && c != '.')
        .unwrap_or(val_str.len());
    val_str[..end_idx].trim().parse::<f64>().ok()
}

/// How far the video's time must have moved before its frame is shown, in seconds (see the time messages).
const REVEAL_AFTER_SECONDS: f64 = 0.35;
/// When YouTube sends no time, how long after "playing" the frame is shown (measured on a Pixel 8: the first picture about 1.4 s
/// after "playing").
const REVEAL_FALLBACK_MS: u64 = 1_500;

fn send_yt_cmd(iframe_id: &str, func: &str) {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.get_element_by_id(iframe_id) {
                if let Ok(iframe) = el.dyn_into::<web_sys::HtmlIFrameElement>() {
                    if let Some(cw) = iframe.content_window() {
                        let cmd = format!(r#"{{"event":"command","func":"{}","args":[]}}"#, func);
                        let _ = cw.post_message(&wasm_bindgen::JsValue::from_str(&cmd), "*");
                    }
                }
            }
        }
    }
}

fn send_yt_seek(iframe_id: &str, seconds: f64) {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.get_element_by_id(iframe_id) {
                if let Ok(iframe) = el.dyn_into::<web_sys::HtmlIFrameElement>() {
                    if let Some(cw) = iframe.content_window() {
                        let cmd = format!(r#"{{"event":"command","func":"seekTo","args":[{},true]}}"#, seconds);
                        let _ = cw.post_message(&wasm_bindgen::JsValue::from_str(&cmd), "*");
                    }
                }
            }
        }
    }
}

fn send_yt_listening(iframe_id: &str) {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.get_element_by_id(iframe_id) {
                if let Ok(iframe) = el.dyn_into::<web_sys::HtmlIFrameElement>() {
                    if let Some(cw) = iframe.content_window() {
                        // the form YouTube's own player API sends (id and channel), which the frame answers with its state and events
                        let cmd = format!(r#"{{"event":"listening","id":"{iframe_id}","channel":"widget"}}"#);
                        let _ = cw.post_message(&wasm_bindgen::JsValue::from_str(&cmd), "*");
                    }
                }
            }
        }
    }
}

/// Detect Safari / WebKit browsers to avoid the GPU compositor 3.5x texture drop bug
fn is_webkit_browser() -> bool {
    if let Some(win) = web_sys::window() {
        if let Ok(ua) = win.navigator().user_agent() {
            let u = ua.to_lowercase();
            let is_apple = (u.contains("safari") && !u.contains("chrome") && !u.contains("android"))
                || u.contains("iphone")
                || u.contains("ipad")
                || u.contains("ipod");
            return is_apple;
        }
    }
    false
}

fn set_timeout_ms<F: FnOnce() + 'static>(cb: F, ms: i32) -> Option<i32> {
    let window = web_sys::window()?;
    let closure = Closure::once_into_js(cb);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms)
        .ok()
}

fn clear_timeout_id(id: Option<i32>) {
    if let Some(id) = id {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(id);
        }
    }
}

/// New & Improved YouTube Trailer Player
/// Implements:
/// 1. Universal Autoplay (always loads with mute=1 to satisfy browser autoplay policies, un-mutes once confirmed playing)
/// 2. Active CUED (5) / UNSTARTED (-1) stall kicking via playVideo postMessage
/// 3. Persistent Iframe Lifecycle (preserves iframe DOM across pause/modal-overlays without expensive re-fetches)
/// 4. Artificial Viewport Scaling (2.2x on WebKit/Safari; 3.5x on Chromium)
/// 5. Smart Intro Cutoff (4s for teasers, 8s for full trailers to skip distributor pre-rolls)
/// 6. Smart Outro Cutoff (pauses/ends when remaining < 7s to prevent YouTube end-screen recommendation cards)
/// 7. Robust postMessage parsing (handles raw JSON string and JavaScript objects)
/// 8. Safety fallback timer so video frame is never stuck invisible at opacity-0
#[component]
pub fn TrailerPlayer(
    #[prop(into)] video_key: Signal<Option<String>>,
    #[prop(default = Signal::derive(|| false), into)] is_teaser: Signal<bool>,
    #[prop(default = Signal::derive(|| true), into)] is_playing: Signal<bool>,
    #[prop(default = Signal::derive(|| true), into)] is_muted: Signal<bool>,
    #[prop(default = Signal::derive(|| 0.0), into)] initial_seek_time: Signal<f64>,
    #[prop(optional, into)] seek_time: Option<Signal<Option<f64>>>,
    #[prop(optional, default = "hero".to_string(), into)] variant: String,
    #[prop(optional, into)] crop_factor: Option<f32>,
    #[prop(optional, into)] on_ready: Option<Callback<()>>,
    #[prop(optional, into)] on_ended: Option<Callback<()>>,
    /// Called instead of `on_ended` when YouTube refuses the video; a caller that picks its trailer again (services/unplayable.rs) keeps
    /// playing with the next one
    #[prop(optional, into)] on_unplayable: Option<Callback<()>>,
    #[prop(optional, into)] on_time_update: Option<Callback<f64>>,
    /// The length of the video in seconds, when the player has told it
    #[prop(optional, into)] on_duration: Option<Callback<f64>>,
    #[prop(optional, default = "trailer-iframe".to_string(), into)] iframe_id: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (is_frame_ready, set_is_frame_ready) = signal(false);
    // YouTube said the video cannot play here (removed, private, not allowed in embeds or in this country): the frame stays hidden, so its
    // error page ("Video unavailable") never shows over the picture
    let (failed, set_failed) = signal(false);
    // a message has come from this player's frame since the video was set: its messages decide when it shows
    let heard = StoredValue::new(false);
    let reveal_timer = StoredValue::new_local(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    let (has_ever_started, set_has_ever_started) = signal(is_playing.get_untracked());
    let (is_tab_hidden, set_is_tab_hidden) = signal(false);
    let (was_playing_before_tab_hidden, set_was_playing_before_tab_hidden) = signal(false);
    let (is_unmounted_by_inactivity, set_is_unmounted_by_inactivity) = signal(false);
    let tab_inactivity_timer = StoredValue::new(None::<i32>);
    let last_tracked_time = StoredValue::new(0.0f64);
    let has_initial_seek_done = StoredValue::new(false);
    let playback_anchor_sec = StoredValue::new(0.0f64);
    let playback_anchor_epoch = StoredValue::new(0.0f64);
    let time_update_timer = StoredValue::new(None::<i32>);
    let on_time_update_stored = StoredValue::new(on_time_update);
    let on_duration_stored = StoredValue::new(on_duration);

    let iframe_id_stored = StoredValue::new(iframe_id);
    // the first time the frame reported for this video (its start), to tell when pictures are moving
    let first_time = StoredValue::new(None::<f64>);

    // ── Artificial Scaling Calculation ───────────────────────────────────────
    let is_webkit = is_webkit_browser();
    let artificial_scale: f32 = if is_webkit { 2.2 } else { 3.5 };
    let zoom = crop_factor.unwrap_or_else(|| match variant.as_str() {
        "hero" => 1.15,
        "modal" => 1.30,
        "card" => 1.35,
        _ => 1.20,
    });
    let net_scale = zoom / artificial_scale;
    let sizing_style = format!(
        "position: absolute; left: 50%; top: 50%; width: {:.1}%; height: {:.1}%; transform: translate(-50%, -50%) scale({:.4}); transform-origin: center center; will-change: transform; border: none;",
        artificial_scale * 115.0,
        artificial_scale * 115.0,
        net_scale
    );

    // ── Track mount & play lifecycle without tearing down the iframe ─────────
    Effect::new(move |_| {
        if is_playing.get() {
            set_has_ever_started.set(true);
        }
    });

    let last_key = StoredValue::new(video_key.get_untracked());
    Effect::new(move |_| {
        let k = video_key.get();
        let prev = last_key.get_value();
        if k != prev {
            last_key.set_value(k.clone());
            set_is_frame_ready.set(false);
            first_time.set_value(None);
            set_failed.set(false);
            heard.set_value(false);
            last_tracked_time.set_value(0.0);
            has_initial_seek_done.set_value(false);
            playback_anchor_sec.set_value(0.0);
            playback_anchor_epoch.set_value(0.0);
            clear_timeout_id(time_update_timer.get_value());
            time_update_timer.set_value(None);
            if !is_playing.get_untracked() {
                set_has_ever_started.set(false);
            }
        }
    });

    // ── Continuous Playback Time Tracking Ticker ─────────────────────────────
    let target_id_for_tick = iframe_id_stored.get_value();
    let start_ticker = move || {
        clear_timeout_id(time_update_timer.get_value());
        let tid = target_id_for_tick.clone();

        fn schedule_next_tick(
            tid: String,
            playback_anchor_sec: StoredValue<f64>,
            playback_anchor_epoch: StoredValue<f64>,
            time_update_timer: StoredValue<Option<i32>>,
            on_time_update_stored: StoredValue<Option<Callback<f64>>>,
            is_playing: Signal<bool>,
            tick_count: StoredValue<u32>,
        ) {
            let timer_id = set_timeout_ms(move || {
                if !is_playing.get_untracked() {
                    time_update_timer.set_value(None);
                    return;
                }
                let now = js_sys::Date::now();
                let epoch = playback_anchor_epoch.get_value();
                let anchor_sec = playback_anchor_sec.get_value();
                if epoch > 0.0 {
                    let elapsed = (now - epoch) / 1000.0;
                    let cur = (anchor_sec + elapsed).max(0.0);
                    if let Some(cb) = on_time_update_stored.get_value() {
                        cb.run(cur);
                    }
                }

                // Periodically ping YouTube player for drift synchronization (~every 2.5s)
                // Avoid flooding postMessages every tick which freezes/buffers Safari's media pipeline!
                let cnt = tick_count.get_value().wrapping_add(1);
                tick_count.set_value(cnt);
                if cnt % 10 == 0 {
                    send_yt_cmd(&tid, "getCurrentTime");
                }

                schedule_next_tick(
                    tid,
                    playback_anchor_sec,
                    playback_anchor_epoch,
                    time_update_timer,
                    on_time_update_stored,
                    is_playing,
                    tick_count,
                );
            }, 250);
            time_update_timer.set_value(timer_id);
        }

        let tick_count = StoredValue::new(0u32);
        schedule_next_tick(
            tid,
            playback_anchor_sec,
            playback_anchor_epoch,
            time_update_timer,
            on_time_update_stored,
            is_playing,
            tick_count,
        );
    };

    // ── Reactive Play / Pause via postMessage & Ticker Control ───────────────
    Effect::new(move |_| {
        let playing = is_playing.get() && !is_tab_hidden.get() && !is_unmounted_by_inactivity.get();
        let target_id = iframe_id_stored.get_value();
        if playing {
            send_yt_cmd(&target_id, "playVideo");
            send_yt_listening(&target_id);

            let epoch = playback_anchor_epoch.get_value();
            if epoch == 0.0 {
                let init_t = initial_seek_time.get_untracked();
                let base_sec = if init_t > 0.5 {
                    init_t
                } else if is_teaser.get_untracked() {
                    4.0
                } else {
                    8.0
                };
                let cur_anchor = playback_anchor_sec.get_value();
                if cur_anchor <= 0.0 {
                    playback_anchor_sec.set_value(base_sec);
                }
                playback_anchor_epoch.set_value(js_sys::Date::now());
            } else {
                playback_anchor_epoch.set_value(js_sys::Date::now());
            }

            start_ticker();
        } else {
            send_yt_cmd(&target_id, "pauseVideo");
            clear_timeout_id(time_update_timer.get_value());
            time_update_timer.set_value(None);

            let now = js_sys::Date::now();
            let epoch = playback_anchor_epoch.get_value();
            if epoch > 0.0 {
                let elapsed = (now - epoch) / 1000.0;
                let paused_at = (playback_anchor_sec.get_value() + elapsed).max(0.0);
                playback_anchor_sec.set_value(paused_at);
                playback_anchor_epoch.set_value(0.0);
                if let Some(cb) = on_time_update_stored.get_value() {
                    cb.run(paused_at);
                }
            }
        }
    });

    // ── Explicit Seek Command (e.g. hand-off resume when modal closes) ────────
    if let Some(seek_sig) = seek_time {
        let tid = iframe_id_stored.get_value();
        Effect::new(move |_| {
            if let Some(t) = seek_sig.get() {
                if t > 0.0 {
                    playback_anchor_sec.set_value(t);
                    playback_anchor_epoch.set_value(js_sys::Date::now());
                    send_yt_seek(&tid, t);
                    if let Some(cb) = on_time_update_stored.get_value() {
                        cb.run(t);
                    }
                }
            }
        });
    }

    // ── Universal Tab Visibility & 15-Second Inactivity Lifecycle (All OS) ───
    // the tab hidden or shown (visibilitychange), and Safari's page cache (pagehide, pageshow); the listeners go with the player
    let on_vis_change = std::rc::Rc::new(move || {
        let is_hidden = web_sys::window().and_then(|w| w.document()).map(|doc| {
            let state = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("visibilityState"))
                .ok()
                .and_then(|v| v.as_string());
            let hidden = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("hidden"))
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            hidden || state.as_deref() == Some("hidden")
        }).unwrap_or(false);

        let target_id = iframe_id_stored.get_value();

        if is_hidden {
            // Tab switched / App backgrounded: PAUSE IMMEDIATELY across all OS!
            let currently_playing = is_playing.get_untracked() && !is_unmounted_by_inactivity.get_untracked();
            if currently_playing {
                set_was_playing_before_tab_hidden.set(true);
            }
            set_is_tab_hidden.set(true);
            send_yt_cmd(&target_id, "pauseVideo");

            // Start 15-second inactivity timer to unmount iframe & free system RAM / GPU
            clear_timeout_id(tab_inactivity_timer.get_value());
            let t = set_timeout_ms(move || {
                set_is_unmounted_by_inactivity.set(true);
                set_has_ever_started.set(false);
                set_is_frame_ready.set(false);
            }, 15_000);
            tab_inactivity_timer.set_value(t);
        } else {
            // Tab / App foregrounded: cancel 15s unmount timer
            clear_timeout_id(tab_inactivity_timer.get_value());
            tab_inactivity_timer.set_value(None);
            set_is_tab_hidden.set(false);

            let was_unmounted = is_unmounted_by_inactivity.get_untracked();
            let should_resume = was_playing_before_tab_hidden.get_untracked() && is_playing.get_untracked();
            set_was_playing_before_tab_hidden.set(false);

            if was_unmounted {
                // Left > 15 seconds: unmounted to free RAM, now re-mount and resume from saved timestamp
                set_is_unmounted_by_inactivity.set(false);
                if should_resume {
                    set_has_ever_started.set(true);
                }
            } else if should_resume {
                // Left < 15 seconds: iframe was kept warm in DOM, resume immediately!
                send_yt_cmd(&target_id, "playVideo");
            }
        }
    });
    {
        let f = on_vis_change.clone();
        crate::utils::listen::listen_document("visibilitychange", move |_| f());
        let f = on_vis_change.clone();
        crate::utils::listen::listen_window(leptos::ev::pagehide, move |_| f());
        let f = on_vis_change;
        crate::utils::listen::listen_window(leptos::ev::pageshow, move |_| f());
    }

    on_cleanup(move || {
        clear_timeout_id(tab_inactivity_timer.get_value());
        clear_timeout_id(time_update_timer.get_value());
    });

    // ── Mute / Unmute command via postMessage ────────────────────────────────
    // NOTE: Unmuting is only permitted once is_frame_ready is true to avoid
    // tripping the browser's autoplay-with-sound gate on fresh/paused players.
    Effect::new(move |_| {
        let muted = is_muted.get();
        let frame_ready = is_frame_ready.get();
        let target_id = iframe_id_stored.get_value();
        if muted {
            send_yt_cmd(&target_id, "mute");
        } else if frame_ready {
            send_yt_cmd(&target_id, "unMute");
        }
    });

    // ── Safety Fallback Timer for Frame Visibility ──────────────────────────
    // When no message at all has come from the frame after 3 s (the browser blocks them), the frame is shown anyway so the video is never
    // stuck behind the picture. Once the frame talks, its messages decide: a video that cannot play sends an error and no time, and stays
    // hidden. The framework's timer, cleared when the video changes or the player goes (the old one ran on after both).
    Effect::new(move |_| {
        let playing = is_playing.get();
        let has_key = video_key.get().is_some();
        reveal_timer.update_value(|t| if let Some(h) = t.take() { h.clear() });
        if playing && has_key {
            let on_ready_cb = on_ready;
            let handle = leptos::leptos_dom::helpers::set_timeout_with_handle(
                move || {
                    if heard.try_get_value() != Some(false) || failed.try_get_untracked() != Some(false) {
                        return;
                    }
                    set_is_frame_ready.try_set(true);
                    if let Some(cb) = on_ready_cb {
                        cb.run(());
                    }
                },
                std::time::Duration::from_millis(3000),
            );
            reveal_timer.set_value(handle.ok());
        } else {
            set_is_frame_ready.set(false);
        }
    });
    on_cleanup(move || reveal_timer.try_update_value(|t| if let Some(h) = t.take() { h.clear() }).unwrap_or(()));

    // ── YouTube iframe event listener (Ready, Playing, Ending, Outro-Cutoff, Time-Update) ─
    // one listener for the player's life (it went on the window for good before, so every player ever shown kept reading every message)
    let on_ready_cb = on_ready;
    let on_ended_cb = on_ended;
    let on_time_update_cb = on_time_update;

    crate::utils::listen::listen_window(leptos::ev::message, move |e: web_sys::MessageEvent| {
        let s: String = if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            txt.into()
        } else if let Ok(json_str) = js_sys::JSON::stringify(&e.data()) {
            json_str.into()
        } else {
            String::new()
        };

        if s.is_empty() {
            return;
        }

        let target_id = iframe_id_stored.get_value();

        // only messages from this player's own frame: another player's, or any once this player's frame has gone, are not ours. Compared as
        // plain values: a frame from another site fails the Window type test (instanceof is false across origins), which made every YouTube
        // message look foreign from 2026-10-23 (ae87281) until this check
        let own: Option<wasm_bindgen::JsValue> = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&target_id))
            .and_then(|el| el.dyn_into::<web_sys::HtmlIFrameElement>().ok())
            .and_then(|f| f.content_window())
            .map(Into::into);
        let Some(own) = own else { return };
        let source: Option<wasm_bindgen::JsValue> = e.source().map(Into::into);
        if source.as_ref() != Some(&own) {
            return;
        }
        heard.set_value(true);

        // The video cannot play here (YouTube's codes 2, 5, 100, 101, 150, 153): hide the frame and end, so the picture stays. Checked
        // first: "info":150 also reads as "info":1 to the loose checks below, which took an error for playback and unmuted it.
        if s.contains(r#""onError""#) {
            set_failed.set(true);
            set_is_frame_ready.set(false);
            if let (Some(code), Some(key)) = (parse_yt_number(&s, "info"), video_key.get_untracked()) {
                if crate::services::unplayable::is_lasting(code as u32) {
                    crate::services::unplayable::mark(&key);
                }
            }
            if let Some(cb) = on_unplayable.or(on_ended_cb) {
                cb.run(());
            }
            return;
        }
        if failed.get_untracked() {
            return;
        }

        // Kick stalled player if stuck in CUED (5), UNSTARTED (-1), or onReady
        let is_stalled_cued = s.contains(r#""playerState":5"#)
            || s.contains(r#""playerState": 5"#)
            || s.contains(r#""info":5"#)
            || s.contains(r#""info": 5"#)
            || s.contains(r#""data":5"#)
            || s.contains(r#""data": 5"#)
            || s.contains(r#""playerState":-1"#)
            || s.contains(r#""playerState": -1"#)
            || s.contains(r#""info":-1"#)
            || s.contains(r#""info": -1"#)
            || s.contains(r#""data":-1"#)
            || s.contains(r#""data": -1"#)
            || s.contains(r#""onReady""#);

        if is_stalled_cued && is_playing.get_untracked() {
            send_yt_cmd(&target_id, "playVideo");
        }

        // Playing confirmed: playerState 1, data 1, info 1
        let is_playing_event = s.contains(r#""info":1"#)
            || s.contains(r#""info": 1"#)
            || s.contains(r#""playerState":1"#)
            || s.contains(r#""playerState": 1"#)
            || s.contains(r#""data":1"#)
            || s.contains(r#""data": 1"#);

        if is_playing_event {
            // the frame is shown once its time has moved (below), not at "playing": YouTube says "playing" up to a second and a half
            // before its first picture is drawn on a phone, which showed as black between the still and the video. YouTube does not
            // always send its time, so the frame is shown REVEAL_FALLBACK_MS after "playing" in any case.
            if !is_frame_ready.get_untracked() {
                leptos::leptos_dom::helpers::set_timeout(
                    move || {
                        if is_playing.try_get_untracked() == Some(true) && is_frame_ready.try_get_untracked() == Some(false) {
                            set_is_frame_ready.try_set(true);
                            if let Some(cb) = on_ready_cb {
                                cb.run(());
                            }
                        }
                    },
                    std::time::Duration::from_millis(REVEAL_FALLBACK_MS),
                );
            }

            // Sound is restored here, now that playback is confirmed active.
            // Unmuting an already-playing element does NOT trip browser autoplay restrictions!
            if !is_muted.get_untracked() {
                send_yt_cmd(&target_id, "unMute");
            }

            // Initial seek performed ONCE upon playback start
            if !has_initial_seek_done.get_value() {
                let init_t = initial_seek_time.get_untracked();
                if init_t > 0.5 {
                    has_initial_seek_done.set_value(true);
                    playback_anchor_sec.set_value(init_t);
                    playback_anchor_epoch.set_value(js_sys::Date::now());
                    send_yt_seek(&target_id, init_t);
                }
            }
        }

        // Normal End: playerState 0, data 0, info 0
        let is_ended_event = s.contains(r#""info":0"#)
            || s.contains(r#""info": 0"#)
            || s.contains(r#""playerState":0"#)
            || s.contains(r#""playerState": 0"#)
            || s.contains(r#""data":0"#)
            || s.contains(r#""data": 0"#);

        if is_ended_event {
            if let Some(cb) = on_ended_cb {
                cb.run(());
            }
        }

        // Live Time Update Reporting (for seamless handoff to modal and back)
        if s.contains(r#""currentTime""#) {
            if let Some(cur) = parse_yt_number(&s, "currentTime") {
                // the frame is shown when its time has moved a little past where it started (pictures are being drawn); the still
                // picture stays until then, as Netflix's cards (a video that cannot play reports 0 and never moves)
                if cur > 0.0 && !is_frame_ready.get_untracked() {
                    match first_time.get_value() {
                        None => first_time.set_value(Some(cur)),
                        Some(start) if cur - start >= REVEAL_AFTER_SECONDS && is_playing.get_untracked() => {
                            set_is_frame_ready.set(true);
                            if let Some(cb) = on_ready_cb {
                                cb.run(());
                            }
                        }
                        _ => {}
                    }
                }
                last_tracked_time.set_value(cur);
                if cur > 0.0 {
                    playback_anchor_sec.set_value(cur);
                    playback_anchor_epoch.set_value(js_sys::Date::now());
                }
                if let Some(cb) = on_time_update_cb {
                    cb.run(cur);
                }
            }
        }

        if let (Some(cb), Some(dur)) = (on_duration_stored.get_value(), parse_yt_number(&s, "duration")) {
            if dur > 0.0 { cb.run(dur); }
        }

        // Smart Outro Cutoff: If remaining < 7.0s, end before YouTube shows recommendation tiles
        if s.contains(r#""currentTime""#) && s.contains(r#""duration""#) {
            if let (Some(cur), Some(dur)) = (parse_yt_number(&s, "currentTime"), parse_yt_number(&s, "duration")) {
                if dur > 25.0 && (dur - cur) < 7.0 && (dur - cur) > 0.0 {
                    if let Some(cb) = on_ended_cb {
                        cb.run(());
                    }
                }
            }
        }
    });

    let extra_class = class.unwrap_or_default();

    view! {
        <div
            class=format!("absolute inset-0 overflow-hidden pointer-events-none transition-opacity duration-(--motion-slowest) ease-in-out {}", extra_class)
            class:opacity-100=move || is_frame_ready.get() && is_playing.get() && !failed.get()
            class:opacity-0=move || !is_frame_ready.get() || !is_playing.get() || failed.get()
        >
            {move || {
                if let Some(key) = video_key.get() {
                    if has_ever_started.get() {
                        let origin = web_sys::window()
                            .and_then(|w| w.location().origin().ok())
                            .unwrap_or_default();

                        // Smart Intro Skip or Seamless Timestamp Continuation
                        let init_t = initial_seek_time.get_untracked();
                        let start_sec = if init_t > 0.5 {
                            init_t.floor() as u32
                        } else if is_teaser.get() {
                            4
                        } else {
                            8
                        };

                        // ALWAYS load with mute=1 so browser autoplay policies unconditionally permit it!
                        // Sound is restored via unMute postMessage once playback is running.
                        let embed_url = format!(
                            "https://www.youtube.com/embed/{}?autoplay=1&mute=1&controls=0&modestbranding=1&rel=0&iv_load_policy=3&playsinline=1&enablejsapi=1&disablekb=1&origin={}&widget_referrer={}&start={}",
                            key,
                            origin,
                            origin,
                            start_sec
                        );

                        let rendered_id = iframe_id_stored.get_value();
                        // Ask the frame for its events once it has loaded, again every 250 ms until it answers (at most 5 s), as YouTube's own API
                        // does: asked only when playback was requested, before the frame had loaded, it never answered, so its errors went unseen
                        // and the reveal timer showed whatever it held, "Video unavailable" included
                        let on_load = {
                            let id = rendered_id.clone();
                            move |_| {
                                fn ask(id: String, heard: StoredValue<bool>, left: u32) {
                                    if heard.try_get_value() != Some(false) || left == 0 {
                                        return;
                                    }
                                    send_yt_listening(&id);
                                    leptos::leptos_dom::helpers::set_timeout(move || ask(id, heard, left - 1), std::time::Duration::from_millis(250));
                                }
                                ask(id.clone(), heard, 20);
                            }
                        };
                        view! {
                            <div class="w-full h-full pointer-events-none flex items-center justify-center overflow-hidden">
                                <iframe
                                    on:load=on_load
                                    id=rendered_id
                                    class="pointer-events-none object-cover"
                                    style=sizing_style.clone()
                                    src=embed_url
                                    allow="autoplay; encrypted-media"
                                    tabindex="-1"
                                />
                            </div>
                        }.into_any()
                    } else {
                        view! { <div /> }.into_any()
                    }
                } else {
                    view! { <div /> }.into_any()
                }
            }}
        </div>
    }
}

