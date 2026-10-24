//! The order of the clips (the owner's feed suggestions 2 and 4, 2026-10-24, note 116): the titles of the rows the engine makes for the
//! profile's Home, taken in turns so the feed mixes what is watched today, what suits the profile, the page's themes, series that
//! everyone is watching and new releases; then spaced so two clips in a row do not share their first genre and films and series take
//! turns where the list allows. A title comes once.
use crate::services::ai_engine::{FeedRow, VibeItem};
use std::collections::HashSet;

/// How far ahead the spacing looks for a clip of another genre or kind.
const LOOK_AHEAD: usize = 6;

/// Where a title comes from. The lanes take turns in `TURNS`, one title each; an empty lane is passed.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Lane {
    /// Watched today: the New & Popular list and the Top 10 rows
    Trending,
    /// Rows made for the profile (Top Picks, Because You Watched, More Like ...), which follow what it watches, saves and swipes
    Personal,
    /// Theme, season and collection rows
    Other,
    /// Everyone's Watching
    Popular,
    New,
}

const TURNS: [Lane; 8] = [Lane::Trending, Lane::Personal, Lane::Other, Lane::Popular, Lane::New, Lane::Personal, Lane::Trending, Lane::Other];

/// A title's key: (is a series, TMDB id).
pub fn key(item: &VibeItem) -> (bool, u32) {
    (is_tv(item), item.id)
}

fn is_tv(item: &VibeItem) -> bool {
    item.media_type.as_deref() == Some("tv")
}

fn first_genre(item: &VibeItem) -> &str {
    item.genres.as_deref().unwrap_or("").split(',').next().unwrap_or("").trim()
}

fn lane_of(row: &FeedRow) -> Lane {
    match (row.row_type.as_str(), row.kind.as_str()) {
        ("top_ten", _) => Lane::Trending,
        (_, "personal") => Lane::Personal,
        (_, "popular") => Lane::Popular,
        (_, "new") => Lane::New,
        _ => Lane::Other,
    }
}

/// The rows of one lane read across: the first title of each row, then the second of each, and so on.
fn across(rows: &[&[VibeItem]]) -> Vec<VibeItem> {
    let longest = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    (0..longest).flat_map(|i| rows.iter().filter_map(move |r| r.get(i).cloned())).collect()
}

/// The clips after `shown`: `trending` (the New & Popular list) and the Home rows in turns, then spaced. Titles with no picture are
/// left out (a clip shows the picture until its video plays).
pub fn order(trending: &[VibeItem], rows: &[FeedRow], shown: &HashSet<(bool, u32)>) -> Vec<VibeItem> {
    let lane = |l: Lane| -> Vec<VibeItem> {
        let mut parts: Vec<&[VibeItem]> = Vec::new();
        if l == Lane::Trending {
            parts.push(trending);
        }
        parts.extend(rows.iter().filter(|r| lane_of(r) == l).map(|r| r.items.as_slice()));
        across(&parts)
    };
    let mut lanes: Vec<(Lane, std::collections::VecDeque<VibeItem>)> =
        [Lane::Trending, Lane::Personal, Lane::Other, Lane::Popular, Lane::New].into_iter().map(|l| (l, lane(l).into())).collect();

    // the lanes in turns, each title once
    let mut taken: HashSet<(bool, u32)> = shown.clone();
    let mut merged: Vec<VibeItem> = Vec::new();
    loop {
        let mut any = false;
        for turn in TURNS {
            let Some((_, queue)) = lanes.iter_mut().find(|(l, _)| *l == turn) else { continue };
            while let Some(mut item) = queue.pop_front() {
                if item.backdrop_path.as_deref().is_some_and(|p| !p.is_empty()) && taken.insert(key(&item)) {
                    // the New & Popular list carries no tags: its genres stand in, as on Netflix's line ("Action • Comedy")
                    if item.tags.is_empty() && item.vibe_pills.is_empty() {
                        item.tags = item.genres.as_deref().unwrap_or("").split(',').map(str::trim).filter(|g| !g.is_empty()).take(3).map(String::from).collect();
                    }
                    merged.push(item);
                    any = true;
                    break;
                }
            }
        }
        if !any && lanes.iter().all(|(_, q)| q.is_empty()) {
            break;
        }
    }
    spaced(merged)
}

