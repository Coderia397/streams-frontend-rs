//! Audio & Subtitles on the phone's player, as Netflix's Android app (view tree and frames on a Pixel 8, 2026-10-24, note 116): the
//! whole screen, two columns, Audio from 36 px in and Subtitles from 508 px, a bold heading 44 px down, rows 58.5 px apart from 112 px
//! down with a check before the choice, the lower rows fading out; the choice is applied with Apply (bottom right, white) or dropped with
//! Cancel (grey). Audio shows when the stream has two or more tracks; subtitles always (Off first).
use leptos::prelude::*;
use crate::services::resolver::SubtitleTrack;

#[component]
pub fn AudioSubtitlesMenu(
    open: RwSignal<bool>,
    subtitles: Signal<Vec<SubtitleTrack>>,
    selected_sub: ReadSignal<Option<String>>,
    audio_tracks: Signal<Vec<(i32, String)>>,
    audio_selected: Signal<i32>,
    on_select_audio: Callback<i32>,
    on_select_sub: Callback<Option<String>>,
) -> impl IntoView {
    // the choice being made, applied only with Apply
    let audio = RwSignal::new(audio_selected.get_untracked());
    let sub = RwSignal::new(selected_sub.get_untracked());
    Effect::new(move |_| {
        if open.get() {
            audio.set(audio_selected.get_untracked());
            sub.set(selected_sub.get_untracked());
        }
    });
    let apply = move |_| {
        if audio.get_untracked() != audio_selected.get_untracked() {
            on_select_audio.run(audio.get_untracked());
        }
        if sub.get_untracked() != selected_sub.get_untracked() {
            on_select_sub.run(sub.get_untracked());
        }
        open.set(false);
    };
    let has_audio = move || audio_tracks.get().len() > 1;
    // the lower rows fade out, as the app's list
    let fade = "mask-image: linear-gradient(to bottom, #000 0, #000 55%, transparent 100%); -webkit-mask-image: linear-gradient(to bottom, #000 0, #000 55%, transparent 100%);";

    view! {
        <Show when=move || open.get()>
            <div class="absolute inset-0 z-50 bg-void text-ink" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()>
                // the columns from the screen's edge, as the app's (Audio at 36 px, Subtitles at 508 px; Subtitles alone at 36 px)
                <Show when=has_audio>
                    <div class="absolute inset-y-0 left-[36px] w-[436px]">
                        <Column title=crate::i18n::t("player-audio") style=fade>
                            {move || audio_tracks.get().into_iter().map(|(i, label)| view! {
                                <Row chosen=Signal::derive(move || audio.get() == i) on_pick=Callback::new(move |_| audio.set(i)) label=label />
                            }).collect_view()}
                        </Column>
                    </div>
                </Show>
                <div class=move || if has_audio() { "absolute inset-y-0 left-[508px] right-[230px]" } else { "absolute inset-y-0 left-[36px] right-[230px]" }>
                    <Column title=crate::i18n::t("subtitle-drawer-subtitles") style=fade>
                        <Row chosen=Signal::derive(move || sub.get().is_none()) on_pick=Callback::new(move |_| sub.set(None)) label=crate::i18n::t("player-subs-off") />
                        {move || subtitles.get().into_iter().map(|s| {
                            let url = s.url.clone();
                            let mine = url.clone();
                            view! { <Row chosen=Signal::derive(move || sub.get().as_deref() == Some(mine.as_str())) on_pick=Callback::new(move |_| sub.set(Some(url.clone()))) label=s.label.clone() /> }
                        }).collect_view()}
                    </Column>
                </div>
                // Cancel and Apply, 20 px above the bottom and 25 px from the right
                <div class="absolute bottom-5 right-[25px] flex gap-2">
                    <button type="button" on:click=move |_| open.set(false) class="btn-pill h-[43px] bg-lift px-5 text-body-lg font-bold text-ink">{crate::i18n::t("common-cancel")}</button>
                    <button type="button" on:click=apply class="btn-pill h-[43px] bg-ink px-5 text-body-lg font-bold text-void">{crate::i18n::t("player-apply")}</button>
                </div>
            </div>
        </Show>
    }
}

/// A column: its heading and the rows under it, scrolling.
#[component]
fn Column(title: String, style: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="flex h-full w-full min-w-0 flex-col">
            <h2 class="pt-[44px] text-title-lg font-bold leading-[33px]">{title}</h2>
            <div class="mt-[24px] min-h-0 flex-1 overflow-y-auto scrollbar-hide pb-24" style=style>{children()}</div>
        </div>
    }
}

/// One choice: a check before it when chosen, the label bold.
#[component]
fn Row(#[prop(into)] chosen: Signal<bool>, on_pick: Callback<()>, label: String) -> impl IntoView {
    view! {
        <button type="button" on:click=move |_| on_pick.run(()) class="flex h-[58.5px] w-full items-center text-start text-title-sm font-bold cursor-pointer">
            <i class="ph ph-check w-[36.7px] shrink-0 text-heading" class=("invisible", move || !chosen.get())></i>
            <span class="truncate">{label}</span>
        </button>
    }
}
