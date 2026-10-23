//! Viewing restrictions of one profile: the British age rating limit (profile_settings.age_rating) and the titles that are blocked.
use super::prefs::use_prefs;
use super::widgets::{toast, ChoiceList, LinkRow, Section, Stepper};
use crate::data::ratings::{limit_label, RatingSystem};
use super::Screen;
use crate::config::{BLOCKED_TITLES_MAX, BLOCKED_TITLE_MAX};
use crate::data::settings::{rating, DEFAULT_RATING, RATINGS};
use crate::i18n::{t, t_args};
use crate::store::use_profile_store;
use leptos::prelude::*;

fn is_kids(id: &str) -> bool {
    use_profile_store().profiles.with(|l| l.iter().find(|p| p.id == id).is_some_and(|p| p.is_kids))
}

#[component]
pub fn Restrictions(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let id = StoredValue::new(id);
    let level_value = Signal::derive(move || {
        let r = rating(prefs.profile(&id.get_value()).rating_for(is_kids(&id.get_value())));
        if r.key == DEFAULT_RATING { t("set-restrictions-none") } else { t_args("set-rating-up-to", &[("rating", limit_label(prefs.rating_system(), r.key).into())]) }
    });
    let blocked_value = Signal::derive(move || {
        let n = prefs.profile(&id.get_value()).blocked_titles.len();
        if n == 0 { t("set-none") } else { t_args("set-blocked-count", &[("count", n.into())]) }
    });
    view! {
        <Section>
            <LinkRow icon="warning-circle" title=t("set-title-ratings") href=Screen::Ratings(id.get_value()).path() value=level_value />
            <LinkRow icon="prohibit" title=t("set-title-blocked") href=Screen::Blocked(id.get_value()).path() value=blocked_value />
        </Section>
        <RatingSystemChoice />
    }
}

/// British age ratings as a stepper: each rating allows everything below it, with what it means written under it. A Kids
/// profile is offered U and PG only. The same ratings are on the titles (TMDB and the catalogue carry them).
#[component]
pub fn Ratings(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let id = StoredValue::new(id);
    let kids = is_kids(&id.get_value());
    let selected = Signal::derive(move || prefs.profile(&id.get_value()).rating_for(kids).to_string());
    let system = prefs.rating_system();
    let options: Vec<(String, String, String)> = RATINGS.iter().filter(|r| !kids || r.kids).map(|r| (r.key.to_string(), limit_label(system, r.key), t(r.hint_key))).collect();
    view! {
        <div>
            <p class="mb-5 text-body-md text-ink/60">{t("set-ratings-hint")}</p>
            <Stepper options=options selected=selected on_pick=Callback::new(move |v: String| { prefs.update_profile(&id.get_value(), |p| p.max_rating = Some(v)); toast(t("set-saved")); }) />
        </div>
    }
}

#[component]
pub fn Blocked(id: String) -> impl IntoView {
    let prefs = use_prefs();
    let id = StoredValue::new(id);
    let text = RwSignal::new(String::new());
    let list = Memo::new(move |_| prefs.profile(&id.get_value()).blocked_titles);
    let add = move || {
        let value = text.get_untracked().trim().chars().take(BLOCKED_TITLE_MAX).collect::<String>();
        if value.is_empty() {
            return;
        }
        prefs.update_profile(&id.get_value(), |p| {
            if p.blocked_titles.len() < BLOCKED_TITLES_MAX && !p.blocked_titles.iter().any(|b| b.to_lowercase() == value.to_lowercase()) {
                p.blocked_titles.push(value);
            }
        });
        text.set(String::new());
    };
    view! {
        <div>
            <p class="mb-5 text-body-md text-ink/60">{t("set-blocked-hint")}</p>
            <div class="mb-5 flex gap-3">
                <input
                    type="text"
                    maxlength=BLOCKED_TITLE_MAX
                    placeholder=move || t("set-blocked-placeholder")
                    aria-label=move || t("set-title-blocked")
                    prop:value=move || text.get()
                    on:input=move |ev| text.set(event_target_value(&ev))
                    on:keydown=move |ev| if ev.key() == "Enter" { add() }
                    class="min-w-0 flex-1 border border-ink/20 bg-ink/10 px-4 py-3 text-body-md text-ink focus:border-ink focus:outline-none"
                />
                <button type="button" on:click=move |_| add() class="btn-ink px-6">{t("set-add")}</button>
            </div>
            <ul class="overflow-hidden rounded-card border border-ink/10 bg-ink/5 divide-y divide-ink/10">
                {move || list.get().into_iter().map(|title| {
                    let gone = title.clone();
                    view! {
                        <li class="flex items-center px-5 py-3">
                            <span class="flex-1 truncate text-body-md">{title}</span>
                            <button type="button" aria-label=t("set-remove") on:click=move |_| prefs.update_profile(&id.get_value(), |p| p.blocked_titles.retain(|b| b != &gone)) class="btn-circle btn-circle-clear btn-circle-md text-ink/70">
                                <i class="ph-bold ph-trash text-xl" aria-hidden="true"></i>
                            </button>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

/// Which age rating labels this person reads. Automatic follows the country of the device the session signed in from; the choice is kept
/// in the account (account_details.rating_system) and the database answer for automatic is asked again after a change.
#[component]
fn RatingSystemChoice() -> impl IntoView {
    let prefs = use_prefs();
    let selected = Signal::derive(move || prefs.account.with(|a| a.rating_system.clone()));
    let options = vec![
        ("auto".to_string(), t("set-rating-system-auto")),
        (RatingSystem::Uk.key().to_string(), t("set-rating-system-uk")),
        (RatingSystem::Us.key().to_string(), t("set-rating-system-us")),
    ];
    view! {
        <div class="mt-8">
            <h2 class="mb-1 text-heading-sm font-bold">{t("set-rating-system-title")}</h2>
            <p class="mb-4 text-body-md text-ink/60">{t("set-rating-system-hint")}</p>
            <ChoiceList
                options=options
                selected=selected
                on_pick=Callback::new(move |v: String| {
                    prefs.account.update(|a| a.rating_system = v.clone());
                    leptos::task::spawn_local(async move {
                        let _ = crate::services::account::rpc("set_rating_system", &serde_json::json!({ "p_system": v })).await;
                        crate::services::device::refresh_rating_system().await;
                    });
                    toast(t("set-saved"));
                })
            />
        </div>
    }
}
