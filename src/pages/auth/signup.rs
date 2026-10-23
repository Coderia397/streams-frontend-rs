use super::{button_class, AuthShell, Field, Look, Notice};
use crate::pages::help::{tr, tr_with};
use crate::services::account::{self, AuthError, SignUp};
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Welcome,
    Details,
    Check,
}

impl Step {
    fn number(self) -> u32 {
        match self {
            Step::Welcome => 1,
            Step::Details => 2,
            Step::Check => 3,
        }
    }
}

/// Minimum password length. Matches what the Supabase project is set to require.
const MIN_PASSWORD: usize = crate::config::MIN_PASSWORD_LENGTH;

#[component]
pub fn SignupPage() -> impl IntoView {
    Effect::new(move |_| crate::pages::help::set_site_title(&tr("auth-signup-title")));
    let query = use_query_map();
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let email = RwSignal::new(query.with_untracked(|q| q.get("email").unwrap_or_default()));
    let password = RwSignal::new(String::new());
    let (step, set_step) = signal(if email.get_untracked().is_empty() { Step::Welcome } else { Step::Details });
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(String::new());
    let (touched, set_touched) = signal(false);
    let navigate = use_navigate();
    let profiles = crate::store::use_profile_store();

    let first_bad = Signal::derive(move || touched.get() && first.get().trim().is_empty());
    let last_bad = Signal::derive(move || touched.get() && last.get().trim().is_empty());
    let email_bad = Signal::derive(move || touched.get() && !account::valid_email(&email.get()));
    let pass_bad = Signal::derive(move || touched.get() && password.get().chars().count() < MIN_PASSWORD);

    let nav_done = navigate.clone();
    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_touched.set(true);
        if first.get_untracked().trim().is_empty() || last.get_untracked().trim().is_empty() {
            set_error.set(tr("auth-error-name"));
            return;
        }
        if !account::valid_email(&email.get_untracked()) {
            set_error.set(tr("auth-error-email"));
            return;
        }
        if password.get_untracked().chars().count() < MIN_PASSWORD {
            set_error.set(tr_with("auth-error-password-short", &[("min", FluentValue::from(MIN_PASSWORD))]));
            return;
        }
        if busy.get_untracked() {
            return;
        }
        set_error.set(String::new());
        set_busy.set(true);
        let nav = nav_done.clone();
        let (f, l, e, p) = (first.get_untracked(), last.get_untracked(), email.get_untracked(), password.get_untracked());
        leptos::task::spawn_local(async move {
            match account::sign_up(&e, &p, &f, &l).await {
                Ok(SignUp::SignedIn(session)) => {
                    crate::store::start_account(profiles, &session);
                    nav(&crate::components::layout::account_gate::after_sign_in(), Default::default());
                }
                Ok(SignUp::ConfirmEmail) => set_step.set(Step::Check),
                // the same words whether the address is new or taken, so nobody can find out who has an account
                Err(AuthError::CouldNotCreate) => set_error.set(tr("auth-error-could-not-create")),
                Err(AuthError::Weak(_)) => set_error.set(tr_with("auth-error-password-short", &[("min", FluentValue::from(MIN_PASSWORD))])),
                Err(AuthError::RateLimited) => set_error.set(tr("auth-error-rate")),
                Err(AuthError::Network) => set_error.set(tr("auth-error-network")),
                Err(_) => set_error.set(tr("auth-error-generic")),
            }
            set_busy.set(false);
        });
    };

    let ticks = ["auth-tick-free", "auth-tick-devices", "auth-tick-profiles"];
    view! {
        <AuthShell look=Look::Light right=("/login", "login-sign-in")>
            <div class="mx-auto w-full max-w-[440px] px-6 pb-24 pt-8">
                <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-full border-2 border-brand text-brand" aria-hidden="true">
                    <i class="ph-bold ph-check text-title"></i>
                </div>
                <p class="text-fine text-paper-ink">{move || tr_with("auth-step", &[("n", FluentValue::from(step.get().number())), ("total", FluentValue::from(3))])}</p>
                {move || match step.get() {
                    Step::Welcome => view! {
                        <h1 class="mb-6 mt-1 text-heading-xl font-bold leading-tight">{tr("auth-welcome-title")}</h1>
                        <ul class="mb-8 space-y-4">
                            {ticks.into_iter().map(|k| view! {
                                <li class="flex items-start gap-3 text-body-lg"><i class="ph-bold ph-check mt-1 text-brand" aria-hidden="true"></i>{tr(k)}</li>
                            }).collect_view()}
                        </ul>
                        <button type="button" on:click=move |_| set_step.set(Step::Details) class=button_class(Look::Light)>{tr("auth-next")}</button>
                    }
                    .into_any(),
                    Step::Details => view! {
                        <h1 class="mb-2 mt-1 text-heading-xl font-bold leading-tight">{tr("auth-details-title")}</h1>
                        <p class="mb-6 text-body-lg text-paper-ink">{tr("auth-details-body")}</p>
                        <Notice text=error look=Look::Light />
                        <form on:submit=on_create.clone() class="space-y-4" novalidate>
                            <div class="grid gap-4 sm:grid-cols-2">
                                <Field id="signup-first" label="auth-first-name" value=first autocomplete="given-name" look=Look::Light invalid=first_bad />
                                <Field id="signup-last" label="auth-last-name" value=last autocomplete="family-name" look=Look::Light invalid=last_bad />
                            </div>
                            <Field id="signup-email" label="auth-email" value=email kind="email" autocomplete="email" look=Look::Light invalid=email_bad />
                            <Field id="signup-password" label="auth-password" value=password kind="password" autocomplete="new-password" look=Look::Light invalid=pass_bad />
                            <p class="text-fine text-paper-mute">{tr_with("auth-password-rule", &[("min", FluentValue::from(MIN_PASSWORD))])}</p>
                            <button type="submit" disabled=move || busy.get() class=button_class(Look::Light)>
                                {move || if busy.get() { tr("auth-creating") } else { tr("auth-create") }}
                            </button>
                            <p class="text-fine leading-5 text-paper-mute">
                                {crate::pages::help::markup::text(&tr("auth-terms"))}
                            </p>
                        </form>
                    }
                    .into_any(),
                    Step::Check => view! {
                        <h1 class="mb-3 mt-1 text-heading-xl font-bold leading-tight">{tr("auth-check-title")}</h1>
                        <p class="mb-8 text-body-lg leading-7 text-paper-ink">{tr_with("auth-check-body", &[("email", FluentValue::from(email.get()))])}</p>
                        <a href="/login" class=button_class(Look::Light)>{tr("auth-back-to-sign-in")}</a>
                        <p class="mt-6 text-body-sm text-paper-mute">{tr("auth-check-spam")}</p>
                    }
                    .into_any(),
                }}
            </div>
        </AuthShell>
    }
}
