//! What a profile is offered: genre menus, pages it cannot fill, season pages and the engine's feature switches.
use super::*;

thread_local! {
    /// (route, age limit, surface) -> names of pages the profile cannot fill
    static HIDDEN_PAGES: std::cell::RefCell<std::collections::HashMap<(String, String, String), std::collections::HashSet<String>>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Asks the engine which of `names` a profile has too few titles for (`route` is `genre-support` or `language-support`, `param` the
/// query field that carries the names, `field` the key of the answer). An engine that cannot be reached hides nothing. Answers are kept per limit, route and surface.
async fn hidden_pages(route: &str, param: &str, field: &str, surface: &str, names: &[String]) -> std::collections::HashSet<String> {
    // a profile without a limit is asked as an adult: a page with too little of its own is not offered to anyone
    let limit = crate::pages::settings::prefs::active_rating().unwrap_or("18");
    let key = (route.to_string(), limit.to_string(), surface.to_string());
    if let Some(hit) = HIDDEN_PAGES.with(|c| c.borrow().get(&key).cloned()) {
        return hit;
    }
    let joined: String = js_sys::encode_uri_component(&names.join(",")).into();
    let url = format!("{}/api/{}?surface={}&{}={}&rating={}", base(), route, surface, param, joined, limit);
    let Some(v) = engine_json::<serde_json::Value>(&url, MENU_FRESH_MS).await else { return Default::default() };
    let hidden: std::collections::HashSet<String> = v
        .get(field)
        .and_then(|g| g.as_object())
        .map(|m| m.iter().filter(|(_, e)| e.get("supported").and_then(|s| s.as_bool()) == Some(false)).map(|(n, _)| n.clone()).collect())
        .unwrap_or_default();
    HIDDEN_PAGES.with(|c| c.borrow_mut().insert(key, hidden.clone()));
    hidden
}

/// One genre page of the engine's list: the id of its address (`/browse/genre/<id>`), its name and whether this profile is offered it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GenreEntry {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub supported: bool,
    /// "movie", "tv" or "both" (a page that mixes films and series: the Kids pages both menus share, the events)
    #[serde(default)]
    pub surface: String,
    /// The calendar event the page is the page of, if it is one
    #[serde(default)]
    pub event: Option<String>,
}

/// A genre menu from the engine: the page the genre addresses hang under (`/browse/genre/<id>?bc=<page_id>`), whether it is the Kids profile's, and its pages.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct GenreMenu {
    #[serde(default)]
    pub page_id: u32,
    #[serde(default)]
    pub kids: bool,
    #[serde(default)]
    pub genres: Vec<GenreEntry>,
}

/// The genre pages of a surface from the engine (`kind` is "movie" or "tv"), in picker order, for the profile in use. None when the engine cannot be reached: the page then uses its own list.
pub async fn fetch_genre_list(kind: &str) -> Option<Vec<GenreEntry>> {
    fetch_genre_menu(kind).await.map(|m| m.genres)
}

/// The genre menu of a surface with the page its addresses hang under (the Kids profile has pages of its own, 20000 and 21000). None when the engine cannot be reached.
pub async fn fetch_genre_menu(kind: &str) -> Option<GenreMenu> {
    let url = format!("{}/api/genre-list?surface={}&rating={}", base(), kind, crate::pages::settings::prefs::active_rating().unwrap_or("18"));
    let m: GenreMenu = engine_json(&url, MENU_FRESH_MS).await?;
    (!m.genres.is_empty()).then_some(m)
}

/// The switches the engine sets for the product (`games`): anything not on is off, also when the engine cannot be reached.
pub async fn fetch_features() -> std::collections::HashMap<String, bool> {
    use futures::future::{FutureExt, LocalBoxFuture, Shared};
    type Answer = std::collections::HashMap<String, bool>;
    thread_local! {
        // one request answers every caller (the navbar, the account gate, the phone bars, the details window asked it separately before),
        // kept for five minutes; a failed answer is not kept, so the next caller asks again
        static FEATURES: std::cell::RefCell<Option<(f64, Shared<LocalBoxFuture<'static, Option<Answer>>>)>> = const { std::cell::RefCell::new(None) };
    }
    let now = js_sys::Date::now();
    let pending = FEATURES.with(|f| {
        let mut f = f.borrow_mut();
        if let Some((at, fut)) = f.as_ref() {
            if now - at < 300_000.0 {
                return fut.clone();
            }
        }
        let fut: LocalBoxFuture<'static, Option<Answer>> = async {
            let resp = engine_get(&format!("{}/api/features", base())).await.ok()?;
            if !resp.ok() { return None; }
            let answer = resp.json::<Answer>().await.ok()?;
            store_answer("features", &answer);
            Some(answer)
        }
        .boxed_local();
        let shared = fut.shared();
        *f = Some((now, shared.clone()));
        shared
    });
    match pending.await {
        Some(answer) => answer,
        None => {
            FEATURES.with(|f| *f.borrow_mut() = None);
            Default::default()
        }
    }
}

/// The feature switches this browser saw last (all off when it has seen none), to draw what depends on them at once.
pub fn stored_features() -> std::collections::HashMap<String, bool> {
    stored_answer("features").unwrap_or_default()
}

/// An event in season that has a page of its own (Halloween): its label key and label for the chip, and the id of its genre page.
#[derive(Clone, Debug, Deserialize, serde::Serialize, PartialEq)]
pub struct SeasonChip {
    pub id: String,
    pub label_key: String,
    pub label: String,
    pub genre_id: u32,
}

/// The chips of the events in season for the mobile top bar, from the engine; empty out of season or when the engine cannot be reached, so nothing about a season stays on screen once it is over.
pub async fn fetch_season_chips() -> Vec<SeasonChip> {
    #[derive(Deserialize)]
    struct Answer { chips: Vec<SeasonChip> }
    let chips = engine_json::<Answer>(&format!("{}/api/season-chips", base()), MENU_FRESH_MS).await.map(|a| a.chips);
    if let Some(c) = &chips {
        store_answer("season_chips", c);
    }
    chips.unwrap_or_default()
}

/// The season chips this browser saw last, to draw the chip row at once (the engine's answer replaces them when it comes).
pub fn stored_season_chips() -> Vec<SeasonChip> {
    stored_answer("season_chips").unwrap_or_default()
}

/// The genre pages a limited profile should not be offered (see `hidden_pages`).
pub async fn hidden_genres(surface: &str, names: &[String]) -> std::collections::HashSet<String> {
    hidden_pages("genre-support", "names", "genres", surface, names).await
}

/// The language pages a limited profile should not be offered.
pub async fn hidden_languages(codes: &[String]) -> std::collections::HashSet<String> {
    hidden_pages("language-support", "codes", "languages", "both", codes).await
}
