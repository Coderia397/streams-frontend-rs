//! Edit Profile and the screens that hang off it: avatar and profile lock. Restrictions, languages and subtitles have their own
//! files. Profiles are kept in this browser; every change goes through `change` (one place), which is where the account tables
//! take over later (public.profiles: name, avatar, kind, has_pin, is_main; the rating limit is in profile_settings.age_rating).
use super::hub::persist_profiles;
use super::prefs::use_prefs;
use super::widgets::{toast, ConfirmDialog, DigitBoxes, Field, FormButtons, LinkRow, Message, OutlineButton, Section, SwitchRow, Tone};
use super::Screen;
use crate::config::{GAME_HANDLE_MAX, GAME_HANDLE_MIN, PIN_LENGTH, PROFILE_NAME_MAX};
use crate::data::settings::{rating, HANDLE_ADJECTIVES, HANDLE_NOUNS, PROFILE_DATA_USAGE};
use crate::i18n::{t, t_args};
use crate::models::profile::Profile;
use crate::store::use_profile_store;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

fn find(id: &str) -> Option<Profile> {
    use_profile_store().profiles.with(|l| l.iter().find(|p| p.id == id).cloned())
}

fn change(id: &str, apply: impl FnOnce(&mut Profile)) {
    let store = use_profile_store();
    store.profiles.update(|l| {
        if let Some(p) = l.iter_mut().find(|p| p.id == id) {
            apply(p);
        }
    });
    persist_profiles(store);
}

fn not_found() -> AnyView {
    view! { <p class="text-body-md text-ink/60">{t("set-profile-missing")}</p> }.into_any()
}

/// A name another profile of the account already uses (the database keeps names unique per account, ignoring case).
fn name_taken(id: &str, name: &str) -> bool {
    use_profile_store().profiles.with(|l| l.iter().any(|p| p.id != id && p.name.trim().to_lowercase() == name.trim().to_lowercase()))
}

