use leptos::prelude::*;
use crate::components::hls_adapter::HlsPlayer;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[component]
pub fn VideoPlayer(
    title: String,
    stream_url: String,
    on_close: Option<Callback<()>>,
) -> impl IntoView {
    let video_ref = NodeRef::<leptos::html::Video>::new();
    let (is_playing, set_is_playing) = signal(true);
    let (is_muted, set_is_muted) = signal(false);
    let (is_idle, set_is_idle) = signal(false);
    let hls_player = RwSignal::new_local(None::<std::rc::Rc<HlsPlayer>>);

    let (progress, set_progress) = signal(0.0);
    let (duration, set_duration) = signal(0.0);

    // Timeout for idle mouse
    let _timeout_handle = RwSignal::new_local(None::<leptos::wasm_bindgen::closure::Closure<dyn FnMut()>>);

    let reset_idle = move |_| {
        set_is_idle.set(false);
        // We could implement an actual setTimeout here, but for brevity in WASM
        // we omit the full Web API binding for setTimeout in this snippet.
        // A full implementation would use window().set_timeout_with_callback_and_timeout_and_arguments
    };

    Effect::new({
        let stream_url = stream_url.clone();
        move |_| {
            if let Some(video) = video_ref.get() {
                let player = HlsPlayer::attach(video.clone(), &stream_url);
                hls_player.set(Some(std::rc::Rc::new(player)));
            }
        }
    });

    let container_ref = NodeRef::<leptos::html::Div>::new();
    let progress_bar_ref = NodeRef::<leptos::html::Div>::new();

    let do_toggle_play = move || {
        if let Some(player) = hls_player.get() {
            if is_playing.get() {
                player.pause();
                set_is_playing.set(false);
            } else {
                player.play();
                set_is_playing.set(true);
            }
        }
    };

    let do_toggle_mute = move || {
        if let Some(video) = video_ref.get() {
            let m = !video.muted();
            video.set_muted(m);
            set_is_muted.set(m);
        }
    };

    let do_skip_backward = move || {
        if let Some(player) = hls_player.get() {
            let target = (player.current_time() - 10.0).max(0.0);
            player.seek(target);
            set_progress.set(target);
        }
    };

    let do_skip_forward = move || {
        if let Some(player) = hls_player.get() {
            let target = (player.current_time() + 10.0).min(player.duration());
            player.seek(target);
            set_progress.set(target);
        }
    };

    let do_toggle_fullscreen = move || {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if doc.fullscreen_element().is_some() {
                let _ = doc.exit_fullscreen();
            } else if let Some(container) = container_ref.get() {
                let _ = container.request_fullscreen();
            }
        }
    };

    let on_time_update = move |_| {
        if let Some(player) = hls_player.get() {
            set_progress.set(player.current_time());
            set_duration.set(player.duration());
        }
    };

    let seek = move |e: leptos::ev::MouseEvent| {
        if let (Some(bar), Some(player)) = (progress_bar_ref.get(), hls_player.get()) {
            let rect = bar.get_bounding_client_rect();
            let click_x = (e.client_x() as f64 - rect.left()).max(0.0);
            let width = rect.width();
            if width > 0.0 {
                let ratio = (click_x / width).clamp(0.0, 1.0);
                let target = ratio * player.duration();
                player.seek(target);
                set_progress.set(target);
            }
        }
    };

    let close_player = move |_| {
        if let Some(cb) = on_close {
            cb.run(());
        }
    };

    // Keyboard shortcuts
    crate::utils::listen::listen_window(leptos::ev::keydown, move |e: web_sys::KeyboardEvent| {
        match e.key().as_str() {
            " " | "k" | "K" => {
                e.prevent_default();
                do_toggle_play();
            }
            "j" | "J" | "ArrowLeft" => {
                e.prevent_default();
                do_skip_backward();
            }
            "l" | "L" | "ArrowRight" => {
                e.prevent_default();
                do_skip_forward();
            }
            "m" | "M" => {
                e.prevent_default();
                do_toggle_mute();
            }
            "f" | "F" => {
                e.prevent_default();
                do_toggle_fullscreen();
            }
            _ => {}
        }
    });

    view! {
        <div
            node_ref=container_ref
            class="fixed inset-0 bg-void z-(--z-modal) flex flex-col group"
            on:mousemove=reset_idle
        >
            <video
                node_ref=video_ref
                id=crate::keys::PLAYER_CONTAINER_ID
                class="w-full h-full object-contain cursor-default"
                controls=false
                autoplay=true
                on:timeupdate=on_time_update
                on:play=move |_| set_is_playing.set(true)
                on:pause=move |_| set_is_playing.set(false)
                on:click=move |_| do_toggle_play()
            ></video>
            
            <div 
                class="absolute inset-0 flex flex-col justify-between p-8 transition-opacity duration-(--motion-slower) bg-gradient-to-t from-void/80 via-transparent to-void/60 pointer-events-none"
                class=("opacity-0", move || is_idle.get())
                class=("opacity-100", move || !is_idle.get())
                class=("group-hover:opacity-100", move || true)
            >
                <div class="flex justify-between items-start pointer-events-auto">
                    <button on:click=close_player class="text-ink text-3xl hover:scale-110 transition-transform drop-shadow-md">
                        <AppIcon glyph=Glyph::Back />
                    </button>
                    <div class="text-ink text-xl font-bold drop-shadow-md">{title}</div>
                    <button class="text-ink text-3xl hover:scale-110 transition-transform drop-shadow-md">
                        <AppIcon glyph=Glyph::Report />
                    </button>
                </div>
                
                <div class="flex flex-col gap-6 pointer-events-auto pb-4 px-4">
                    <div
                        node_ref=progress_bar_ref
                        class="w-full h-1.5 bg-line-strong rounded-full cursor-pointer relative group/progress"
                        on:click=seek
                    >
                        <div 
                            class="h-full bg-brand rounded-full transition-all duration-(--motion-quick)"
                            style=move || format!("width: {}%", if duration.get() > 0.0 { (progress.get() / duration.get()) * 100.0 } else { 0.0 })
                        ></div>
                        <div 
                            class="absolute top-1/2 w-4 h-4 bg-brand rounded-full transform -translate-y-1/2 -translate-x-1/2 scale-0 group-hover/progress:scale-100 transition-transform duration-(--motion-base)"
                            style=move || format!("left: {}%", if duration.get() > 0.0 { (progress.get() / duration.get()) * 100.0 } else { 0.0 })
                        ></div>
                    </div>
                    
                    <div class="flex justify-between items-center text-ink">
                        <div class="flex items-center gap-8">
                            <button on:click=move |_| do_toggle_play() class="text-display hover:scale-110 transition-transform">
                                {move || if is_playing.get() { view!{<i class="ph-fill ph-pause"></i>}.into_any() } else { view!{<AppIcon glyph=Glyph::Play />}.into_any() }}
                            </button>
                            <button on:click=move |_| do_skip_backward() class="text-3xl hover:scale-110 transition-transform" title=crate::i18n::t("video-player-rewind-10s")>
                                <i class="ph-bold ph-clock-counter-clockwise"></i>
                            </button>
                            <button on:click=move |_| do_skip_forward() class="text-3xl hover:scale-110 transition-transform" title=crate::i18n::t("video-player-forward-10s")>
                                <i class="ph-bold ph-clock-clockwise"></i>
                            </button>
                            <button on:click=move |_| do_toggle_mute() class="text-3xl hover:scale-110 transition-transform">
                                {move || if is_muted.get() { view!{<AppIcon glyph=Glyph::Muted />} } else { view!{<AppIcon glyph=Glyph::Speaker />} }}
                            </button>
                        </div>
                        
                        <div class="flex items-center gap-8 text-3xl">
                            <span class="text-sm font-medium font-mono">
                                {move || {
                                    let p = progress.get() as u32;
                                    let d = duration.get() as u32;
                                    format!("{:02}:{:02} / {:02}:{:02}", p / 60, p % 60, d / 60, d % 60)
                                }}
                            </span>
                            <button class="hover:scale-110 transition-transform"><i class="ph-bold ph-closed-captioning"></i></button>
                            <button on:click=move |_| do_toggle_fullscreen() class="hover:scale-110 transition-transform" title=crate::i18n::t("video-player-fullscreen")>
                                <AppIcon glyph=Glyph::FullScreen />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
