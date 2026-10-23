//! Account screens (per account): security, devices, privacy and data, viewing activity, delete. The screens that change account
//! data sit behind a password check (`Gate`), like the "make sure it's you" page in the Netflix app.

use super::prefs::use_prefs;
use super::widgets::{Field, FormButtons, LinkRow, Message, OutlineButton, PasswordField, Section, Tone};
use super::Screen;
use crate::config::{routes, MIN_PASSWORD_LENGTH, NAME_MAX, PHONE_DIGITS};
use crate::i18n::t;
use crate::services::account::{self, AuthError};
use crate::store::use_profile_store;
use leptos::prelude::*;

pub(super) fn error_text(e: &AuthError) -> String {
    t(match e {
        AuthError::Invalid => "set-err-invalid",
        AuthError::Weak(_) => "set-err-weak",
        AuthError::RateLimited => "set-err-rate",
        AuthError::RecentSignIn => "set-err-recent",
        AuthError::Network => "set-err-network",
        _ => "set-err-other",
    })
}

pub(super) fn go(path: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_href(path);
    }
}

/// Shown instead of account values when nobody is signed in.
#[component]
fn SignInBanner() -> impl IntoView {
    account::session().is_none().then(|| view! {
        <div class="mb-8 rounded-card border border-ink/10 bg-ink/5 p-5 text-center">
            <p class="mb-4 text-body-md text-ink/70">{t("set-signin-needed")}</p>
            <a href=routes::LOGIN class="btn-ink inline-block px-8 py-3">{t("help-ui-signin")}</a>
        </div>
    })
}

/// Name, password, email and phone with their current values.
#[component]
fn AccountDetails(#[prop(optional)] wide: bool) -> impl IntoView {
    let session = account::session();
    let email = session.as_ref().map(|s| s.email.clone()).unwrap_or_default();
    let phone = session.as_ref().map(|s| s.phone.clone()).unwrap_or_default();
    let full_name = session.as_ref().map(|s| format!("{} {}", s.first_name, s.last_name).trim().to_string()).unwrap_or_default();
    let or_none = |v: String| Signal::derive(move || if v.is_empty() { t("set-none") } else { v.clone() });
    view! {
        <Section title=t("set-account-details")>
            {(!wide).then(|| view! { <LinkRow icon="identification-card" title=t("set-title-name") href=Screen::Name.path() value=or_none(full_name.clone()) /> })}
            <LinkRow icon="lock-simple" title=t("set-title-password") href=Screen::Password.path() />
            <LinkRow icon="envelope-simple" title=t("set-title-email") href=Screen::Email.path() value=or_none(email) verified=true />
            <LinkRow icon="device-mobile" title=t("set-title-phone") href=Screen::Phone.path() value=or_none(phone) />
        </Section>
    }
}

/// Access and devices, privacy, profile transfer, personal information access and feature testing.
#[component]
fn AccessAndPrivacy() -> impl IntoView {
    let prefs = use_prefs();
    let on_off = move |on: bool| if on { t("set-lock-on") } else { t("set-lock-off") };
    view! {
        <Section title=t("set-security-access")>
            <LinkRow icon="devices" title=t("set-title-devices") hint=t("set-access-hint") href=Screen::Access.path() />
            <LinkRow icon="arrows-left-right" title=t("set-title-transfer") href=Screen::TransferSetting.path() value=Signal::derive(move || on_off(prefs.account.get().profile_transfer)) />
            <LinkRow icon="file-arrow-down" title=t("set-title-personal-info") hint=t("set-personal-info-hint") href=Screen::PersonalInfo.path() />
            <LinkRow icon="flask" title=t("set-row-feature-testing") href=Screen::Testing.path() value=Signal::derive(move || on_off(prefs.account.get().test_participation)) />
        </Section>
    }
}

