//! Lists of titles: trending, top rated, by genre or language, search, recommendations and the rows built from them.
use super::*;

/// Fetch trending items (used for the hero carousel and home page rows).
pub async fn fetch_trending(kind: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/trending/{}/week?api_key={}&language=en-US",
        TMDB_BASE_URL, kind, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let items = resp.results.into_iter().filter(|i| !i.is_nsfw() && (i.backdrop_path.is_some() || i.poster_path.is_some())).collect();
    Ok(items)
}

/// Fetch top rated items
pub async fn fetch_top_rated(kind: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/{}/top_rated?api_key={}&language=en-US",
        TMDB_BASE_URL, kind, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let items = resp.results.into_iter().filter(|i| !i.is_nsfw() && (i.backdrop_path.is_some() || i.poster_path.is_some())).collect();
    Ok(items)
}

/// Fetch items for a specific TMDB genre ID.
pub async fn fetch_by_genre(kind: &str, genre_id: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/discover/{}?api_key={}&with_genres={}&language=en-US&sort_by=popularity.desc",
        TMDB_BASE_URL, kind, TMDB_API_KEY, genre_id
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    Ok(resp.results)
}

/// Fetch items for a British region filter (used by the 'British TV' category).
pub async fn fetch_british_tv() -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url = format!(
        "{}/discover/tv?api_key={}&with_origin_country=GB&language=en-US&sort_by=popularity.desc",
        TMDB_BASE_URL, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    Ok(resp.results)
}

/// Primary entry point: given a Netflix-style genre ID from the URL,
/// resolve the correct TMDB query and return its results.
pub async fn fetch_for_genre_route(genre_key: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let kind = if let Some(ctx) = map_genre_key_to_tmdb(genre_key) {
        match ctx.kind {
            MediaKind::Tv => "tv",
            MediaKind::Movie => "movie",
        }
    } else if genre_key == "british" || genre_key == "binge" || genre_key == "52117" || genre_key == "1191605" {
        "tv"
    } else {
        "movie"
    };
    fetch_row_content(kind, Some(genre_key), None, None, None, 1).await
}

/// Search across TV and Movies
pub async fn search_media(query: &str) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let clean = query.trim();
    if clean.is_empty() {
        return Ok(vec![]);
    }
    let encoded_query: String = js_sys::encode_uri_component(clean)
        .as_string()
        .unwrap_or_else(|| clean.to_string());
    let url = format!(
        "{}/search/multi?api_key={}&language=en-US&query={}&page=1&include_adult=false",
        TMDB_BASE_URL, TMDB_API_KEY, encoded_query
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    // Filter out people, nsfw content, and items without artwork
    let filtered = resp.results.into_iter().filter(|i| {
        i.media_type.as_deref() != Some("person")
            && !i.is_nsfw()
            && (i.backdrop_path.is_some() || i.poster_path.is_some())
    }).collect();
    Ok(filtered)
}

/// Fetch recommendations (More Like This)
pub async fn fetch_recommendations(id: u32, is_tv: bool) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/recommendations?api_key={}&language=en-US&page=1",
        TMDB_BASE_URL, kind, id, TMDB_API_KEY
    );
    let resp: TmdbResponse<MediaItem> = get(&url).send().await?.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let filtered = resp.results.into_iter().filter(|i| {
        !i.is_nsfw() && (i.backdrop_path.is_some() || i.poster_path.is_some())
    }).collect();
    Ok(filtered)
}

