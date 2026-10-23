//! Settings widgets: small helpers for dates and files.

/// A time from the database (ISO text) written for the reader's language.
pub fn local_time(iso: &str) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    if date.get_time().is_nan() {
        return iso.to_string();
    }
    date.to_locale_string(&crate::i18n::language(), &wasm_bindgen::JsValue::UNDEFINED).as_string().unwrap_or_else(|| iso.to_string())
}

/// Offers `text` to the browser as a file to save.
pub fn save_as_file(name: &str, text: &str, mime: &str) {
    use wasm_bindgen::JsCast;
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(a) = doc.create_element("a") else { return };
    let href = format!("data:{mime};charset=utf-8,{}", js_sys::encode_uri_component(text));
    let _ = a.set_attribute("href", &href);
    let _ = a.set_attribute("download", name);
    if let Some(el) = a.dyn_ref::<web_sys::HtmlElement>() {
        el.click();
    }
}


/// A time written for the reader's language: date only.
pub fn local_date(millis: f64) -> String {
    js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(millis)).to_locale_date_string(&crate::i18n::language(), &wasm_bindgen::JsValue::UNDEFINED).as_string().unwrap_or_default()
}


