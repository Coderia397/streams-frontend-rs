//! Where the links in our emails land (/auth/confirm). The branded emails carry `token_hash` and `type` in the address; links
//! made by the default Supabase email carry the sign in after the #. Either way this page signs the person in, then sends them on:
//! a new account to the profile chooser, a password reset to Change password, an email change to Account.
use super::{button_class, AuthShell, Look};
use crate::config::routes;
use crate::pages::help::tr;
use crate::services::account;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Working,
    Failed,
}

#[component]
pub fn ConfirmPage() -> impl IntoView {
    Effect::new(move |_| crate::pages::help::set_site_title(&tr("auth-confirm-title")));
    let query = use_query_map();
    let navigate = use_navigate();
    let profiles = crate::store::use_profile_store();
    let prefs = crate::pages::settings::use_prefs();
    let state = RwSignal::new(State::Working);

    let token_hash = query.with_untracked(|q| q.get("token_hash"));
    let kind = query.with_untracked(|q| q.get("type")).unwrap_or_default();
    let fragment = web_sys::window().and_then(|w| w.location().hash().ok()).unwrap_or_default();

    leptos::task::spawn_local(async move {
        let outcome = match token_hash {
            Some(hash) => account::confirm_token(&kind, &hash).await.map(|s| (s, kind.clone())),
            None => match account::session_from_fragment(&fragment).await {
                Some(r) => r,
                None => {
                    state.set(State::Failed);
                    return;
                }
            },
        };
        match outcome {
            Ok((session, kind)) => {
                // clear the token out of the address bar and the history
                if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) {
                    let _ = h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(routes::AUTH_CONFIRM));
                }
                let next = match kind.as_str() {
                    // they just proved they own the mailbox, so the password screen does not ask again
                    "recovery" => {
                        prefs.mark_verified();
                        crate::pages::settings::Screen::Password.path()
                    }
                    "email_change" => crate::pages::settings::Screen::Account.path(),
                    _ => {
                        crate::store::start_account(profiles, &session.first_name);
                        routes::BROWSE.to_string()
                    }
                };
                navigate(&next, Default::default());
            }
            Err(_) => state.set(State::Failed),
        }
    });

    view! {
        <AuthShell look=Look::Dark>
            <div class="mx-auto w-full max-w-[440px] px-6 pb-24 pt-10">
                {move || match state.get() {
                    State::Working => view! {
                        <h1 class="text-heading-xl font-bold leading-tight">{tr("auth-confirm-working")}</h1>
                        <p class="mt-3 text-body-md text-ink-body">{tr("auth-confirm-working-body")}</p>
                    }.into_any(),
                    State::Failed => view! {
                        <h1 class="text-heading-xl font-bold leading-tight">{tr("auth-confirm-failed-title")}</h1>
                        <p class="mb-6 mt-3 text-body-md leading-7 text-ink-body">{tr("auth-confirm-failed-body")}</p>
                        <a href=routes::LOGIN class=button_class(Look::Dark)>{tr("auth-back-to-sign-in")}</a>
                        <p class="mt-5 text-body-md"><a href="/forgot" class="text-ink-body underline underline-offset-2 hover:text-ink">{tr("auth-confirm-new-link")}</a></p>
                    }.into_any(),
                }}
            </div>
        </AuthShell>
    }
}
