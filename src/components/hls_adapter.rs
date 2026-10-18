//! HLS.js WebAssembly Adapter
use wasm_bindgen::prelude::*;
use web_sys::HtmlVideoElement;

#[wasm_bindgen(module = "/interop/dist/index.js")]
extern "C" {
    #[wasm_bindgen(js_name = hlsAttach)]
    pub fn hls_attach(video_id: &str, stream_url: &str) -> String;

    #[wasm_bindgen(js_name = hlsDestroy)]
    pub fn hls_destroy(handle: &str);

    #[wasm_bindgen(js_name = hlsSetAudioTrack)]
    pub fn hls_set_audio_track(handle: &str, track_index: i32);

    #[wasm_bindgen(js_name = hlsAudioTracks)]
    pub fn hls_audio_tracks(handle: &str) -> String;

    #[wasm_bindgen(js_name = hlsGetAudioTrack)]
    pub fn hls_get_audio_track(handle: &str) -> i32;

    #[wasm_bindgen(js_name = hlsSetQuality)]
    pub fn hls_set_quality(handle: &str, level: i32);

    #[wasm_bindgen(js_name = hlsSeek)]
    pub fn hls_seek(handle: &str, seconds: f64);

    #[wasm_bindgen(js_name = hlsCurrentTime)]
    pub fn hls_current_time(handle: &str) -> f64;

    #[wasm_bindgen(js_name = hlsDuration)]
    pub fn hls_duration(handle: &str) -> f64;
}

pub struct HlsPlayer {
    handle: String,
    pub video_element: HtmlVideoElement,
}

impl HlsPlayer {
    pub fn attach(video_element: HtmlVideoElement, stream_url: &str) -> Self {
        let video_id = video_element
            .get_attribute("id")
            .unwrap_or_else(|| crate::keys::VIDEO_ELEMENT_ID.to_string());
        let handle = hls_attach(&video_id, stream_url);
        Self { handle, video_element }
    }

    pub fn play(&self) {
        let _ = self.video_element.play();
    }

    pub fn pause(&self) {
        let _ = self.video_element.pause();
    }

    pub fn seek(&self, seconds: f64) {
        hls_seek(&self.handle, seconds);
    }

    pub fn set_audio_track(&self, index: i32) {
        hls_set_audio_track(&self.handle, index);
    }

    /// The audio tracks of the stream as (index, label); empty when the stream has fewer than two.
    pub fn audio_tracks(&self) -> Vec<(i32, String)> {
        #[derive(serde::Deserialize)]
        struct Track { i: i32, name: String, lang: String }
        serde_json::from_str::<Vec<Track>>(&hls_audio_tracks(&self.handle)).unwrap_or_default().into_iter()
            .map(|t| (t.i, if t.name.is_empty() { t.lang } else { t.name })).collect()
    }

    pub fn audio_track(&self) -> i32 {
        hls_get_audio_track(&self.handle)
    }

    pub fn set_quality(&self, level: i32) {
        hls_set_quality(&self.handle, level);
    }

    pub fn current_time(&self) -> f64 {
        hls_current_time(&self.handle)
    }

    pub fn duration(&self) -> f64 {
        hls_duration(&self.handle)
    }
}

impl Drop for HlsPlayer {
    fn drop(&mut self) {
        hls_destroy(&self.handle);
    }
}
