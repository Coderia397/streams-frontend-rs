//! The controls over the playing video, one design per form (note 114): the website's (website.rs, Netflix's website player) and the
//! phone's (phone.rs, Netflix's Android app player, note 116). The player (player/mod.rs) owns the video and its state and hands both
//! the same bundle; only one design is mounted.
use leptos::prelude::*;

pub mod phone;
pub mod website;

/// What the controls show and can do, from the player.
#[derive(Clone, Copy)]
pub struct ControlsState {
    pub title: StoredValue<String>,
    /// The website's episode line (`E23  Red John's Footsteps`), empty for a film
    pub subtitle: Signal<String>,
    /// The next episode: number, name, summary and still
    pub next_info: Signal<Option<(u32, String, String, Option<String>)>>,
    /// The season and episode playing, for a series
    pub episode: Option<(u32, u32)>,
    /// The episode's name, once known
    pub episode_name: Signal<Option<String>>,
    pub media_id: u32,
    pub is_tv: bool,
    pub is_playing: ReadSignal<bool>,
    pub is_buffering: ReadSignal<bool>,
    pub is_muted: ReadSignal<bool>,
    pub volume: ReadSignal<f64>,
    pub current_time: ReadSignal<f64>,
    pub duration: ReadSignal<f64>,
    pub buffered_fraction: ReadSignal<f64>,
    /// The website hides its controls after a while without the pointer moving
    pub is_idle: ReadSignal<bool>,
    /// The playback speed (1.0 normal)
    pub speed: ReadSignal<f64>,
    pub on_set_speed: Callback<f64>,
    pub on_toggle_play: Callback<()>,
    pub on_seek: Callback<f64>,
    pub on_skip_by: Callback<f64>,
    pub on_toggle_mute: Callback<()>,
    pub on_set_volume: Callback<f64>,
    pub on_toggle_episodes: Callback<()>,
    pub on_toggle_subtitles: Callback<()>,
    pub on_next_episode: Option<Callback<()>>,
    pub on_toggle_fullscreen: Callback<()>,
    pub on_back: Callback<()>,
    /// The phone's own menus (the website keeps its drawers in player/mod.rs): the subtitles and audio of the stream, the choice made,
    /// and moving to another episode
    pub subtitles: Signal<Vec<crate::services::resolver::SubtitleTrack>>,
    pub selected_sub: ReadSignal<Option<String>>,
    pub audio_tracks: Signal<Vec<(i32, String)>>,
    pub audio_selected: Signal<i32>,
    pub on_select_audio: Callback<i32>,
    pub on_select_sub: Callback<Option<String>>,
    /// Reads the stream's audio tracks again (they come a moment after it starts)
    pub on_refresh_audio: Callback<()>,
    pub on_select_episode: Option<Callback<(u32, u32)>>,
}

/// The time left, as the players show it (`06:55`, `1:02:10`).
pub fn time_left(current: f64, duration: f64) -> String {
    if duration <= 0.0 || current.is_nan() || duration.is_nan() {
        return "00:00".to_string();
    }
    let rem = (duration - current).max(0.0).floor() as u64;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m:02}:{s:02}") }
}

/// The controls of the form in use.
#[component]
pub fn PlayerControls(state: ControlsState) -> impl IntoView {
    view! {
        <crate::form::ByForm
            phone=move || view! { <phone::PhoneControls state=state /> }
            desktop=move || view! { <website::WebsiteControls state=state /> }
        />
    }
}