#[component]
pub fn EditProfile(id: String, #[prop(optional)] edit_page: bool) -> impl IntoView {
    let store = use_profile_store();
    let prefs = use_prefs();
    let navigate = use_navigate();
    let confirm_delete = RwSignal::new(false);
    let note = RwSignal::new(None::<(Tone, String)>);
    let name = RwSignal::new(find(&id).map(|p| p.name).unwrap_or_default());
    let id = StoredValue::new(id);

    let profile = Memo::new(move |_| store.profiles.with(|l| l.iter().find(|p| p.id == id.get_value()).cloned()));
    let p_prefs = Memo::new(move |_| prefs.profile(&id.get_value()));
    // follows the profile when the address changes (the sidebar switches between profiles on the same component)
    Effect::new(move |_| {
        if let Some(p) = profile.get() {
            name.set(p.name);
        }
    });
    let save_name = move || {
        let value = name.get_untracked().trim().chars().take(PROFILE_NAME_MAX).collect::<String>();
        let current = profile.get_untracked().map(|p| p.name).unwrap_or_default();
        if value.is_empty() {
            name.set(current);
        } else if name_taken(&id.get_value(), &value) {
            note.set(Some((Tone::Error, t("set-name-taken"))));
            name.set(current);
        } else if value != current {
            note.set(None);
            change(&id.get_value(), |p| p.name = value);
            toast(t("set-saved"));
        }
    };
    let delete = Callback::new(move |_| {
        remove_profile(&id.get_value());
        navigate(&Screen::Profiles.path(), Default::default());
    });
    move || {
        let Some(p) = profile.get() else { return not_found() };
        let kids = p.is_kids;
        let pid = p.id.clone();
        // the main profile cannot be deleted, and the account keeps at least one profile
        let removable = store.profiles.with(|l| crate::models::profile_rules::can_delete(l, &p.id));
        let can_be_main = p.is_default != Some(true) && !kids;
        let maturity_value = Signal::derive(move || { let r = rating(p_prefs.get().rating_for(kids)); if r.key == crate::data::settings::DEFAULT_RATING { t("set-restrictions-none") } else { t_args("set-rating-up-to", &[("rating", r.key.into())]) } });
        let handle_value = Signal::derive(move || p_prefs.get().game_handle.unwrap_or_else(|| t("set-handle-create")));
        let lock_value = Signal::derive(move || if profile.get().is_some_and(|p| p.pin.is_some()) { t("set-lock-on") } else { t("set-lock-off") });
        let language_value = Signal::derive(move || crate::i18n::native_name(&p_prefs.get().ui_language.unwrap_or_else(crate::i18n::language)));
        let screen = move |s: fn(String) -> Screen| s(pid.clone()).path();
        view! {
            <div>
                <Message text=note />
                <div class="flex flex-col items-center gap-5 max-lg:mb-1 lg:mb-8 lg:flex-row lg:gap-6">
                    // radius: the avatar tile
                    <a href=screen(Screen::Avatar) class="group relative block h-29 w-29 shrink-0 overflow-hidden rounded-card lg:h-24 lg:w-24" aria-label=t("set-title-avatar")>
                        <img src=p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string()) alt="" class="h-full w-full object-cover" />
                        <span class="absolute bottom-1 end-1 flex h-7 w-7 items-center justify-center rounded-full bg-ink text-void">
                            <i class="ph-bold ph-pencil-simple text-fine" aria-hidden="true"></i>
                        </span>
                    </a>
                    <div class="w-full min-w-0 max-lg:max-w-75 lg:flex-1" on:focusout=move |_| if !crate::form::current().is_desktop() { save_name() } on:keydown=move |ev| if ev.key() == "Enter" { save_name() }>
                        <Field label=t("set-profile-name") label_wide_only=true value=name maxlength=PROFILE_NAME_MAX as u32 />
                        // the website saves with a button; the app saves when you leave the field
                        <div class="hidden gap-3 lg:flex">
                            <button type="button" on:click=move |_| save_name() class="btn-ink px-6 py-2">{t("set-save")}</button>
                            <button type="button" on:click=move |_| name.set(profile.get_untracked().map(|p| p.name).unwrap_or_default()) class="rounded-button px-6 py-2 text-body-md font-bold text-ink/80 hover:bg-ink/10">{t("set-cancel")}</button>
                        </div>
                    </div>
                </div>
                {edit_page.then(|| view! {
                    <div class="hidden lg:block">
                        <Section title=t("set-title-game-handle")>
                            <LinkRow icon="game-controller" title=t("set-title-game-handle") href=Screen::GameHandle(id.get_value()).path() value=handle_value />
                        </Section>
                        <Section title=t("set-contact-info")>
                            <LinkRow icon="identification-card" title=t("set-title-name") href=Screen::Name.path() />
                            <LinkRow icon="envelope-simple" title=t("set-title-email") href=Screen::Email.path() />
                            <LinkRow icon="device-mobile" title=t("set-title-phone") href=Screen::Phone.path() />
                        </Section>
                    </div>
                })}
                <div class=if edit_page { "lg:hidden" } else { "contents" }>
                <Section>
                    <LinkRow icon="game-controller" title=t("set-title-game-handle") href=screen(Screen::GameHandle) value=handle_value />
                    <LinkRow icon="warning-circle" title=t("set-title-restrictions") href=screen(Screen::Restrictions) value=maturity_value />
                    <LinkRow icon="lock-simple" title=t("set-title-lock") href=screen(Screen::Lock) value=lock_value />
                    <LinkRow icon="translate" title=t("set-title-language") hint=t("set-language-hint") href=screen(Screen::Language) value=language_value />
                    <LinkRow icon="subtitles" title=t("set-title-audio-subtitles") hint=t("set-audio-subtitles-hint") href=screen(Screen::AudioSubtitles) />
                    <LinkRow icon="text-aa" title=t("set-title-subtitles") hint=t("set-subtitles-hint") href=screen(Screen::Subtitles) />
                    // phones and tablets list the autoplay switches here; the website has them on a Playback page (note 114, one mounted)
                    <crate::form::DesktopOr
                        compact=move || view! { <AutoplaySwitches id=id.get_value() /> }
                        desktop={
                            let screen = screen.clone();
                            move || view! {
                            <LinkRow icon="play-circle" title=t("set-title-playback") hint=t("set-playback-hint") href=screen(Screen::Playback) />
                            <LinkRow icon="bell" title=t("set-title-notifications") hint=t("set-notifications-hint") href=screen(Screen::Notifications) />
                            <LinkRow icon="clock-counter-clockwise" title=t("set-title-activity") hint=t("set-privacy-activity-hint") href=Screen::Activity.path() />
                            <LinkRow icon="shield-check" title=t("set-title-privacy") hint=t("set-privacy-hint") href=Screen::Privacy.path() />
                            <LinkRow icon="arrows-left-right" title=t("set-title-transfer-profile") hint=t("set-transfer-profile-hint") href=screen(Screen::TransferProfile) />
                            }
                        }
                    />
                    {can_be_main.then(|| view! { <LinkRow icon="crown-simple" title=t("set-main-profile") hint=t("set-main-profile-hint") href=screen(Screen::MainProfile) /> })}
                </Section>
                {removable.then(|| view! { <OutlineButton label=t("set-delete-profile") danger=true on_click=Callback::new(move |_| confirm_delete.set(true)) /> })}
                <ConfirmDialog open=confirm_delete title=t("set-delete-profile") body=t("set-delete-profile-body") confirm_label=t("set-delete") danger=true on_confirm=delete />
                </div>
            </div>
        }
        .into_any()
    }
}