pub async fn fetch_trending_page(kind: &str, page: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let safe_kind = if kind == "tv" { "tv" } else { "movie" };
    let url = format!("{}/trending/{}/day?api_key={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, page);
    let resp = get(&url).send().await?;
    let data: TmdbResponse<MediaItem> = resp.json::<TmdbResponse<MediaItem>>().await?.rated().await;
    let items = data.results.into_iter().filter_map(|mut item| {
        if item.is_nsfw() {
            return None;
        }
        if item.backdrop_path.is_none() && item.poster_path.is_none() {
            return None;
        }
        if item.media_type.is_none() {
            item.media_type = Some(safe_kind.to_string());
        }
        Some(item)
    }).collect();
    Ok(items)
}

/// Robust TMDB Query Engine for Rows, Feeds, and Category Pages.
/// Dispatches to distinct endpoints and queries with correct media_type separation,
/// quality floors, content moderation, and anti-leakage filters.
pub async fn fetch_row_content(
    kind: &str,
    genre_id: Option<&str>,
    sort_by: Option<&str>,
    extra_params: Option<&str>,
    endpoint: Option<&str>,
    page: u32,
) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let safe_kind = if kind == "tv" { "tv" } else { "movie" };

    // 1. Explicit Endpoint (e.g. "/movie/now_playing", "/movie/upcoming", "/trending/tv/week")
    if let Some(ep) = endpoint {
        let ep_clean = ep.trim_start_matches('/');
        let sep = if ep_clean.contains('?') { "&" } else { "?" };
        let mut url = format!("{}/{}{}api_key={}&include_adult=false&page={}", TMDB_BASE_URL, ep_clean, sep, TMDB_API_KEY, page);
        if let Some(extra) = extra_params {
            if !extra.is_empty() {
                let extra_clean = extra.trim_start_matches('&');
                url.push_str(&format!("&{}", extra_clean));
            }
        }
        if let Ok(resp) = get(&url).send().await {
            if let Ok(data) = resp.json::<TmdbResponse<MediaItem>>().await {
                let data = data.rated().await;
                let items: Vec<MediaItem> = data.results.into_iter().filter_map(|mut item| {
                    if item.is_nsfw() {
                        return None;
                    }
                    if item.backdrop_path.is_none() && item.poster_path.is_none() {
                        return None;
                    }
                    if item.media_type.is_none() {
                        item.media_type = Some(safe_kind.to_string());
                    }
                    Some(item)
                }).collect();
                if !items.is_empty() {
                    return Ok(items);
                }
            }
        }
        return fetch_trending_page(safe_kind, page).await;
    }

    // 2. Genre or Category ID
    if let Some(gid) = genre_id {
        let gid_lower = gid.to_lowercase();
        let default_sort = sort_by.unwrap_or("popularity.desc");
        let has_custom_votes = extra_params.map(|e| e.contains("vote_count.gte")).unwrap_or(false);

        let mut url = match gid_lower.as_str() {
            // LGBTQ / Pride
            "10001" | "lgbtq" | "pride" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                format!("{}/discover/{}?api_key={}&with_keywords=158718%7C380747%7C250606%7C363345%7C264386%7C290527%7C329968%7C300642%7C195624{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Black Stories
            "10003" | "black stories" | "black" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                format!("{}/discover/{}?api_key={}&with_keywords=256015%7C358563%7C190675%7C251436%7C288242%7C195624%7C339021{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // British origin
            "british" | "52117" | "10005" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_origin_country=GB{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Classics (pre-1985 for movies, pre-1998 for TV)
            "10009" | "classics" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                    format!("{}/discover/tv?api_key={}&first_air_date.lte=1998-12-31{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=80" };
                    format!("{}/discover/movie?api_key={}&primary_release_date.lte=1985-12-31{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Cult Classics & Cult TV
            "10010" | "cult" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_keywords=374649%7C367333%7C9840{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // European origin
            "european" | "10006" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_origin_country=FR%7CDE%7CIT%7CES%7CNL%7CDK%7CSE%7CNO%7CFI%7CPL{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // US / Hollywood origin
            "us" | "10008" | "hollywood" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                format!("{}/discover/{}?api_key={}&with_origin_country=US{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Halloween (Spooky & Supernatural)
            "10016" | "halloween" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                    format!("{}/discover/tv?api_key={}&with_keywords=3335%7C3358%7C616%7C162846%7C3133%7C12339%7C1299%7C6152{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                    format!("{}/discover/movie?api_key={}&with_genres=27&with_keywords=3335%7C3358%7C616%7C162846%7C3133%7C12339%7C1299%7C6152{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Independent / Indie Cinema
            "10011" | "independent" | "indie" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                format!("{}/discover/{}?api_key={}&with_keywords=281237%7C10183{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // International (non-English)
            "international" | "10012" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                format!("{}/discover/{}?api_key={}&without_original_language=en{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Shorts (under 45 minutes)
            "10013" | "shorts" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=10" };
                format!("{}/discover/movie?api_key={}&with_runtime.lte=40&with_runtime.gte=2{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Sports
            "10014" | "sport" | "sports" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                format!("{}/discover/{}?api_key={}&with_keywords=333328%7C13042%7C6496%7C1480%7C209476%7C315138{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Stand-up Comedy (Movies/Specials)
            "10017" | "stand-up comedy" | "stand-up" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                format!("{}/discover/movie?api_key={}&with_genres=35&with_keywords=9716%7C356038{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Stand-up & Chat Shows (TV)
            "10020" | "chat shows" | "stand-up & chat shows" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=10" };
                format!("{}/discover/tv?api_key={}&with_genres=10767&with_keywords=9716%7C356038{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Emmy or Emmys (Award-winning prestige television)
            "10018" | "emmy" | "emmys" | "emmy or emmys" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=80" };
                format!("{}/discover/tv?api_key={}&with_genres=18,35&vote_average.gte=7.8{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Science & Nature
            "10019" | "science & nature" | "nature" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                if safe_kind == "tv" {
                    format!("{}/discover/tv?api_key={}&with_genres=99&with_keywords=221355%7C9902%7C270%7C33577%7C305903{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    format!("{}/discover/movie?api_key={}&with_genres=99&with_keywords=221355%7C9902%7C270%7C33577%7C305903{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Teen
            "10015" | "teen" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                format!("{}/discover/{}?api_key={}&with_genres=18,35&with_keywords=10683%7C6270%7C193400%7C206720{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Anime (Strict vote floor + Japanese language + exclude adult tags)
            "anime" | "7424" | "16" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                format!("{}/discover/{}?api_key={}&with_genres=16&with_original_language=ja{}&without_keywords=190370,222243,267498&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, safe_kind, TMDB_API_KEY, vote_filter, default_sort, page)
            }
            // Horror (Cross-media: TV uses Mystery/Sci-Fi + Horror keywords)
            "27" | "8711" | "horror" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/tv?api_key={}&with_keywords=6152%7C3358%7C12377%7C162846%7C295907%7C3335{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=27{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Romance (Cross-media: TV uses Drama + Romance keywords)
            "10749" | "8883" | "romance" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/tv?api_key={}&with_genres=18&with_keywords=304976%7C128%7C3691%7C13072{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=10749{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Thriller (Cross-media: TV uses Mystery & Crime)
            "53" | "8933" | "thriller" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                    format!("{}/discover/tv?api_key={}&with_genres=9648,80{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=53{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Fantasy (Cross-media: TV uses 10765)
            "14" | "fantasy" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/tv?api_key={}&with_genres=10765&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=14{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Sci-Fi (Cross-media: TV uses 10765)
            "878" | "1492" | "1372" | "sci-fi" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                    format!("{}/discover/tv?api_key={}&with_genres=10765&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres=878{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Music & Musicals
            "10402" | "music & musicals" => {
                if safe_kind == "tv" {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=15" };
                    format!("{}/discover/tv?api_key={}&with_keywords=4344%7C246377%7C6029%7C18001{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                    format!("{}/discover/movie?api_key={}&with_genres=10402{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Binge-worthy / Boredom Busters
            "binge" | "1191605" | "boredom" => {
                let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=40" };
                if safe_kind == "tv" {
                    format!("{}/discover/tv?api_key={}&with_genres=10759,80,10765&without_genres=16,10764{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                } else {
                    format!("{}/discover/movie?api_key={}&with_genres=28,12,53,878&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                }
            }
            // Numeric IDs or Netflix codes
            _ => {
                let resolved_genre_id = match gid_lower.as_str() {
                    "1365" => if safe_kind == "tv" { "10759" } else { "28" },
                    "6548" | "10375" => "35",
                    "5763" | "11714" => "18",
                    "6839" => "99",
                    "9875" | "26146" => "80",
                    "783" => if safe_kind == "tv" { "10762" } else { "10751" },
                    "10673" => "10759",
                    // TMDB cross-media resolution
                    "28" | "12" => if safe_kind == "tv" { "10759" } else { gid },
                    "10751" => if safe_kind == "tv" { "10762" } else { "10751" },
                    "10759" => if safe_kind == "movie" { "28" } else { "10759" },
                    "10765" => if safe_kind == "movie" { "878" } else { "10765" },
                    "10762" => if safe_kind == "movie" { "10751" } else { "10762" },
                    _ => gid,
                };

                if safe_kind == "tv" {
                    match resolved_genre_id {
                        // Docuseries: require vote threshold and modern era to avoid 1-vote local foreign broadcasts
                        "99" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                            format!("{}/discover/tv?api_key={}&with_genres=99{}&first_air_date.gte=2000-01-01&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Action & Adventure TV: exclude animation & kids cartoons (Doraemon, etc.)
                        "10759" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                            format!("{}/discover/tv?api_key={}&with_genres=10759&without_genres=16,10762{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Kids & Family TV: modern era threshold to avoid 1960s relics
                        "10762" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=20" };
                            format!("{}/discover/tv?api_key={}&with_genres=10762&first_air_date.gte=1990-01-01{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Sci-Fi & Fantasy TV: exclude pure animation to keep live-action series prominent
                        "10765" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                            format!("{}/discover/tv?api_key={}&with_genres=10765&without_genres=16{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        // Comedies & Dramas: exclude reality/talk shows
                        "35" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                            format!("{}/discover/tv?api_key={}&with_genres=35&without_genres=10764,10767{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        "18" => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=30" };
                            format!("{}/discover/tv?api_key={}&with_genres=18&without_genres=10764,10767{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, vote_filter, default_sort, page)
                        }
                        _ => {
                            let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=25" };
                            format!("{}/discover/tv?api_key={}&with_genres={}{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, resolved_genre_id, vote_filter, default_sort, page)
                        }
                    }
                } else {
                    let vote_filter = if has_custom_votes { "" } else { "&vote_count.gte=35" };
                    format!("{}/discover/movie?api_key={}&with_genres={}{}&include_adult=false&sort_by={}&page={}", TMDB_BASE_URL, TMDB_API_KEY, resolved_genre_id, vote_filter, default_sort, page)
                }
            }
        };

        if let Some(extra) = extra_params {
            if !extra.is_empty() {
                let extra_clean = extra.trim_start_matches('&');
                url.push_str(&format!("&{}", extra_clean));
            }
        }

        if let Ok(resp) = get(&url).send().await {
            if let Ok(data) = resp.json::<TmdbResponse<MediaItem>>().await {
                let data = data.rated().await;
                let items: Vec<MediaItem> = data.results.into_iter().filter_map(|mut item| {
                    if item.is_nsfw() {
                        return None;
                    }
                    if item.backdrop_path.is_none() && item.poster_path.is_none() {
                        return None;
                    }
                    if item.media_type.is_none() {
                        item.media_type = Some(safe_kind.to_string());
                    }
                    Some(item)
                }).collect();
                if !items.is_empty() {
                    return Ok(items);
                }
            }
        }

        // If extra_params caused 0 results, retry without the restrictive extra_params
        if extra_params.is_some() {
            // Strip extra_params from url and retry
            let base_url = if url.contains('&') {
                if let Some(extra) = extra_params {
                    url.replace(extra, "")
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            };
            if let Ok(resp) = get(&base_url).send().await {
                if let Ok(data) = resp.json::<TmdbResponse<MediaItem>>().await {
                let data = data.rated().await;
                    let items: Vec<MediaItem> = data.results.into_iter().filter_map(|mut item| {
                        if item.is_nsfw() || (item.backdrop_path.is_none() && item.poster_path.is_none()) {
                            None
                        } else {
                            if item.media_type.is_none() {
                                item.media_type = Some(safe_kind.to_string());
                            }
                            Some(item)
                        }
                    }).collect();
                    if !items.is_empty() {
                        return Ok(items);
                    }
                }
            }
        }
    }

    // 3. Fallback: ALWAYS respect safe_kind so TV series never show movies
    fetch_trending_page(safe_kind, page).await
}

pub async fn fetch_for_genre_route_page(genre_key: &str, page: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    fetch_row_content("movie", Some(genre_key), None, None, None, page).await
}

pub async fn fetch_by_language(lang: &str, page: u32) -> Result<Vec<MediaItem>, gloo_net::Error> {
    let url_movie = format!(
        "{}/discover/movie?api_key={}&with_original_language={}&sort_by=popularity.desc&page={}",
        TMDB_BASE_URL, TMDB_API_KEY, lang, page
    );
    let resp_movie: Result<TmdbResponse<MediaItem>, _> = get(&url_movie).send().await?.json().await;

    let url_tv = format!(
        "{}/discover/tv?api_key={}&with_original_language={}&sort_by=popularity.desc&page={}",
        TMDB_BASE_URL, TMDB_API_KEY, lang, page
    );
    let resp_tv: Result<TmdbResponse<MediaItem>, _> = get(&url_tv).send().await?.json().await;

    let resp_movie = match resp_movie { Ok(r) => Ok(r.rated().await), e => e };
    let resp_tv = match resp_tv { Ok(r) => Ok(r.rated().await), e => e };
    let mut combined = resp_movie.map(|r| r.results).unwrap_or_default();
    if let Ok(tv_resp) = resp_tv {
        for mut item in tv_resp.results {
            item.media_type = Some("tv".to_string());
            combined.push(item);
        }
    }
    Ok(combined)
}
