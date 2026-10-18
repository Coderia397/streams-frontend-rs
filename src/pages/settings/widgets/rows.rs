//! Settings widgets: cards, link rows, switches and choice lists.
use crate::i18n::t;
use leptos::prelude::*;

pub(super) const ROW: &str = "set-row flex min-h-settings-row w-full items-center border-b border-set-line px-1 py-3 text-start last:border-b-0 hover:bg-ink/5 focus-visible:bg-ink/5 focus-visible:outline-none lg:relative lg:min-h-[80px] lg:border-b-0 lg:px-6 lg:py-6 lg:before:absolute lg:before:inset-x-4 lg:before:top-0 lg:before:h-px lg:before:bg-set-line lg:first:before:hidden";
pub(super) const CARD: &str = "set-card rounded-card border border-set-edge bg-set-card px-4 lg:rounded-card lg:px-1";

#[component]
pub fn Section(#[prop(optional, into)] title: Option<String>, children: Children) -> impl IntoView {
    view! {
        <section class="mb-8 lg:mb-[33px]">
            {title.map(|t| view! { <h2 class="set-section-title mb-3 text-body-md leading-row text-ink/85 lg:mb-[17px] lg:text-[18px] lg:font-medium lg:leading-[27px] lg:text-ink">{t}</h2> })}
            <div class=CARD>{children()}</div>
        </section>
    }
}

pub(super) fn row_body(icon: &'static str, title: String, hint: Option<String>, value: Option<Signal<String>>, chevron: bool, verified: bool) -> impl IntoView {
    view! {
        <i class=format!("ph ph-{icon} me-3 text-2xl text-ink/70 lg:me-4") aria-hidden="true"></i>
        <span class="min-w-0 flex-1">
            <span class="block text-body-md font-semibold leading-row text-ink lg:text-[18px] lg:font-medium lg:leading-[27px]">{title}</span>
            {hint.clone().map(|h| view! { <span class="block pt-0.5 text-label leading-normal text-ink/70 lg:pt-0 lg:text-[14px] lg:leading-[25px]">{h}</span> })}
            // on a phone the value sits under the title, as in the Android app, unless the row has a hint of its own
            {hint.is_none().then(|| value.map(|v| view! { <span class="block truncate pt-0.5 text-label text-ink/70 lg:pt-0 lg:text-[14px] lg:leading-[25px]">{move || v.get()}</span> }))}
            {verified.then(|| view! { <span class="mt-1 hidden items-center gap-1 text-[12px] font-medium text-ink lg:flex"><i class="ph-fill ph-check-circle text-[16px]" aria-hidden="true"></i>{t("set-verified")}</span> })}
        </span>
        {chevron.then(|| view! { <i class="ph ph-caret-right ms-3 text-2xl text-ink/70 lg:text-[16px] rtl:-scale-x-100" aria-hidden="true"></i> })}
    }
}

/// A row that opens another screen.
#[component]
pub fn LinkRow(
    icon: &'static str,
    #[prop(into)] title: String,
    #[prop(into)] href: String,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(optional)] value: Option<Signal<String>>,
    /// a "Verified" line under the value (the website shows it under the email address)
    #[prop(optional)] verified: bool,
) -> impl IntoView {
    view! {
        <a href=href class=ROW>
            {row_body(icon, title, hint, value, true, verified)}
        </a>
    }
}

/// A row that does something here (opens a dialog, runs a test).
#[component]
pub fn ActionRow(
    icon: &'static str,
    #[prop(into)] title: String,
    on_click: Callback<()>,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional)] chevron: bool,
) -> impl IntoView {
    view! {
        <button type="button" on:click=move |_| on_click.run(()) class=ROW>
            {row_body(icon, title, hint, value, chevron, false)}
        </button>
    }
}

