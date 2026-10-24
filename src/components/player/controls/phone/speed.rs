//! The speed slider of the phone's player, as Netflix's Android app (frames on a Pixel 8, 2026-10-24, note 116): the controls give way
//! to one thick track at the bottom (from 176 px to 812 px across, 16 px high, its middle 87 px above the bottom), white up to the speed
//! in use and grey after it, a white upright knob with a gap either side; the five speeds under it, the one in use in bold and 1x with
//! "Normal" under it. A tap on a speed (or a drag along the track) sets it; a tap anywhere else closes the slider.
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// The speeds, as Netflix's.
pub const SPEEDS: [f64; 5] = [0.5, 0.75, 1.0, 1.25, 1.5];

/// `1.25x`
pub fn speed_label(v: f64) -> String {
    format!("{v}x")
}

#[component]
pub fn SpeedSlider(open: RwSignal<bool>, speed: ReadSignal<f64>, on_set_speed: Callback<f64>) -> impl IntoView {
    // where the speed in use sits along the track (0 to 1)
    let place = move || {
        let v = speed.get();
        SPEEDS.iter().position(|s| (s - v).abs() < 0.01).map(|i| i as f64 / (SPEEDS.len() - 1) as f64).unwrap_or(0.5)
    };
    let pick_at = move |e: &web_sys::PointerEvent| {
        let Some(el) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let r = el.get_bounding_client_rect();
        if r.width() > 0.0 {
            let f = ((e.client_x() as f64 - r.left()) / r.width()).clamp(0.0, 1.0);
            let i = (f * (SPEEDS.len() - 1) as f64).round() as usize;
            on_set_speed.run(SPEEDS[i]);
        }
    };
    view! {
        <Show when=move || open.get()>
            <div class="absolute inset-0 z-50" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); open.set(false); }>
                // a soft shade under the slider, so it reads over a bright picture
                <div class="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-linear-to-t from-void/80 to-transparent"></div>
                <div class="absolute bottom-[82px] left-[176px] right-[125px] h-4 touch-none" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()
                    on:pointerdown=move |e: web_sys::PointerEvent| {
                        if let Some(t) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) { let _ = t.set_pointer_capture(e.pointer_id()); }
                        pick_at(&e);
                    }
                    on:pointermove=move |e: web_sys::PointerEvent| { if e.buttons() > 0 { pick_at(&e) } }
                >
                    // two pieces either side of the knob (white before it, grey after), round at the ends of the track and nearly square
                    // at the knob, 6 px from it; the knob a white upright bar 3.5 x 43 px; a dot at the last stop (Netflix's, zoomed frames)
                    <div class="absolute inset-y-0 left-0 rounded-s-full rounded-e-[2px] bg-ink" style=move || format!("width:max(0px, calc({:.3} * (100% - 16px) + 8px - 8px))", place())></div>
                    <div class="absolute inset-y-0 right-0 rounded-e-full rounded-s-[2px] bg-speed-track" style=move || format!("width:max(0px, calc({:.3} * (100% - 16px) + 8px - 8px))", 1.0 - place())></div>
                    <div class="absolute right-[6px] top-1/2 h-1 w-1 -translate-y-1/2 rounded-full bg-ink" class=("hidden", move || place() >= 0.999)></div>
                    <div class="absolute top-1/2 h-[43px] w-[3.5px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-ink" style=move || format!("left:calc({:.3} * (100% - 16px) + 8px)", place())></div>
                </div>
                <div class="absolute bottom-[16px] left-[176px] right-[125px] h-[52px]" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()>
                    {SPEEDS.into_iter().enumerate().map(|(i, v)| {
                        let at = i as f64 / (SPEEDS.len() - 1) as f64;
                        let chosen = move || (speed.get() - v).abs() < 0.01;
                        view! {
                            <button type="button" on:click=move |_| on_set_speed.run(v) class="absolute top-0 flex -translate-x-1/2 flex-col items-center px-2 text-body-md text-ink cursor-pointer" style=format!("left:calc({at:.3} * (100% - 16px) + 8px)")>
                                <span class=move || if chosen() { "font-bold" } else { "" }>{speed_label(v)}</span>
                                {((v - 1.0).abs() < 0.01).then(|| view! { <span class="font-bold">{crate::i18n::t("player-speed-normal")}</span> })}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
        </Show>
    }
}
