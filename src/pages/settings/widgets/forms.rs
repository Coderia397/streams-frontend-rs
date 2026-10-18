//! Settings widgets: fields, buttons, steppers and pickers.
use crate::i18n::t;
use leptos::prelude::*;

/// Labelled text field.
#[component]
pub fn Field(
    #[prop(into)] label: String,
    /// Show the label only from 1024 px (the Android app has none above the name field).
    #[prop(default = false)] label_wide_only: bool,
    value: RwSignal<String>,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = "off")] autocomplete: &'static str,
    #[prop(optional)] maxlength: Option<u32>,
    #[prop(optional)] inputmode: Option<&'static str>,
) -> impl IntoView {
    view! {
        <label class="mb-4 block">
            <span class=if label_wide_only { "mb-1.5 hidden text-label font-semibold text-ink/60 lg:block" } else { "mb-1.5 block text-label font-semibold text-ink/60" }>{label}</span>
            <input
                type=kind
                autocomplete=autocomplete
                maxlength=maxlength
                inputmode=inputmode
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                class="w-full rounded-control border border-set-edge bg-set-card px-4 py-3 text-body-md text-ink focus:border-ink focus:outline-none max-lg:h-14"
            />
        </label>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Error,
    Done,
}

/// A line under a form saying what went wrong or that it worked.
#[component]
pub fn Message(text: RwSignal<Option<(Tone, String)>>) -> impl IntoView {
    move || {
        text.get().map(|(tone, msg)| {
            let look = if tone == Tone::Error { "border-danger/50 bg-danger/10 text-danger" } else { "border-ink/20 bg-ink/10 text-ink" };
            view! { <p role="status" class=format!("mb-4 rounded-control border px-4 py-3 text-body-sm {look}")>{msg}</p> }
        })
    }
}

/// Buttons at the bottom of a form: the main one and a plain cancel.
#[component]
pub fn FormButtons(#[prop(into)] save_label: String, on_save: Callback<()>, #[prop(into)] cancel_href: String, #[prop(optional, into)] busy: Option<Signal<bool>>) -> impl IntoView {
    view! {
        // a phone stacks them full width, like the Android app; wide screens put them side by side
        <div class="mt-2 flex flex-col items-stretch gap-3 lg:flex-row lg:flex-wrap lg:items-center">
            <button
                type="button"
                on:click=move |_| on_save.run(())
                disabled=move || busy.map(|b| b.get()).unwrap_or(false)
                class="btn-ink rounded-control px-8 py-3 disabled:opacity-50 lg:py-2.5"
            >
                {save_label}
            </button>
            <a href=cancel_href class="rounded-control px-6 py-2.5 text-center text-body-md font-bold text-ink/80 hover:bg-ink/10">{t("set-cancel")}</a>
        </div>
    }
}

/// Outline button at the bottom of a page, for the actions that remove things.
#[component]
pub fn OutlineButton(#[prop(into)] label: String, on_click: Callback<()>, #[prop(optional)] danger: bool) -> impl IntoView {
    let look = if danger { "border-danger/60 text-danger hover:bg-danger/10" } else { "border-ink/40 text-ink hover:bg-ink/10" };
    view! {
        <button type="button" on:click=move |_| on_click.run(()) class=format!("w-full rounded-control border px-6 py-3 text-body-md font-bold {look}")>{label}</button>
    }
}

/// Password field with a show or hide button.
#[component]
pub fn PasswordField(#[prop(into)] label: String, value: RwSignal<String>, #[prop(default = "current-password")] autocomplete: &'static str) -> impl IntoView {
    let shown = RwSignal::new(false);
    view! {
        <label class="mb-4 block">
            <span class="mb-1.5 block text-label font-semibold text-ink/60">{label}</span>
            <span class="flex items-center rounded-control border border-set-edge bg-set-card focus-within:border-ink">
                <input
                    type=move || if shown.get() { "text" } else { "password" }
                    autocomplete=autocomplete
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev))
                    class="min-w-0 flex-1 bg-transparent px-4 py-3 text-body-md text-ink focus:outline-none"
                />
                <button
                    type="button"
                    aria-label=move || if shown.get() { t("set-password-hide") } else { t("set-password-show") }
                    aria-pressed=move || shown.get().to_string()
                    on:click=move |_| shown.update(|s| *s = !*s)
                    class="flex h-11 w-11 shrink-0 items-center justify-center text-ink/70 hover:text-ink"
                >
                    <i class=move || if shown.get() { "ph-bold ph-eye-slash text-xl" } else { "ph-bold ph-eye text-xl" } aria-hidden="true"></i>
                </button>
            </span>
        </label>
    }
}

/// A row of digit boxes for a PIN or a code. One real input sits over the boxes, so typing, pasting, the keyboard type and
/// one time code autofill all work; the boxes only show what was typed.
#[component]
pub fn DigitBoxes(#[prop(into)] label: String, value: RwSignal<String>, length: usize, #[prop(default = false)] secret: bool, #[prop(default = false)] one_time_code: bool, #[prop(default = false)] plain: bool) -> impl IntoView {
    let focused = RwSignal::new(false);
    view! {
        <div class=if plain { "mb-7" } else { "mb-4" }>
            <span class=if plain { "sr-only" } else { "mb-1.5 block text-label font-semibold text-ink/60" }>{label.clone()}</span>
            <div class="relative inline-block">
                <div class="flex gap-2" aria-hidden="true">
                    {(0..length).map(|i| view! {
                        <span
                            class=if plain { "flex h-[72px] w-10 items-center justify-center rounded-control border bg-page text-heading font-medium text-ink" } else { "flex h-14 w-11 items-center justify-center rounded-control border bg-ink/10 text-title font-bold text-ink" }
                            class=("border-ink", move || focused.get() && value.with(|v| v.chars().count()) == i)
                            class=("border-ink/50", move || plain && !(focused.get() && value.with(|v| v.chars().count()) == i))
                            class=("border-ink/20", move || !plain && !(focused.get() && value.with(|v| v.chars().count()) == i))
                        >
                            {move || value.with(|v| v.chars().nth(i).map(|c| if secret { '•' } else { c }).map(String::from).unwrap_or_default())}
                        </span>
                    }).collect_view()}
                </div>
                <input
                    type="text"
                    inputmode="numeric"
                    pattern="[0-9]*"
                    autocomplete=if one_time_code { "one-time-code" } else { "off" }
                    maxlength=length
                    aria-label=label
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev).chars().filter(|c| c.is_ascii_digit()).take(length).collect())
                    on:focus=move |_| focused.set(true)
                    on:blur=move |_| focused.set(false)
                    class="absolute inset-0 h-full w-full cursor-text bg-transparent text-transparent caret-transparent opacity-0"
                />
            </div>
        </div>
    }
}