/// A row with an on/off switch.
#[component]
pub fn SwitchRow(
    icon: &'static str,
    #[prop(into)] title: String,
    #[prop(optional, into)] hint: Option<String>,
    checked: Signal<bool>,
    on_change: Callback<bool>,
) -> impl IntoView {
    view! {
        <div class=ROW>
            {row_body(icon, title.clone(), hint, None, false, false)}
            <button
                type="button"
                role="switch"
                aria-checked=move || checked.get().to_string()
                aria-label=title
                on:click=move |_| on_change.run(!checked.get_untracked())
                class="relative ms-3 inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
                class=("bg-brand", move || checked.get())
                class=("bg-ink/25", move || !checked.get())
            >
                <span
                    class="inline-block h-4 w-4 rounded-full bg-ink transition-transform"
                    class=("translate-x-6", move || checked.get())
                    class=("rtl:-translate-x-6", move || checked.get())
                    class=("translate-x-1", move || !checked.get())
                    class=("rtl:-translate-x-1", move || !checked.get())
                ></span>
            </button>
        </div>
    }
}

/// Radio list: one choice picked, each row a full width button.
#[component]
pub fn ChoiceList(options: Vec<(String, String)>, selected: Signal<String>, on_pick: Callback<String>, #[prop(default = false)] plain: bool) -> impl IntoView {
    view! {
        <div role="radiogroup" class=if plain { "" } else { CARD }>
            {options
                .into_iter()
                .map(|(value, label)| {
                    let v = value.clone();
                    let v2 = value.clone();
                    view! {
                        <button
                            type="button"
                            role="radio"
                            aria-checked=move || (selected.get() == v).to_string()
                            on:click=move |_| on_pick.run(v2.clone())
                            class=if plain { "flex min-h-12 w-full items-center gap-4 py-2 text-start focus-visible:outline-none" } else { ROW }
                        >
                            {plain.then(|| view! {
                                <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border-2 border-ink/70" class=("border-brand", { let s = value.clone(); move || selected.get() == s })>
                                    <span class="h-3 w-3 rounded-full bg-brand" class=("invisible", { let s = value.clone(); move || selected.get() != s })></span>
                                </span>
                            })}
                            <span class="flex-1 text-body-md leading-row text-ink lg:text-title-sm">{label}</span>
                            {(!plain).then(|| view! { <i class="ph-bold ph-check text-xl text-brand" class=("invisible", { let s = value.clone(); move || selected.get() != s }) aria-hidden="true"></i> })}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// A row that opens to show more (a device card).
#[component]
pub fn Expandable(icon: &'static str, #[prop(into)] title: String, #[prop(optional, into)] meta: Option<String>, #[prop(optional, into)] badge: Option<String>, children: ChildrenFn) -> impl IntoView {
    let open = RwSignal::new(false);
    view! {
        <div class="rounded-card border border-set-edge bg-set-card px-4">
            <button type="button" aria-expanded=move || open.get().to_string() on:click=move |_| open.update(|o| *o = !*o) class="flex w-full items-center gap-3 py-4 text-start focus-visible:outline-none">
                <i class=format!("ph-bold ph-{icon} text-2xl text-ink/70") aria-hidden="true"></i>
                <span class="min-w-0 flex-1">
                    {badge.filter(|b| !b.is_empty()).map(|b| view! { <Badge text=b /> })}
                    <span class="block truncate text-title-sm font-semibold leading-row text-ink">{title}</span>
                    {meta.filter(|m| !m.is_empty()).map(|m| view! { <span class="block text-body-sm text-ink/70">{m}</span> })}
                </span>
                <i class="ph-bold ph-caret-down text-2xl text-ink/70 transition-transform" class=("rotate-180", move || open.get()) aria-hidden="true"></i>
            </button>
            <Show when=move || open.get()>
                <div class="border-t border-set-line py-3 text-body-sm text-ink/80">{children()}</div>
            </Show>
        </div>
    }
}

/// Small label: New, Verified, This device.
#[component]
pub fn Badge(#[prop(into)] text: String) -> impl IntoView {
    view! { <span class="mb-1 inline-block rounded-control bg-badge px-1.5 pb-0.5 pt-px text-body-sm font-medium text-paper-ink">{text}</span> }
}

/// A banner that explains something and may carry a button.
#[component]
pub fn Notice(icon: &'static str, #[prop(into)] title: Signal<String>, #[prop(into)] body: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="mb-6 rounded-card border border-set-edge bg-set-card p-5">
            <p class="mb-1 flex items-center gap-2 text-title-sm font-semibold leading-row"><i class=format!("ph-bold ph-{icon} text-2xl") aria-hidden="true"></i>{move || title.get()}</p>
            <p class="mb-4 text-body-md text-ink/70">{move || body.get()}</p>
            {children()}
        </div>
    }
}
