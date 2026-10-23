//! Age rating labels in two systems. The British one reads U, PG, 12, 15, 18; the American one reads G, PG, PG-13, R, NC-17 (films) and
//! TV-Y ... TV-MA (series). A title shows the real certificate of the system the person reads; when the title only has the other system's
//! certificate, a neutral age is shown instead ("All", "8+", "12+", "15+", "18+"): nothing is converted into a label of the other system
//! and nothing is guessed. All badges share one look (see `colors`), so the two systems differ in their words only.
//! Which system a person reads: their choice in the settings, else the country of the device this session signed in from (resolved by the
//! database, `my_rating_system`), else the country the connection came from, else British.
use super::settings::{bbfc_rank, us_to_bbfc_rank};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingSystem {
    Uk,
    Us,
}

impl RatingSystem {
    pub fn key(self) -> &'static str {
        match self {
            Self::Uk => "uk",
            Self::Us => "us",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "uk" => Some(Self::Uk),
            "us" => Some(Self::Us),
            _ => None,
        }
    }
}

/// Rating labels by country, the same data as public.rating_system_by_country. Used when the database has not answered; a country that
/// is not listed reads the British labels.
pub const SYSTEM_BY_COUNTRY: &[(&str, RatingSystem)] = &[
    ("US", RatingSystem::Us), ("CA", RatingSystem::Us), ("MX", RatingSystem::Us), ("PR", RatingSystem::Us), ("GU", RatingSystem::Us),
    ("VI", RatingSystem::Us), ("AS", RatingSystem::Us), ("MP", RatingSystem::Us), ("GB", RatingSystem::Uk), ("IE", RatingSystem::Uk),
];

pub fn system_for_country(country: &str) -> RatingSystem {
    SYSTEM_BY_COUNTRY.iter().find(|(c, _)| c.eq_ignore_ascii_case(country.trim())).map_or(RatingSystem::Uk, |(_, s)| *s)
}

/// What a badge says, and how to read it aloud.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Say {
    /// The title's own certificate in the reader's system ("15", "PG-13", "TV-MA")
    Rated(String),
    /// Suitable for everyone (neutral age band 1)
    All,
    /// Neutral "N+" age taken from the other system's certificate
    Plus(u8),
    NotRated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeLabel {
    /// 0 not rated, 1 to 5 from the lowest age to the highest (colour and order)
    pub rank: u8,
    pub say: Say,
}

fn neutral(rank: u8) -> AgeLabel {
    let say = match rank {
        1 => Say::All,
        2 => Say::Plus(8),
        3 => Say::Plus(12),
        4 => Say::Plus(15),
        _ => Say::Plus(18),
    };
    AgeLabel { rank, say }
}

/// The label for a title with these certificates (either may be missing) for a reader of `system`.
pub fn age_label(system: RatingSystem, gb: Option<&str>, us: Option<&str>) -> AgeLabel {
    let gb_rank = gb.and_then(bbfc_rank).map(|r| r + 1);
    let us_rank = us.and_then(us_to_bbfc_rank).map(|r| r + 1);
    let (own, own_rank, other_rank) = match system {
        RatingSystem::Uk => (gb, gb_rank, us_rank),
        RatingSystem::Us => (us, us_rank, gb_rank),
    };
    match (own, own_rank, other_rank) {
        (Some(text), Some(rank), _) => AgeLabel { rank, say: Say::Rated(text.trim().to_uppercase()) },
        (_, _, Some(rank)) => neutral(rank),
        _ => AgeLabel { rank: 0, say: Say::NotRated },
    }
}

/// The text of an age limit step in the reader's system. The limit itself is kept by its British key (profile_settings.age_rating);
/// the American words are the nearest ones for the limit.
pub fn limit_label(system: RatingSystem, key: &str) -> String {
    match (system, key) {
        (RatingSystem::Us, "U") => "G".to_string(),
        (RatingSystem::Us, "PG") => "PG".to_string(),
        (RatingSystem::Us, "12") => "PG-13".to_string(),
        (RatingSystem::Us, "15") => "R".to_string(),
        (RatingSystem::Us, "18") => "NC-17".to_string(),
        _ => key.to_string(),
    }
}

