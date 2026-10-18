use wasm_bindgen::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmbientRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[wasm_bindgen(module = "/interop/dist/streams-interop.js")]
extern "C" {
    #[wasm_bindgen(js_name = getCachedAmbientColor)]
    fn js_get_cached_ambient_color(url: &str) -> String;

    #[wasm_bindgen(js_name = getLastAmbientColor)]
    fn js_get_last_ambient_color() -> String;

    #[wasm_bindgen(js_name = extractAmbientColor)]
    fn js_extract_ambient_color(url: &str, callback: &js_sys::Function);

    #[wasm_bindgen(js_name = applyAmbientColor)]
    fn js_apply_ambient_color(r: u8, g: u8, b: u8);

    #[wasm_bindgen(js_name = setThemeColor)]
    fn js_set_theme_color(r: u8, g: u8, b: u8);
}

pub fn get_cached_ambient_color(url: &str) -> Option<(u8, u8, u8)> {
    if url.is_empty() {
        return None;
    }
    let res = js_get_cached_ambient_color(url);
    if res.is_empty() {
        None
    } else {
        serde_json::from_str::<AmbientRGB>(&res).ok().map(|c| (c.r, c.g, c.b))
    }
}

pub fn get_last_ambient_color() -> Option<(u8, u8, u8)> {
    let res = js_get_last_ambient_color();
    if res.is_empty() {
        Some((16, 21, 25))
    } else {
        serde_json::from_str::<AmbientRGB>(&res)
            .ok()
            .map(|c| (c.r, c.g, c.b))
            .or(Some((16, 21, 25)))
    }
}

pub fn extract_ambient_color<F: FnOnce(Option<(u8, u8, u8)>) + 'static>(url: &str, cb: F) {
    if url.is_empty() {
        cb(None);
        return;
    }
    let closure = Closure::once_into_js(move |val: JsValue| {
        let rgb = val.as_string()
            .and_then(|s| serde_json::from_str::<AmbientRGB>(&s).ok())
            .map(|c| (c.r, c.g, c.b));
        cb(rgb);
    });
    js_extract_ambient_color(url, closure.as_ref().unchecked_ref());
}

pub fn apply_ambient_color(r: u8, g: u8, b: u8) {
    js_apply_ambient_color(r, g, b);
}

pub fn set_theme_color(r: u8, g: u8, b: u8) {
    js_set_theme_color(r, g, b);
}
