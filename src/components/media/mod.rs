//! Everything that shows titles, grouped by kind. Each group's modules are re-exported here, so `media::info_modal` and
//! `media::details::info_modal` name the same module and the paths used across the app keep working.

pub mod buttons;
pub mod cards;
pub mod collections;
pub mod details;
pub mod billboard;
pub mod icons;
pub mod rows;

pub use buttons::{action_buttons, tooltip_wrapper};
pub use cards::{movie_card, movie_card_badges, movie_card_popup, movie_card_rating};
pub use collections::{collection_logo, collection_row, kids_banner};
pub use details::{info_modal, info_modal_episodes, info_modal_recommendations, share_sheet};
pub use billboard::{hero, spotlight_card, spotlight_artwork};
pub use icons::{chip_icons, shelf_icons};
pub use rows::{row, row_scroll, feed_row, top_ten_row, top_ten_numerals, continue_watching_row, continue_watching_options_sheet, vibe_row, title_strip};