// Colours. The British ratings use the colours the classification is known by (green U, yellow PG, orange 12, pink 15, red 18, blue R18).
// The American ratings have no official colours, so they use the same logical scale: green for all ages, yellow, orange, red, and a dark red
// for NC-17. Where white text on the official colour is too faint to read, the text is near black instead (each pair is tested).
const GREEN: (&str, &str) = ("#00a826", "#0b0b0c"); // colour: the classification board\'s own colour (a value, for the contrast test)
const YELLOW: (&str, &str) = ("#f5a800", "#0b0b0c"); // colour: the classification board\'s own colour (a value, for the contrast test)
const ORANGE: (&str, &str) = ("#f05a1a", "#0b0b0c"); // colour: the classification board\'s own colour (a value, for the contrast test)
const PINK: (&str, &str) = ("#eb287f", "#0b0b0c"); // colour: the classification board\'s own colour (a value, for the contrast test)
const RED: (&str, &str) = ("#dc1420", "#ffffff"); // colour: the classification board\'s own colour (a value, for the contrast test)
const DARK_RED: (&str, &str) = ("#7f1d1d", "#ffffff"); // colour: the classification board\'s own colour (a value, for the contrast test)
const BLUE: (&str, &str) = ("#0060cf", "#ffffff"); // colour: the classification board\'s own colour (a value, for the contrast test)
const GREY: (&str, &str) = ("#6b6b72", "#ffffff"); // colour: the classification board\'s own colour (a value, for the contrast test)
/// Every colour pair in use, for the contrast test.
pub const PALETTE: [(&str, &str); 8] = [GREEN, YELLOW, ORANGE, PINK, RED, DARK_RED, BLUE, GREY];

/// Fill and text colour by rank, the British scale: 1 U green, 2 PG yellow, 3 12 orange, 4 15 pink, 5 18 red, 0 not rated grey.
pub const fn colors(rank: u8) -> (&'static str, &'static str) {
    match rank {
        1 => GREEN,
        2 => YELLOW,
        3 => ORANGE,
        4 => PINK,
        5 => RED,
        _ => GREY,
    }
}

