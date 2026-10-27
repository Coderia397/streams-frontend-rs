use super::tr;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Sets the browser tab title of a page of the app itself (sign in, sign up and so on) to "<page> | <brand>", and the canonical link of that page.
pub fn set_site_title(page: &str) {
    let full = if page.is_empty() { crate::brand::NAME.to_string() } else { format!("{page} | {}", crate::brand::NAME) };
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            doc.set_title(&full);
            let path = window.location().pathname().unwrap_or_default();
            set_attr(&doc, "link[rel=canonical]", "href", &format!("{}{path}", crate::config::SITE_URL));
            set_attr(&doc, "meta[property='og:url']", "content", &format!("{}{path}", crate::config::SITE_URL));
            set_attr(&doc, "meta[property='og:title']", "content", &full);
            set_attr(&doc, "meta[name='twitter:title']", "content", &full);
        }
    }
}

/// Sets the browser tab title to "<page> | <brand> Help Center" and points the canonical link and the sharing title at this page.
pub fn set_title(page: &str) {
    let suffix = tr("help-ui-doc-title");
    let full = if page.is_empty() { suffix } else { format!("{page} | {suffix}") };
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            doc.set_title(&full);
            let path = window.location().pathname().unwrap_or_default();
            // the Help Centre, with the policies under /help/legal, lives on its own host
            let origin = if path == "/help" || path.starts_with("/help/") { crate::config::HELP_URL } else { crate::config::SITE_URL };
            set_attr(&doc, "link[rel=canonical]", "href", &format!("{origin}{path}"));
            set_attr(&doc, "meta[property='og:url']", "content", &format!("{origin}{path}"));
            set_attr(&doc, "meta[property='og:title']", "content", &full);
            set_attr(&doc, "meta[name='twitter:title']", "content", &full);
        }
    }
}

/// Sets the description that search results and link previews show for this page.
pub fn set_description(text: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        set_attr(&doc, "meta[name=description]", "content", text);
        set_attr(&doc, "meta[property='og:description']", "content", text);
        set_attr(&doc, "meta[name='twitter:description']", "content", text);
    }
}

fn set_attr(doc: &web_sys::Document, selector: &str, name: &str, value: &str) {
    if let Ok(Some(el)) = doc.query_selector(selector) {
        let _ = el.set_attribute(name, value);
    }
}

/// Name of a language in that language, for the picker.
pub fn native_name(code: &str) -> String {
    crate::i18n::native_name(code)
}

/// Home is a light page under a white bar. Every other help page (articles, search, contact) has a black bar and a dark footer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Light,
    Dark,
}

/// True when this browser has opened the app and chosen a profile. A first visit sees Browse and Sign in instead.
fn has_profile() -> bool {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten()).is_some()
}

/// The signed in view of the header: the profile name with a menu that holds Account and Sign out.
#[component]
fn AccountMenu(name: Signal<String>, dark: bool) -> impl IntoView {
    let (open, set_open) = signal(false);
    let root = NodeRef::<leptos::html::Div>::new();
    let on_focusout = move |ev: leptos::ev::FocusEvent| {
        let inside = ev.related_target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()).zip(root.get_untracked()).is_some_and(|(n, r)| r.contains(Some(&n)));
        if !inside {
            set_open.set(false);
        }
    };
    let text = if dark { "text-ink" } else { "text-paper-ink" };
    view! {
        <div node_ref=root class="relative" on:focusout=on_focusout>
            <button
                aria-haspopup="menu"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| set_open.update(|o| *o = !*o)
                class=format!("flex items-center gap-2 rounded-button border-2 border-transparent px-2 py-1 text-body-md focus-visible:border-brand-active {text}")
            >
                <span class="underline-offset-2 hover:underline">{move || name.get()}</span>
                <i class="ph-fill ph-caret-down text-fine text-brand" aria-hidden="true"></i>
            </button>
            <Show when=move || open.get()>
                <div role="menu" class="absolute end-0 top-full z-(--z-dialog) mt-2 w-56 rounded-menu border border-paper-line bg-paper py-2 shadow-popover">
                    <a role="menuitem" href=crate::config::app_link(&crate::pages::settings::Screen::Account.path()) class="flex items-center gap-2 px-4 py-2 text-body-md text-paper-ink hover:bg-paper-soft">
                        {tr("help-ui-account")}
                        <i class="ph-bold ph-caret-right text-fine text-brand" aria-hidden="true"></i>
                    </a>
                    <a role="menuitem" href=crate::config::app_link(crate::config::routes::SIGN_OUT) class="flex items-center gap-2 px-4 py-2 text-body-md text-paper-ink hover:bg-paper-soft">
                        {tr("help-ui-signout")}
                        <i class="ph-bold ph-caret-right text-fine text-brand" aria-hidden="true"></i>
                    </a>
                </div>
            </Show>
        </div>
    }
}

