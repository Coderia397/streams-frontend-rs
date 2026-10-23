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
