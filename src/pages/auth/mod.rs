//! Sign in, sign up and password reset pages, laid out like the Netflix ones: a black page with a colored glow for sign in, a
//! white page with steps for sign up, floating label fields, a full width action button and a footer of links.
mod confirm;
mod forgot;
mod login;
mod signup;

pub use confirm::ConfirmPage;
pub use forgot::ForgotPage;
pub use login::LoginPage;
pub use signup::SignupPage;

use crate::pages::help::tr;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// Black page, used for sign in and password reset.
    Dark,
    /// White page, used for sign up.
    Light,
}

/// Header with the logo (and a link on the right), the page, and the footer with help links and the language picker.
#[component]
pub fn AuthShell(children: Children, #[prop(default = Look::Dark)] look: Look, #[prop(optional)] right: Option<(&'static str, &'static str)>, #[prop(default = false)] back: bool, /// false takes the footer off on a phone (the sign in page, as in the app)
    #[prop(default = true)] phone_footer: bool) -> impl IntoView {
    let dark = look == Look::Dark;
    let page = if dark { "bg-linear-to-b from-brand-deep/40 via-void to-void text-ink" } else { "bg-paper text-paper-ink" };
    let header = if dark { "sm:border-b sm:border-ink/10" } else { "border-b border-paper-line bg-paper" };
    let main = if dark { "" } else { "bg-paper" };
    let logo = if dark { crate::brand::LOGO_WORDMARK } else { crate::brand::LOGO_WORDMARK_PRIMARY };
    let right_link = if dark { "text-ink" } else { "text-paper-ink" };
    view! {
        <div class=format!("flex min-h-screen flex-col font-sans {page}")>
            <header class=header>
                <div class="mx-auto flex h-16 max-w-[1280px] sm:h-[90px] items-center justify-between px-6 md:px-12">
                    <div class="flex items-center gap-2">
                        {back.then(|| view! { <a href="/" aria-label=move || tr("auth-back") class="-ms-2 flex h-10 w-10 items-center justify-center text-title sm:hidden"><AppIcon glyph=Glyph::Back class="rtl:-scale-x-100" /></a> })}
                        <a href="/" aria-label=crate::brand::NAME><img src=logo alt=crate::brand::NAME class="h-6 w-auto sm:h-8 md:h-10" /></a>
                    </div>
                    {right.map(|(href, key)| view! { <a href=href class=format!("text-body-sm font-bold hover:underline {right_link}")>{tr(key)}</a> })}
                </div>
            </header>
            <main class=format!("flex-1 {main}")>{children()}</main>
            <div class=if phone_footer { "contents" } else { "contents max-sm:hidden" }><AuthFooter look=look /></div>
        </div>
    }
}


/// Footer with the help links, the cookie preferences and the language picker, shared by the auth pages and the landing page.
#[component]
pub fn AuthFooter(#[prop(default = Look::Dark)] look: Look) -> impl IntoView {
    let dark = look == Look::Dark;
    let consent = crate::consent::use_ui();
    let current = crate::i18n::language();
    let on_language = move |ev: leptos::ev::Event| {
        if let Some(select) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) {
            crate::i18n::set_language(&select.value());
        }
    };
    let footer = if dark { "bg-page text-ink-ash" } else { "bg-paper-soft text-paper-mute" };
    let link = if dark { "hover:text-ink" } else { "hover:text-paper-ink" };
    let select = if dark { "border-ink/40 bg-void/60 text-ink-soft" } else { "border-paper-edge bg-paper text-paper-ink" };
    let links: [(&str, &str); 8] = [
        ("/help", "footer-help"),
        ("/help/contact", "footer-contact"),
        ("/terms", "help-ui-legal-terms"),
        ("/privacy", "help-ui-legal-privacy"),
        ("/cookies", "help-ui-legal-cookies"),
        ("/dmca", "help-ui-legal-dmca"),
        ("/disclaimer", "help-ui-legal-disclaimer"),
        ("/credits", "help-ui-legal-credits"),
    ];
    view! {
            <footer class=footer>
                <div class="mx-auto max-w-[1280px] px-6 py-10 md:px-12">
                    <p class="mb-6 text-body-md">{tr("auth-questions")}" "<a href="/help/contact" class="underline underline-offset-2">{tr("help-ui-contact-us")}</a></p>
                    <ul class="grid grid-cols-2 gap-x-8 gap-y-4 text-body-sm md:grid-cols-4">
                        {links.into_iter().map(|(href, key)| view! { <li><a href=href class=format!("underline underline-offset-2 {link}")>{tr(key)}</a></li> }).collect_view()}
                        <li><button on:click=move |_| consent.open_center() class=format!("text-start underline underline-offset-2 {link}")>{tr("cookie-link")}</button></li>
                    </ul>
                    <label for="auth-language" class="sr-only">{tr("help-ui-language-label")}</label>
                    <div class="relative mt-8 inline-block">
                        <select id="auth-language" on:change=on_language class=format!("min-w-40 cursor-pointer appearance-none border py-2.5 ps-4 pe-10 text-body-sm {select}")>
                            {crate::i18n::AVAILABLE.iter().map(|code| view! { <option value=*code selected=*code == current class="bg-paper text-paper-ink">{crate::i18n::native_name(code)}</option> }).collect_view()}
                        </select>
                        <i class="ph-fill ph-caret-down pointer-events-none absolute end-3 top-1/2 -translate-y-1/2 text-fine"></i>
                    </div>
                </div>
            </footer>
    }
}

