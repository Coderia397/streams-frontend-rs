//! Subtitle appearance: the same six choices as the Netflix app, with a preview that follows them.
use super::prefs::{use_prefs, SubtitleStyle};
use crate::data::settings::{PREVIEW_PAD_X_PX, PREVIEW_PAD_Y_PX, PREVIEW_RADIUS_PX, PREVIEW_WINDOW_PX, SUBTITLE_COLORS, SUBTITLE_EDGES, SUBTITLE_SIZES};
use super::widgets::{Section, SwatchPicker, SwitchRow};
use crate::i18n::t;
use leptos::prelude::*;

pub(crate) fn hex(name: &str) -> &'static str {
    SUBTITLE_COLORS.iter().find(|(n, _)| *n == name).map(|(_, h)| *h).unwrap_or(SUBTITLE_COLORS[0].1)
}

pub(crate) fn size_px(size: u8) -> u32 {
    SUBTITLE_SIZES.iter().find(|(v, _, _)| *v == size).map(|(_, _, px)| *px).unwrap_or(SUBTITLE_SIZES[1].2)
}

pub(crate) fn edge_css(edge: &str, color: &str) -> String {
    let c = hex(color);
    match edge {
        "raised" => format!("1px 1px 0 {c}, 2px 2px 0 {c}"),
        "depressed" => format!("-1px -1px 0 {c}, -2px -2px 0 {c}"),
        "uniform" => format!("0 0 3px {c}, 0 0 3px {c}, 0 0 3px {c}"),
        "shadow" => format!("2px 2px 4px {c}"),
        _ => "none".to_string(),
    }
}

pub(crate) fn paint(name: &str, clear: bool) -> String {
    if name == "none" {
        return "transparent".to_string();
    }
    let h = hex(name);
    let (r, g, b) = (u8::from_str_radix(&h[1..3], 16).unwrap_or(0), u8::from_str_radix(&h[3..5], 16).unwrap_or(0), u8::from_str_radix(&h[5..7], 16).unwrap_or(0));
    format!("rgba({r},{g},{b},{})", if clear { 0.5 } else { 1.0 })
}

/// The style of a subtitle line in the player, the same as the preview above with the size taken from the player's height (`cqh`, the
/// overlay is a size container) so a line is as tall on a phone as on a computer, as on Netflix: small 3.6%, medium 4.6%, large 6%.
pub(crate) fn player_cue_css(s: &SubtitleStyle) -> String {
    let height = match s.size { 0 => 3.6, 2 => 6.0, _ => 4.6 };
    let window = paint(&s.window, s.window_clear);
    format!(
        "font-size:{height}cqh;line-height:1.25;color:{};text-shadow:{};background:{};box-shadow:0 0 0 0.35em {window};border-radius:0.15em;padding:0.05em 0.3em",
        hex(&s.color), edge_css(&s.edge, &s.edge_color), paint(&s.background, s.background_clear)
    )
}

#[component]
fn Pick(label_key: &'static str, value: Signal<String>, options: Vec<(String, String)>, on_pick: Callback<String>, ) -> impl IntoView {
    view! {
        <label class="set-row flex min-h-settings-row w-full items-center justify-between gap-4 border-b border-set-line px-1 py-3 last:border-b-0">
            <span class="text-title-sm font-semibold leading-row">{t(label_key)}</span>
            <select
                on:change=move |ev| on_pick.run(event_target_value(&ev))
                prop:value=move || value.get()
                class="max-w-1/2 border border-ink/20 bg-ink/10 px-3 py-2 text-body-md text-ink focus:border-ink focus:outline-none"
            >
                {options.into_iter().map(|(v, l)| view! { <option value=v class="bg-page text-ink">{l}</option> }).collect_view()}
            </select>
        </label>
    }
}

