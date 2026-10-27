//! Settings and Account, one route per screen at netflix.com's own addresses (`/account...`, `/settings/<page>/<profile>`, `/password`...,
//! read live on 2026-10-27, note 138). Phones and tablets show full screens with a back arrow and reach the top ones from the profile
//! sheet of the phone's My List tab; from 1024 px wide the same routes show a sidebar next to the screen. The tree is recorded in
//! notes/32-settings-rebuild-plan-2026-10-11.md.
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
pub(crate) mod subtitles;
mod widgets;

pub use prefs::{provide_prefs, use_prefs};

use crate::config::routes;
use crate::i18n::t;
use crate::store::use_profile_store;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// One screen of the tree. Profile screens carry the profile id they edit; the account's name, privacy and viewing activity carry the
/// profile in use, as Netflix's addresses do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
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
    Name(String),
    Password,
    Email,
    Phone,
    Access,
    Privacy(String),
    Activity(String),
    PersonalInfo,
    Testing,
    TransferSetting,
    Delete,
}

/// Which sidebar entry a screen belongs to (desktop). The phone has no sidebar, it has the profile sheet.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Overview,
    Security,
    Devices,
    Profiles,
    App,
}

/// The words after `/settings/` that name a page of a profile (`/settings/<word>/<profile>`, or the profile in use without one).
const PROFILE_PAGES: &[&str] = &["lock", "language", "audio-subtitles", "restrictions", "ratings", "blocked", "main", "subtitles", "playback", "viewed", "privacy", "name"];

impl Screen {
    /// Reads an address the way netflix.com does (read live on 2026-10-27, notes 137 and 138): the account under `/account` and at
    /// `/password`, `/email`, `/phonenumber`, `/manageaccountaccess` and `/donottest`; a profile under `/settings/<page>/<profile>` and
    /// `/notificationSettings`. `active` is the profile used when an address names none, and `/settings` alone or with a word that is
    /// neither a page nor a profile (`is_profile`) is the profile in use, as on Netflix. The addresses of before (`/settings/account/...`,
    /// `/settings/profile/<id>/...`) still read (`is_old`). None: an address the site does not have.
    pub fn parse(path: &str, active: &str, is_profile: impl Fn(&str) -> bool) -> Option<Screen> {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let id = |rest: &[&str]| rest.first().map(|s| s.to_string()).unwrap_or_else(|| active.to_string());
        let channel = |c: &str| crate::data::settings::NOTIFICATION_CHANNELS.iter().any(|(name, _, _)| *name == c);
        Some(match parts.as_slice() {
            ["account"] => Screen::Account,
            ["account", "security"] => Screen::AccountSecurity,
            ["account", "devices"] => Screen::AccountDevices,
            ["account", "profiles"] => Screen::AccountProfiles,
            ["account", "getmyinfo"] => Screen::PersonalInfo,
            ["account", "delete"] => Screen::Delete,
            ["account", "profile", "transfer", rest @ ..] if rest.len() <= 1 => Screen::TransferProfile(id(rest)),
            ["password"] => Screen::Password,
            ["email"] => Screen::Email,
            ["phonenumber"] => Screen::Phone,
            ["manageaccountaccess"] => Screen::Access,
            ["donottest"] => Screen::Testing,
            ["notificationSettings"] => Screen::Notifications(active.to_string()),
            ["notificationSettings", c, rest @ ..] if channel(c) && rest.len() <= 1 => Screen::NotificationChannel(id(rest), c.to_string()),
            ["notificationSettings", profile] => Screen::Notifications(profile.to_string()),
            ["settings", rest @ ..] => Self::parse_settings(rest, active, is_profile),
            _ => return None,
        })
    }

