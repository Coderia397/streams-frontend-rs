use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const WATCH_STORAGE_KEY: &str = "streams_watch_progress_v1";
const EPISODE_STORAGE_KEY: &str = "streams_episode_progress_v1";

fn storage_key() -> String {
    crate::store::scoped::key(WATCH_STORAGE_KEY)
}

/// How far one episode of a series was watched (the record of the series keeps only the episode watched last).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct EpisodeProgress {
    pub position: f64,
    pub duration: f64,
    pub updated_at: u64,
}

impl EpisodeProgress {
    pub fn percent(&self) -> f32 {
        if self.duration > 0.0 { ((self.position / self.duration) * 100.0).clamp(0.0, 100.0) as f32 } else { 0.0 }
    }
}

pub fn episode_key(series_id: u32, season: u32, episode: u32) -> String {
    format!("tv_{series_id}_{season}_{episode}")
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WatchRecord {
    pub media_id: u32,
    pub is_tv: bool,
    pub title: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub episode_title: Option<String>,
    pub current_time: f64,
    pub duration: f64,
    pub percentage: f32,
    pub updated_at: u64,
}

impl WatchRecord {
    pub fn composite_key(media_id: u32, is_tv: bool) -> String {
        format!("{}_{}", if is_tv { "tv" } else { "movie" }, media_id)
    }

    pub fn key(&self) -> String {
        Self::composite_key(self.media_id, self.is_tv)
    }

    pub fn backdrop_url(&self, size: &str) -> Option<String> {
        self.backdrop_path.as_ref().map(|p| {
            if p.starts_with("http") {
                p.clone()
            } else {
                format!("{}/{}{}", crate::config::TMDB_IMAGE_URL, size, p)
            }
        })
    }

    pub fn poster_url(&self, size: &str) -> Option<String> {
        self.poster_path.as_ref().map(|p| {
            if p.starts_with("http") {
                p.clone()
            } else {
                format!("{}/{}{}", crate::config::TMDB_IMAGE_URL, size, p)
            }
        })
    }
}

#[derive(Clone, Copy)]
pub struct WatchStore {
    pub records: RwSignal<HashMap<String, WatchRecord>>,
    /// one entry per episode watched, key `tv_<series>_<season>_<episode>`
    pub episodes: RwSignal<HashMap<String, EpisodeProgress>>,
}

fn load_episodes() -> HashMap<String, EpisodeProgress> {
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    storage
        .and_then(|s| s.get_item(&crate::store::scoped::key(EPISODE_STORAGE_KEY)).ok().flatten())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_episodes(map: &HashMap<String, EpisodeProgress>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(raw) = serde_json::to_string(map) {
            let _ = storage.set_item(&crate::store::scoped::key(EPISODE_STORAGE_KEY), &raw);
        }
    }
}

fn load_from_storage() -> HashMap<String, WatchRecord> {
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    storage
        .as_ref()
        .and_then(|s| s.get_item(&storage_key()).ok().flatten())
        .and_then(|raw| serde_json::from_str::<HashMap<String, WatchRecord>>(&raw).ok())
        .unwrap_or_default()
}

fn save_to_storage(records: &HashMap<String, WatchRecord>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(raw) = serde_json::to_string(records) {
            let _ = storage.set_item(&storage_key(), &raw);
        }
    }
}

pub fn provide_watch_store() {
    let initial_records = load_from_storage();
    let records_signal = RwSignal::new(initial_records);
    let store = WatchStore {
        records: records_signal,
        episodes: RwSignal::new(load_episodes()),
    };
    crate::store::provide_and_keep(store);
}

pub fn use_watch_store() -> WatchStore {
    crate::store::use_kept::<WatchStore>("WatchStore")
}

impl WatchStore {
    /// Reads the records of the profile now in use (after the profile changed).
    pub fn reload(&self) {
        self.records.set(load_from_storage());
        self.episodes.set(load_episodes());
    }

    /// How far an episode was watched, 0 to 100 (0 when never started).
    pub fn episode_percent(&self, series_id: u32, season: u32, episode: u32) -> f32 {
        let key = episode_key(series_id, season, episode);
        self.episodes.with(|m| m.get(&key).map(|e| e.percent()).unwrap_or_else(|| {
            // the series record also tells how far its last episode went (a copy made before episodes were kept one by one)
            self.records.with(|r| r.get(&WatchRecord::composite_key(series_id, true)).filter(|r| r.season == Some(season) && r.episode == Some(episode)).map(|r| r.percentage).unwrap_or(0.0))
        }))
    }