#[component]
fn DevicesSection(#[prop(optional)] wide: bool) -> impl IntoView {
    view! {
        <Section title=t(if wide { "set-devices-account-access" } else { "set-account-devices" })>
            <LinkRow icon="devices" title=t("set-title-devices") hint=t("set-access-hint") href=Screen::Access.path() />
        </Section>
    }
}

#[component]
fn ProfilesSection() -> impl IntoView {
    view! {
        <Section title=t("set-account-profiles")>
            <LinkRow icon="users-three" title=t("set-nav-profiles") href=Screen::Profiles.path() />
            <LinkRow icon="shield-check" title=t("set-title-privacy") href=Screen::Privacy.path() />
        </Section>
    }
}

/// Delete account: an outline button on phones, a red text link on the website.
fn delete_button() -> impl IntoView {
    view! {
        <div class="lg:hidden"><OutlineButton label=t("set-title-delete") danger=true on_click=Callback::new(|_| go(&Screen::Delete.path())) /></div>
        <a href=Screen::Delete.path() class="hidden h-12 items-center rounded-button px-4 text-title-sm font-medium text-danger hover:bg-danger/10 lg:flex">{t("set-title-delete")}</a>
    }
}

fn active_profile_id() -> String {
    let store = use_profile_store();
    store.active_profile_id.get_untracked().or_else(|| store.profiles.with_untracked(|l| l.first().map(|p| p.id.clone()))).unwrap_or_default()
}

/// The Account screen. A phone or tablet gets one page with Security, Devices and Profiles stacked, like the Netflix app. From
/// 1024 px the same address is the Overview with quick links, and Security, Devices and Profiles are pages of their own.
#[component]
pub fn Account() -> impl IntoView {
    let store = use_profile_store();
    let profile_count = Memo::new(move |_| store.profiles.with(|l| l.len()));
    view! {
        <div>
            <SignInBanner />
            // phones and tablets: the app's account list; computers: the website's overview (note 114, one mounted)
            <crate::form::DesktopOr
                compact=|| view! {
                <AccountDetails />
                <DevicesSection />
                <ProfilesSection />
                {delete_button()}
                }
                desktop=move || view! {
                <Section title=t("set-overview-quick")>
                    <LinkRow icon="devices" title=t("set-qlink-access") href=Screen::Access.path() />
                    <LinkRow icon="lock-simple" title=t("set-overview-update-password") href=Screen::Password.path() />
                    <LinkRow icon="user-switch" title=t("set-title-transfer-profile") href=Screen::TransferProfile(active_profile_id()).path() />
                    <LinkRow icon="warning-circle" title=t("set-po-restrictions") href=Screen::Restrictions(active_profile_id()).path() />
                </Section>
                <Section>
                    <LinkRow icon="sliders-horizontal" title=t("set-overview-edit") hint=t("set-overview-edit-hint") href=Screen::Profile(active_profile_id()).path() />
                    <LinkRow icon="users-three" title=t("set-nav-profiles") hint=crate::i18n::t_args("set-overview-profiles", &[("count", profile_count.get_untracked().into())]) href=Screen::Profiles.path() />
                </Section>
                }
            />
        </div>
    }
}

/// Wide screens: Security page (account details, access and privacy, delete).
#[component]
pub fn SecurityPage() -> impl IntoView {
    view! {
        <div>
            <SignInBanner />
            <AccountDetails wide=true />
            <AccessAndPrivacy />
            {delete_button()}
        </div>
    }
}

/// Wide screens: Devices page.
#[component]
pub fn DevicesPage() -> impl IntoView {
    view! { <div><SignInBanner /><DevicesSection wide=true /></div> }
}