    /// The part after `/settings`.
    fn parse_settings(rest: &[&str], active: &str, is_profile: impl Fn(&str) -> bool) -> Screen {
        let id = |rest: &[&str]| rest.first().map(|s| s.to_string()).unwrap_or_else(|| active.to_string());
        match rest {
            ["app"] => Screen::App,
            ["app", "licences"] => Screen::Licences,
            ["app", "network"] => Screen::CheckNetwork,
            ["app", "playback-spec"] => Screen::PlaybackSpec,
            ["app", "speed-test"] => Screen::SpeedTest,
            ["profileTransfer", "enable"] => Screen::TransferSetting,
            ["profile", "edit", r @ ..] if r.len() <= 1 => Screen::ProfileEdit(id(r)),
            ["profile", "icon", r @ ..] if r.len() <= 1 => Screen::Avatar(id(r)),
            ["profile", "gamehandle", r @ ..] if r.len() <= 1 => Screen::GameHandle(id(r)),
            ["account", r @ ..] => Self::old_account(r, active),
            ["profile", profile, r @ ..] => Self::old_profile(profile, r),
            // a profile's own id first: the account's first profiles are called "main" and "kids" before the account names them
            [profile] if is_profile(profile) => Screen::Profile(profile.to_string()),
            [page, r @ ..] if r.len() <= 1 && PROFILE_PAGES.contains(page) => {
                let id = id(r);
                match *page {
                    "lock" => Screen::Lock(id),
                    "language" => Screen::Language(id),
                    "audio-subtitles" => Screen::AudioSubtitles(id),
                    "restrictions" => Screen::Restrictions(id),
                    "ratings" => Screen::Ratings(id),
                    "blocked" => Screen::Blocked(id),
                    "main" => Screen::MainProfile(id),
                    "subtitles" => Screen::Subtitles(id),
                    "playback" => Screen::Playback(id),
                    "viewed" => Screen::Activity(id),
                    "privacy" => Screen::Privacy(id),
                    _ => Screen::Name(id),
                }
            }
            _ => Screen::Profile(active.to_string()),
        }
    }

    /// `/settings/account/...` before 2026-10-27.
    fn old_account(rest: &[&str], active: &str) -> Screen {
        let active = active.to_string();
        match rest {
            ["security"] => Screen::AccountSecurity,
            ["devices"] => Screen::AccountDevices,
            ["profiles"] => Screen::AccountProfiles,
            ["name"] => Screen::Name(active),
            ["password"] => Screen::Password,
            ["email"] => Screen::Email,
            ["phone"] => Screen::Phone,
            ["access"] => Screen::Access,
            ["privacy"] => Screen::Privacy(active),
            ["privacy", "activity"] => Screen::Activity(active),
            ["personal-info"] => Screen::PersonalInfo,
            ["testing"] => Screen::Testing,
            ["transfer"] => Screen::TransferSetting,
            ["delete"] => Screen::Delete,
            _ => Screen::Account,
        }
    }

    /// `/settings/profile/<id>/...` before 2026-10-27.
    fn old_profile(profile: &str, rest: &[&str]) -> Screen {
        let id = profile.to_string();
        match rest {
            ["edit"] => Screen::ProfileEdit(id),
            ["avatar"] => Screen::Avatar(id),
            ["restrictions"] => Screen::Restrictions(id),
            ["restrictions", "ratings"] => Screen::Ratings(id),
            ["restrictions", "blocked"] => Screen::Blocked(id),
            ["lock"] => Screen::Lock(id),
            ["main"] => Screen::MainProfile(id),
            ["language"] => Screen::Language(id),
            ["audio-subtitles"] => Screen::AudioSubtitles(id),
            ["subtitles"] => Screen::Subtitles(id),
            ["playback"] => Screen::Playback(id),
            ["notifications"] => Screen::Notifications(id),
            ["notifications", c] => Screen::NotificationChannel(id, c.to_string()),
            ["game-handle"] => Screen::GameHandle(id),
            ["transfer"] => Screen::TransferProfile(id),
            _ => Screen::Profile(id),
        }
    }

    /// An address of before 2026-10-27, which moves to the screen's address now.
    pub fn is_old(path: &str) -> bool {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        matches!(parts.as_slice(), ["settings", "account", ..]) || matches!(parts.as_slice(), ["settings", "profile", second, ..] if !["edit", "icon", "gamehandle"].contains(second))
    }

