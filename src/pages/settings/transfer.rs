//! Profile transfer: the account setting that allows or blocks copying profiles out of the account, and the per profile
//! "Transfer a profile" screen that saves a profile as a file. Importing a profile from such a file is on Manage profiles.
//! The file holds the name, picture, kind, settings and viewing history of one profile; it never holds the PIN.
use super::hub::persist_profiles;
use super::prefs::{use_prefs, Prefs, ProfilePrefs};
use super::widgets::{save_as_file, toast, Message, Notice, Tone};
use super::Screen;
use crate::config::{MAX_PROFILES, PROFILE_NAME_MAX};
use crate::i18n::t;
use crate::models::profile::Profile;
use crate::services::account;
use crate::store::{use_profile_store, use_watch_store, ProfileStore, WatchRecord};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Name of the format, so a file from somewhere else is refused.
const FORMAT: &str = "streams-profile";
const VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
struct ProfileFile {
    format: String,
    version: u8,
    name: String,
    avatar_url: Option<String>,
    is_kids: bool,
    prefs: ProfilePrefs,
    history: Vec<WatchRecord>,
}

#[derive(Debug, PartialEq)]
pub enum ImportError {
    NotAProfile,
    TooNew,
    Full,
}

fn export_text(profile: &Profile, prefs: ProfilePrefs, history: Vec<WatchRecord>) -> String {
    let file = ProfileFile { format: FORMAT.into(), version: VERSION, name: profile.name.clone(), avatar_url: profile.avatar_url.clone(), is_kids: profile.is_kids, prefs, history };
    serde_json::to_string_pretty(&file).unwrap_or_default()
}

fn parse_file(text: &str) -> Result<ProfileFile, ImportError> {
    let file: ProfileFile = serde_json::from_str(text).map_err(|_| ImportError::NotAProfile)?;
    if file.format != FORMAT {
        return Err(ImportError::NotAProfile);
    }
    if file.version > VERSION {
        return Err(ImportError::TooNew);
    }
    Ok(file)
}

/// A name no other profile uses: the wanted one, or with " (2)", " (3)" and so on added.
fn free_name(taken: &[String], wanted: &str) -> String {
    let base: String = wanted.trim().chars().take(PROFILE_NAME_MAX).collect();
    let used = |n: &str| taken.iter().any(|t| t.trim().to_lowercase() == n.to_lowercase());
    if !used(&base) {
        return base;
    }
    (2..)
        .map(|n| {
            let suffix = format!(" ({n})");
            let room = PROFILE_NAME_MAX.saturating_sub(suffix.chars().count());
            format!("{}{suffix}", base.chars().take(room).collect::<String>())
        })
        .find(|c| !used(c))
        .unwrap_or(base)
}

/// Adds the profile in `text` to this account. Returns the name it was given.
pub fn import_profile(text: &str, store: ProfileStore, prefs: Prefs) -> Result<String, ImportError> {
    let file = parse_file(text)?;
    let regular = store.profiles.with_untracked(|l| l.iter().filter(|p| !p.is_kids).count());
    if !file.is_kids && regular >= MAX_PROFILES {
        return Err(ImportError::Full);
    }
    let taken: Vec<String> = store.profiles.with_untracked(|l| l.iter().map(|p| p.name.clone()).collect());
    let name = free_name(&taken, &file.name);
    let id = format!("p{}", js_sys::Date::now() as u64);
    store.profiles.update(|l| {
        let order = l.iter().map(|p| p.sort_order).max().unwrap_or(0) + 1;
        l.push(Profile { id: id.clone(), name: name.clone(), avatar_url: file.avatar_url, is_kids: file.is_kids, is_default: Some(false), pin: None, sort_order: order });
    });
    persist_profiles(store);
    prefs.update_profile(&id, |p| *p = file.prefs);
    let watch = use_watch_store();
    for r in file.history {
        if watch.get_record(r.media_id, r.is_tv).is_none() {
            watch.records.update(|m| {
                m.insert(WatchRecord::composite_key(r.media_id, r.is_tv), r);
            });
        }
    }
    Ok(name)
}