#[component]
pub fn Subtitles(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let id = StoredValue::new(id);
    let style = Memo::new(move |_| prefs.profile(&id.get_value()).subtitle_style);
    let set = move |apply: Box<dyn FnOnce(&mut SubtitleStyle)>| prefs.update_profile(&id.get_value(), |p| apply(&mut p.subtitle_style));
    let swatches = |none: bool| {
        let mut v: Vec<(String, String, String)> = Vec::new();
        if none {
            v.push(("none".into(), "transparent".into(), t("set-sub-none")));
        }
        v.extend(SUBTITLE_COLORS.iter().map(|(n, h)| (n.to_string(), h.to_string(), t(&format!("set-color-{n}")))));
        v
    };

    view! {
        <div>
            <div class="mb-6 flex h-40 items-center justify-center overflow-hidden rounded-card border border-ink/10 bg-linear-to-b from-ink/10 to-void px-4">
                <span
                    class="text-center font-bold"
                    style=move || {
                        let s = style.get();
                        let window = paint(&s.window, s.window_clear);
                        format!("font-size:{}px;color:{};text-shadow:{};background:{};box-shadow:0 0 0 {}px {window};border-radius:{}px;padding:{}px {}px", size_px(s.size), hex(&s.color), edge_css(&s.edge, &s.edge_color), paint(&s.background, s.background_clear), PREVIEW_WINDOW_PX, PREVIEW_RADIUS_PX, PREVIEW_PAD_Y_PX, PREVIEW_PAD_X_PX)
                    }
                >
                    {t("set-sub-preview")}
                </span>
            </div>
            <Section>
                <div class="set-row flex min-h-settings-row w-full items-center justify-between gap-4 border-b border-set-line px-1 py-3 last:border-b-0">
                    <span class="text-title-sm font-semibold leading-row">{t("set-sub-size")}</span>
                    <div role="radiogroup" class="flex gap-2">
                        {SUBTITLE_SIZES.iter().map(|(n, key, px)| { let (n, key, px) = (*n, *key, *px); view! {
                            <button
                                type="button"
                                role="radio"
                                aria-checked=move || (style.get().size == n).to_string()
                                aria-label=t(key)
                                on:click=move |_| set(Box::new(move |s| s.size = n))
                                class="flex h-11 w-11 items-center justify-center rounded-button border font-bold"
                                class=("border-ink", move || style.get().size == n)
                                class=("bg-ink", move || style.get().size == n)
                                class=("text-void", move || style.get().size == n)
                                class=("border-ink/30", move || style.get().size != n)
                            >
                                <span style=format!("font-size:{}px", px - 6)>"A"</span>
                            </button>
                        } }).collect_view()}
                    </div>
                </div>
                <SwatchPicker label=t("set-sub-color") colors=swatches(false) selected=Signal::derive(move || style.get().color) on_pick=Callback::new(move |v: String| set(Box::new(move |s| s.color = v))) />
                <Pick
                    label_key="set-sub-edge"
                    value=Signal::derive(move || style.get().edge)
                    options=SUBTITLE_EDGES.iter().map(|e| (e.to_string(), t(&format!("set-edge-{e}")))).collect()
                    on_pick=Callback::new(move |v: String| set(Box::new(move |s| s.edge = v)))
                />
                <SwatchPicker label=t("set-sub-edge-color") colors=swatches(false) selected=Signal::derive(move || style.get().edge_color) on_pick=Callback::new(move |v: String| set(Box::new(move |s| s.edge_color = v))) />
                <SwatchPicker label=t("set-sub-background") colors=swatches(true) selected=Signal::derive(move || style.get().background) on_pick=Callback::new(move |v: String| set(Box::new(move |s| s.background = v))) />
                <SwitchRow icon="circle-half" title=t("set-sub-transparent") checked=Signal::derive(move || style.get().background_clear) on_change=Callback::new(move |v| set(Box::new(move |s| s.background_clear = v))) />
                <SwatchPicker label=t("set-sub-window") colors=swatches(true) selected=Signal::derive(move || style.get().window) on_pick=Callback::new(move |v: String| set(Box::new(move |s| s.window = v))) />
                <SwitchRow icon="circle-half" title=t("set-sub-transparent") checked=Signal::derive(move || style.get().window_clear) on_change=Callback::new(move |v| set(Box::new(move |s| s.window_clear = v))) />
            </Section>
            <button type="button" on:click=move |_| set(Box::new(|s| *s = SubtitleStyle::default())) class="rounded-button border border-ink/40 px-6 py-3 text-body-md font-bold hover:bg-ink/10">{t("set-reset")}</button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_become_css() {
        assert_eq!(paint("none", false), "transparent");
        assert_eq!(paint("black", true), "rgba(0,0,0,0.5)");
        assert_eq!(edge_css("none", "black"), "none");
        assert_eq!(size_px(2), 30);
    }
}
