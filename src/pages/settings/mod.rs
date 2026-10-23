//! Settings and Account, built as a folder of screens like the Netflix app: a hub (profile card, other profiles, App settings,
//! Account, Help, Sign out) and one route per screen below it. Phones and tablets show the hub and full screens with a back
//! arrow; from 1024 px wide the same routes show a sidebar next to the screen, and the hub address opens the profile screen.
//! The tree is recorded in notes/32-settings-rebuild-plan-2026-10-11.md.
mod account;
mod account_menu;
mod activity;
mod app;
mod delete_account;
mod devices;
mod edit_profile_page;
mod footer;
mod diagnostics;
mod stepup;
mod notifications;
mod privacy;
mod transfer;
pub mod hub;
mod languages;
pub(crate) mod prefs;
mod profile;
mod restrictions;
mod shell;
mod subtitles;
mod widgets;

pub use prefs::{provide_prefs, use_prefs};

use crate::config::routes;
use crate::i18n::t;
use crate::store::use_profile_store;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// One screen of the tree. Profile screens carry the profile id they edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    Hub,
    Profiles,
    Profile(String),
    ProfileEdit(String),
    Avatar(String),
    Restrictions(String),
    Ratings(String),
    Blocked(String),
    Lock(String),
    MainProfile(String),
    Language(String),
    AudioSubtitles(String),
    Subtitles(String),
    Playback(String),
    Notifications(String),
    NotificationChannel(String, String),
    GameHandle(String),
    TransferProfile(String),
    App,
    Licences,
    CheckNetwork,
    PlaybackSpec,
    SpeedTest,
    Account,
    AccountSecurity,
    AccountDevices,
    AccountProfiles,
    Name,
    Password,
    Email,
    Phone,
    Access,
    Privacy,
    Activity,
    PersonalInfo,
    Testing,
    TransferSetting,
    Delete,
}

/// Which sidebar entry a screen belongs to (desktop). The phone has no sidebar, it has the hub list.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Overview,
    Security,
    Devices,
    Profiles,
    App,
}

const ROOT: &str = "/settings";

