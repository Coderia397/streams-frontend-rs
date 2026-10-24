use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MovieRating {
    Like,
    Dislike,
    Love,
}

/// The "Love this!" icon: IBM Carbon's own pair of thumbs (thumbs-up-double), drawn in the same square as one thumb, so it sits in the
/// same button as I like this (owner's choice, 2026-10-24, note 117).
#[component]
pub fn DoubleThumbsUpIcon(
    #[prop(default = 22)] size: u32,
    /// the solid icon (a rating that is set) instead of the outline
    #[prop(default = false)] filled: bool,
    /// kept for callers that passed a background colour; the pair is one drawing now
    #[prop(default = String::new())] mask_color: String,
) -> impl IntoView {
    let _ = mask_color;
    view! { <AppIcon glyph=if filled { Glyph::LoveFilled } else { Glyph::Love } style=format!("font-size: {size}px;") /> }
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
                <AppIcon glyph=Glyph::ThumbDown style=format!("font-size: {}px;", size) />
            }.into_any(),
            _ => view! {
                <AppIcon glyph=Glyph::ThumbUp style=format!("font-size: {}px;", size) />
            }.into_any(),
        }}
    }
}