/// Removes a profile and everything kept for it; the person falls back to the profile picker if it was the one in use.
pub(super) fn remove_profile(pid: &str) {
    let store = use_profile_store();
    let prefs = use_prefs();
    store.profiles.update(|l| l.retain(|p| p.id != pid));
    if store.active_profile_id.get_untracked().as_deref() == Some(pid) {
        store.active_profile_id.set(None);
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = s.remove_item(crate::keys::ACTIVE_PROFILE);
        }
    }
    prefs.forget_profile(pid);
    persist_profiles(store);
}

/// From 1024 px the profile address is the website's "Manage profile and preferences": the profile with a link to Edit profile,
/// the profile lock, then the preferences, then Delete Profile.
#[component]
pub fn ProfileOverview(id: String) -> impl IntoView {
    let store = use_profile_store();
    let navigate = use_navigate();
    let confirm_delete = RwSignal::new(false);
    let id = StoredValue::new(id);
    let profile = Memo::new(move |_| store.profiles.with(|l| l.iter().find(|p| p.id == id.get_value()).cloned()));
    let delete = Callback::new(move |_| {
        remove_profile(&id.get_value());
        navigate(&Screen::Profiles.path(), Default::default());
    });
    move || {
        let Some(p) = profile.get() else { return not_found() };
        let pid = p.id.clone();
        let removable = store.profiles.with(|l| crate::models::profile_rules::can_delete(l, &p.id));
        let can_be_main = p.is_default != Some(true) && !p.is_kids;
        let path = |s: fn(String) -> Screen| s(pid.clone()).path();
        let (edit, lock, language, restrictions, subtitles, playback, notifications, transfer, main) = (path(Screen::ProfileEdit), path(Screen::Lock), path(Screen::Language), path(Screen::Restrictions), path(Screen::Subtitles), path(Screen::Playback), path(Screen::Notifications), path(Screen::TransferProfile), path(Screen::MainProfile));
        view! {
            <div>
                <Section>
                    <a href=edit.clone() class="flex items-center gap-4 px-5 py-3 hover:bg-ink/10">
                        <img src=p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string()) alt="" class="h-10 w-10 rounded-control object-cover" />
                        <span class="min-w-0 flex-1">
                            <span class="block truncate text-body-md font-bold">{p.name.clone()}</span>
                            <span class="block text-label text-ink/60">{t("set-po-edit-hint")}</span>
                        </span>
                        <i class="ph-bold ph-caret-right text-xl text-ink/40 rtl:-scale-x-100" aria-hidden="true"></i>
                    </a>
                    <LinkRow icon="lock-simple" title=t("set-title-lock") hint=t("set-po-lock-hint") href=lock />
                </Section>
                <Section title=t("set-po-preferences")>
                    <LinkRow icon="translate" title=t("set-po-languages") hint=t("set-po-languages-hint") href=language />
                    <LinkRow icon="warning-circle" title=t("set-po-restrictions") hint=t("set-po-restrictions-hint") href=restrictions />
                    <LinkRow icon="text-aa" title=t("set-title-subtitles") hint=t("set-po-subtitles-hint") href=subtitles />
                    <LinkRow icon="play-circle" title=t("set-title-playback") hint=t("set-po-playback-hint") href=playback />
                    <LinkRow icon="bell" title=t("set-title-notifications") hint=t("set-po-notifications-hint") href=notifications />
                    <LinkRow icon="clock-counter-clockwise" title=t("set-title-activity") hint=t("set-po-activity-hint") href=Screen::Activity.path() />
                    <LinkRow icon="shield-check" title=t("set-title-privacy") hint=t("set-po-privacy-hint") href=Screen::Privacy.path() />
                    <LinkRow icon="arrows-left-right" title=t("set-title-transfer") hint=t("set-transfer-profile-hint") href=transfer />
                    {can_be_main.then(|| view! { <LinkRow icon="crown-simple" title=t("set-main-profile") hint=t("set-main-profile-hint") href=main /> })}
                </Section>
                {removable.then(|| view! { <OutlineButton label=t("set-po-delete-profile") danger=true on_click=Callback::new(move |_| confirm_delete.set(true)) /> })}
                <ConfirmDialog open=confirm_delete title=t("set-delete-profile") body=t("set-delete-profile-body") confirm_label=t("set-delete") danger=true on_confirm=delete />
            </div>
        }
        .into_any()
    }
}