impl Screen {
    /// Reads an address like `/settings/profile/p1/subtitles`. `active` is the profile used when the address names none.
    pub fn parse(path: &str, active: &str) -> Screen {
        let rest = path.trim_start_matches('/').strip_prefix("settings").unwrap_or("");
        let parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
        let id = || parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| active.to_string());
        match parts.as_slice() {
            ["profiles"] => Screen::Profiles,
            ["profile"] | ["profile", _] => Screen::Profile(id()),
            ["profile", _, "edit"] => Screen::ProfileEdit(id()),
            ["profile", _, "avatar"] => Screen::Avatar(id()),
            ["profile", _, "restrictions"] => Screen::Restrictions(id()),
            ["profile", _, "restrictions", "ratings"] => Screen::Ratings(id()),
            ["profile", _, "restrictions", "blocked"] => Screen::Blocked(id()),
            ["profile", _, "lock"] => Screen::Lock(id()),
            ["profile", _, "main"] => Screen::MainProfile(id()),
            ["profile", _, "language"] => Screen::Language(id()),
            ["profile", _, "audio-subtitles"] => Screen::AudioSubtitles(id()),
            ["profile", _, "subtitles"] => Screen::Subtitles(id()),
            ["profile", _, "playback"] => Screen::Playback(id()),
            ["profile", _, "notifications"] => Screen::Notifications(id()),
            ["profile", _, "notifications", channel] if crate::data::settings::NOTIFICATION_CHANNELS.iter().any(|(c, _, _)| c == channel) => Screen::NotificationChannel(id(), channel.to_string()),
            ["profile", _, "game-handle"] => Screen::GameHandle(id()),
            ["profile", _, "transfer"] => Screen::TransferProfile(id()),
            ["app"] => Screen::App,
            ["app", "licences"] => Screen::Licences,
            ["app", "network"] => Screen::CheckNetwork,
            ["app", "playback-spec"] => Screen::PlaybackSpec,
            ["app", "speed-test"] => Screen::SpeedTest,
            ["account"] => Screen::Account,
            ["account", "security"] => Screen::AccountSecurity,
            ["account", "devices"] => Screen::AccountDevices,
            ["account", "profiles"] => Screen::AccountProfiles,
            ["account", "name"] => Screen::Name,
            ["account", "password"] => Screen::Password,
            ["account", "email"] => Screen::Email,
            ["account", "phone"] => Screen::Phone,
            ["account", "access"] => Screen::Access,
            ["account", "privacy"] => Screen::Privacy,
            ["account", "privacy", "activity"] => Screen::Activity,
            ["account", "personal-info"] => Screen::PersonalInfo,
            ["account", "testing"] => Screen::Testing,
            ["account", "transfer"] => Screen::TransferSetting,
            ["account", "delete"] => Screen::Delete,
            _ => Screen::Hub,
        }
    }

    /// The address of this screen. The only place addresses are written, so pages link with `Screen::X.path()`.
    pub fn path(&self) -> String {
        let profile = |id: &String, tail: &str| if id.is_empty() { format!("{ROOT}/profile") } else { format!("{ROOT}/profile/{id}{tail}") };
        match self {
            Screen::Hub => ROOT.into(),
            Screen::Profiles => format!("{ROOT}/profiles"),
            Screen::Profile(id) => profile(id, ""),
            Screen::ProfileEdit(id) => profile(id, "/edit"),
            Screen::Avatar(id) => profile(id, "/avatar"),
            Screen::Restrictions(id) => profile(id, "/restrictions"),
            Screen::Ratings(id) => profile(id, "/restrictions/ratings"),
            Screen::Blocked(id) => profile(id, "/restrictions/blocked"),
            Screen::Lock(id) => profile(id, "/lock"),
            Screen::MainProfile(id) => profile(id, "/main"),
            Screen::Language(id) => profile(id, "/language"),
            Screen::AudioSubtitles(id) => profile(id, "/audio-subtitles"),
            Screen::Subtitles(id) => profile(id, "/subtitles"),
            Screen::Playback(id) => profile(id, "/playback"),
            Screen::Notifications(id) => profile(id, "/notifications"),
            Screen::NotificationChannel(id, channel) => profile(id, &format!("/notifications/{channel}")),
            Screen::GameHandle(id) => profile(id, "/game-handle"),
            Screen::TransferProfile(id) => profile(id, "/transfer"),
            Screen::App => format!("{ROOT}/app"),
            Screen::Licences => format!("{ROOT}/app/licences"),
            Screen::CheckNetwork => format!("{ROOT}/app/network"),
            Screen::PlaybackSpec => format!("{ROOT}/app/playback-spec"),
            Screen::SpeedTest => format!("{ROOT}/app/speed-test"),
            Screen::Account => format!("{ROOT}/account"),
            Screen::AccountSecurity => format!("{ROOT}/account/security"),
            Screen::AccountDevices => format!("{ROOT}/account/devices"),
            Screen::AccountProfiles => format!("{ROOT}/account/profiles"),
            Screen::Name => format!("{ROOT}/account/name"),
            Screen::Password => format!("{ROOT}/account/password"),
            Screen::Email => format!("{ROOT}/account/email"),
            Screen::Phone => format!("{ROOT}/account/phone"),
            Screen::Access => format!("{ROOT}/account/access"),
            Screen::Privacy => format!("{ROOT}/account/privacy"),
            Screen::Activity => format!("{ROOT}/account/privacy/activity"),
            Screen::PersonalInfo => format!("{ROOT}/account/personal-info"),
            Screen::Testing => format!("{ROOT}/account/testing"),
            Screen::TransferSetting => format!("{ROOT}/account/transfer"),
            Screen::Delete => format!("{ROOT}/account/delete"),
        }
    }

    /// The screen one level up on a phone or tablet. The back arrow goes here, so back always lands inside the tree.
    pub fn parent(&self) -> String {
        match self {
            Screen::Hub => routes::BROWSE.into(),
            Screen::Profiles | Screen::App | Screen::Account | Screen::Profile(_) | Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles => Screen::Hub.path(),
            Screen::ProfileEdit(id) | Screen::Avatar(id) | Screen::Restrictions(id) | Screen::Lock(id) | Screen::MainProfile(id) | Screen::Language(id) | Screen::AudioSubtitles(id) | Screen::Subtitles(id) | Screen::Playback(id) | Screen::Notifications(id) | Screen::GameHandle(id) | Screen::TransferProfile(id) => Screen::Profile(id.clone()).path(),
            Screen::NotificationChannel(id, _) => Screen::Notifications(id.clone()).path(),
            Screen::Ratings(id) | Screen::Blocked(id) => Screen::Restrictions(id.clone()).path(),
            Screen::Licences | Screen::CheckNetwork | Screen::PlaybackSpec | Screen::SpeedTest => Screen::App.path(),
            Screen::Name | Screen::Password | Screen::Email | Screen::Phone | Screen::Access | Screen::Privacy | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete => Screen::Account.path(),
            Screen::Activity => Screen::Privacy.path(),
        }
    }

    /// The screen one level up from 1024 px wide, where the account area is split into Security, Devices and Profiles pages
    /// like the Netflix website. Everything else goes where the phone goes.
    pub fn parent_wide(&self) -> String {
        match self {
            Screen::Name | Screen::Password | Screen::Email | Screen::Phone | Screen::Privacy | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete => Screen::AccountSecurity.path(),
            Screen::Access => Screen::AccountDevices.path(),
            Screen::Profiles | Screen::Profile(_) => Screen::AccountProfiles.path(),
            Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles | Screen::App => Screen::Account.path(),
            Screen::Account => routes::BROWSE.into(),
            other => other.parent(),
        }
    }

    pub fn title_key(&self) -> &'static str {
        match self {
            Screen::Hub => "set-title",
            Screen::Profiles => "set-title-profiles",
            Screen::Profile(_) | Screen::ProfileEdit(_) => "set-title-edit-profile",
            Screen::Avatar(_) => "set-title-avatar",
            Screen::Restrictions(_) => "set-title-restrictions",
            Screen::Ratings(_) => "set-title-ratings",
            Screen::Blocked(_) => "set-title-blocked",
            Screen::Lock(_) => "set-title-lock",
            Screen::MainProfile(_) => "set-main-profile",
            Screen::Language(_) => "set-title-language",
            Screen::AudioSubtitles(_) => "set-title-audio-subtitles",
            Screen::Subtitles(_) => "set-title-subtitles",
            Screen::Playback(_) => "set-title-playback",
            Screen::Notifications(_) => "set-title-notifications",
            Screen::NotificationChannel(_, channel) => if channel == "push" { "set-notify-push" } else { "set-notify-email" },
            Screen::GameHandle(_) => "set-title-game-handle",
            Screen::TransferProfile(_) => "set-title-transfer-profile",
            Screen::App => "set-title-app",
            Screen::Licences => "set-app-licences",
            Screen::CheckNetwork => "set-check-network",
            Screen::PlaybackSpec => "set-playback-spec",
            Screen::SpeedTest => "set-title-speed-test",
            Screen::Account => "set-title-account",
            Screen::AccountSecurity => "set-account-security",
            Screen::AccountDevices => "set-account-devices",
            Screen::AccountProfiles => "set-account-profiles",
            Screen::Name => "set-title-name",
            Screen::Password => "set-title-change-password",
            Screen::Email => "set-title-email",
            Screen::Phone => "set-title-phone",
            Screen::Access => "set-title-devices",
            Screen::Privacy => "set-title-privacy",
            Screen::Activity => "set-title-activity",
            Screen::PersonalInfo => "set-title-personal-info",
            Screen::Testing => "set-title-testing",
            Screen::TransferSetting => "set-title-transfer",
            Screen::Delete => "set-title-delete",
        }
    }

    /// Screens Netflix draws white on a phone (the Account pages); the rest are dark there. From 1024 px everything is white.
    fn light_on_phone(&self) -> bool {
        matches!(self, Screen::Account | Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles | Screen::Name | Screen::Password | Screen::Email | Screen::Phone | Screen::Access | Screen::Privacy | Screen::Activity | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete)
    }

    /// Pages the website draws on their own, with only the logo above them and no side menu (the pages of one profile's preferences). Phones keep the app's frame.
    pub(crate) fn standalone(&self) -> bool {
        matches!(self, Screen::Language(_) | Screen::AudioSubtitles(_) | Screen::Subtitles(_) | Screen::Playback(_) | Screen::Notifications(_) | Screen::NotificationChannel(_, _) | Screen::GameHandle(_) | Screen::TransferProfile(_) | Screen::Lock(_) | Screen::Privacy | Screen::Activity)
    }

    /// The profile a page is about: the one in its address, else (privacy, activity) the one in use.
    pub(crate) fn profile_of(&self) -> Option<String> {
        match self {
            Screen::Language(id) | Screen::AudioSubtitles(id) | Screen::Subtitles(id) | Screen::Playback(id) | Screen::Notifications(id) | Screen::NotificationChannel(id, _) | Screen::GameHandle(id) | Screen::TransferProfile(id) | Screen::Lock(id) => Some(id.clone()),
            _ => None,
        }
    }

    fn section(&self) -> Section {
        match self {
            Screen::Hub | Screen::Account => Section::Overview,
            Screen::AccountSecurity | Screen::Name | Screen::Password | Screen::Email | Screen::Phone | Screen::Privacy | Screen::Activity | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete => Section::Security,
            Screen::AccountDevices | Screen::Access => Section::Devices,
            Screen::App | Screen::Licences | Screen::CheckNetwork | Screen::PlaybackSpec | Screen::SpeedTest => Section::App,
            _ => Section::Profiles,
        }
    }

    /// Screens that change account data and ask for the password first.
    fn needs_account_check(&self) -> bool {
        matches!(self, Screen::Name | Screen::Email | Screen::Phone | Screen::Access | Screen::PersonalInfo | Screen::Delete)
    }

    /// Screens that change a profile's protection and ask for the password when signed in. Changing the maturity level is one
    /// of them in the database too (it needs a recent sign in).
    fn needs_profile_check(&self) -> bool {
        matches!(self, Screen::Ratings(_) | Screen::Blocked(_) | Screen::MainProfile(_))
    }
}