/// Colours of a label for a reader of `system`. An American certificate has its own logical colour (R and TV-MA red, PG-13 and TV-14 orange,
/// NC-17 dark red); every other label takes the colour of its rank. R18 is blue.
pub fn colors_for(system: RatingSystem, label: &AgeLabel) -> (&'static str, &'static str) {
    if let Say::Rated(text) = &label.say {
        if text == "R18" {
            return BLUE;
        }
        if system == RatingSystem::Us {
            return match text.as_str() {
                "G" | "TV-Y" | "TV-G" => GREEN,
                "PG" | "TV-PG" | "TV-Y7" | "TV-Y7-FV" => YELLOW,
                "PG-13" | "TV-14" => ORANGE,
                "R" | "TV-MA" => RED,
                "NC-17" => DARK_RED,
                _ => colors(label.rank),
            };
        }
    }
    colors(label.rank)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(hex: &str) -> f64 {
        let v = |i: usize| {
            let c = u8::from_str_radix(&hex[1 + i * 2..3 + i * 2], 16).unwrap() as f64 / 255.0;
            if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * v(0) + 0.7152 * v(1) + 0.0722 * v(2)
    }

    fn contrast(a: &str, b: &str) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn every_badge_colour_pair_is_readable() {
        for (bg, fg) in PALETTE {
            assert!(contrast(bg, fg) >= 4.5, "{bg} with {fg} is {:.2}", contrast(bg, fg));
        }
    }

    #[test]
    fn american_ratings_have_their_own_logical_colours() {
        let label = |us: &str| age_label(RatingSystem::Us, None, Some(us));
        assert_eq!(colors_for(RatingSystem::Us, &label("G")), GREEN);
        assert_eq!(colors_for(RatingSystem::Us, &label("PG-13")), ORANGE);
        assert_eq!(colors_for(RatingSystem::Us, &label("R")), RED);
        assert_eq!(colors_for(RatingSystem::Us, &label("TV-MA")), RED);
        assert_eq!(colors_for(RatingSystem::Us, &label("NC-17")), DARK_RED);
        // the British reader keeps the British scale for the same title
        assert_eq!(colors_for(RatingSystem::Uk, &age_label(RatingSystem::Uk, Some("15"), Some("R"))), PINK);
        assert_eq!(colors_for(RatingSystem::Uk, &age_label(RatingSystem::Uk, Some("R18"), None)), BLUE);
    }

    #[test]
    fn a_british_reader_sees_the_british_certificate_first() {
        assert_eq!(age_label(RatingSystem::Uk, Some("15"), Some("TV-MA")), AgeLabel { rank: 4, say: Say::Rated("15".into()) });
        assert_eq!(age_label(RatingSystem::Uk, Some("12A"), None).say, Say::Rated("12A".into()));
    }

    #[test]
    fn an_american_reader_sees_the_american_certificate_first() {
        assert_eq!(age_label(RatingSystem::Us, Some("15"), Some("PG-13")), AgeLabel { rank: 3, say: Say::Rated("PG-13".into()) });
        assert_eq!(age_label(RatingSystem::Us, None, Some("tv-ma")).say, Say::Rated("TV-MA".into()));
    }

    #[test]
    fn only_the_other_systems_certificate_gives_a_neutral_age() {
        assert_eq!(age_label(RatingSystem::Us, Some("U"), None).say, Say::All);
        assert_eq!(age_label(RatingSystem::Us, Some("PG"), None).say, Say::Plus(8));
        assert_eq!(age_label(RatingSystem::Us, Some("12A"), None).say, Say::Plus(12));
        assert_eq!(age_label(RatingSystem::Uk, None, Some("R")).say, Say::Plus(15));
        assert_eq!(age_label(RatingSystem::Uk, None, Some("NC-17")).say, Say::Plus(18));
    }

    #[test]
    fn no_usable_certificate_is_not_rated() {
        for (gb, us) in [(None, None), (Some(""), Some("NR")), (Some("X"), Some("Unrated"))] {
            assert_eq!(age_label(RatingSystem::Uk, gb, us), AgeLabel { rank: 0, say: Say::NotRated });
            assert_eq!(age_label(RatingSystem::Us, gb, us).rank, 0);
        }
    }

    #[test]
    fn colour_follows_the_rank_in_both_systems() {
        assert_eq!(age_label(RatingSystem::Uk, Some("18"), None).rank, age_label(RatingSystem::Us, None, Some("NC-17")).rank);
        assert_eq!(colors(5), colors(age_label(RatingSystem::Us, Some("18"), None).rank));
    }

    #[test]
    fn the_country_table_decides_the_system() {
        assert_eq!(system_for_country("us"), RatingSystem::Us);
        assert_eq!(system_for_country("CA"), RatingSystem::Us);
        assert_eq!(system_for_country("GB"), RatingSystem::Uk);
        assert_eq!(system_for_country("JP"), RatingSystem::Uk);
        assert_eq!(system_for_country(""), RatingSystem::Uk);
    }

    #[test]
    fn limits_read_in_the_system_of_the_reader() {
        assert_eq!(limit_label(RatingSystem::Uk, "15"), "15");
        assert_eq!(limit_label(RatingSystem::Us, "U"), "G");
        assert_eq!(limit_label(RatingSystem::Us, "12"), "PG-13");
        assert_eq!(limit_label(RatingSystem::Us, "18"), "NC-17");
    }
}
