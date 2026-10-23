//! Layout tokens that live in CSS (`style/input.css`) and are also needed by Rust code that positions overlays.

/// Page side margin in px, read from the `--app-x` custom property of the root element.
pub fn app_x() -> f64 {
    css_px("--app-x").unwrap_or(48.0)
}

/// The nav's search field when open, in px (`--search-open-w`).
pub fn search_open_w() -> f64 {
    css_px("--search-open-w").unwrap_or(400.0)
}

fn css_px(name: &str) -> Option<f64> {
    let window = web_sys::window()?;
    let root = window.document()?.document_element()?;
    let style = window.get_computed_style(&root).ok()??;
    let raw = style.get_property_value(name).ok()?;
    raw.trim().trim_end_matches("px").trim().parse().ok()
}

/// How long the pointer must rest on a card (a tile, a Top 10 card) before its hover card opens, in milliseconds: moving across a row on the way to a button opens nothing.
pub const CARD_HOVER_DELAY_MS: i32 = 350;