/// The sidebar entries from 1024 px wide: icon, translation key, the screen and the section it highlights. The same groups as the
/// Netflix website's account menu (Overview, Security, Devices, Profiles) plus the app's own settings.
fn sidebar() -> Vec<(&'static str, &'static str, Screen, Section)> {
    vec![
        ("squares-four", "set-nav-overview", Screen::Account, Section::Overview),
        ("shield-check", "set-account-security", Screen::AccountSecurity, Section::Security),
        ("devices", "set-account-devices", Screen::AccountDevices, Section::Devices),
        ("users-three", "set-account-profiles", Screen::AccountProfiles, Section::Profiles),
        ("gear-six", "set-nav-app", Screen::App, Section::App),
    ]
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let location = use_location();
    let profiles = use_profile_store();
    let active = Memo::new(move |_| profiles.active_profile_id.get().unwrap_or_default());
    let screen = Memo::new(move |_| Screen::parse(&location.pathname.get(), &active.get()));
    Effect::new(move |_| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let title = if screen.get() == Screen::Hub { t(Screen::Account.title_key()) } else { t(screen.get().title_key()) };
            doc.set_title(&format!("{title} | {}", crate::brand::NAME));
        }
    });
    let signout_open = RwSignal::new(false);
    provide_context(hub::SignOutOpen(signout_open));
    // while the password check is showing it has its own heading, so the page title steps aside
    let gate_active = RwSignal::new(false);
    provide_context(stepup::GateActive(gate_active));
    provide_context(widgets::Toast(RwSignal::new(None)));

    let avatar = Memo::new(move |_| profiles.profiles.with(|l| profiles.active_profile_id.with(|id| l.iter().find(|p| Some(&p.id) == id.as_ref()).or(l.first()).and_then(|p| p.avatar_url.clone()))).unwrap_or_else(|| crate::data::DEFAULT_AVATAR.to_string()));
    let light_phone = move || screen.get().light_on_phone();
    let standalone = Memo::new(move |_| screen.get().standalone());

    view! {
        <div class="theme-paper-wide min-h-screen bg-void font-sans text-ink selection:bg-brand selection:text-ink" class=("theme-paper", light_phone) class=("set-tiles", move || !light_phone())>
            <Show when=light_phone><shell::PhoneBar screen=screen /></Show>
            <div class=("lg:hidden", move || standalone.get())><shell::TopBar avatar=avatar signout_open=signout_open /></div>
            <div class="set-gutter mx-auto flex pb-16 pt-2 max-lg:px-4 max-lg:max-w-settings max-lg:gap-settings-gap max-lg:sm:px-8 lg:gap-[2.45%] lg:pt-6" class=("max-lg:pt-8", light_phone)>
                <div class=move || if standalone.get() { "contents lg:hidden" } else { "contents" }><shell::SideMenu screen=screen signout_open=signout_open /></div>
                <main class=move || if standalone.get() { "mx-auto w-full min-w-0 max-w-settings-main flex-1 lg:fixed lg:inset-0 lg:z-(--z-screen) lg:mx-0 lg:w-auto lg:max-w-none lg:overflow-auto lg:bg-page lg:pb-16" } else { "mx-auto w-full min-w-0 max-w-settings-main flex-1 lg:mx-0 lg:w-[70%] lg:max-w-none lg:flex-none" }>
                    <shell::StandaloneBar screen=screen avatar=avatar signout_open=signout_open />
                    <div class=move || if standalone.get() { "lg:mx-auto lg:max-w-[690px] lg:pt-[13px]" } else { "" }>
                        <shell::PageHeading screen=screen gate_active=gate_active light_phone=Signal::derive(light_phone) />
                        <shell::ForProfile screen=screen />
                        {move || {
                            let current = screen.get();
                            if current.needs_account_check() || current.needs_profile_check() {
                                let account = current.needs_account_check();
                                let shown = StoredValue::new(current);
                                view! { <stepup::Gate account=account>{move || screen_body(shown.get_value())}</stepup::Gate> }.into_any()
                            } else {
                                screen_body(current)
                            }
                        }}
                    </div>
                    <div class=move || if standalone.get() { "hidden lg:block" } else { "hidden" }><footer::AccountFooter /></div>
                </main>
            </div>
            <widgets::Toaster />
            <widgets::ConfirmDialog open=signout_open title=t("set-sign-out") body=t("set-sign-out-body") confirm_label=t("set-sign-out") on_confirm=Callback::new(|_| hub::sign_out_now()) />
        </div>
    }
}

