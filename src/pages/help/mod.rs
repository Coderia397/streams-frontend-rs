//! Help center: home, articles, search results and contact. The structure is in data/help.rs and the words are in i18n/en.ftl.
//! Pages share a light shell (dark bar on top, dark footer) because the Netflix help center is a light site; colors come from
//! the paper tokens in style/brand.css.
mod actions;
mod article;
mod contact;
mod home;
mod legal;
pub(crate) mod markup;
mod recent;
mod results;
mod search;
mod shell;
mod toc;
mod widgets;

pub use article::ArticlePage;
pub use contact::ContactPage;
pub use home::HelpPage;
pub use legal::LegalPage;
pub use results::ResultsPage;

use fluent_bundle::FluentValue;

/// Interface text with the values every help message may use: the product name and the two mailboxes.
pub fn tr(key: &str) -> String {
    tr_with(key, &[])
}

pub fn tr_with(key: &str, extra: &[(&str, FluentValue)]) -> String {
    let mut args: Vec<(&str, FluentValue)> = vec![
        ("brand", FluentValue::from(crate::brand::NAME)),
        ("help_email", FluentValue::from(crate::brand::HELP_EMAIL)),
        ("contribute_email", FluentValue::from(crate::brand::CONTRIBUTE_EMAIL)),
    ];
    args.extend(extra.iter().cloned());
    crate::i18n::t_args(key, &args)
}

/// A policy text with the product name and the one contact address filled in.
pub(crate) fn legal_text(key: &str) -> String {
    crate::i18n::t_args(
        key,
        &[("brand", FluentValue::from(crate::brand::NAME)), ("support_email", FluentValue::from(crate::brand::HELP_EMAIL)), ("help_email", FluentValue::from(crate::brand::HELP_EMAIL))],
    )
}

/// One part (title, summary, keywords, body) of an article.
pub fn art(slug: &str, part: &str) -> String {
    tr(&crate::data::help::key(slug, part))
}

pub fn set_site_title(page: &str) {
    shell::set_site_title(page);
}

pub fn set_page_title(page: &str) {
    shell::set_title(page);
}

pub fn set_page_description(text: &str) {
    shell::set_description(text);
}

pub fn encode(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap_or_default()
}

/// A mailto link to the help mailbox with a subject and a body that ends with the visitor's browser details.
pub(crate) fn mailto(subject: &str, intro: &str) -> String {
    let window = web_sys::window();
    let (agent, page, screen) = match &window {
        Some(w) => (
            w.navigator().user_agent().unwrap_or_default(),
            w.location().href().unwrap_or_default(),
            format!("{} x {}", w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0), w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0)),
        ),
        None => Default::default(),
    };
    let body = format!("{intro}\n\n\n--- {} ---\nPage: {page}\nBrowser: {agent}\nLanguage: {}\nWindow: {screen}\n", tr("help-ui-mail-details"), crate::i18n::language());
    format!("mailto:{}?subject={}&body={}", crate::brand::HELP_EMAIL, encode(subject), encode(&body))
}