    /// The address of this screen, the one netflix.com uses (note 138). The only place addresses are written, so pages link with
    /// `Screen::X.path()`. Without an id a profile's page is written without one, which reads back as the profile in use.
    pub fn path(&self) -> String {
        let page = |name: &str, id: &str| if id.is_empty() { format!("/settings/{name}") } else { format!("/settings/{name}/{id}") };
        let under = |base: &str, id: &str| if id.is_empty() { base.to_string() } else { format!("{base}/{id}") };
        match self {
            Screen::Profile(id) => under("/settings", id),
            Screen::ProfileEdit(id) => page("profile/edit", id),
            Screen::Avatar(id) => page("profile/icon", id),
            Screen::GameHandle(id) => page("profile/gamehandle", id),
            Screen::Restrictions(id) => page("restrictions", id),
            Screen::Ratings(id) => page("ratings", id),
            Screen::Blocked(id) => page("blocked", id),
            Screen::Lock(id) => page("lock", id),
            Screen::MainProfile(id) => page("main", id),
            Screen::Language(id) => page("language", id),
            Screen::AudioSubtitles(id) => page("audio-subtitles", id),
            Screen::Subtitles(id) => page("subtitles", id),
            Screen::Playback(id) => page("playback", id),
            Screen::Name(id) => page("name", id),
            Screen::Privacy(id) => page("privacy", id),
            Screen::Activity(id) => page("viewed", id),
            Screen::Notifications(id) => under("/notificationSettings", id),
            Screen::NotificationChannel(id, channel) => under(&format!("/notificationSettings/{channel}"), id),
            Screen::TransferProfile(id) => under("/account/profile/transfer", id),
            Screen::App => "/settings/app".into(),
            Screen::Licences => "/settings/app/licences".into(),
            Screen::CheckNetwork => "/settings/app/network".into(),
            Screen::PlaybackSpec => "/settings/app/playback-spec".into(),
            Screen::SpeedTest => "/settings/app/speed-test".into(),
            Screen::Account => "/account".into(),
            Screen::AccountSecurity => "/account/security".into(),
            Screen::AccountDevices => "/account/devices".into(),
            Screen::AccountProfiles => "/account/profiles".into(),
            Screen::Password => "/password".into(),
            Screen::Email => "/email".into(),
            Screen::Phone => "/phonenumber".into(),
            Screen::Access => "/manageaccountaccess".into(),
            Screen::PersonalInfo => "/account/getmyinfo".into(),
            Screen::Testing => "/donottest".into(),
            Screen::TransferSetting => "/settings/profileTransfer/enable".into(),
            Screen::Delete => "/account/delete".into(),
        }
    }

    /// The screen one level up on a phone or tablet. The back arrow goes here, so back always lands inside the tree; the top screens
    /// go back to the phone's My List tab, whose profile sheet opens them (nav/phone), as Netflix's app opens them from My Netflix.
    pub fn parent(&self) -> String {
        match self {
            Screen::App | Screen::Account | Screen::Profile(_) | Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles => routes::MY_LIST.into(),
            Screen::ProfileEdit(id) | Screen::Avatar(id) | Screen::Restrictions(id) | Screen::Lock(id) | Screen::MainProfile(id) | Screen::Language(id) | Screen::AudioSubtitles(id) | Screen::Subtitles(id) | Screen::Playback(id) | Screen::Notifications(id) | Screen::GameHandle(id) | Screen::TransferProfile(id) => Screen::Profile(id.clone()).path(),
            Screen::NotificationChannel(id, _) => Screen::Notifications(id.clone()).path(),
            Screen::Ratings(id) | Screen::Blocked(id) => Screen::Restrictions(id.clone()).path(),
            Screen::Licences | Screen::CheckNetwork | Screen::PlaybackSpec | Screen::SpeedTest => Screen::App.path(),
            Screen::Name(_) | Screen::Password | Screen::Email | Screen::Phone | Screen::Access | Screen::Privacy(_) | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete => Screen::Account.path(),
            Screen::Activity(id) => Screen::Privacy(id.clone()).path(),
        }
    }

