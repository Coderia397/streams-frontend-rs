//! Sharing a title, from anywhere on the site. A component takes the handle once (`use_share()`) and its button calls
//! `share.open(ShareItem::title(..))`; the one sheet, mounted at the app's root (`ShareHost`, lib.rs), opens with that title on every
//! form. Each target is in targets.rs; the system share sheet and copying are in interop/share.ts.
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

pub mod sheet;
pub mod targets;

pub use sheet::ShareHost;

#[wasm_bindgen(module = "/interop/dist/streams-interop.js")]
extern "C" {
    #[wasm_bindgen(js_name = sharePrepare)]
    fn share_prepare(image_url: &str, file_name: &str);
    #[wasm_bindgen(js_name = shareSystem)]
    fn share_system(title: &str, url: &str, image_url: &str) -> js_sys::Promise;
    #[wasm_bindgen(js_name = copyText)]
    fn copy_text(text: &str) -> js_sys::Promise;
}

/// What is shared: the title's name, the link that opens it here, and its poster for the services that take a picture.
#[derive(Clone, Debug, PartialEq)]
pub struct ShareItem {
    pub title: String,
    pub url: String,
    pub image: Option<String>,
}

impl ShareItem {
    pub fn title(is_tv: bool, id: u32, title: impl Into<String>, poster_path: Option<&str>) -> Self {
        Self {
            title: title.into(),
            url: title_url(is_tv, id),
            image: poster_path.map(|p| format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, p)),
        }
    }
}

/// The address of a title's page on this site.
pub fn title_url(is_tv: bool, id: u32) -> String {
    let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
    format!("{origin}/title/{}", crate::pages::title::slug_for(is_tv, id))
}

/// The handle a share button opens the sheet with.
#[derive(Clone, Copy)]
pub struct Share(RwSignal<Option<ShareItem>>);

impl Share {
    pub fn open(&self, item: ShareItem) {
        if let Some(image) = item.image.as_deref() {
            // the picture is fetched now, so a tap on Instagram can share it while the tap still allows the system sheet
            share_prepare(image, &file_name(&item.title));
        }
        self.0.set(Some(item));
    }
}

/// The picture's file name: the title in small letters, words joined by dashes.
fn file_name(title: &str) -> String {
    let words: Vec<String> = title.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_lowercase).collect();
    format!("{}.jpg", if words.is_empty() { "poster".to_string() } else { words.join("-") })
}

/// Called once, at the app's root.
pub fn provide_share() {
    provide_context(Share(RwSignal::new(None)));
}

pub fn use_share() -> Share {
    use_context::<Share>().expect("provide_share() runs in App")
}

#[cfg(test)]
mod tests {
    #[test]
    fn picture_file_name() {
        assert_eq!(super::file_name("Game of Thrones: Season 1!"), "game-of-thrones-season-1.jpg");
        assert_eq!(super::file_name("Amélie"), "amélie.jpg");
        assert_eq!(super::file_name("?!"), "poster.jpg");
    }
}
