//! Privacy and data settings, Personal information access and Test participation.
use super::widgets::{save_as_file, ActionRow, LinkRow, Message, Section, SwitchRow, Tone};
use super::prefs::use_prefs;
use super::Screen;
use crate::i18n::{t, t_args};
use crate::services::account::{self, AuthError};
use crate::store::use_profile_store;
use leptos::prelude::*;

fn error_text(e: &AuthError) -> String {
    t(match e {
        AuthError::RecentSignIn => "set-err-recent",
        AuthError::Network => "set-err-network",
        AuthError::RateLimited => "set-err-rate",
        _ => "set-err-other",
    })
}

/// Privacy and data settings of the active profile: who can open it, what is kept, and how recommendations use it.
#[component]
pub fn Privacy() -> impl IntoView {
    let consent = crate::consent::use_ui();
    let profiles = use_profile_store();
    let active = Memo::new(move |_| profiles.profiles.with(|l| profiles.active_profile_id.with(|id| l.iter().find(|p| Some(&p.id) == id.as_ref()).or(l.first()).cloned())));
    view! {
        <div>
            {move || active.get().map(|p| view! { <p class="mb-6 text-body-md text-ink/60">{t_args("set-for-profile", &[("name", p.name.clone().into())])}</p> })}
            <Section title=t("set-privacy-viewing")>
                {move || active.get().map(|p| view! { <LinkRow icon="lock-simple" title=t("set-title-lock") hint=t("set-privacy-lock-hint") href=Screen::Lock(p.id).path() /> })}
                <LinkRow icon="clock-counter-clockwise" title=t("set-title-activity") hint=t("set-privacy-activity-hint") href=Screen::Activity(active.get_untracked().map(|p| p.id).unwrap_or_default()).path() />
            </Section>
            <Section title=t("set-privacy-data")>
                <SwitchRow
                    icon="sparkle"
                    title=t("set-privacy-personalization")
                    hint=t("set-privacy-personalization-hint")
                    checked=Signal::derive(move || consent.personalization.get())
                    on_change=Callback::new(move |v| consent.decide(v))
                />
                <ActionRow icon="cookie" title=t("cookie-link") hint=t("set-privacy-cookies-hint") chevron=true on_click=Callback::new(move |_| consent.open_center()) />
                <LinkRow icon="file-arrow-down" title=t("set-title-personal-info") hint=t("set-personal-info-hint") href=Screen::PersonalInfo.path() />
            </Section>
        </div>
    }
}

/// Request a copy of the personal information held about the account (`export_my_data`).
#[component]
pub fn PersonalInfo() -> impl IntoView {
    let note = RwSignal::new(None::<(Tone, String)>);
    let busy = RwSignal::new(false);
    let download = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        note.set(None);
        leptos::task::spawn_local(async move {
            match account::export_data().await {
                Ok(text) => {
                    save_as_file(&crate::data::settings::export_file_name(), &text, crate::data::settings::EXPORT_MIME);
                    note.set(Some((Tone::Done, t("set-personal-info-done"))));
                }
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    };
    view! {
        <div class="max-w-xl">
            <p class="mb-4 text-body-md text-ink/70">{t("set-personal-info-body")}</p>
            <ul class="mb-6 list-disc space-y-1 ps-6 text-body-md text-ink/70">
                {["account", "profiles", "lists", "history", "settings", "security"].iter().map(|k| view! { <li>{t(&format!("set-personal-info-{k}"))}</li> }).collect_view()}
            </ul>
            <Message text=note />
            <button type="button" on:click=download disabled=move || busy.get() class="btn-ink px-8 py-3 disabled:opacity-50">{t("set-privacy-export")}</button>
            <p class="mt-4 text-body-sm text-ink/60">{t("set-personal-info-note")}</p>
        </div>
    }
}

/// Test participation: whether the account is included in tests and previews of new features.
#[component]
pub fn Testing() -> impl IntoView {
    let prefs = use_prefs();
    let note = RwSignal::new(None::<(Tone, String)>);
    let set = Callback::new(move |on: bool| {
        prefs.account.update(|a| a.test_participation = on);
        if account::session().is_some() {
            leptos::task::spawn_local(async move {
                if let Err(e) = account::set_switch("test_participation", on).await {
                    note.set(Some((Tone::Error, error_text(&e))));
                }
            });
        }
    });
    view! {
        <div class="max-w-xl">
            <p class="mb-5 text-body-md text-ink/70">{t("set-testing-body")}</p>
            <Message text=note />
            <Section>
                <SwitchRow icon="flask" title=t("set-testing-switch") hint=t("set-testing-hint") checked=Signal::derive(move || prefs.account.get().test_participation) on_change=set />
            </Section>
            <a href=Screen::AccountSecurity.path() class="btn-ink inline-block px-8 py-3">{t("set-done")}</a>
        </div>
    }
}