/// Wide screens: Profiles page (parental controls, then one row per profile).
#[component]
pub fn ProfilesPage() -> impl IntoView {
    let store = use_profile_store();
    view! {
        <div>
            <Section title=t("set-profiles-parental")>
                <LinkRow icon="warning-circle" title=t("set-title-restrictions") hint=t("set-profiles-parental-hint") href=Screen::Restrictions(active_profile_id()).path() />
                <LinkRow icon="user-switch" title=t("set-title-transfer-profile") hint=t("set-profiles-transfer-hint") href=Screen::TransferProfile(active_profile_id()).path() />
            </Section>
            <Section title=t("set-profiles-settings")>
                {move || {
                    let active = store.active_profile_id.get();
                    store.profiles.get().into_iter().map(|p| {
                        let now = Some(&p.id) == active.as_ref();
                        view! {
                            <a href=Screen::Profile(p.id.clone()).path() class="flex items-center gap-4 px-5 py-3 hover:bg-ink/10">
                                <img src=p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string()) alt="" class="h-10 w-10 rounded-control object-cover" />
                                <span class="min-w-0 flex-1 truncate text-body-md font-bold">{p.name.clone()}</span>
                                {now.then(|| view! { <span class="rounded-control bg-badge px-[8px] py-[2px] text-body-sm font-medium leading-[22px] text-chrome/70">{t("set-now-watching")}</span> })}
                                <i class="ph-bold ph-caret-right text-xl text-ink/40 rtl:-scale-x-100" aria-hidden="true"></i>
                            </a>
                        }
                    }).collect_view()
                }}
                <super::hub::AddProfile />
            </Section>
        </div>
    }
}

