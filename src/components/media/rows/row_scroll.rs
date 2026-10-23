//! What every horizontal row shares: how a page turn ends, and the buttons at the two edges.
//!
//! A page turn moves the row by one screenful; the last page is cut to the end of the row (so it shows the last cards rather than a half empty page); one more turn forward goes back to
//! the start and one more turn back from the start goes to the end, as in the app, so a row can be paged round for ever.

use leptos::prelude::*;

/// The scroll position a page turn ends at. `step` is the width of a page.
pub fn page_target(forward: bool, current: f64, client_width: f64, scroll_width: f64, step: f64) -> f64 {
    let max = (scroll_width - client_width).max(0.0);
    if forward {
        if current >= max - 4.0 { 0.0 } else { (current + step).min(max) }
    } else if current <= 4.0 {
        max
    } else {
        (current - step).max(0.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
}

/// The control at one edge of a row: a tall area over the margin (and the cards that peek into it), invisible until the row is hovered; hovering the area darkens and blurs the peeking cards,
/// and leaving it lets them clear again slowly. In its middle a round pill with the arrow. `inset` is the top and bottom offset in Tailwind classes; the area reaches past the cards above and below
/// and a little way over the card beside it (14px), so the pointer finds the button without crossing a card (which would open its hover card).
#[component]
pub fn EdgeButton(
    edge: Edge,
    #[prop(into)] visible: Signal<bool>,
    on_click: Callback<()>,
    #[prop(into)] label: String,
    #[prop(default = "-top-4 -bottom-4")] inset: &'static str,
) -> impl IntoView {
    let side = if edge == Edge::Left { "left-0" } else { "right-0" };
    let icon = if edge == Edge::Left { "ph-bold ph-caret-left" } else { "ph-bold ph-caret-right" };
    view! {
        <button
            type="button"
            class=move || format!(
                "edge-btn hover-only hidden sm:flex absolute {inset} {side} z-30 items-center justify-center cursor-pointer select-none border-none outline-none opacity-0 group-hover/row:opacity-100 {}",
                if visible.get() { "" } else { "!pointer-events-none !opacity-0" }
            )
            style="width: calc(var(--app-x, 56px) + 14px);"
            on:click=move |e: web_sys::MouseEvent| { e.stop_propagation(); on_click.run(()); }
            aria-label=label
        >
            <span class="edge-pill"><i class=icon></i></span>
        </button>
    }
}

// ── The endless row ────────────────────────────────────────────────────────────────────────────────
// A row that loops carries a copy of its last cards in front (`data-loop-lead`, hidden at first), a copy of its first cards behind (`data-loop-clone` marks the first of them, `data-loop-first` marks
// the first real card) and the empty margin before its first card (`data-loop-spacer`). The first page turn starts the loop: the copy in front is shown and the empty margin goes, so the end of the
// row shows at the left edge as a sliver, as on every other page. From then on a page turn that runs into a copy is put back, when it has ended, to the same view among the real cards
// without animation, so the row always moves in the direction of the button and never shows a cut.

use wasm_bindgen::JsCast;

struct Geometry {
    /// scroll position of the first page (the first real card at the margin)
    first_page: f64,
    /// distance from the first real card to the first card of the copy behind
    length: f64,
    /// scroll position at which the last real card ends flush with the right margin
    last_page: f64,
    /// width of a page turn
    step: f64,
}

fn element(strip: &web_sys::HtmlElement, selector: &str) -> Option<web_sys::HtmlElement> {
    strip.query_selector(selector).ok()??.dyn_into().ok()
}

fn geometry(strip: &web_sys::HtmlElement, margin: f64) -> Option<Geometry> {
    let clone = element(strip, "[data-loop-clone]")?;
    let first = element(strip, "[data-loop-first]")?;
    let prev: web_sys::HtmlElement = clone.previous_element_sibling()?.dyn_into().ok()?;
    let end_of_last = (prev.offset_left() + prev.offset_width()) as f64;
    let gap = clone.offset_left() as f64 - end_of_last;
    let client = strip.client_width() as f64;
    Some(Geometry {
        first_page: first.offset_left() as f64 - margin,
        length: (clone.offset_left() - first.offset_left()) as f64,
        last_page: (end_of_last - client + margin).max(0.0),
        step: (client - 2.0 * margin + gap).max(100.0),
    })
}

fn go(strip: &web_sys::HtmlElement, x: f64, instant: bool) {
    let opts = web_sys::ScrollToOptions::new();
    opts.set_left(x.max(0.0));
    opts.set_behavior(if instant { web_sys::ScrollBehavior::Instant } else { web_sys::ScrollBehavior::Smooth });
    strip.scroll_to_with_scroll_to_options(&opts);
}

/// Starts the loop of a row (once): shows the copy in front, takes the empty margin away and keeps the first card where it was on the screen.
fn start_loop(strip: &web_sys::HtmlElement) {
    if strip.get_attribute("data-loop-on").is_some() { return; }
    let (Some(lead), Some(spacer)) = (element(strip, "[data-loop-lead]"), element(strip, "[data-loop-spacer]")) else { return };
    let Some(first) = element(strip, "[data-loop-first]") else { return };
    let before = strip.scroll_left() as f64;
    let first_before = first.offset_left() as f64;
    let _ = lead.style().set_property("display", "flex");
    let _ = spacer.style().set_property("display", "none");
    // the first card has moved by the width of the copy (and its gap) less the margin that went: scroll by as much, so that it stays where it was on the screen
    go(strip, before + (first.offset_left() as f64 - first_before), true);
    let _ = strip.set_attribute("data-loop-on", "1");
}

/// A position in the copy in front or in the copy behind, as the same view among the real cards.
fn settle(strip: &web_sys::HtmlElement, margin: f64) {
    let Some(g) = geometry(strip, margin) else { return };
    if strip.get_attribute("data-loop-on").is_none() { return; }
    let cur = strip.scroll_left() as f64;
    if cur < g.first_page - 2.0 {
        go(strip, cur + g.length, true);
    } else if cur > g.first_page + g.length - 2.0 {
        go(strip, cur - g.length, true);
    }
}

/// One page turn of an endless row. Returns false when the row has no copy (the caller then uses `page_target`).
pub fn turn_page(strip: &web_sys::HtmlElement, forward: bool, margin: f64) -> bool {
    if element(strip, "[data-loop-clone]").is_none() || element(strip, "[data-loop-lead]").is_none() { return false; }
    start_loop(strip);
    settle(strip, margin);
    let Some(g) = geometry(strip, margin) else { return false };
    let cur = strip.scroll_left() as f64;
    let target = if forward {
        // the last page ends flush; one more turn goes on into the copy behind, which looks like the first page
        if cur < g.last_page - 4.0 { (cur + g.step).min(g.last_page) } else { g.first_page + g.length }
    } else if cur > g.first_page + 4.0 {
        (cur - g.step).max(g.first_page)
    } else {
        // back from the first page: into the copy in front, which looks like the last page
        g.first_page - g.step
    };
    go(strip, target, false);
    let strip = strip.clone();
    leptos::leptos_dom::helpers::set_timeout(move || settle(&strip, margin), std::time::Duration::from_millis(900));
    true
}

/// Which page a looping row is on and how many pages it has, for the dashes at the top of the row: (index, pages). None for a row without a copy. Positions inside the copies
/// count as the same page among the real cards; the first page seen through the copy behind (after the last page) is page one again.
pub fn page_position(strip: &web_sys::HtmlElement, margin: f64) -> Option<(usize, usize)> {
    let g = geometry(strip, margin)?;
    let last = (g.last_page - g.first_page).max(1.0);
    let pages = ((last / g.step).ceil() as usize + 1).max(2);
    let pos = (strip.scroll_left() as f64 - g.first_page).rem_euclid(g.length.max(1.0));
    let index = if pos > last + g.step * 0.5 {
        0
    } else if pos >= last - 4.0 {
        pages - 1
    } else {
        ((pos / g.step).round() as usize).min(pages - 2)
    };
    Some((index, pages))
}
