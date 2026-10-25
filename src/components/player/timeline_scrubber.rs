use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/interop/dist/streams-interop.js")]
extern "C" {
    /// A picture source for the timeline (interop/preview.ts): a hidden copy of the stream at its lowest quality
    #[wasm_bindgen(js_name = previewOpen)]
    fn preview_open(url: &str, is_hls: bool) -> String;
    /// Draws the picture of a moment into the canvas with this id (hidden until it is there)
    #[wasm_bindgen(js_name = previewShow)]
    fn preview_show(handle: &str, seconds: f64, canvas_id: &str);
    #[wasm_bindgen(js_name = previewClose)]
    fn preview_close(handle: &str);
}

/// The canvas the timeline's picture is drawn into (one player at a time).
const PREVIEW_CANVAS: &str = "timeline-preview";

/// The time under the pointer as Netflix writes it in its pill: `8:43`, `1:02:10` (note 126).
fn format_time(seconds: f64) -> String {
    if seconds.is_nan() || seconds < 0.0 {
        return "0:00".to_string();
    }
    let total_secs = seconds.floor() as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}

/// The website's timeline, as on Netflix (note 126): a 3 px track that grows to 7 px while the pointer is on it, a 14 px knob, and over
/// the pointer the picture of that moment with the time below it in a round pill (the pill alone until the picture is there, or when the
/// stream gives none).
#[component]
pub fn TimelineScrubber(
    current_time: ReadSignal<f64>,
    duration: ReadSignal<f64>,
    buffered_fraction: ReadSignal<f64>,
    on_seek: Callback<f64>,
    /// the stream playing (none for an embedded player): where the pictures come from
    stream_url: Signal<Option<String>>,
    #[prop(default = Signal::derive(|| false))]
    is_hls: Signal<bool>,
) -> impl IntoView {
    use wasm_bindgen::JsCast;
    let scrubber_ref = NodeRef::<leptos::html::Div>::new();
    let (is_hovering, set_is_hovering) = signal(false);
    let (hover_pct, set_hover_pct) = signal(0.0);
    let (hover_time_str, set_hover_time_str) = signal("0:00".to_string());
    // where the pointer holds the knob while dragging, as a share of the bar. The video seeks once, when the button is let go, as
    // Netflix's player and the phone's bar: a seek on every move made hls.js drop and fetch segments again dozens of times a second, and
    // playback stuttered after a long drag.
    let (held, set_held) = signal(None::<f64>);
    // the picture source, opened at the first hover (no data is spent for a viewer who never points at the bar), and the stream it is for
    let preview = StoredValue::new(None::<(String, String)>);
    let close_preview = move || {
        if let Some((handle, _)) = preview.get_value() {
            preview_close(&handle);
        }
        preview.set_value(None);
    };
    // another source plays: its pictures come from its own stream
    Effect::new(move |_| {
        stream_url.track();
        close_preview();
    });
    on_cleanup(close_preview);
    let has_pictures = move || stream_url.get().is_some();

    let progress_pct = move || {
        if let Some(f) = held.get() {
            return f * 100.0;
        }
        let d = duration.get();
        if d <= 0.0 {
            0.0
        } else {
            ((current_time.get() / d) * 100.0).clamp(0.0, 100.0)
        }
    };

    let buffer_pct = move || {
        (buffered_fraction.get() * 100.0).clamp(0.0, 100.0)
    };

    let fraction_at = move |e: &web_sys::PointerEvent| -> Option<f64> {
        let rect = scrubber_ref.get()?.get_bounding_client_rect();
        (rect.width() > 0.0).then(|| ((e.client_x() as f64 - rect.left()) / rect.width()).clamp(0.0, 1.0))
    };

    let on_pointer_move = move |e: web_sys::PointerEvent| {
        let is_fine_pointer = e.pointer_type() == "mouse" || e.pointer_type() == "pen";
        if let Some(fraction) = fraction_at(&e) {
            let at = fraction * duration.get();
            set_hover_pct.set(fraction * 100.0);
            set_hover_time_str.set(format_time(at));
            if is_fine_pointer {
                set_is_hovering.set(true);
            }
            if held.get_untracked().is_some() {
                set_held.set(Some(fraction));
            }
            if is_fine_pointer {
                if let Some(url) = stream_url.get_untracked().filter(|_| at.is_finite() && at > 0.0) {
                    if preview.with_value(|p| p.as_ref().map(|(_, u)| u != &url).unwrap_or(true)) {
                        close_preview();
                        let handle = preview_open(&url, is_hls.get_untracked());
                        preview.set_value((!handle.is_empty()).then(|| (handle, url)));
                    }
                    if let Some((handle, _)) = preview.get_value() {
                        preview_show(&handle, at, PREVIEW_CANVAS);
                    }
                }
            }
        }
    };

    let on_pointer_down = move |e: web_sys::PointerEvent| {
        if let Some(t) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let _ = t.set_pointer_capture(e.pointer_id());
        }
        set_held.set(fraction_at(&e));
    };

    let on_pointer_up = move |e: web_sys::PointerEvent| {
        if let Some(fraction) = fraction_at(&e).or(held.get_untracked()) {
            if held.get_untracked().is_some() {
                on_seek.run(fraction * duration.get_untracked());
            }
        }
        set_held.set(None);
    };

    // the picture is 245 x 138 and the pill 55 wide: the pair keeps inside the bar at its ends
    let place = move || {
        let half = if has_pictures() { "122.5px" } else { "28px" };
        format!("left: clamp({half}, {}%, calc(100% - {half}));", hover_pct.get())
    };

    // a 23 px high area to click (data-player-control: a click here is not a play or pause of the picture)
    view! {
        <div
            node_ref=scrubber_ref
            data-player-control=""
            on:pointermove=on_pointer_move
            on:pointerleave=move |_| set_is_hovering.set(false)
            on:pointerdown=on_pointer_down
            on:pointerup=on_pointer_up
            on:pointercancel=move |_| set_held.set(None)
            class="group/bar relative w-full h-[23px] flex items-center cursor-pointer select-none touch-none"
        >
            // Track
            <div class="relative w-full h-[3px] bg-mute pointer-events-none transition-[height] duration-(--motion-quick) group-hover/bar:h-[7px]">
                // Buffered range
                <div class="absolute top-0 bottom-0 left-0 bg-ink-mist" style=move || format!("width: {}%;", buffer_pct()) />
                // Played part
                <div class="absolute top-0 bottom-0 left-0 bg-brand" style=move || format!("width: {}%;", progress_pct()) />
            </div>

            // Knob
            <div
                class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 size-[14px] bg-brand rounded-full pointer-events-none"
                style=move || format!("left: {}%;", progress_pct())
            />

            // the picture of the moment under the pointer, and its time in a pill, 10 px apart and 10 px above the bar
            <div
                class=move || format!(
                    "pointer-events-none absolute bottom-full mb-2.5 flex -translate-x-1/2 flex-col items-center gap-2.5 {}",
                    if is_hovering.get() && duration.get() > 0.0 { "" } else { "hidden" }
                )
                style=place
            >
                <Show when=has_pictures>
                    <canvas id=PREVIEW_CANVAS width="256" height="144" class="invisible w-[245px] aspect-video rounded-card bg-void shadow-lg" />
                </Show>
                <span class="flex h-7 items-center rounded-button bg-field px-3.5 text-body-sm font-medium text-ink tabular-nums">
                    {move || hover_time_str.get()}
                </span>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::format_time;

    #[test]
    fn times_as_netflix_writes_them() {
        assert_eq!(format_time(523.4), "8:43");
        assert_eq!(format_time(5.0), "0:05");
        assert_eq!(format_time(3730.0), "1:02:10");
        assert_eq!(format_time(f64::NAN), "0:00");
    }
}
