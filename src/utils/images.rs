//! TMDB pictures in the size the screen needs: TMDB offers every picture at several widths; `srcset` lists them and `sizes` says how wide
//! the picture is drawn, so the browser fetches the smallest one that is sharp on this screen (a tile on a laptop the 780 px backdrop, the
//! billboard on a tablet 1280 px, the original only where it is needed) instead of one size everywhere (the billboard, the details window
//! and the title page asked for the 3840 px original on every screen).

const BACKDROPS: [(&str, u32); 4] = [("w300", 300), ("w780", 780), ("w1280", 1280), ("original", 3840)];
const POSTERS: [(&str, u32); 5] = [("w185", 185), ("w342", 342), ("w500", 500), ("w780", 780), ("original", 2000)];

/// How wide a row's tile is drawn: the columns of `.tile-width` in style/input.css (3.29 posters a row on phones, then 3 to 8 tiles).
pub const TILE_SIZES: &str = "(max-width: 599px) 29vw, (max-width: 959px) 31vw, (max-width: 1279px) 23vw, (max-width: 1599px) 19vw, (max-width: 1919px) 16vw, (max-width: 2559px) 14vw, 12vw";
/// A poster tile on a phone.
pub const PHONE_POSTER_SIZES: &str = "29vw";
/// The billboard, the details window's picture and the title page's: about the width of the window.
pub const WIDE_SIZES: &str = "100vw";

/// The TMDB backdrop at `url` at every width TMDB offers, for `srcset` (empty for an address that is not TMDB's).
pub fn backdrop_srcset(url: &str) -> String {
    srcset(url, &BACKDROPS)
}

/// The TMDB poster at `url` at every width TMDB offers, for `srcset`.
pub fn poster_srcset(url: &str) -> String {
    srcset(url, &POSTERS)
}

fn srcset(url: &str, steps: &[(&str, u32)]) -> String {
    let base = crate::config::TMDB_IMAGE_URL;
    let Some((_, file)) = url.strip_prefix(base).and_then(|r| r.strip_prefix('/')).and_then(|r| r.split_once('/')) else {
        return String::new();
    };
    steps.iter().map(|(size, width)| format!("{base}/{size}/{file} {width}w")).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tmdb_picture_is_offered_at_every_width() {
        let base = crate::config::TMDB_IMAGE_URL;
        let set = backdrop_srcset(&format!("{base}/w780/abc.jpg"));
        assert_eq!(set, format!("{base}/w300/abc.jpg 300w, {base}/w780/abc.jpg 780w, {base}/w1280/abc.jpg 1280w, {base}/original/abc.jpg 3840w"));
        assert!(backdrop_srcset("https://example.test/picture.jpg").is_empty());
    }
}