/// A field whose label sits inside it and moves up when there is something to read.
#[component]
pub fn Field(
    id: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = "off")] autocomplete: &'static str,
    #[prop(default = Look::Dark)] look: Look,
    #[prop(optional, into)] invalid: Option<Signal<bool>>,
) -> impl IntoView {
    let dark = look == Look::Dark;
    let (shown, set_shown) = signal(false);
    let is_password = kind == "password";
    let input = if dark {
        "border-ink/40 bg-void/60 text-ink focus:border-ink"
    } else {
        "border-paper-edge bg-paper text-paper-ink focus:border-paper-ink"
    };
    let label_color = if dark { "text-ink-ash" } else { "text-paper-mute" };
    view! {
        <div class="relative">
            <input
                id=id
                type=move || if is_password && shown.get() { "text" } else { kind }
                autocomplete=autocomplete
                placeholder=" "
                prop:value=value
                on:input=move |ev| value.set(event_target_value(&ev))
                aria-invalid=move || invalid.is_some_and(|i| i.get()).to_string()
                class=format!("peer h-14 w-full border px-4 pt-5 text-body-md placeholder-transparent focus:outline-none {input}")
                class=("!border-danger", move || invalid.is_some_and(|i| i.get()))
            />
            <label
                for=id
                class=format!("pointer-events-none absolute start-4 top-4 text-body-md transition-all duration-(--motion-fast) peer-focus:top-1.5 peer-focus:text-fine peer-[:not(:placeholder-shown)]:top-1.5 peer-[:not(:placeholder-shown)]:text-fine {label_color}")
            >
                {move || tr(label)}
            </label>
            {is_password.then(|| view! {
                <button
                    type="button"
                    on:click=move |_| set_shown.update(|s| *s = !*s)
                    aria-label=move || if shown.get() { tr("auth-hide-password") } else { tr("auth-show-password") }
                    class=format!("absolute end-3 top-1/2 -translate-y-1/2 text-title {label_color}")
                >
                    <i class=move || if shown.get() { "ph-bold ph-eye-slash" } else { "ph-bold ph-eye" }></i>
                </button>
            })}
        </div>
    }
}

/// The full width action button.
pub fn button_class(look: Look) -> &'static str {
    match look {
        Look::Dark => "flex h-12 w-full items-center justify-center rounded-button bg-brand text-body-md font-bold text-ink hover:bg-brand-hover disabled:opacity-60",
        Look::Light => "flex h-14 w-full items-center justify-center rounded-button bg-brand text-title-sm font-bold text-ink hover:bg-brand-hover disabled:opacity-60",
    }
}

/// A message above the form, used for errors and for notices.
#[component]
pub fn Notice(#[prop(into)] text: Signal<String>, #[prop(default = Look::Dark)] look: Look, #[prop(default = true)] error: bool) -> impl IntoView {
    let tone = match (look, error) {
        (Look::Dark, true) => "border-danger bg-danger/15 text-ink",
        (Look::Light, true) => "border-danger bg-danger/10 text-paper-ink",
        (Look::Dark, false) => "border-brand bg-brand/15 text-ink",
        (Look::Light, false) => "border-brand bg-brand/10 text-paper-ink",
    };
    move || {
        let t = text.get();
        (!t.is_empty()).then(|| view! { <div role=if error { "alert" } else { "status" } class=format!("mb-4 rounded-control border-s-4 p-3 text-body-sm leading-6 {tone}")>{t.clone()}</div> })
    }
}
