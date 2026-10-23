//! TMDB, the source of title data: the shared plumbing here (address, key, language, lenient fields), the parts by subject, all re-exported
//! so `services::tmdb::MediaItem` and the other paths used across the app keep working.
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::models::mapping::{map_genre_key_to_tmdb, MediaKind};

const TMDB_BASE_URL: &str = crate::config::TMDB_API_URL;

const TMDB_API_KEY: &str = crate::config::TMDB_API_KEY;
const TMDB_IMAGE_BASE: &str = crate::config::TMDB_IMAGE_URL;

/// Adds the interface language to a catalog request so titles, summaries and genre names come back translated. Trailers and
/// logos are left alone because their language filters would hide most of them.
fn localised(url: &str) -> String {
    if url.contains("/videos") || url.contains("/images") {
        return url.to_string();
    }
    let lang = crate::i18n::tmdb_language();
    if url.contains("language=") {
        url.replace("language=en-US", &format!("language={lang}"))
    } else if url.contains('?') {
        format!("{url}&language={lang}")
    } else {
        format!("{url}?language={lang}")
    }
}

fn get(url: &str) -> gloo_net::http::RequestBuilder {
    Request::get(&localised(url))
}

/// An answer on its way or arrived: the body as text, or why it failed.
type Pending = futures::future::Shared<futures::future::LocalBoxFuture<'static, Result<std::rc::Rc<str>, std::rc::Rc<str>>>>;

thread_local! {
    /// Every TMDB address asked for in this session and its answer (or the request still on its way). Before, each card asked for its
    /// title's logo itself and a title in three rows asked three times: opening Home made 233 TMDB requests, most of them repeats.
    static RESPONSES: std::cell::RefCell<std::collections::HashMap<String, Pending>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// The most answers kept; past it the store starts again (a session that browses for hours).
const KEEP_RESPONSES: usize = 2000;

/// A TMDB request made once per address for the session: a second caller gets the same answer, or joins the request on its way; a failed
/// request is forgotten so the next caller tries again. TMDB's data does not change within a visit.
async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, gloo_net::Error> {
    use futures::FutureExt;
    let address = localised(url);
    let pending = RESPONSES.with(|r| {
        let mut r = r.borrow_mut();
        if r.len() >= KEEP_RESPONSES {
            r.clear();
        }
        r.entry(address.clone())
            .or_insert_with(|| {
                let a = address.clone();
                async move {
                    let resp = Request::get(&a).send().await.map_err(|e| std::rc::Rc::from(e.to_string()))?;
                    if !resp.ok() {
                        return Err(std::rc::Rc::from(format!("{} {}", resp.status(), resp.status_text())));
                    }
                    resp.text().await.map(std::rc::Rc::from).map_err(|e| std::rc::Rc::from(e.to_string()))
                }
                .boxed_local()
                .shared()
            })
            .clone()
    });
    match pending.await {
        Ok(text) => serde_json::from_str(&text).map_err(gloo_net::Error::SerdeError),
        Err(why) => {
            RESPONSES.with(|r| r.borrow_mut().remove(&address));
            Err(gloo_net::Error::GlooError(why.to_string()))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TmdbResponse<T> {
    pub page: u32,
    pub results: Vec<T>,
    pub total_pages: u32,
    pub total_results: u32,
}

fn deserialize_null_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

fn deserialize_null_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;
    Ok(opt.unwrap_or(0.0))
}

mod lists;
mod media;
mod titles;
mod videos;

pub use lists::*;
pub use media::*;
pub use titles::*;
pub use videos::*;
