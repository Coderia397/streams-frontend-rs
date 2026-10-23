use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovieRating {
    Like,
    Dislike,
    Love,
}

/// The "Love this!" icon: two thumbs up, the one in front low on the left, the one behind high on the right and cut away round the front one, so a gap shows between them
/// (a mask, so it needs no background colour and works on any button state).
#[component]
pub fn DoubleThumbsUpIcon(
    #[prop(default = 22)] size: u32,
    /// the solid icon (a rating that is set) instead of the outline
    #[prop(default = false)] filled: bool,
    /// kept for callers that passed a background colour; the gap is cut with a mask now
    #[prop(default = String::new())] mask_color: String,
) -> impl IntoView {
    let _ = mask_color;
    let s = size as f64;
    let (dx, dy) = ((s * 0.55).round(), (s * 0.45).round());
    let gap = s * 0.58;
    // centre of the front thumb, in the coordinates of the thumb behind it
    let (cx, cy) = (s / 2.0 - dx + s * 0.02, s / 2.0 + dy - s * 0.02);
    let class = if filled { "ph-fill ph-thumbs-up" } else { "ph-bold ph-thumbs-up" };
    let behind_mask = format!("radial-gradient(circle at {cx:.1}px {cy:.1}px, transparent {gap:.1}px, #000 {:.1}px)", gap + 0.6);
    view! {
        <div class="relative inline-flex" style=format!("width: {}px; height: {}px;", s + dx, s + dy)>
            <div class="absolute" style=format!("left: {dx}px; top: 0; -webkit-mask-image: {behind_mask}; mask-image: {behind_mask};")>
                <i class=class style=format!("font-size: {size}px;")></i>
            </div>
            <div class="absolute" style=format!("left: 0; top: {dy}px;")>
                <i class=class style=format!("font-size: {size}px;")></i>
            </div>
        </div>
    }
}

#[component]
pub fn RatingIcon(
    rating: Option<MovieRating>,
    #[prop(default = 22)] size: u32,
    #[prop(default = "var(--color-panel)".to_string())] mask_color: String,
) -> impl IntoView {
    view! {
        {match rating {
            Some(MovieRating::Love) => view! {
                <DoubleThumbsUpIcon size=size mask_color=mask_color />
            }.into_any(),
            Some(MovieRating::Dislike) => view! {
                <i class="ph-bold ph-thumbs-down" style=format!("font-size: {}px;", size)></i>
            }.into_any(),
            _ => view! {
                <i class="ph-bold ph-thumbs-up" style=format!("font-size: {}px;", size)></i>
            }.into_any(),
        }}
    }
}
