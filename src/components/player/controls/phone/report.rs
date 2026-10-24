//! Report a problem on the phone's player, as Netflix's Android app (frames on a Pixel 8, 2026-10-24, note 116): "What is the issue?"
//! at the top left; four choices on dark cards in the middle (Buffering & loading, Subtitles, Audio & video, Another issue), an icon
//! before and a chevron after each; Cancel and Send at the bottom right, Send greyed until a choice is made. Send opens the Help Centre's
//! contact page with the issue, since the site has no report service of its own.
use leptos::prelude::*;

const ISSUES: [(&str, &str, &str); 4] = [
    ("buffering", "ph-arrow-clockwise", "player-issue-buffering"),
    ("subtitles", "ph-subtitles", "player-issue-subtitles"),
    ("audio-video", "ph-picture-in-picture", "player-issue-audio-video"),
    ("other", "ph-question", "player-issue-other"),
];

#[component]
pub fn ReportMenu(open: RwSignal<bool>) -> impl IntoView {
    let chosen = RwSignal::new(None::<&'static str>);
    let send = move |_| {
        let Some(issue) = chosen.get_untracked() else { return };
        super::super::super::frame::leave_landscape();
        if let Some(w) = web_sys::window() {
            let _ = w.location().set_href(&format!("/help/contact?issue={issue}"));
        }
    };
    view! {
        <Show when=move || open.get()>
            <div class="absolute inset-0 z-50 bg-void text-ink" on:click=|e: leptos::ev::MouseEvent| e.stop_propagation()>
                <h2 class="absolute left-[67px] top-4 text-title-lg font-bold">{crate::i18n::t("player-issue-title")}</h2>
                <div class="absolute left-1/2 top-[94px] flex w-[509px] max-w-[70%] -translate-x-1/2 flex-col gap-2.5">
                    {ISSUES.into_iter().map(|(id, icon, key)| view! {
                        <button
                            type="button"
                            on:click=move |_| chosen.set(Some(id))
                            class=move || format!("flex h-[63px] w-full items-center gap-4 rounded-button px-5 text-start text-title-sm font-bold cursor-pointer {}", if chosen.get() == Some(id) { "bg-ink text-void" } else { "bg-sheet" })
                        >
                            <i class=format!("ph {icon} text-heading")></i>
                            <span class="flex-1">{crate::i18n::t(key)}</span>
                            <i class="ph ph-caret-right text-title-sm"></i>
                        </button>
                    }).collect_view()}
                </div>
                <div class="absolute bottom-5 right-[25px] flex gap-2">
                    <button type="button" on:click=move |_| { chosen.set(None); open.set(false); } class="btn-pill h-[43px] bg-lift px-5 text-body-lg font-bold text-ink">{crate::i18n::t("set-cancel")}</button>
                    <button
                        type="button"
                        on:click=send
                        disabled=move || chosen.get().is_none()
                        class=move || format!("btn-pill h-[43px] px-5 text-body-lg font-bold {}", if chosen.get().is_some() { "bg-ink text-void" } else { "bg-lift/60 text-ink/40" })
                    >
                        {crate::i18n::t("player-issue-send")}
                    </button>
                </div>
            </div>
        </Show>
    }
}