    /// The screen one level up from 1024 px wide, where the account area is split into Security, Devices and Profiles pages
    /// like the Netflix website. Everything else goes where the phone goes.
    pub fn parent_wide(&self) -> String {
        match self {
            Screen::Name(_) | Screen::Password | Screen::Email | Screen::Phone | Screen::Privacy(_) | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete => Screen::AccountSecurity.path(),
            Screen::Access => Screen::AccountDevices.path(),
            Screen::Profile(_) => Screen::AccountProfiles.path(),
            Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles | Screen::App => Screen::Account.path(),
            Screen::Account => routes::BROWSE.into(),
            other => other.parent(),
        }
    }

    pub fn title_key(&self) -> &'static str {
        match self {
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
            Screen::Name(_) => "set-title-name",
            Screen::Password => "set-title-change-password",
            Screen::Email => "set-title-email",
            Screen::Phone => "set-title-phone",
            Screen::Access => "set-title-devices",
            Screen::Privacy(_) => "set-title-privacy",
            Screen::Activity(_) => "set-title-activity",
            Screen::PersonalInfo => "set-title-personal-info",
            Screen::Testing => "set-title-testing",
            Screen::TransferSetting => "set-title-transfer",
            Screen::Delete => "set-title-delete",
        }
    }

    /// Screens Netflix draws white on a phone (the Account pages); the rest are dark there. From 1024 px everything is white.
    fn light_on_phone(&self) -> bool {
        matches!(self, Screen::Account | Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles | Screen::Name(_) | Screen::Password | Screen::Email | Screen::Phone | Screen::Access | Screen::Privacy(_) | Screen::Activity(_) | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete)
    }

    /// Pages the website draws on their own, with only the logo above them and no side menu (the pages of one profile's preferences). Phones keep the app's frame.
    pub(crate) fn standalone(&self) -> bool {
        matches!(self, Screen::Language(_) | Screen::AudioSubtitles(_) | Screen::Subtitles(_) | Screen::Playback(_) | Screen::Notifications(_) | Screen::NotificationChannel(_, _) | Screen::GameHandle(_) | Screen::TransferProfile(_) | Screen::Lock(_) | Screen::Privacy(_) | Screen::Activity(_))
    }

    /// The profile a page is about: the one in its address, else (privacy, activity) the one in use.
    pub(crate) fn profile_of(&self) -> Option<String> {
        match self {
            Screen::Language(id) | Screen::AudioSubtitles(id) | Screen::Subtitles(id) | Screen::Playback(id) | Screen::Notifications(id) | Screen::NotificationChannel(id, _) | Screen::GameHandle(id) | Screen::TransferProfile(id) | Screen::Lock(id) => Some(id.clone()),
            _ => None,
        }
    }

    /// The profile whose settings this screen shows (Manage Profiles opens them without a profile in use, store::shown_without_profile).
    pub fn edited_profile(&self) -> Option<&str> {
        match self {
            Screen::Profile(id) | Screen::ProfileEdit(id) | Screen::Avatar(id) | Screen::Restrictions(id) | Screen::Ratings(id) | Screen::Blocked(id) | Screen::Lock(id) | Screen::MainProfile(id) | Screen::Language(id) | Screen::AudioSubtitles(id) | Screen::Subtitles(id) | Screen::Playback(id) | Screen::Notifications(id) | Screen::NotificationChannel(id, _) | Screen::GameHandle(id) | Screen::TransferProfile(id) => Some(id),
            _ => None,
        }
    }

    fn section(&self) -> Section {
        match self {
            Screen::Account => Section::Overview,
            Screen::AccountSecurity | Screen::Name(_) | Screen::Password | Screen::Email | Screen::Phone | Screen::Privacy(_) | Screen::Activity(_) | Screen::PersonalInfo | Screen::Testing | Screen::TransferSetting | Screen::Delete => Section::Security,
            Screen::AccountDevices | Screen::Access => Section::Devices,
            Screen::App | Screen::Licences | Screen::CheckNetwork | Screen::PlaybackSpec | Screen::SpeedTest => Section::App,
            _ => Section::Profiles,
        }
    }

    /// Screens that change account data and ask for the password first.
    fn needs_account_check(&self) -> bool {
        matches!(self, Screen::Name(_) | Screen::Email | Screen::Phone | Screen::Access | Screen::PersonalInfo | Screen::Delete)
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
    let parsed = Memo::new(move |_| Screen::parse(&location.pathname.get(), &active.get(), |id| profiles.profiles.with(|l| l.iter().any(|p| p.id == id))));
    let screen = Memo::new(move |_| parsed.get().unwrap_or(Screen::Account));
    let navigate = leptos_router::hooks::use_navigate();
    Effect::new(move |_| {
        let path = location.pathname.get();
        // the address has left this page (it moved to "Lost your way?" below and this page is being taken down)
        if path == routes::NOT_FOUND {
            return;
        }
        let replace = leptos_router::NavigateOptions { replace: true, ..Default::default() };
        match parsed.get() {
            // an address the site does not have: Netflix's "Lost your way?" (pages/not_found.rs)
            None => navigate(&crate::pages::not_found::address_for_lost(), replace),
            // the addresses of before 2026-10-27 move to Netflix's
            Some(s) if Screen::is_old(&path) => navigate(&format!("{}{}", s.path(), location.search.get_untracked()), replace),
            Some(_) => {}
        }
    });
    Effect::new(move |_| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            doc.set_title(&format!("{} | {}", t(screen.get().title_key()), crate::brand::NAME));
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
        Screen::Name(_) => view! { <account::Name /> }.into_any(),
        Screen::Password => view! { <account::Password /> }.into_any(),
        Screen::Email => view! { <account::Email /> }.into_any(),
        Screen::Phone => view! { <account::Phone /> }.into_any(),
        Screen::Access => view! { <devices::Access /> }.into_any(),
        Screen::Privacy(_) => view! { <privacy::Privacy /> }.into_any(),
        Screen::Activity(_) => view! { <activity::Activity /> }.into_any(),
        Screen::PersonalInfo => view! { <privacy::PersonalInfo /> }.into_any(),
        Screen::Testing => view! { <privacy::Testing /> }.into_any(),
        Screen::TransferSetting => view! { <transfer::TransferSetting /> }.into_any(),
        Screen::Delete => view! { <delete_account::DeleteAccount /> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(path: &str) -> Option<Screen> {
        Screen::parse(path, "active", |id| id == "p1" || id == "main")
    }

    fn every_screen() -> Vec<Screen> {
        let id = || "p1".to_string();
        vec![
            Screen::Profile(id()),
            Screen::ProfileEdit(id()),
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
            Screen::Name(id()),
            Screen::Password,
            Screen::Email,
            Screen::Phone,
            Screen::Access,
            Screen::Privacy(id()),
            Screen::Activity(id()),
            Screen::PersonalInfo,
            Screen::Testing,
            Screen::TransferSetting,
            Screen::Delete,
        ]
    }

    #[test]
    fn every_address_reads_back_to_its_screen() {
        for s in every_screen() {
            assert_eq!(read(&s.path()), Some(s.clone()), "{}", s.path());
            assert!(!Screen::is_old(&s.path()), "{} reads as an old address", s.path());
        }
    }

    #[test]
    fn the_addresses_are_netflixs() {
        // read live on netflix.com, 2026-10-27 (note 138)
        let p = || "p1".to_string();
        for (s, address) in [
            (Screen::Account, "/account"),
            (Screen::AccountSecurity, "/account/security"),
            (Screen::AccountDevices, "/account/devices"),
            (Screen::AccountProfiles, "/account/profiles"),
            (Screen::Password, "/password"),
            (Screen::Email, "/email"),
            (Screen::Phone, "/phonenumber"),
            (Screen::Access, "/manageaccountaccess"),
            (Screen::PersonalInfo, "/account/getmyinfo"),
            (Screen::Testing, "/donottest"),
            (Screen::TransferSetting, "/settings/profileTransfer/enable"),
            (Screen::Delete, "/account/delete"),
            (Screen::Profile(p()), "/settings/p1"),
            (Screen::ProfileEdit(p()), "/settings/profile/edit/p1"),
            (Screen::Name(p()), "/settings/name/p1"),
            (Screen::Lock(p()), "/settings/lock/p1"),
            (Screen::Language(p()), "/settings/language/p1"),
            (Screen::Restrictions(p()), "/settings/restrictions/p1"),
            (Screen::Subtitles(p()), "/settings/subtitles/p1"),
            (Screen::Playback(p()), "/settings/playback/p1"),
            (Screen::Activity(p()), "/settings/viewed/p1"),
            (Screen::Privacy(p()), "/settings/privacy/p1"),
            (Screen::Notifications(p()), "/notificationSettings/p1"),
            (Screen::NotificationChannel(p(), "email".into()), "/notificationSettings/email/p1"),
            (Screen::NotificationChannel(p(), "push".into()), "/notificationSettings/push/p1"),
            (Screen::TransferProfile(p()), "/account/profile/transfer/p1"),
        ] {
            assert_eq!(s.path(), address);
        }
    }

    #[test]
    fn netflixs_fallbacks_hold() {
        // /settings alone, and with a word that is no page and no profile, is the profile in use; the account's first profile "main" is a profile
        assert_eq!(read("/settings"), Some(Screen::Profile("active".into())));
        assert_eq!(read("/settings/devices"), Some(Screen::Profile("active".into())));
        assert_eq!(read("/settings/main"), Some(Screen::Profile("main".into())));
        assert_eq!(read("/settings/restrictions"), Some(Screen::Restrictions("active".into())));
        assert_eq!(read("/notificationSettings"), Some(Screen::Notifications("active".into())));
        // any other address the site does not have
        assert_eq!(read("/account/nosuchpage"), None);
        assert_eq!(read("/nosuchpage"), None);
    }

    #[test]
    fn the_addresses_of_before_move() {
        for (old, now) in [
            ("/settings/account", Screen::Account),
            ("/settings/account/security", Screen::AccountSecurity),
            ("/settings/account/profiles", Screen::AccountProfiles),
            ("/settings/account/password", Screen::Password),
            ("/settings/account/phone", Screen::Phone),
            ("/settings/account/privacy/activity", Screen::Activity("active".into())),
            ("/settings/account/transfer", Screen::TransferSetting),
            ("/settings/profile/p1", Screen::Profile("p1".into())),
            ("/settings/profile/p1/edit", Screen::ProfileEdit("p1".into())),
            ("/settings/profile/p1/restrictions/ratings", Screen::Ratings("p1".into())),
            ("/settings/profile/p1/notifications/push", Screen::NotificationChannel("p1".into(), "push".into())),
            ("/settings/profile/p1/transfer", Screen::TransferProfile("p1".into())),
        ] {
            assert!(Screen::is_old(old), "{old}");
            assert_eq!(read(old), Some(now), "{old}");
        }
    }

    #[test]
    fn back_stays_inside_the_tree() {
        for s in every_screen() {
            // a phone ends at the My List tab, whose profile sheet opens the top screens
            let mut at = s.path();
            for _ in 0..6 {
                match read(&at) {
                    Some(screen) => at = screen.parent(),
                    None => break,
                }
            }
            assert_eq!(at, routes::MY_LIST, "{:?} never reaches the My List tab on a phone", s);
            // from 1024 px the website's tree is used; it ends at the overview, which ends at /browse
            let mut at = s.clone();
            for _ in 0..6 {
                if at == Screen::Account {
                    break;
                }
                at = read(&at.parent_wide()).expect("a settings address");
            }
            assert_eq!(at, Screen::Account, "{:?} never reaches the overview on a wide screen", s);
        }
        assert_eq!(Screen::Account.parent_wide(), routes::BROWSE);
        assert_eq!(Screen::Ratings("p1".into()).parent(), "/settings/restrictions/p1");
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
        assert!(crate::data::help::QUICK_LINKS.iter().any(|(id, _, href)| *id == "profiles" && *href == Screen::AccountProfiles.path()));
    }

    #[test]
    fn only_account_changes_need_the_check() {
        // the Netflix website opens Change password and Create a profile lock directly (the password page asks the current password itself)
        assert!(!Screen::Password.needs_account_check());
        assert!(!Screen::Lock("p1".into()).needs_profile_check());
        assert!(Screen::Name("p1".into()).needs_account_check());
        assert!(Screen::Ratings("p1".into()).needs_profile_check());
        assert!(!Screen::Subtitles("p1".into()).needs_profile_check());
        assert!(!Screen::Privacy("p1".into()).needs_account_check());
    }
}
