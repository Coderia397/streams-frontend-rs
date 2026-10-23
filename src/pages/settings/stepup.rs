//! "First, let's make sure it's you": the check before the screens that change account data. Two ways, like the Netflix page: a
//! code sent by email, or the account password. A check counts for `RECENT_SIGN_IN_SECONDS` (the database asks for a recent
//! sign in too). Without an account session the profile screens open freely and the account screens ask to sign in.
use super::account::error_text;
use super::prefs::{now_secs, use_prefs};
use super::widgets::{DigitBoxes, Message, PasswordField, Tone};
use crate::config::{routes, CODE_LENGTH, CODE_VALID_MINUTES};
use crate::i18n::{t, t_args};
use crate::services::account::{self, AuthError};
use leptos::prelude::*;

/// Set while the check is on screen, so the page can hide its own title.
#[derive(Clone, Copy)]
pub struct GateActive(pub RwSignal<bool>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Choose,
    Password,
    Code,
}

#[component]
pub fn Gate(account: bool, children: ChildrenFn) -> impl IntoView {
    let prefs = use_prefs();
    let step = RwSignal::new(Step::Choose);
    let password = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());
    let note = RwSignal::new(None::<(Tone, String)>);
    let busy = RwSignal::new(false);
    let session = account::session();
    let email = StoredValue::new(session.as_ref().map(|s| s.email.clone()).unwrap_or_default());
    let signed_in = session.is_some();
    let verified = move || prefs.verified_until.get() > now_secs();

    let fail = move |e: AuthError| note.set(Some((Tone::Error, if e == AuthError::Invalid { t("set-code-wrong") } else { error_text(&e) })));
    let done = move || {
        password.set(String::new());
        code.set(String::new());
        prefs.mark_verified();
    };
    let submit_password = Callback::new(move |_| {
        if busy.get_untracked() || password.get_untracked().is_empty() {
            return;
        }
        busy.set(true);
        note.set(None);
        let value = password.get_untracked();
        leptos::task::spawn_local(async move {
            match account::verify_password(&value).await {
                Ok(()) => done(),
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    });
    let send_code = Callback::new(move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        note.set(None);
        leptos::task::spawn_local(async move {
            match account::send_code().await {
                Ok(()) => {
                    code.set(String::new());
                    step.set(Step::Code);
                }
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
            busy.set(false);
        });
    });
    let submit_code = Callback::new(move |_| {
        if busy.get_untracked() || code.get_untracked().chars().count() != CODE_LENGTH {
            return;
        }
        busy.set(true);
        note.set(None);
        let value = code.get_untracked();
        leptos::task::spawn_local(async move {
            match account::verify_code(&value).await {
                Ok(()) => done(),
                Err(e) => fail(e),
            }
            busy.set(false);
        });
    });
    let children = StoredValue::new(children);
    let showing = move || signed_in && !verified();
    if let Some(active) = use_context::<GateActive>() {
        Effect::new(move |_| active.0.set(showing()));
        on_cleanup(move || active.0.set(false));
    }

    move || {
        if !signed_in {
            return if account {
                view! {
                    <div class="text-center">
                        <p class="mb-6 text-body-md text-ink/70">{t("set-signin-needed")}</p>
                        <a href=routes::LOGIN class="btn-ink inline-block px-8 py-3">{t("help-ui-signin")}</a>
                    </div>
                }
                .into_any()
            } else {
                children.with_value(|c| c()).into_any()
            };
        }
        if verified() {
            return children.with_value(|c| c()).into_any();
        }
        // from 1024 px the check is a page of its own, as on the website: no side menu, one centred column of 400 px
        view! {
            <div class="lg:fixed lg:inset-x-0 lg:bottom-0 lg:top-settings-header lg:z-(--z-screen) lg:overflow-auto lg:bg-page">
            <div class="w-full max-w-[400px] max-lg:max-w-md max-lg:pt-8 lg:mx-auto lg:translate-x-4 lg:pt-[81px]">
                <h2 class="mb-2 text-account-title-sm font-bold leading-[41px]">{move || if step.get() == Step::Code { t("set-code-title") } else { t("set-verify-title") }}</h2>
                <p class="mb-7 text-body-md leading-5 text-ink/70 lg:text-body-md" class=("lg:mb-9", move || step.get() == Step::Code)>
                    {move || if step.get() == Step::Code {
                        view! { {t("set-code-body-1")}" "<strong class="font-bold">{email.get_value()}</strong>" "{t_args("set-code-body-2", &[("minutes", CODE_VALID_MINUTES.into())])} }.into_any()
                    } else { t("set-verify-body").into_any() }}
                </p>
                <Message text=note />
                {move || match step.get() {
                    Step::Choose => view! {
                        <div class="flex flex-col gap-5">
                            // radius: an option card
                            <button type="button" on:click=move |_| send_code.run(()) class="flex h-[69px] w-full items-center gap-[15px] rounded-card bg-ink/5 px-3 text-start hover:bg-ink/10">
                                <i class="ph ph-envelope-simple shrink-0 text-body-md text-ink" aria-hidden="true"></i>
                                <span class="min-w-0 flex-1"><span class="block text-title-sm font-bold leading-6">{t("set-code-email")}</span><span class="block text-body-sm leading-[22px] text-ink/70">{email.get_value()}</span></span>
                                <i class="ph-bold ph-caret-right shrink-0 text-body-md text-ink rtl:-scale-x-100" aria-hidden="true"></i>
                            </button>
                            // radius: an option card
                            <button type="button" on:click=move |_| { note.set(None); step.set(Step::Password) } class="flex h-[69px] w-full items-center gap-[15px] rounded-card bg-ink/5 px-3 text-start hover:bg-ink/10">
                                <i class="ph ph-lock-simple shrink-0 text-body-md text-ink" aria-hidden="true"></i>
                                <span class="min-w-0 flex-1"><span class="block text-title-sm font-bold leading-6">{t("set-password-confirm")}</span></span>
                                <i class="ph-bold ph-caret-right shrink-0 text-body-md text-ink rtl:-scale-x-100" aria-hidden="true"></i>
                            </button>
                        </div>
                    }.into_any(),
                    Step::Password => view! {
                        <form on:submit=move |ev| { ev.prevent_default(); submit_password.run(()); }>
                            <PasswordField label=t("set-password-current") value=password />
                            <div class="flex flex-wrap items-center gap-4">
                                <button type="submit" disabled=move || busy.get() class="btn-ink px-8 py-2.5 disabled:opacity-50">{t("set-continue")}</button>
                                <a href="/forgot" class="text-body-md text-ink/70 underline underline-offset-2 hover:text-ink">{t("set-forgot-password")}</a>
                            </div>
                            <button type="button" on:click=move |_| { note.set(None); step.set(Step::Choose) } class="mt-6 text-body-md font-semibold text-ink underline underline-offset-2">{t("set-try-another-way")}</button>
                        </form>
                    }.into_any(),
                    Step::Code => view! {
                        <form on:submit=move |ev| { ev.prevent_default(); submit_code.run(()); }>
                            <DigitBoxes label=t("set-code-label") value=code length=CODE_LENGTH one_time_code=true plain=true />
                            <div class="flex flex-col gap-2">
                                <button type="submit" disabled=move || busy.get() || code.with(|c| c.chars().count() != CODE_LENGTH) class="h-10 w-full rounded-button bg-ink px-4 text-body-md font-medium text-void disabled:opacity-50">{t("set-code-submit")}</button>
                                <button type="button" on:click=move |_| send_code.run(()) disabled=move || busy.get() class="h-10 w-full rounded-button bg-glass-hover px-4 text-body-md font-medium text-ink disabled:opacity-50">{t("set-code-resend")}</button>
                                <button type="button" on:click=move |_| { note.set(None); step.set(Step::Choose) } class="h-10 w-full rounded-button px-4 text-body-md font-medium text-ink">{t("set-try-another-way")}</button>
                            </div>
                        </form>
                    }.into_any(),
                }}
                <p class="mt-8 text-body-md text-ink/70 lg:text-label" class=("lg:mt-4", move || step.get() == Step::Code) class=("lg:mt-[52px]", move || step.get() != Step::Code) class=("lg:text-body-sm", move || step.get() != Step::Code) class=("lg:text-ink", move || step.get() != Step::Code)>{t("set-need-help")}" "<a href=routes::HELP class="underline underline-offset-2 hover:text-ink">{t("set-visit-help")}</a></p>
            </div>
            </div>
        }
        .into_any()
    }
}
