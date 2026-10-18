use super::{button_class, AuthShell, Field, Look, Notice};
use crate::pages::help::{tr, tr_with};
use crate::services::account::{self, AuthError};
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Email,
    Password,
}

fn message(error: &AuthError, email: &str) -> String {
    match error {
        AuthError::Invalid => tr("auth-error-invalid"),
        AuthError::Unconfirmed => tr_with("auth-error-unconfirmed", &[("email", FluentValue::from(email.to_string()))]),
        AuthError::RateLimited => tr("auth-error-rate"),
        AuthError::Network => tr("auth-error-network"),
        _ => tr("auth-error-generic"),
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    Effect::new(move |_| crate::pages::help::set_page_title(&tr("login-sign-in")));
    let query = use_query_map();
    let email = RwSignal::new(query.with_untracked(|q| q.get("email").unwrap_or_default()));
    let password = RwSignal::new(String::new());
    let (remember, set_remember) = signal(true);
    let (step, set_step) = signal(Step::Email);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(String::new());
    let (notice, set_notice) = signal(String::new());
    let (resend, set_resend) = signal(false);
    let email_bad = Signal::derive(move || !error.get().is_empty() && step.get() == Step::Email);
    let navigate = use_navigate();
    let profiles = crate::store::use_profile_store();

    let on_continue = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if !account::valid_email(&email.get_untracked()) {
            set_error.set(tr("auth-error-email"));
            return;
        }
        set_error.set(String::new());
        set_step.set(Step::Password);
    };
    let nav_done = navigate.clone();
    let on_sign_in = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if busy.get_untracked() {
            return;
        }
        set_busy.set(true);
        set_error.set(String::new());
        set_notice.set(String::new());
        set_resend.set(false);
        let nav = nav_done.clone();
        let (e, p, r) = (email.get_untracked(), password.get_untracked(), remember.get_untracked());
        leptos::task::spawn_local(async move {
            match account::sign_in(&e, &p, r).await {
                Ok(session) => {
                    crate::store::start_account(profiles, &session.first_name);
                    nav(&crate::components::layout::account_gate::after_sign_in(), Default::default());
                }
                Err(err) => {
                    set_resend.set(err == AuthError::Unconfirmed);
                    set_error.set(message(&err, &e));
                }
            }
            set_busy.set(false);
        });
    };
    let on_resend = move |_| {
        let e = email.get_untracked();
        leptos::task::spawn_local(async move {
            match account::resend_confirmation(&e).await {
                Ok(()) => set_notice.set(tr_with("auth-resent", &[("email", FluentValue::from(e.clone()))])),
                Err(err) => set_error.set(message(&err, &e)),
            }
        });
    };

    view! {
        <AuthShell look=Look::Dark back=true>
            <div class="mx-auto w-full max-w-[440px] px-6 pb-24 pt-10">
                {move || match step.get() {
                    Step::Email => view! {
                        // the phone version asks the way the Netflix app does; wider screens use the web wording
                        <h1 class="text-heading-xl font-bold leading-tight sm:hidden">{tr("auth-login-app-title")}</h1>
                        <p class="mb-6 mt-2 text-body-md text-ink-soft sm:hidden">{tr("auth-login-app-subtitle")}</p>
                        <h1 class="hidden text-heading-xl font-bold leading-tight sm:block">{tr("auth-login-title")}</h1>
                        <p class="mb-6 mt-2 hidden text-body-md text-ink-body sm:block">{tr("auth-login-subtitle")}</p>
                        <Notice text=error />
                        <form on:submit=on_continue class="space-y-3" novalidate>
                            <Field id="login-email" label="auth-email" value=email kind="email" autocomplete="email" invalid=email_bad />
                            <button type="submit" class=button_class(Look::Dark)>{tr("auth-continue")}</button>
                        </form>
                        <p class="mt-4 text-body-md text-ink-body">
                            {tr("auth-new-here")}" "
                            <a href=move || format!("/signup?email={}", crate::pages::help::encode(&email.get())) class="font-semibold text-ink underline underline-offset-2">{tr("auth-signup-link")}</a>
                        </p>
                    }
                    .into_any(),
                    Step::Password => view! {
                        <h1 class="text-heading-xl font-bold leading-tight">{tr("auth-welcome-back")}</h1>
                        <p class="mb-6 mt-2 flex flex-wrap items-center gap-x-2 text-body-md text-ink-body">
                            <span class="break-all">{move || email.get()}</span>
                            <button type="button" on:click=move |_| { set_step.set(Step::Email); set_error.set(String::new()); } class="text-ink underline underline-offset-2">{tr("auth-not-you")}</button>
                        </p>
                        <Notice text=error />
                        <Notice text=notice error=false />
                        {move || resend.get().then(|| view! { <button type="button" on:click=on_resend class="mb-4 text-body-sm font-semibold underline underline-offset-2">{tr("auth-resend")}</button> })}
                        <form on:submit=on_sign_in.clone() class="space-y-3">
                            <Field id="login-password" label="auth-password" value=password kind="password" autocomplete="current-password" />
                            <button type="submit" disabled=move || busy.get() class=button_class(Look::Dark)>
                                {move || if busy.get() { tr("auth-signing-in") } else { tr("login-sign-in") }}
                            </button>
                            <label class="flex cursor-pointer items-center gap-3 pt-1 text-body-sm text-ink-body">
                                <input type="checkbox" prop:checked=remember on:change=move |ev| set_remember.set(event_target_checked(&ev)) class="h-4 w-4 accent-brand" />
                                {tr("auth-remember")}
                            </label>
                        </form>
                        <p class="mt-5 text-body-md"><a href="/forgot" class="text-ink-body underline underline-offset-2 hover:text-ink">{tr("auth-forgot-link")}</a></p>
                    }
                    .into_any(),
                }}
                <details class="group mt-8">
                    <summary class="flex cursor-pointer list-none items-center gap-1 text-body-md font-bold sm:font-normal">
                        {tr("auth-get-help")}
                        <i class="ph-bold ph-caret-down text-body-sm transition-transform duration-(--motion-base) group-open:rotate-180" aria-hidden="true"></i>
                    </summary>
                    <ul class="mt-3 space-y-3 text-body-sm font-semibold text-ink sm:space-y-2 sm:font-normal sm:text-ink-body">
                        <li><a href="/help/article/sign-in" class="underline underline-offset-2 hover:text-ink">{tr("auth-forgot-email")}</a></li>
                        <li><a href="/forgot" class="underline underline-offset-2 hover:text-ink">{tr("auth-forgot-link")}</a></li>
                        <li><a href="/help/article/sign-in" class="underline underline-offset-2 hover:text-ink">{tr("auth-learn-signin")}</a></li>
                    </ul>
                </details>
            </div>
        </AuthShell>
    }
}