/// Levels joined by a line, each level allowing everything below it (the age ratings stepper in the Netflix app). `options` run
/// from the lowest to the highest level: (value, label, hint).
#[component]
pub fn Stepper(options: Vec<(String, String, String)>, selected: Signal<String>, on_pick: Callback<String>) -> impl IntoView {
    let values: Vec<String> = options.iter().map(|(v, _, _)| v.clone()).collect();
    let count = options.len();
    view! {
        <div role="radiogroup" class="px-0 py-2 lg:rounded-card lg:border lg:border-set-edge lg:bg-set-card lg:px-4">
            {options
                .into_iter()
                .enumerate()
                .map(|(i, (value, label, hint))| {
                    let values = values.clone();
                    let v = value.clone();
                    let reached = {
                        let values = values.clone();
                        move || values.iter().position(|x| *x == selected.get()).is_some_and(|at| i <= at)
                    };
                    let reached2 = reached.clone();
                    view! {
                        <button
                            type="button"
                            role="radio"
                            aria-checked=move || (selected.get() == v).to_string()
                            on:click=move |_| on_pick.run(value.clone())
                            class="relative flex w-full items-start gap-4 py-3 text-start focus-visible:outline-none"
                        >
                            <span class="relative flex w-7 shrink-0 justify-center">
                                {(i + 1 < count).then(|| view! { <span class="absolute start-1/2 top-7 -bottom-6 w-0.5 -translate-x-1/2 bg-set-line" class=("bg-brand", { let r = reached2.clone(); move || r() }) aria-hidden="true"></span> })}
                                <span
                                    class="relative flex h-7 w-7 items-center justify-center rounded-full border-2"
                                    class=("border-brand", { let r = reached.clone(); move || r() })
                                    class=("bg-brand", { let r = reached.clone(); move || r() })
                                    class=("border-set-edge", { let r = reached.clone(); move || !r() })
                                >
                                    <i class="ph-bold ph-check text-fine text-chrome-ink" class=("invisible", { let r = reached.clone(); move || !r() }) aria-hidden="true"></i>
                                </span>
                            </span>
                            <span class="min-w-0 flex-1">
                                <span class="block text-body-md font-semibold leading-row text-ink lg:text-title-sm">{label}</span>
                                <span class="block text-label text-ink/70 lg:text-body-sm">{hint}</span>
                            </span>
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Round colour swatches, one selected.
#[component]
pub fn SwatchPicker(#[prop(into)] label: String, colors: Vec<(String, String, String)>, selected: Signal<String>, on_pick: Callback<String>) -> impl IntoView {
    view! {
        <div class="set-row block w-full border-b border-set-line px-1 py-3 last:border-b-0">
            <span class="mb-2 block text-title-sm font-semibold leading-row text-ink">{label}</span>
            <div role="radiogroup" class="flex flex-wrap gap-3">
                {colors
                    .into_iter()
                    .map(|(id, hex, name)| {
                        let hex_none = hex == "transparent";
                        let sel = id.clone();
                        let pick = id.clone();
                        view! {
                            <button
                                type="button"
                                role="radio"
                                aria-label=name
                                aria-checked=move || (selected.get() == sel).to_string()
                                on:click=move |_| on_pick.run(pick.clone())
                                style=format!("background-color:{hex}")
                                class="h-9 w-9 rounded-full border-2 border-set-edge focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
                                class=("swatch-none", hex_none)
                                class=("ring-2", { let s = id.clone(); move || selected.get() == s })
                                class=("ring-brand", { let s = id.clone(); move || selected.get() == s })
                                class=("ring-offset-2", { let s = id.clone(); move || selected.get() == s })
                            ></button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}


/// A field as on the website's Edit Profile: a 56 px box with a 1 px grey edge, the label small above the value, and an optional icon on the left.
#[component]
pub fn FloatField(
    #[prop(into)] label: String,
    value: RwSignal<String>,
    #[prop(optional)] maxlength: Option<u32>,
    #[prop(optional)] icon: Option<&'static str>,
) -> impl IntoView {
    view! {
        <label class="relative flex h-[56px] w-full cursor-text items-center gap-4 rounded-control border border-[rgba(128,128,128,0.7)] bg-page px-4 focus-within:border-ink">
            {icon.map(|i| view! { <i class=format!("ph ph-{i} shrink-0 text-heading") aria-hidden="true"></i> })}
            <span class="flex min-w-0 flex-1 flex-col">
                <span class="text-fine leading-[14px] text-ink/70">{label}</span>
                <input
                    type="text"
                    maxlength=maxlength
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev))
                    class="min-w-0 bg-transparent text-body-md leading-[22px] text-ink focus:outline-none"
                />
            </span>
        </label>
    }
}
