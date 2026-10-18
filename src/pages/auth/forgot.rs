use super::{button_class, AuthShell, Field, Look, Notice};
use crate::pages::help::{tr, tr_with};
use crate::services::account::{self, AuthError};
use fluent_bundle::FluentValue;
use leptos::prelude::*;

#[component]
pub fn ForgotPage() -> impl IntoView {
    Effect::new(move |_| crate::pages::help::set_site_title(&tr("auth-forgot-title")));
    let email = RwSignal::new(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(String::new());
    let (sent, set_sent) = signal(false);
    let bad = Signal::derive(move || !error.get().is_empty());
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let e = email.get_untracked();
        if !account::valid_email(&e) {
            set_error.set(tr("auth-error-email"));
            return;
        }
        set_error.set(String::new());
        set_busy.set(true);
        leptos::task::spawn_local(async move {
            match account::recover(&e).await {
                Ok(()) => set_sent.set(true),
                Err(AuthError::RateLimited) => set_error.set(tr("auth-error-rate")),
                Err(AuthError::Network) => set_error.set(tr("auth-error-network")),
                Err(_) => set_error.set(tr("auth-error-generic")),
            }
            set_busy.set(false);
        });
    };
    view! {
        <AuthShell look=Look::Dark>
            <div class="mx-auto w-full max-w-[440px] px-6 pb-24 pt-10">
                {move || if sent.get() {
                    view! {
                        <h1 class="text-heading-xl font-bold leading-tight">{tr("auth-forgot-sent-title")}</h1>
                        <p class="mb-6 mt-3 text-body-md leading-7 text-ink-body">{tr_with("auth-forgot-sent", &[("email", FluentValue::from(email.get()))])}</p>
                        <a href="/login" class=button_class(Look::Dark)>{tr("auth-back-to-sign-in")}</a>
                    }
                    .into_any()
                } else {
                    view! {
                        <h1 class="text-heading-xl font-bold leading-tight">{tr("auth-forgot-title")}</h1>
                        <p class="mb-6 mt-2 text-body-md text-ink-body">{tr("auth-forgot-body")}</p>
                        <Notice text=error />
                        <form on:submit=on_submit class="space-y-3" novalidate>
                            <Field id="forgot-email" label="auth-email" value=email kind="email" autocomplete="email" invalid=bad />
                            <button type="submit" disabled=move || busy.get() class=button_class(Look::Dark)>{tr("auth-send-link")}</button>
                        </form>
                        <p class="mt-5 text-body-md"><a href="/login" class="text-ink-body underline underline-offset-2 hover:text-ink">{tr("auth-back-to-sign-in")}</a></p>
                    }
                    .into_any()
                }}
            </div>
        </AuthShell>
    }
}
