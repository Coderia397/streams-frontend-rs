//! Cookie banner and Privacy Preference Center. State and rules are in crate::consent.
use crate::config::{cookie_banner_position, BannerPosition};
use crate::consent::use_ui;
use crate::pages::help::markup;
use crate::pages::help::tr;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    General,
    Essential,
    Personal,
    Third,
    Ads,
}

const SECTIONS: [(Section, &str); 5] = [
    (Section::General, "cookie-nav-general"),
    (Section::Essential, "cookie-nav-essential"),
    (Section::Personal, "cookie-nav-personal"),
    (Section::Third, "cookie-nav-third"),
    (Section::Ads, "cookie-nav-ads"),
];

#[component]
fn Toggle(on: RwSignal<bool>, label: &'static str) -> impl IntoView {
    view! {
        <button
            role="switch"
            aria-checked=move || on.get().to_string()
            aria-label=move || tr(label)
            on:click=move |_| on.update(|v| *v = !*v)
            class="relative h-6 w-11 shrink-0 rounded-full transition-colors duration-(--motion-base)"
            class=("bg-brand", move || on.get())
            class=("bg-paper-edge", move || !on.get())
        >
            <span
                class="absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-paper transition-transform duration-(--motion-base)"
                class=("translate-x-5", move || on.get())
            ></span>
        </button>
    }
}

#[component]
fn Details(list_key: &'static str) -> impl IntoView {
    let (open, set_open) = signal(false);
    view! {
        <button on:click=move |_| set_open.update(|o| *o = !*o) class="mt-4 text-body-sm text-brand-active hover:underline">
            {move || if open.get() { tr("cookie-hide-details") } else { tr("cookie-details") }}
        </button>
        <Show when=move || open.get()>
            <div class="mt-3 text-body-sm">{markup::blocks(markup::parse(&tr(list_key)))}</div>
        </Show>
    }
}