/// Header, page and footer for every help page. The language picker defaults to the browser language on a first visit and
/// remembers the choice. The Contact Us band above the footer is left out on the contact page itself.
#[component]
pub fn HelpShell(children: Children, #[prop(default = Look::Dark)] look: Look, #[prop(default = true)] contact_band: bool) -> impl IntoView {
    let dark = look == Look::Dark;
    let current = crate::i18n::language();
    let profiles = crate::store::use_profile_store();
    // on its own host the Help Centre cannot read the app's storage: the app leaves the profile's name in a cookie (services/help_cookie.rs)
    let hello = if crate::config::on_help_host() { Some(crate::services::help_cookie::read()) } else { None };
    let signed_in = match &hello {
        Some(name) => name.is_some(),
        None => has_profile(),
    };
    let active_name = Signal::derive(move || {
        if let Some(name) = &hello {
            return name.clone().unwrap_or_default();
        }
        let id = profiles.active_profile_id.get();
        profiles.profiles.with(|list| list.iter().find(|p| Some(&p.id) == id.as_ref()).or(list.first()).map(|p| p.name.clone()).unwrap_or_default())
    });
    let consent = crate::consent::use_ui();
    let on_language = move |ev: leptos::ev::Event| {
        if let Some(select) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) {
            crate::i18n::set_language(&select.value());
        }
    };
    let legal = [(crate::config::routes::TERMS, "help-ui-legal-terms"), (crate::config::routes::PRIVACY, "help-ui-legal-privacy"), (crate::config::routes::COOKIES, "help-ui-legal-cookies"), (crate::config::routes::DMCA, "help-ui-legal-dmca"), (crate::config::routes::DISCLAIMER, "help-ui-legal-disclaimer"), (crate::config::routes::CREDITS, "help-ui-legal-credits")];

    let header_bg = if dark { "bg-void text-ink" } else { "bg-paper text-paper-ink" };
    let logo = if dark { crate::brand::LOGO_WORDMARK } else { crate::brand::LOGO_WORDMARK_PRIMARY };
    let divider = if dark { "bg-ink/60" } else { "bg-paper-line" };
    let outline = if dark { "border-ink/70 text-ink hover:bg-ink/10" } else { "border-paper-edge text-paper-ink hover:bg-paper-soft" };
    let footer_bg = if dark { "bg-radial-[at_100%_0%] from-brand-deep to-void to-60% text-ink" } else { "bg-paper-wash text-paper-mute" };
    let select_look = if dark { "border-ink/40 bg-void/40 text-ink-soft focus:border-ink" } else { "border-paper-edge bg-paper text-paper-ink focus:border-paper-ink" };
    let caret = if dark { "text-ink-soft" } else { "text-paper-ink" };
    let link_look = if dark { "text-ink-silver hover:text-ink" } else { "text-paper-mute hover:text-paper-ink" };

    view! {
        <div class="flex min-h-screen flex-col bg-paper font-sans text-paper-ink selection:bg-brand selection:text-ink">
            <header class=format!("{header_bg} print:hidden")>
                <div class="mx-auto flex h-16 max-w-[1280px] items-center justify-between gap-4 px-6 md:px-12">
                    <div class="flex min-w-0 items-center gap-4">
                        <a href=crate::config::app_link("/browse") class="flex shrink-0 items-center" aria-label=crate::brand::NAME>
                            <img src=logo alt=crate::brand::NAME class="h-7 w-auto" />
                        </a>
                        <span class=format!("h-6 w-px {divider}") aria-hidden="true"></span>
                        <a href="/help" class="truncate text-body-lg hover:underline">{tr("help-ui-center")}</a>
                    </div>
                    <div class="flex shrink-0 items-center gap-3">
                        {if signed_in {
                            view! { <AccountMenu name=active_name dark=dark /> }.into_any()
                        } else {
                            view! {
                                <a href=crate::config::app_link("/browse") class=format!("hidden rounded-button border px-4 py-1.5 text-fine font-semibold sm:block {outline}")>{tr("help-ui-browse")}</a>
                                <a href=crate::config::app_link("/login") class="rounded-button bg-brand px-4 py-1.5 text-fine font-semibold text-ink hover:bg-brand-hover">{tr("help-ui-signin")}</a>
                            }
                            .into_any()
                        }}
                    </div>
                </div>
            </header>

            <div class="flex-1">{children()}</div>

            {(contact_band && !dark).then(|| view! {
                <section class="px-6 pb-16 pt-6 text-center">
                    <h2 class="mb-6 text-title font-bold text-paper-ink">{tr("help-ui-need-more")}</h2>
                    <a href="/help/contact" class="inline-block min-w-52 rounded-button bg-void px-10 py-3.5 text-body-md font-bold text-ink hover:bg-brand-deep">{tr("help-ui-contact-us")}</a>
                </section>
            })}

            <footer class=format!("{footer_bg} print:hidden")>
                <div class="mx-auto max-w-[1280px] px-6 py-12 md:px-12">
                    {(contact_band && dark).then(|| view! {
                        <div class="mb-8 flex flex-wrap items-center gap-6 border-b border-ink/20 pb-8">
                            <h2 class="text-heading font-bold">{tr("help-ui-need-more")}</h2>
                            <a href="/help/contact" class="rounded-button bg-ink px-6 py-3 text-body-sm font-bold text-void hover:bg-ink-soft">{tr("help-ui-contact-us")}</a>
                        </div>
                    })}
                    <label for="help-language" class="sr-only">{tr("help-ui-language-label")}</label>
                    <div class="relative mb-8 inline-block">
                        <select
                            id="help-language"
                            on:change=on_language
                            class=format!("min-w-48 cursor-pointer appearance-none border py-2.5 ps-4 pe-10 text-body-sm focus:outline-none {select_look}")
                        >
                            {crate::i18n::AVAILABLE
                                .iter()
                                .map(|code| view! { <option value=*code selected=*code == current class="bg-paper text-paper-ink">{native_name(code)}</option> })
                                .collect_view()}
                        </select>
                        <i class=format!("ph-fill ph-caret-down pointer-events-none absolute end-3 top-1/2 -translate-y-1/2 text-fine {caret}")></i>
                    </div>
                    <nav class="flex flex-col items-start gap-5 text-body-sm">
                        {legal.into_iter().map(|(href, key)| view! { <a href=href class=format!("w-fit underline-offset-2 hover:underline {link_look}")>{tr(key)}</a> }).collect_view()}
                        <button on:click=move |_| consent.open_center() class=format!("w-fit text-start underline-offset-2 hover:underline {link_look}")>{tr("cookie-link")}</button>
                    </nav>
                </div>
            </footer>
        </div>
    }
}
