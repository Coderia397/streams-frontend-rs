use leptos::prelude::*;
use crate::services::resolver::SubtitleTrack;

/// The audio and subtitles panel, laid out from the owner's Netflix screenshots (note 70): a dark panel above the controls on the right, a heading in 28 px, rows of 21 px with a
/// check in front of the chosen one, the row under the pointer on a lighter fill. The audio column (left) shows only when the stream carries two or more audio tracks; the subtitles
/// column is always there.
/// The timing offset stays, as a small row under the list.
#[component]
pub fn SubtitleDrawer(
    is_open: ReadSignal<bool>,
    subtitles: Signal<Vec<SubtitleTrack>>,
    selected_sub_url: ReadSignal<Option<String>>,
    sub_delay: ReadSignal<f64>,
    /// The audio tracks of the stream as (index, label); the audio column shows when there are two or more
    audio_tracks: Signal<Vec<(i32, String)>>,
    audio_selected: Signal<i32>,
    on_select_audio: Callback<i32>,
    on_select_sub: Callback<Option<String>>,
    on_change_delay: Callback<f64>,
    /// True when the chosen language has another file to try; `on_other_version` moves to it (subtitles out of step with the video)
    #[prop(optional, into)] has_other_version: Signal<bool>,
    #[prop(optional)] on_other_version: Option<Callback<()>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let row = "flex h-[59px] w-full items-center px-[10px] text-start text-title-lg transition-colors duration-(--motion-quick) hover:bg-lift hover:text-ink focus-visible:outline-2 focus-visible:outline-ink";
    let has_audio = move || audio_tracks.get().len() > 1;
    view! {
        <Show when=move || is_open.get()>
            // a click anywhere else closes the panel (no dimming)
            <div on:click=move |_| on_close.run(()) class="fixed inset-0 z-40" />

            <div class="fixed bottom-[94px] right-0 z-50 flex max-h-[calc(100vh-190px)] w-[min(681px,100vw)] flex-col overflow-hidden rounded-card bg-field shadow-2xl" class=("w-[min(1000px,100vw)]", has_audio)>
              <div class="flex min-h-0 flex-1">
                <Show when=has_audio>
                    <div class="w-[40%] shrink-0 overflow-y-auto border-e border-line-strong px-[35px] pb-[20px] pt-[26px]">
                        <h2 class="mb-[18px] text-heading-lg font-medium leading-[34px] text-ink">{crate::i18n::t("player-audio")}</h2>
                        {move || audio_tracks.get().into_iter().map(|(i, label)| view! {
                            <button on:click=move |_| on_select_audio.run(i) class=move || format!("{row} {}", if audio_selected.get() == i { "text-ink" } else { "text-ink-body" })>
                                <i class=move || format!("ph ph-check me-[14px] w-[24px] text-heading {}", if audio_selected.get() == i { "" } else { "invisible" })></i>
                                <span class="truncate">{label}</span>
                            </button>
                        }).collect::<Vec<_>>()}
                    </div>
                </Show>
                <div class="min-w-0 flex-1 overflow-y-auto px-[35px] pb-[20px] pt-[26px]">
                    <h2 class="mb-[18px] text-heading-lg font-medium leading-[34px] text-ink">{crate::i18n::t("subtitle-drawer-subtitles")}</h2>

                    <button on:click=move |_| on_select_sub.run(None) class=move || format!("{row} {}", if selected_sub_url.get().is_none() { "text-ink" } else { "text-ink-body" })>
                        <i class=move || format!("ph ph-check me-[14px] w-[24px] text-heading {}", if selected_sub_url.get().is_none() { "" } else { "invisible" })></i>
                        <span>{crate::i18n::t("player-subs-off")}</span>
                    </button>

                    {move || {
                        let subs = subtitles.get();
                        if subs.is_empty() {
                            view! { <p class="px-[10px] py-3 text-title-sm text-ink-body">{crate::i18n::t("subtitle-drawer-no-subtitles-detected-for-this")}</p> }.into_any()
                        } else {
                            subs.into_iter().map(|s| {
                                let url = s.url.clone();
                                let label = s.label.clone();
                                let u_active = url.clone();
                                let u_icon = url.clone();
                                let select_cb = on_select_sub;
                                view! {
                                    <button on:click=move |_| select_cb.run(Some(url.clone())) class=move || format!("{row} {}", if selected_sub_url.get().as_deref() == Some(u_active.as_str()) { "text-ink" } else { "text-ink-body" })>
                                        <i class=move || format!("ph ph-check me-[14px] w-[24px] text-heading {}", if selected_sub_url.get().as_deref() == Some(u_icon.as_str()) { "" } else { "invisible" })></i>
                                        <span class="truncate">{label}</span>
                                    </button>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}

                    <Show when=move || selected_sub_url.get().is_some()>
                        <div class="mt-[14px] flex items-center justify-between border-t border-line-strong px-[10px] pt-[14px] text-title-sm text-ink-body">
                            <span>{crate::i18n::t("subtitle-drawer-subtitle-timing-offset")}</span>
                            <span class="flex items-center gap-4 text-ink">
                                <button on:click=move |_| on_change_delay.run(sub_delay.get() - 0.5) class="h-9 w-12 rounded-button hover:bg-lift">"-0.5s"</button>
                                <span class="w-14 text-center">{move || format!("{:+.1}s", sub_delay.get())}</span>
                                <button on:click=move |_| on_change_delay.run(sub_delay.get() + 0.5) class="h-9 w-12 rounded-button hover:bg-lift">"+0.5s"</button>
                            </span>
                        </div>
                    </Show>
                    <Show when=move || selected_sub_url.get().is_some() && has_other_version.get() && on_other_version.is_some()>
                        <button
                            on:click=move |_| { if let Some(cb) = on_other_version { cb.run(()) } }
                            class="mt-[6px] flex w-full items-center gap-3 rounded-button px-[10px] py-3 text-start text-title-sm text-ink-body hover:bg-lift hover:text-ink"
                        >
                            <i class="ph-bold ph-arrows-clockwise text-title" aria-hidden="true"></i>
                            {crate::i18n::t("subtitle-drawer-another-version")}
                        </button>
                    </Show>
                </div>
              </div>
            </div>
        </Show>
    }
}