#[component]
fn Center() -> impl IntoView {
    let ui = use_ui();
    let (section, set_section) = signal(Section::General);
    let personal = ui.personalization;
    view! {
        <div class="fixed inset-0 z-(--z-consent) flex items-center justify-center bg-void/60 p-3" on:click=move |_| ui.center.set(false)>
            <div
                role="dialog"
                aria-modal="true"
                aria-label=move || tr("cookie-center-title")
                on:click=|ev| ev.stop_propagation()
                class="flex h-[min(620px,92dvh)] w-full max-w-[830px] flex-col overflow-hidden bg-paper font-sans text-paper-ink shadow-popover"
            >
                <div class="flex items-center justify-between gap-4 border-b border-paper-line px-5 py-3">
                    <img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="h-6 w-auto shrink-0" />
                    <h2 class="flex-1 text-center text-title font-bold md:text-start md:ps-10">{tr("cookie-center-title")}</h2>
                    <button aria-label=move || tr("cookie-dismiss") on:click=move |_| ui.center.set(false) class="flex h-9 w-9 items-center justify-center text-title hover:bg-paper-soft">
                        <i class="ph-bold ph-x"></i>
                    </button>
                </div>
                <div class="flex min-h-0 flex-1 flex-col md:flex-row">
                    <nav class="flex shrink-0 overflow-x-auto bg-paper-soft md:w-[250px] md:flex-col md:overflow-visible">
                        {SECTIONS
                            .iter()
                            .map(|(s, key)| {
                                let s = *s;
                                view! {
                                    <button
                                        on:click=move |_| set_section.set(s)
                                        aria-current=move || (section.get() == s).then_some("true")
                                        class="shrink-0 border-b border-paper-line border-s-4 px-4 py-3.5 text-start text-body-sm font-bold"
                                        class=("border-s-brand", move || section.get() == s)
                                        class=("bg-paper", move || section.get() == s)
                                        class=("border-s-transparent", move || section.get() != s)
                                    >
                                        {tr(key)}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </nav>
                    <div class="min-h-0 flex-1 overflow-y-auto p-5 text-body-sm leading-6 md:p-7">
                        {move || match section.get() {
                            Section::General => view! {
                                <h3 class="mb-4 text-body-md font-bold">{tr("cookie-nav-general")}</h3>
                                {markup::blocks(markup::parse(&tr("cookie-general")))}
                            }
                            .into_any(),
                            Section::Essential => view! {
                                <div class="mb-4 flex items-center justify-between gap-4">
                                    <h3 class="text-body-md font-bold">{tr("cookie-nav-essential")}</h3>
                                    <span class="text-fine font-bold text-brand-active">{tr("cookie-always-active")}</span>
                                </div>
                                {markup::blocks(markup::parse(&tr("cookie-essential")))}
                                <Details list_key="cookie-essential-list" />
                            }
                            .into_any(),
                            Section::Personal => view! {
                                <div class="mb-4 flex items-center justify-between gap-4">
                                    <h3 class="text-body-md font-bold">{tr("cookie-nav-personal")}</h3>
                                    <Toggle on=personal label="cookie-nav-personal" />
                                </div>
                                {markup::blocks(markup::parse(&tr("cookie-personal")))}
                                <Details list_key="cookie-personal-list" />
                            }
                            .into_any(),
                            Section::Third => view! {
                                <h3 class="mb-4 text-body-md font-bold">{tr("cookie-nav-third")}</h3>
                                {markup::blocks(markup::parse(&tr("cookie-third")))}
                            }
                            .into_any(),
                            Section::Ads => view! {
                                <h3 class="mb-4 text-body-md font-bold">{tr("cookie-nav-ads")}</h3>
                                {markup::blocks(markup::parse(&tr("cookie-ads")))}
                            }
                            .into_any(),
                        }}
                    </div>
                </div>
                <div class="border-t border-paper-line px-5 py-4">
                    <button on:click=move |_| ui.decide(personal.get_untracked()) class="rounded-control bg-brand px-6 py-3 text-body-sm font-bold text-ink hover:bg-brand-hover">
                        {tr("cookie-save")}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Mounted once in App. Shows the consent sheet until a choice is saved, and the Privacy Preference Center when asked.
/// On a phone the sheet is a modal that fills the width at the bottom and takes about a fifth of the screen, with rounded-control top
/// corners; from the `sm` width (a foldable or a tablet) it becomes a floating pop-up whose place is a setting (crate::config).
#[component]
pub fn CookieManager() -> impl IntoView {
    let ui = use_ui();
    let location = use_location();
    // Show only after a short pause, on pages where it makes sense, and not over the Who's watching screen.
    let profiles = crate::store::use_profile_store();
    let (ready, set_ready) = signal(false);
    if let Some(w) = web_sys::window() {
        let on_ready = wasm_bindgen::closure::Closure::once_into_js(move || set_ready.set(true));
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(wasm_bindgen::JsCast::unchecked_ref(&on_ready), crate::consent::SHOW_DELAY_MS);
    }
    let visible = move || ui.banner.get() && ready.get() && profiles.active_profile_id.get().is_some() && !crate::consent::suppressed_on(&location.pathname.get());
    // Where the pop-up sits from the `sm` width is a setting; a phone always gets the bottom sheet.
    let place = match cookie_banner_position() {
        BannerPosition::Top => "sm:inset-x-0 sm:top-6 sm:bottom-auto sm:mx-auto",
        BannerPosition::Bottom => "sm:inset-x-0 sm:bottom-6 sm:mx-auto",
        BannerPosition::BottomStart => "sm:bottom-6 sm:start-6 sm:inset-x-auto",
        BannerPosition::BottomEnd => "sm:bottom-6 sm:end-6 sm:inset-x-auto",
        BannerPosition::Center => "sm:inset-x-0 sm:top-1/2 sm:bottom-auto sm:mx-auto sm:-translate-y-1/2",
    };
    view! {
        <Show when=visible>
            <div class="fixed inset-0 z-(--z-consent) bg-void/50" aria-hidden="true"></div>
            <div
                role="dialog"
                aria-modal="true"
                aria-label=move || tr("cookie-center-title")
                class=format!("fixed inset-x-0 bottom-0 z-(--z-consent) min-h-sheet-min w-full rounded-t-chip bg-paper px-5 pb-[max(1.25rem,env(safe-area-inset-bottom))] pt-3 font-sans text-paper-ink shadow-popover sm:w-sheet sm:max-w-[calc(100%-3rem)] sm:rounded-chip sm:p-6 {place}")
            >
                <span class="mx-auto mb-3 block h-1 w-10 rounded-full bg-paper-line sm:hidden" aria-hidden="true"></span>
                <div class="flex items-start gap-3">
                    <p class="flex-1 text-body-sm leading-6">{markup::text(&tr("cookie-sheet"))}</p>
                    <button aria-label=move || tr("cookie-dismiss") on:click=move |_| ui.dismiss() class="-me-1 -mt-1 flex h-9 w-9 shrink-0 items-center justify-center text-title text-paper-ink">
                        <i class="ph-bold ph-x"></i>
                    </button>
                </div>
                <div class="mt-4 grid grid-cols-3 gap-2 sm:flex sm:flex-wrap sm:gap-3">
                    <button on:click=move |_| ui.open_center() class="rounded-control border border-paper-edge bg-paper px-3 py-3 text-fine font-bold hover:bg-paper-soft sm:px-5 sm:text-body-sm">{tr("cookie-learn")}</button>
                    <button on:click=move |_| ui.decide(false) class="rounded-control bg-brand px-3 py-3 text-fine font-bold text-ink hover:bg-brand-hover sm:px-5 sm:text-body-sm">{tr("cookie-reject")}</button>
                    <button on:click=move |_| ui.decide(true) class="rounded-control bg-brand px-3 py-3 text-fine font-bold text-ink hover:bg-brand-hover sm:px-5 sm:text-body-sm">{tr("cookie-accept")}</button>
                </div>
            </div>
        </Show>
        <Show when=move || ui.center.get()>
            <Center />
        </Show>
    }
}