/// Two clips in a row do not share their first genre, and after two films or two series the next is the other kind, when one of the
/// next LOOK_AHEAD titles allows it; otherwise the next in line comes.
fn spaced(mut rest: Vec<VibeItem>) -> Vec<VibeItem> {
    let mut out: Vec<VibeItem> = Vec::with_capacity(rest.len());
    while !rest.is_empty() {
        let fits = |c: &VibeItem, out: &[VibeItem]| {
            let other_genre = out.last().is_none_or(|p| first_genre(p).is_empty() || first_genre(p) != first_genre(c));
            let other_kind = match out {
                [.., a, b] if is_tv(a) == is_tv(b) => is_tv(c) != is_tv(b),
                _ => true,
            };
            other_genre && other_kind
        };
        let at = (0..rest.len().min(LOOK_AHEAD))
            .find(|&i| fits(&rest[i], &out))
            .or_else(|| (0..rest.len().min(LOOK_AHEAD)).find(|&i| out.last().is_none_or(|p| first_genre(p) != first_genre(&rest[i]))))
            .unwrap_or(0);
        out.push(rest.remove(at));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, tv: bool, genres: &str) -> VibeItem {
        VibeItem {
            id,
            media_type: Some(if tv { "tv" } else { "movie" }.into()),
            genres: Some(genres.into()),
            backdrop_path: Some(format!("/{id}.jpg")),
            ..Default::default()
        }
    }

    fn row(kind: &str, row_type: &str, items: Vec<VibeItem>) -> FeedRow {
        FeedRow { kind: kind.into(), row_type: row_type.into(), items, ..Default::default() }
    }

    #[test]
    fn lanes_take_turns_and_titles_come_once() {
        let trending = vec![item(1, false, "Action"), item(2, false, "Comedy")];
        let rows = vec![
            row("personal", "vibe", vec![item(10, true, "Drama"), item(1, false, "Action")]),
            row("theme", "vibe", vec![item(20, false, "Horror")]),
            row("popular", "vibe", vec![item(30, true, "Crime")]),
            row("new", "vibe", vec![item(40, false, "Romance")]),
        ];
        let ids: Vec<u32> = order(&trending, &rows, &HashSet::new()).iter().map(|i| i.id).collect();
        assert_eq!(ids, vec![1, 10, 20, 30, 40, 2]);
    }

    #[test]
    fn shown_titles_and_titles_without_a_picture_are_left_out() {
        let mut bare = item(3, false, "Drama");
        bare.backdrop_path = None;
        let trending = vec![item(1, false, "Action"), item(2, true, "Comedy"), bare];
        let shown: HashSet<(bool, u32)> = [(false, 1)].into_iter().collect();
        let ids: Vec<u32> = order(&trending, &[], &shown).iter().map(|i| i.id).collect();
        assert_eq!(ids, vec![2]);
    }

    #[test]
    fn genres_stand_in_for_missing_tags() {
        let mut tagged = item(2, true, "Drama");
        tagged.tags = vec!["Witty".into()];
        let got = order(&[item(1, false, "Animation, Adventure, Fantasy, Family"), tagged], &[], &HashSet::new());
        assert_eq!(got[0].tags, vec!["Animation", "Adventure", "Fantasy"]);
        assert_eq!(got[1].tags, vec!["Witty"]);
    }

    #[test]
    fn spacing_breaks_up_genres_and_kinds() {
        let list = vec![item(1, false, "Action"), item(2, false, "Action"), item(3, false, "Comedy"), item(4, true, "Drama"), item(5, false, "Horror")];
        let ids: Vec<u32> = spaced(list).iter().map(|i| i.id).collect();
        // 2 shares 1's genre, so 3 comes second; after two films a series (4)
        assert_eq!(ids, vec![1, 3, 4, 2, 5]);
    }
}