/// The content of one screen. Some addresses are two screens, one per form (note 114, only one is mounted): phones and tablets get
/// the app's list, computers the website's page.
fn screen_body(screen: Screen) -> AnyView {
    use crate::form::DesktopOr;
    match screen {
        // phones and tablets: the hub list; computers: the hub address shows the account overview
        Screen::Hub => view! { <DesktopOr compact=|| view! { <hub::Hub /> } desktop=|| view! { <account::Account /> } /> }.into_any(),
        Screen::Profiles => view! { <hub::ManageProfiles /> }.into_any(),
        // phones and tablets: the Edit profile list of the app; computers: "Manage profile and preferences" like the website, and Edit profile is its own page
        Screen::Profile(id) => {
            let (a, b) = (id.clone(), id);
            view! { <DesktopOr compact=move || view! { <profile::EditProfile id=a.clone() /> } desktop=move || view! { <profile::ProfileOverview id=b.clone() /> } /> }.into_any()
        }
        // phones and tablets: the Edit profile list; computers: the Edit Profile page of the website
        Screen::ProfileEdit(id) => {
            let (a, b) = (id.clone(), id);
            view! { <DesktopOr compact=move || view! { <profile::EditProfile id=a.clone() /> } desktop=move || view! { <edit_profile_page::EditProfilePage id=b.clone() /> } /> }.into_any()
        }
        Screen::Avatar(id) => view! { <profile::AvatarPicker id=id /> }.into_any(),
        Screen::Restrictions(id) => view! { <restrictions::Restrictions id=id /> }.into_any(),
        Screen::Ratings(id) => view! { <restrictions::Ratings id=id /> }.into_any(),
        Screen::Blocked(id) => view! { <restrictions::Blocked id=id /> }.into_any(),
        Screen::Lock(id) => view! { <profile::Lock id=id /> }.into_any(),
        Screen::MainProfile(id) => view! { <profile::MainProfile id=id /> }.into_any(),
        Screen::Language(id) => view! { <languages::DisplayLanguage id=id /> }.into_any(),
        Screen::AudioSubtitles(id) => view! { <languages::AudioSubtitles id=id /> }.into_any(),
        Screen::Subtitles(id) => view! { <subtitles::Subtitles id=id /> }.into_any(),
        Screen::Playback(id) => view! { <profile::Playback id=id /> }.into_any(),
        Screen::Notifications(id) => view! { <notifications::Notifications id=id /> }.into_any(),
        Screen::NotificationChannel(id, channel) => view! { <notifications::Channel id=id channel=channel /> }.into_any(),
        Screen::GameHandle(id) => view! { <profile::GameHandle id=id /> }.into_any(),
        Screen::TransferProfile(id) => view! { <transfer::TransferProfile id=id /> }.into_any(),
        Screen::App => view! { <app::AppSettings /> }.into_any(),
        Screen::Licences => view! { <app::Licences /> }.into_any(),
        Screen::CheckNetwork => view! { <diagnostics::CheckNetwork /> }.into_any(),
        Screen::PlaybackSpec => view! { <diagnostics::PlaybackSpec /> }.into_any(),
        Screen::SpeedTest => view! { <diagnostics::SpeedTest /> }.into_any(),
        Screen::Account => view! { <account::Account /> }.into_any(),
        Screen::AccountSecurity => view! { <account::SecurityPage /> }.into_any(),
        Screen::AccountDevices => view! { <account::DevicesPage /> }.into_any(),
        Screen::AccountProfiles => view! { <account::ProfilesPage /> }.into_any(),
        Screen::Name => view! { <account::Name /> }.into_any(),
        Screen::Password => view! { <account::Password /> }.into_any(),
        Screen::Email => view! { <account::Email /> }.into_any(),
        Screen::Phone => view! { <account::Phone /> }.into_any(),
        Screen::Access => view! { <devices::Access /> }.into_any(),
        Screen::Privacy => view! { <privacy::Privacy /> }.into_any(),
        Screen::Activity => view! { <activity::Activity /> }.into_any(),
        Screen::PersonalInfo => view! { <privacy::PersonalInfo /> }.into_any(),
        Screen::Testing => view! { <privacy::Testing /> }.into_any(),
        Screen::TransferSetting => view! { <transfer::TransferSetting /> }.into_any(),
        Screen::Delete => view! { <delete_account::DeleteAccount /> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_screen() -> Vec<Screen> {
        let id = || "p1".to_string();
        vec![
            Screen::Profiles,
            Screen::Profile(id()),
            Screen::Avatar(id()),
            Screen::Restrictions(id()),
            Screen::Ratings(id()),
            Screen::Blocked(id()),
            Screen::Lock(id()),
            Screen::MainProfile(id()),
            Screen::Language(id()),
            Screen::AudioSubtitles(id()),
            Screen::Subtitles(id()),
            Screen::Playback(id()),
            Screen::Notifications(id()),
            Screen::NotificationChannel(id(), "email".into()),
            Screen::NotificationChannel(id(), "push".into()),
            Screen::GameHandle(id()),
            Screen::TransferProfile(id()),
            Screen::App,
            Screen::Licences,
            Screen::CheckNetwork,
            Screen::PlaybackSpec,
            Screen::SpeedTest,
            Screen::Account,
            Screen::AccountSecurity,
            Screen::AccountDevices,
            Screen::AccountProfiles,
            Screen::Name,
            Screen::Password,
            Screen::Email,
            Screen::Phone,
            Screen::Access,
            Screen::Privacy,
            Screen::Activity,
            Screen::PersonalInfo,
            Screen::Testing,
            Screen::TransferSetting,
            Screen::Delete,
        ]
    }

    #[test]
    fn every_address_reads_back_to_its_screen() {
        for s in every_screen() {
            assert_eq!(Screen::parse(&s.path(), "other"), s, "{}", s.path());
        }
        assert_eq!(Screen::parse("/settings", "p1"), Screen::Hub);
        assert_eq!(Screen::parse("/settings/profile", "p1"), Screen::Profile("p1".into()));
        assert_eq!(Screen::parse("/settings/nonsense", "p1"), Screen::Hub);
    }

    #[test]
    fn back_stays_inside_the_tree_and_ends_at_browse() {
        for s in every_screen() {
            let mut at = s.clone();
            for _ in 0..6 {
                if at == Screen::Hub {
                    break;
                }
                at = Screen::parse(&at.parent(), "p1");
            }
            assert_eq!(at, Screen::Hub, "{:?} never reaches the hub on a phone", s);
            // from 1024 px the website's tree is used; it ends at the overview, which ends at /browse
            let mut at = s.clone();
            for _ in 0..6 {
                if at == Screen::Account {
                    break;
                }
                at = Screen::parse(&at.parent_wide(), "p1");
            }
            assert_eq!(at, Screen::Account, "{:?} never reaches the overview on a wide screen", s);
        }
        assert_eq!(Screen::Hub.parent(), routes::BROWSE);
        assert_eq!(Screen::Ratings("p1".into()).parent(), "/settings/profile/p1/restrictions");
    }

    #[test]
    fn every_screen_has_a_title_that_exists_in_english() {
        let en = include_str!("../../../i18n/en.ftl");
        for s in every_screen() {
            assert!(en.lines().any(|l| l.starts_with(&format!("{} =", s.title_key()))), "missing {}", s.title_key());
        }
    }

    #[test]
    fn addresses_written_in_data_files_match_the_tree() {
        assert!(crate::data::settings::LEGAL_LINKS.iter().any(|(_, _, href)| *href == Screen::Licences.path()));
        assert!(crate::data::help::QUICK_LINKS.iter().any(|(id, _, href)| *id == "profiles" && *href == Screen::Profiles.path()));
    }

    #[test]
    fn only_account_changes_need_the_check() {
        // the Netflix website opens Change password and Create a profile lock directly (the password page asks the current password itself)
        assert!(!Screen::Password.needs_account_check());
        assert!(!Screen::Lock("p1".into()).needs_profile_check());
        assert!(Screen::Name.needs_account_check());
        assert!(Screen::Ratings("p1".into()).needs_profile_check());
        assert!(!Screen::Subtitles("p1".into()).needs_profile_check());
        assert!(!Screen::Privacy.needs_account_check());
    }
}