/// The account setting: whether profiles may be copied out of the account. A dark red page with the state in large type and one
/// outlined button, like the Netflix page; the same on every width.
#[component]
pub fn TransferSetting() -> impl IntoView {
    let prefs = use_prefs();
    let note = RwSignal::new(None::<(Tone, String)>);
    let on = Signal::derive(move || prefs.account.get().profile_transfer);
    let toggle = move |_| {
        let value = !on.get_untracked();
        prefs.account.update(|a| a.profile_transfer = value);
        if account::session().is_some() {
            leptos::task::spawn_local(async move {
                if let Err(e) = account::set_switch("profile_transfer", value).await {
                    note.set(Some((Tone::Error, t(if e == account::AuthError::Network { "set-err-network" } else { "set-err-other" }))));
                }
            });
        }
        toast(t("set-saved"));
    };
    view! {
        <div class="-mx-4 -mt-4 flex min-h-settings-hero flex-col items-center justify-center bg-linear-to-b from-chrome via-danger-deep to-chrome px-6 py-10 text-center text-chrome-ink sm:-mx-8 lg:mx-0 lg:rounded-card">
            <i class="ph-bold ph-arrows-left-right mb-6 text-8xl text-chrome-ink/90" aria-hidden="true"></i>
            <h2 class="mb-4 max-w-xl text-heading-lg font-bold leading-tight">{move || if on.get() { t("set-transfer-on-title") } else { t("set-transfer-off-title") }}</h2>
            <p class="mb-8 max-w-md text-body-md text-chrome-ink/80">
                {move || t(if on.get() { "set-transfer-on-body" } else { "set-transfer-off-body" })}
                <Show when=move || on.get()>
                    " "<a href=crate::config::routes::HELP class="underline">{t("set-transfer-help-centre")}</a>"."
                </Show>
            </p>
            <Message text=note />
            <div class="flex w-full max-w-xl flex-col gap-3 sm:flex-row">
                <a href=crate::config::routes::BROWSE class="flex-1 rounded-button bg-chrome-ink/90 px-6 py-3 text-body-md font-bold text-chrome hover:bg-chrome-ink">{t("set-transfer-go-home")}</a>
                <button type="button" on:click=toggle class="flex-1 rounded-button border border-danger px-6 py-3 text-body-md font-bold text-chrome-ink hover:bg-danger/20">{move || if on.get() { t("set-transfer-turn-off") } else { t("set-transfer-turn-on") }}</button>
            </div>
        </div>
    }
}

/// Saves one profile as a file that another account can import.
#[component]
pub fn TransferProfile(id: String) -> impl IntoView {
    let store = use_profile_store();
    let prefs = use_prefs();
    let watch = use_watch_store();
    let id = StoredValue::new(id);
    let allowed = Signal::derive(move || prefs.account.get().profile_transfer);
    let download = Callback::new(move |_| {
        let Some(profile) = store.profiles.with_untracked(|l| l.iter().find(|p| p.id == id.get_value()).cloned()) else { return };
        let history: Vec<WatchRecord> = watch.records.with_untracked(|m| m.values().cloned().collect());
        let text = export_text(&profile, prefs.profile(&profile.id), history);
        save_as_file(&crate::data::settings::export_file_name_for(&format!("profile-{}", profile.name.to_lowercase().replace(' ', "-")), "json"), &text, crate::data::settings::EXPORT_MIME);
        toast(t("set-transfer-saved"));
    });
    view! {
        <div class="max-w-xl">
            <Show
                when=move || allowed.get()
                fallback=move || view! {
                    <Notice icon="prohibit" title=t("set-transfer-off-title") body=t("set-transfer-blocked-body")>
                        <a href=Screen::TransferSetting.path() class="btn-ink inline-block px-6 py-2.5">{t("set-transfer-open-setting")}</a>
                    </Notice>
                }
            >
                <p class="mb-4 text-body-md text-ink/70">{t("set-transfer-profile-body")}</p>
                <ol class="mb-6 list-decimal space-y-1 ps-6 text-body-md text-ink/70">
                    <li>{t("set-transfer-step-1")}</li>
                    <li>{t("set-transfer-step-2")}</li>
                    <li>{t("set-transfer-step-3")}</li>
                </ol>
                <button type="button" on:click=move |_| download.run(()) class="btn-ink px-8 py-3">{t("set-transfer-download")}</button>
                <p class="mt-4 text-body-sm text-ink/60">{t("set-transfer-note")}</p>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_profile_file_reads_back() {
        let profile = Profile { id: "p1".into(), name: "Ada".into(), avatar_url: None, is_kids: false, is_default: Some(false), pin: Some("1234".into()), sort_order: 0 };
        let text = export_text(&profile, ProfilePrefs::default(), vec![]);
        assert!(!text.contains("1234"), "the PIN must not be in the file");
        let back = parse_file(&text).unwrap();
        assert_eq!((back.name.as_str(), back.is_kids), ("Ada", false));
    }

    #[test]
    fn other_files_are_refused() {
        assert_eq!(parse_file("{}").err(), Some(ImportError::NotAProfile));
        assert_eq!(parse_file("not json").err(), Some(ImportError::NotAProfile));
        let newer = r#"{"format":"streams-profile","version":9,"name":"X","avatar_url":null,"is_kids":false,"prefs":{},"history":[]}"#;
        assert_eq!(parse_file(newer).err(), Some(ImportError::TooNew));
    }

    #[test]
    fn a_taken_name_gets_a_number() {
        let taken = vec!["Ada".to_string(), "ada (2)".to_string()];
        assert_eq!(free_name(&taken, "Ada"), "Ada (3)");
        assert_eq!(free_name(&taken, "Bob"), "Bob");
    }
}