/// The two autoplay switches of a profile (profile_settings.autoplay_next and autoplay_previews).
#[component]
fn AutoplaySwitches(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let id = StoredValue::new(id);
    let p = Memo::new(move |_| prefs.profile(&id.get_value()));
    view! {
        <SwitchRow
            icon="skip-forward"
            title=t("set-autoplay-next")
            hint=t("set-autoplay-next-hint")
            checked=Signal::derive(move || p.get().autoplay_next)
            on_change=Callback::new(move |v| prefs.update_profile(&id.get_value(), |x| x.autoplay_next = v))
        />
        <SwitchRow
            icon="play-circle"
            title=t("set-autoplay-previews")
            hint=t("set-autoplay-previews-hint")
            checked=Signal::derive(move || p.get().autoplay_previews)
            on_change=Callback::new(move |v| prefs.update_profile(&id.get_value(), |x| x.autoplay_previews = v))
        />
    }
}

/// Playback settings (wide screens; phones have the autoplay switches on Edit profile): autoplay and data use per profile.
#[component]
pub fn Playback(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let id = StoredValue::new(id);
    let options: Vec<(String, String, String)> = PROFILE_DATA_USAGE.iter().map(|(v, label, hint)| (v.to_string(), t(label), t(hint))).collect();
    view! {
        <div>
            <Section title=t("set-playback-autoplay")><AutoplaySwitches id=id.get_value() /></Section>
            <h2 class="set-section-title mb-4 text-title-sm leading-row text-ink/85">{t("set-usage-title")}</h2>
            <div role="radiogroup" class="mb-8 rounded-card border border-set-edge bg-set-card px-4">
                {options.into_iter().map(|(value, label, hint)| {
                    let v = value.clone();
                    let pick = value.clone();
                    let sel = value.clone();
                    view! {
                        <button
                            type="button"
                            role="radio"
                            aria-checked=move || (prefs.profile(&id.get_value()).data_usage == v).to_string()
                            on:click=move |_| { let x = pick.clone(); prefs.update_profile(&id.get_value(), |p| p.data_usage = x); toast(t("set-saved")); }
                            class="set-row flex min-h-settings-row w-full items-center border-b border-set-line px-1 py-3 text-start last:border-b-0 hover:bg-ink/5"
                        >
                            <span class="min-w-0 flex-1">
                                <span class="block text-title-sm font-semibold leading-row text-ink">{label}</span>
                                <span class="block text-body-sm text-ink/70">{hint}</span>
                            </span>
                            <i class="ph-bold ph-check text-2xl text-brand" class=("invisible", move || prefs.profile(&id.get_value()).data_usage != sel) aria-hidden="true"></i>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Game handle: the name shown in games, 3 to 16 letters and digits. "Create" suggests one made of two words and a number.
#[component]
pub fn GameHandle(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let navigate = use_navigate();
    let id = StoredValue::new(id);
    let value = RwSignal::new(prefs.profile(&id.get_value()).game_handle.unwrap_or_default());
    let note = RwSignal::new(None::<(Tone, String)>);
    let suggest = move |_| {
        fn pick(list: &'static [&'static str]) -> &'static str {
            list[(js_sys::Math::random() * list.len() as f64) as usize % list.len()]
        }
        let digits = (js_sys::Math::random() * 1000.0) as u32;
        let handle: String = format!("{}{}{}", pick(HANDLE_ADJECTIVES), pick(HANDLE_NOUNS), digits).chars().take(GAME_HANDLE_MAX).collect();
        value.set(handle);
        note.set(None);
    };
    let save = Callback::new(move |_| {
        let handle = value.get_untracked().trim().to_string();
        if !valid_handle(&handle) {
            note.set(Some((Tone::Error, t_args("set-handle-invalid", &[("min", GAME_HANDLE_MIN.into()), ("max", GAME_HANDLE_MAX.into())]))));
            return;
        }
        prefs.update_profile(&id.get_value(), |p| p.game_handle = Some(handle));
        toast(t("set-saved"));
        navigate(&Screen::Profile(id.get_value()).path(), Default::default());
    });
    view! {
        <div class="max-w-md">
            <p class="mb-2 text-body-md text-ink/70">{t("set-handle-body")}</p>
            <p class="mb-6 text-body-md text-ink/70">{t("set-handle-change")}</p>
            {move || find(&id.get_value()).map(|p| view! {
                <div class="mb-6 flex flex-col items-center gap-1 text-center">
                    <img src=p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string()) alt="" class="mb-2 h-24 w-24 rounded-card object-cover" />
                    <span class="text-title font-bold">{p.name.clone()}</span>
                    <span class="flex items-center gap-1 text-body-sm text-ink/70"><i class="ph-bold ph-game-controller" aria-hidden="true"></i>{t("set-title-game-handle")}</span>
                </div>
            })}
            <Message text=note />
            <Field label=t("set-title-game-handle") value=value maxlength=GAME_HANDLE_MAX as u32 autocomplete="off" />
            <p class="-mt-2 mb-4 text-end text-label text-ink/50">{move || format!("{}/{}", value.with(|v| v.chars().count()), GAME_HANDLE_MAX)}</p>
            <button type="button" on:click=suggest class="mb-6 rounded-button border border-ink/40 px-5 py-2.5 text-body-md font-bold hover:bg-ink/10">{t("set-handle-suggest")}</button>
            <FormButtons save_label=t("set-save") on_save=save cancel_href=Screen::Profile(id.get_value()).path() busy=Signal::derive(move || { let v = value.get(); !valid_handle(v.trim()) || prefs.profile(&id.get_value()).game_handle.as_deref() == Some(v.trim()) }) />
        </div>
    }
}

fn valid_handle(handle: &str) -> bool {
    (GAME_HANDLE_MIN..=GAME_HANDLE_MAX).contains(&handle.chars().count()) && handle.chars().all(|c| c.is_ascii_alphanumeric())
}

#[component]
pub fn AvatarPicker(id: String) -> impl IntoView {
    use crate::components::profiles::icon_picker::IconPicker;
    let navigate = use_navigate();
    let id = StoredValue::new(id);
    let name = Signal::derive(move || find(&id.get_value()).map(|p| p.name).unwrap_or_default());
    let current = Signal::derive(move || find(&id.get_value()).and_then(|p| p.avatar_url));
    let kids = find(&id.get_value()).is_some_and(|p| p.is_kids);
    let (pick_nav, back_nav) = (navigate.clone(), navigate);
    // a page of its own, as on the website: the top bar over the picker, no side menu
    view! {
        <div class="fixed inset-0 z-(--z-screen) overflow-y-auto bg-page text-ink" style="--app-x: calc(8.3333vw + 72px);">
            <header class="flex h-16 items-center px-[var(--app-x)]">
                <a href=crate::config::routes::BROWSE aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="h-7 w-auto" /></a>
            </header>
            <IconPicker
                name=name
                current=current
                kids=kids
                on_pick=Callback::new(move |url: String| {
                    change(&id.get_value(), |p| p.avatar_url = Some(url));
                    pick_nav(&Screen::ProfileEdit(id.get_value()).path(), Default::default());
                })
                on_back=Callback::new(move |_| back_nav(&Screen::ProfileEdit(id.get_value()).path(), Default::default()))
            />
        </div>
    }
}

/// Makes this profile the main one. It sits behind the password check like the database function (set_main_profile).
#[component]
pub fn MainProfile(id: String) -> impl IntoView {
    let store = use_profile_store();
    let navigate = use_navigate();
    let id = StoredValue::new(id);
    let name = Memo::new(move |_| find(&id.get_value()).map(|p| p.name).unwrap_or_default());
    let confirm = Callback::new(move |_| {
        let pid = id.get_value();
        store.profiles.update(|l| {
            for p in l.iter_mut() {
                p.is_default = Some(p.id == pid);
            }
        });
        persist_profiles(store);
        navigate(&Screen::Profile(pid).path(), Default::default());
    });
    view! {
        <div class="max-w-md">
            <p class="mb-6 text-body-md text-ink/70">{move || t_args("set-main-profile-body", &[("name", name.get().into())])}</p>
            <FormButtons save_label=t("set-continue") on_save=confirm cancel_href=Screen::Profile(id.get_value()).path() />
        </div>
    }
}

fn valid_pin(pin: &str) -> bool {
    pin.chars().count() == PIN_LENGTH && pin.chars().all(|c| c.is_ascii_digit())
}

/// What the Lock screen is doing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LockMode {
    /// Showing the state and the actions.
    Idle,
    /// Entering a first PIN, or a new one together with the current one.
    Set,
    /// Asking for the current PIN before removing the lock.
    Remove,
}

#[component]
pub fn Lock(id: String) -> impl IntoView {
    let navigate = use_navigate();
    let id = StoredValue::new(id);
    let current = RwSignal::new(String::new());
    let pin = RwSignal::new(String::new());
    let note = RwSignal::new(None::<(Tone, String)>);
    let mode = RwSignal::new(LockMode::Idle);
    let signed_in = crate::services::account::session().is_some();
    let has_pin = Memo::new(move |_| find(&id.get_value()).is_some_and(|p| p.pin.is_some()));
    let back = move || Screen::Profile(id.get_value()).path();
    // with a PIN already set the current one is asked first, like the database functions do
    let current_ok = move || find(&id.get_value()).and_then(|p| p.pin).is_none_or(|saved| saved == current.get_untracked());

    // a profile of the account keeps its PIN in the database (hashed); a profile of this browser keeps it here
    let on_account = move || signed_in && crate::services::sync::is_uuid(&id.get_value());
    let lock_error = move |e: crate::services::account::AuthError| note.set(Some((Tone::Error, crate::pages::settings::account::error_text(&e))));
    let nav_save = navigate.clone();
    let save = Callback::new(move |_| {
        let value = pin.get_untracked();
        if !valid_pin(&value) {
            note.set(Some((Tone::Error, t_args("set-lock-invalid", &[("digits", PIN_LENGTH.into())]))));
            return;
        }
        if on_account() {
            let (pid, cur, had) = (id.get_value(), current.get_untracked(), has_pin.get_untracked());
            if had && cur.is_empty() {
                note.set(Some((Tone::Error, t("set-lock-wrong"))));
                return;
            }
            let nav = nav_save.clone();
            leptos::task::spawn_local(async move {
                match crate::services::account::pin_set(&pid, &value, had.then_some(cur.as_str())).await {
                    Ok(true) => {
                        change(&pid, |p| p.pin = Some(String::new()));
                        toast(t("set-saved"));
                        nav(&back(), Default::default());
                    }
                    Ok(false) => note.set(Some((Tone::Error, t("set-lock-wrong")))),
                    Err(e) => lock_error(e),
                }
            });
            return;
        }
        if !current_ok() {
            note.set(Some((Tone::Error, t("set-lock-wrong"))));
            return;
        }
        change(&id.get_value(), |p| p.pin = Some(value));
        toast(t("set-saved"));
        nav_save(&back(), Default::default());
    });
    let nav_remove = navigate.clone();
    let remove = Callback::new(move |_| {
        if on_account() {
            let (pid, cur) = (id.get_value(), current.get_untracked());
            let nav = nav_remove.clone();
            leptos::task::spawn_local(async move {
                match crate::services::account::pin_clear(&pid, &cur).await {
                    Ok(true) => {
                        change(&pid, |p| p.pin = None);
                        nav(&back(), Default::default());
                    }
                    Ok(false) => note.set(Some((Tone::Error, t("set-lock-wrong")))),
                    Err(e) => lock_error(e),
                }
            });
            return;
        }
        if !current_ok() {
            note.set(Some((Tone::Error, t("set-lock-wrong"))));
            return;
        }
        change(&id.get_value(), |p| p.pin = None);
        nav_remove(&back(), Default::default());
    });
    // forgot the PIN: only for a signed in account, after the password check this screen already asked for
    let nav_reset = navigate;
    let reset = Callback::new(move |_| {
        if on_account() {
            let pid = id.get_value();
            let nav = nav_reset.clone();
            leptos::task::spawn_local(async move {
                match crate::services::account::pin_reset(&pid).await {
                    Ok(()) => {
                        change(&pid, |p| p.pin = None);
                        nav(&back(), Default::default());
                    }
                    Err(e) => lock_error(e),
                }
            });
            return;
        }
        change(&id.get_value(), |p| p.pin = None);
        nav_reset(&back(), Default::default());
    });
    let confirm_reset = RwSignal::new(false);

    view! {
        <div class="max-lg:flex max-lg:min-h-settings-hero-min max-lg:flex-col max-lg:justify-center">
            <div class="mb-6 flex flex-col items-center gap-4 text-center">
                <span class="flex h-36 w-36 items-center justify-center rounded-full bg-ink/10 lg:h-24 lg:w-24"><AppIcon glyph=Glyph::Locked class="text-7xl lg:text-5xl" /></span>
                <h2 class="text-heading-lg font-bold lg:text-title">{move || if has_pin.get() { t("set-lock-title-on") } else { t("set-lock-title-off") }}</h2>
                <p class="max-w-md text-body-md text-ink/60">{move || t_args("set-lock-body", &[("digits", PIN_LENGTH.into())])}</p>
            </div>
            <Message text=note />
            {move || match (mode.get(), has_pin.get()) {
                (LockMode::Idle, false) => view! {
                    <button type="button" on:click=move |_| mode.set(LockMode::Set) class="btn-ink mx-auto block px-8 py-3 lg:w-full">{t("set-lock-create")}</button>
                }.into_any(),
                (LockMode::Idle, true) => view! {
                    <div class="space-y-3">
                        <button type="button" on:click=move |_| { note.set(None); mode.set(LockMode::Set) } class="btn-ink w-full px-8 py-3">{t("set-lock-change")}</button>
                        <OutlineButton label=t("set-lock-remove") danger=true on_click=Callback::new(move |_| { note.set(None); mode.set(LockMode::Remove) }) />
                        {signed_in.then(|| view! {
                            <button type="button" on:click=move |_| confirm_reset.set(true) class="w-full rounded-button px-6 py-3 text-body-md text-ink/70 underline underline-offset-2 hover:text-ink">{t("set-lock-forgot")}</button>
                        })}
                    </div>
                }.into_any(),
                (LockMode::Set, with_pin) => view! {
                    <div class="max-w-md">
                        {with_pin.then(|| view! { <DigitBoxes label=t("set-lock-current") value=current length=PIN_LENGTH secret=true /> })}
                        <DigitBoxes label=t("set-lock-pin") value=pin length=PIN_LENGTH secret=true />
                        <FormButtons save_label=t("set-save") on_save=save cancel_href=back() />
                    </div>
                }.into_any(),
                (LockMode::Remove, _) => view! {
                    <div class="max-w-md">
                        <p class="mb-4 text-body-md text-ink/70">{t("set-lock-remove-body")}</p>
                        <DigitBoxes label=t("set-lock-current") value=current length=PIN_LENGTH secret=true />
                        <FormButtons save_label=t("set-lock-remove") on_save=remove cancel_href=back() />
                    </div>
                }.into_any(),
            }}
            <ConfirmDialog open=confirm_reset title=t("set-lock-forgot") body=t("set-lock-reset-body") confirm_label=t("set-lock-remove") danger=true on_confirm=reset />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::valid_pin;

    #[test]
    fn pin_is_four_digits() {
        assert!(valid_pin("0123"));
        assert!(!valid_pin("123"));
        assert!(!valid_pin("12a4"));
        assert!(!valid_pin("12345"));
    }
}