    /// An episode position that came from the account (kept here, not sent back).
    pub fn insert_episode_remote(&self, series_id: u32, season: u32, episode: u32, progress: EpisodeProgress) {
        let key = episode_key(series_id, season, episode);
        self.episodes.update(|m| {
            if m.get(&key).is_some_and(|e| e.updated_at >= progress.updated_at) { return; }
            m.insert(key, progress);
            save_episodes(m);
        });
    }

    /// A record that came from the account: kept here, not sent back.
    pub fn insert_remote(&self, record: WatchRecord) {
        let key = record.key();
        self.records.update(|map| {
            map.insert(key, record);
            save_to_storage(map);
        });
    }

    /// Removes a record because the account says it was removed elsewhere (not sent back).
    pub fn remove_record_quiet(&self, media_id: u32, is_tv: bool) {
        let key = WatchRecord::composite_key(media_id, is_tv);
        self.records.update(|map| {
            map.remove(&key);
            save_to_storage(map);
        });
    }

    /// Adds the name and pictures to a record that came from the account without them.
    pub fn fill_details(&self, media_id: u32, is_tv: bool, title: String, poster_path: Option<String>, backdrop_path: Option<String>) {
        let key = WatchRecord::composite_key(media_id, is_tv);
        self.records.update(|map| {
            if let Some(r) = map.get_mut(&key) {
                r.title = title;
                r.poster_path = poster_path;
                r.backdrop_path = backdrop_path;
            }
            save_to_storage(map);
        });
    }

    pub fn get_record(&self, media_id: u32, is_tv: bool) -> Option<WatchRecord> {
        let key = WatchRecord::composite_key(media_id, is_tv);
        self.records.with(|m| m.get(&key).cloned())
    }

    pub fn get_resume_time(&self, media_id: u32, is_tv: bool, season: Option<u32>, episode: Option<u32>) -> f64 {
        if let Some(record) = self.get_record(media_id, is_tv) {
            if is_tv {
                if record.season == season && record.episode == episode {
                    // If within last 30s of completion, don't resume right at the credits
                    if record.duration > 60.0 && record.current_time >= record.duration - 30.0 {
                        0.0
                    } else {
                        record.current_time
                    }
                } else {
                    0.0
                }
            } else {
                if record.duration > 60.0 && record.current_time >= record.duration - 30.0 {
                    0.0
                } else {
                    record.current_time
                }
            }
        } else {
            0.0
        }
    }

    pub fn save_progress(
        &self,
        media_id: u32,
        is_tv: bool,
        title: String,
        poster_path: Option<String>,
        backdrop_path: Option<String>,
        season: Option<u32>,
        episode: Option<u32>,
        episode_title: Option<String>,
        current_time: f64,
        duration: f64,
    ) {
        if media_id == 0 || duration <= 0.0 {
            return;
        }

        let percentage = ((current_time / duration) * 100.0).clamp(0.0, 100.0) as f32;
        let now = js_sys::Date::now() as u64;

        let record = WatchRecord {
            media_id,
            is_tv,
            title,
            poster_path,
            backdrop_path,
            season,
            episode,
            episode_title,
            current_time,
            duration,
            percentage,
            updated_at: now,
        };

        if let (true, Some(s), Some(e)) = (is_tv, season, episode) {
            self.episodes.update(|m| {
                m.insert(episode_key(media_id, s, e), EpisodeProgress { position: current_time, duration, updated_at: now });
                save_episodes(m);
            });
        }
        let key = record.key();
        self.records.update(|map| {
            map.insert(key, record);
            save_to_storage(map);
        });
        crate::services::sync::schedule_push();
    }

    /// Forgets every saved position and rewrites the saved copy.
    pub fn clear_all(&self) {
        self.records.update(|map| {
            map.clear();
            save_to_storage(map);
        });
        self.episodes.update(|m| {
            m.clear();
            save_episodes(m);
        });
    }

    pub fn remove_record(&self, media_id: u32, is_tv: bool) {
        let key = WatchRecord::composite_key(media_id, is_tv);
        self.records.update(|map| {
            map.remove(&key);
            save_to_storage(map);
        });
        crate::services::sync::schedule_push();
    }

    pub fn get_continue_watching_list(&self) -> Vec<WatchRecord> {
        self.records.with(|map| {
            let mut list: Vec<WatchRecord> = map
                .values()
                .filter(|r| {
                    // Show items that have been watched more than 15s and less than 95%
                    r.current_time > 15.0 && r.percentage < 95.0
                })
                .cloned()
                .collect();
            // Sort newest updated first
            list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            list
        })
    }
}
