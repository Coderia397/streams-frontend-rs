use leptos::prelude::*;

fn format_time(seconds: f64) -> String {
    if seconds.is_nan() || seconds < 0.0 {
        return "00:00".to_string();
    }
    let total_secs = seconds.floor() as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

#[component]
pub fn TimelineScrubber(
    current_time: ReadSignal<f64>,
    duration: ReadSignal<f64>,
    buffered_fraction: ReadSignal<f64>,
    on_seek: Callback<f64>,
) -> impl IntoView {
    use wasm_bindgen::JsCast;
    let scrubber_ref = NodeRef::<leptos::html::Div>::new();
    let (is_hovering, set_is_hovering) = signal(false);
    let (hover_pct, set_hover_pct) = signal(0.0);
    let (hover_time_str, set_hover_time_str) = signal("00:00".to_string());
    // where the pointer holds the knob while dragging, as a share of the bar. The video seeks once, when the button is let go, as
    // Netflix's player and the phone's bar: a seek on every move made hls.js drop and fetch segments again dozens of times a second, and
    // playback stuttered after a long drag.
    let (held, set_held) = signal(None::<f64>);

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
        if let Some(fraction) = fraction_at(&e) {
            set_hover_pct.set(fraction * 100.0);
            set_hover_time_str.set(format_time(fraction * duration.get()));
            set_is_hovering.set(true);
            if held.get_untracked().is_some() {
                set_held.set(Some(fraction));
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

    // as in the app's player: a 23 px high area to click, a 4 px track in grey, the played part and a 15 px knob in the accent colour (the knob is always there)
    view! {
        <div
            node_ref=scrubber_ref
            on:pointermove=on_pointer_move
            on:pointerleave=move |_| set_is_hovering.set(false)
            on:pointerdown=on_pointer_down
            on:pointerup=on_pointer_up
            on:pointercancel=move |_| set_held.set(None)
            class="relative w-full h-[23px] flex items-center cursor-pointer select-none touch-none"
        >
            // Track
            <div class="relative w-full h-[4px] bg-mute pointer-events-none">
                // Buffered range
                <div class="absolute top-0 bottom-0 left-0 bg-ink-mist" style=move || format!("width: {}%;", buffer_pct()) />
                // Played part
                <div class="absolute top-0 bottom-0 left-0 bg-brand" style=move || format!("width: {}%;", progress_pct()) />
            </div>

            // Knob
            <div
                class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-[15px] h-[15px] bg-brand rounded-full pointer-events-none"
                style=move || format!("left: {}%;", progress_pct())
            />

            // Time under the pointer
            {move || {
                if is_hovering.get() && duration.get() > 0.0 {
                    view! {
                        <div
                            class="absolute bottom-8 px-2 py-1 bg-field text-ink text-body-sm font-medium rounded-control pointer-events-none -translate-x-1/2"
                            style=move || format!("left: {}%;", hover_pct.get())
                        >
                            {hover_time_str.get()}
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}