/// First and last name of the person behind the account (account_details). The main profile follows its first name.
#[component]
pub fn Name() -> impl IntoView {
    let session = account::session();
    let first = RwSignal::new(session.as_ref().map(|s| s.first_name.clone()).unwrap_or_default());
    let last = RwSignal::new(session.as_ref().map(|s| s.last_name.clone()).unwrap_or_default());
    let note = RwSignal::new(None::<(Tone, String)>);
    let busy = RwSignal::new(false);
    let save = Callback::new(move |_| {
        let (f, l) = (first.get_untracked(), last.get_untracked());
        if f.trim().is_empty() || l.trim().is_empty() {
            note.set(Some((Tone::Error, t("set-name-required"))));
            return;
        }
        busy.set(true);
        note.set(None);
        leptos::task::spawn_local(async move {
            match account::update_names(&f, &l).await {
                Ok(()) => note.set(Some((Tone::Done, t("set-name-saved")))),
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    });
    view! {
        <div class="max-w-md">
            <p class="mb-6 text-body-md text-ink/60">{t("set-name-hint")}</p>
            <Message text=note />
            <Field label=t("set-name-first") value=first autocomplete="given-name" maxlength=NAME_MAX as u32 />
            <Field label=t("set-name-last") value=last autocomplete="family-name" maxlength=NAME_MAX as u32 />
            <FormButtons save_label=t("set-save") on_save=save cancel_href=Screen::Account.path() busy=busy />
        </div>
    }
}

#[component]
pub fn Password() -> impl IntoView {
    let current = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let again = RwSignal::new(String::new());
    let all_devices = RwSignal::new(true);
    let note = RwSignal::new(None::<(Tone, String)>);
    let busy = RwSignal::new(false);
    let save = Callback::new(move |_| {
        let value = new_password.get_untracked();
        let old = current.get_untracked();
        if old.is_empty() {
            note.set(Some((Tone::Error, t("set-password-current-required"))));
            return;
        }
        if value.chars().count() < MIN_PASSWORD_LENGTH {
            note.set(Some((Tone::Error, crate::i18n::t_args("set-password-short", &[("min", MIN_PASSWORD_LENGTH.into())]))));
            return;
        }
        if value != again.get_untracked() {
            note.set(Some((Tone::Error, t("set-password-mismatch"))));
            return;
        }
        busy.set(true);
        note.set(None);
        let everywhere = all_devices.get_untracked();
        leptos::task::spawn_local(async move {
            // like the Netflix page, the current password is asked on the page itself instead of a check before it
            if let Err(e) = account::verify_password(&old).await {
                note.set(Some((Tone::Error, error_text(&e))));
                busy.set(false);
                return;
            }
            match account::change_password(&value).await {
                Ok(()) if everywhere => {
                    current.set(String::new());
                    let _ = account::sign_out_everywhere().await;
                    go(routes::LOGIN);
                }
                Ok(()) => {
                    current.set(String::new());
                    new_password.set(String::new());
                    again.set(String::new());
                    note.set(Some((Tone::Done, t("set-password-saved"))));
                }
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    });
    view! {
        <div class="max-w-md">
            <p class="mb-6 text-body-md text-ink/60">{crate::i18n::t_args("set-password-hint", &[("min", MIN_PASSWORD_LENGTH.into())])}</p>
            <Message text=note />
            <PasswordField label=t("set-password-current") value=current />
            <a href="/forgot" class="-mt-2 mb-5 inline-block text-body-md font-medium underline">{t("set-password-forgot")}</a>
            <PasswordField label=t("set-password-new") value=new_password autocomplete="new-password" />
            <PasswordField label=t("set-password-again") value=again autocomplete="new-password" />
            <label class="mb-6 flex items-center gap-3 text-body-md">
                <input type="checkbox" prop:checked=move || all_devices.get() on:change=move |ev| all_devices.set(event_target_checked(&ev)) class="h-5 w-5 accent-brand" />
                {t("set-password-signout-all")}
            </label>
            <FormButtons save_label=t("set-save") on_save=save cancel_href=Screen::Account.path() busy=busy />
        </div>
    }
}

#[component]
pub fn Email() -> impl IntoView {
    let current = account::session().map(|s| s.email).unwrap_or_default();
    let new_email = RwSignal::new(String::new());
    let note = RwSignal::new(None::<(Tone, String)>);
    let busy = RwSignal::new(false);
    let save = Callback::new(move |_| {
        let value = new_email.get_untracked();
        if !account::valid_email(&value) {
            note.set(Some((Tone::Error, t("set-email-invalid"))));
            return;
        }
        busy.set(true);
        note.set(None);
        leptos::task::spawn_local(async move {
            match account::change_email(&value).await {
                Ok(()) => note.set(Some((Tone::Done, t("set-email-sent")))),
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    });
    view! {
        <div class="max-w-md">
            <p class="mb-6 text-body-md text-ink/60">{t("set-email-hint")}</p>
            <Message text=note />
            <Field label=t("set-email-current") value=RwSignal::new(current) kind="email" />
            <Field label=t("set-email-new") value=new_email kind="email" autocomplete="email" />
            <FormButtons save_label=t("set-save") on_save=save cancel_href=Screen::Account.path() busy=busy />
        </div>
    }
}

#[component]
pub fn Phone() -> impl IntoView {
    let number = RwSignal::new(account::session().map(|s| s.phone).unwrap_or_default());
    let note = RwSignal::new(None::<(Tone, String)>);
    let busy = RwSignal::new(false);
    let save = Callback::new(move |_| {
        let value = number.get_untracked();
        let digits = value.chars().filter(|c| c.is_ascii_digit()).count();
        if !value.trim().is_empty() && !PHONE_DIGITS.contains(&digits) {
            note.set(Some((Tone::Error, t("set-phone-invalid"))));
            return;
        }
        busy.set(true);
        note.set(None);
        leptos::task::spawn_local(async move {
            match account::save_phone(&value).await {
                Ok(()) => note.set(Some((Tone::Done, t("set-phone-saved")))),
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    });
    view! {
        <div class="max-w-md">
            <p class="mb-6 text-body-md text-ink/60">{t("set-phone-hint")}</p>
            <Message text=note />
            <Field label=t("set-title-phone") value=number kind="tel" autocomplete="tel" />
            <FormButtons save_label=t("set-save") on_save=save cancel_href=Screen::Account.path() busy=busy />
        </div>
    }
}

