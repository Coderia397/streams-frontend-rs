//! Signing in, up and out, confirming an address, and taking a kept sign-in back (Remember me).
use super::*;

pub async fn sign_in(email: &str, password: &str, remember: bool) -> Result<Session, AuthError> {
    let previous = session().map(|s| s.access_token);
    let (status, body) = post("/auth/v1/token?grant_type=password", &json!({ "email": email.trim(), "password": password }), None).await?;
    if status == 200 {
        let session = session_from(&body).ok_or_else(|| AuthError::Other("unexpected answer".into()))?;
        save(&session, remember);
        clear_resume();
        if let Some(old) = previous {
            end_old_session(&old).await;
        }
        crate::services::device::register_in_background();
        return Ok(session);
    }
    Err(classify(status, &body))
}

pub async fn sign_up(email: &str, password: &str, first: &str, last: &str) -> Result<SignUp, AuthError> {
    let data = json!({ "first_name": first.trim(), "last_name": last.trim(), "full_name": format!("{} {}", first.trim(), last.trim()).trim() });
    let (status, body) = post(&format!("/auth/v1/signup?redirect_to={}", confirm_address()), &json!({ "email": email.trim(), "password": password, "data": data }), None).await?;
    if status == 200 {
        return Ok(match session_from(&body) {
            Some(session) => {
                save(&session, true);
                SignUp::SignedIn(session)
            }
            None => SignUp::ConfirmEmail,
        });
    }
    Err(classify(status, &body))
}

/// Asks for a reset link. The answer is the same whether or not the address has an account.
pub async fn recover(email: &str) -> Result<(), AuthError> {
    let (status, body) = post(&format!("/auth/v1/recover?redirect_to={}", confirm_address()), &json!({ "email": email.trim() }), None).await?;
    if status == 200 || status == 400 && classify(status, &body) == AuthError::Invalid { Ok(()) } else { Err(classify(status, &body)) }
}

/// Resends the confirmation email for an address that signed up.
pub async fn resend_confirmation(email: &str) -> Result<(), AuthError> {
    let (status, body) = post("/auth/v1/resend", &json!({ "type": "signup", "email": email.trim() }), None).await?;
    if status == 200 { Ok(()) } else { Err(classify(status, &body)) }
}

pub fn sign_out() {
    // plain sign out: the session ends at once (a tab only session, or a browser that cannot seal the token, ends here too)
    if let Some(s) = session() {
        leptos::task::spawn_local(async move {
            let _ = post("/auth/v1/logout", &json!({}), Some(&s.access_token)).await;
        });
    }
    clear_session(SessionEnd::SignedOut);
}

#[wasm_bindgen::prelude::wasm_bindgen(module = "/interop/dist/streams-interop.js")]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = resumeSeal, catch)]
    async fn resume_seal(plain: &str) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = resumeOpen, catch)]
    async fn resume_open(sealed: &str) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = resumeForget, catch)]
    async fn resume_forget() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = resumeFingerprint, catch)]
    async fn resume_fingerprint() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}

/// Signs out of this browser. A browser that stays signed in ("Remember me") keeps the session for a day, sealed on the device (see `interop/resume.ts`), so that typing the
/// same address on the sign in page brings the person back without the password (`resume_sign_in`). The session at the account ends when the day is over
/// (`end_expired_resume`), at once with "Sign out of all devices" or "Forget this browser", and as soon as the account refuses the token (password changed, signed out elsewhere).
/// A tab only session, or one that cannot be sealed, ends at once.
pub async fn sign_out_async() {
    if let Some((s, true)) = stored_session() {
        if !s.refresh_token.is_empty() {
            let fp = resume_fingerprint().await.ok().and_then(|v| v.as_string()).unwrap_or_default();
            let payload = json!({ "t": s.refresh_token, "f": s.first_name, "l": s.last_name, "p": fp }).to_string();
            if let Some(sealed) = resume_seal(&payload).await.ok().and_then(|v| v.as_string()) {
                let resume = Resume { email: s.email.trim().to_lowercase(), sealed, until: now_secs() + RESUME_SECONDS };
                if let (Some(st), Ok(raw)) = (storage(true), serde_json::to_string(&resume)) {
                    let _ = st.set_item(RESUME_KEY, &raw);
                }
                clear_session(SessionEnd::SignedOut);
                return;
            }
        }
    }
    sign_out();
}

const RESUME_KEY: &str = "streams_resume";

/// How long a signed out browser can come back with the address alone.
const RESUME_SECONDS: u64 = 24 * 60 * 60;

#[derive(serde::Serialize, serde::Deserialize)]
struct Resume {
    email: String,
    sealed: String,
    until: u64,
}

fn read_resume() -> Option<Resume> {
    serde_json::from_str(&storage(true)?.get_item(RESUME_KEY).ok().flatten()?).ok()
}

pub(super) fn clear_resume() {
    if let Some(st) = storage(true) {
        let _ = st.remove_item(RESUME_KEY);
    }
}

/// The address this browser signed out with less than a day ago, when it can be resumed.
pub fn resume_email() -> Option<String> {
    read_resume().filter(|r| r.until > now_secs()).map(|r| r.email)
}

/// True when `email` is the address this browser signed out with less than a day ago.
pub fn can_resume(email: &str) -> bool {
    resume_email().is_some_and(|e| e == email.trim().to_lowercase())
}

/// What the sealed text holds: the refresh token, the names and the device fingerprint taken at sign out.
struct Opened { token: String, first: String, last: String, fingerprint: String }

async fn open_resume(r: &Resume) -> Option<Opened> {
    let plain = resume_open(&r.sealed).await.ok()?.as_string().filter(|p| !p.is_empty())?;
    let v: Value = serde_json::from_str(&plain).ok()?;
    let g = |k: &str| v[k].as_str().unwrap_or_default().to_string();
    Some(Opened { token: g("t"), first: g("f"), last: g("l"), fingerprint: g("p") })
}

/// Ends a kept session for good: the token is renewed once (it is single use) and that session is logged out.
async fn end_kept_session(token: &str) {
    if let Ok((200, body)) = post("/auth/v1/token?grant_type=refresh_token", &json!({ "refresh_token": token }), None).await {
        if let Some(s) = session_from(&body) {
            let _ = post("/auth/v1/logout?scope=local", &json!({}), Some(&s.access_token)).await;
        }
    }
}

/// Signs the person back in with the token kept at sign out, for the same address typed again (no password). It is refused, and the password asked, when the day is over, the
/// sealed text cannot be opened (the device key is gone), the device does not look like the one that signed out, or the account no longer accepts the token. The token is single use
/// (the account gives a new one with every renewal) and the kept copy is removed at once.
pub async fn resume_sign_in(email: &str) -> Option<Session> {
    let r = read_resume().filter(|r| r.until > now_secs() && r.email == email.trim().to_lowercase())?;
    clear_resume();
    let opened = open_resume(&r).await?;
    let here = resume_fingerprint().await.ok().and_then(|v| v.as_string()).unwrap_or_default();
    if opened.token.is_empty() || (!opened.fingerprint.is_empty() && opened.fingerprint != here) {
        // not the device it was kept on: the kept session is ended and the password is asked
        end_kept_session(&opened.token).await;
        let _ = resume_forget().await;
        return None;
    }
    match post("/auth/v1/token?grant_type=refresh_token", &json!({ "refresh_token": opened.token }), None).await {
        Ok((200, body)) => {
            let mut fresh = session_from(&body)?;
            if fresh.first_name.is_empty() { fresh.first_name = opened.first; fresh.last_name = opened.last; }
            save(&fresh, true);
            let _ = resume_forget().await;
            crate::services::device::register_in_background();
            Some(fresh)
        }
        _ => None,
    }
}

/// "Not you? Forget this browser": the kept session is ended and the key that sealed it is deleted.
pub fn forget_this_browser() {
    let Some(r) = read_resume() else { return };
    clear_resume();
    leptos::task::spawn_local(async move {
        if let Some(o) = open_resume(&r).await {
            end_kept_session(&o.token).await;
        }
        let _ = resume_forget().await;
    });
}

/// When the day after sign out is over, the session that was kept is ended for good (best effort: it needs a connection).
pub fn end_expired_resume() {
    let Some(r) = read_resume().filter(|r| r.until <= now_secs()) else { return };
    clear_resume();
    leptos::task::spawn_local(async move {
        if let Some(o) = open_resume(&r).await {
            end_kept_session(&o.token).await;
        }
        let _ = resume_forget().await;
    });
}

/// Completes a link from one of our emails: the address carries the token hash and the kind (signup, magiclink, recovery,
/// email_change, invite). The sign in it creates is saved and kept.
pub async fn confirm_token(kind: &str, token_hash: &str) -> Result<Session, AuthError> {
    let (status, body) = post("/auth/v1/verify", &json!({ "type": kind, "token_hash": token_hash }), None).await?;
    if status == 200 {
        let session = session_from(&body).ok_or_else(|| AuthError::Other("unexpected answer".into()))?;
        save(&session, true);
        crate::services::device::register_in_background();
        return Ok(session);
    }
    Err(classify(status, &body))
}

/// Links from the default Supabase email come back with the sign in in the address after the #: access_token, refresh_token,
/// expires_in and type. Reads them, asks who the user is, and saves the session. None when there is nothing to read.
pub async fn session_from_fragment(fragment: &str) -> Option<Result<(Session, String), AuthError>> {
    let pairs: Vec<(String, String)> = fragment.trim_start_matches('#').split('&').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), js_sys::decode_uri_component(v).ok().and_then(|s| s.as_string()).unwrap_or_else(|| v.to_string()))).collect();
    let get = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
    let access = get("access_token")?;
    let kind = get("type").unwrap_or_default();
    let result = async {
        let resp = Request::get(&url("/auth/v1/user")).header("apikey", crate::config::SUPABASE_KEY).header("Authorization", &format!("Bearer {access}")).send().await.map_err(|_| AuthError::Network)?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let user: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if status != 200 {
            return Err(classify(status, &user));
        }
        let expires_in = get("expires_in").and_then(|v| v.parse::<u64>().ok()).unwrap_or(3600);
        let value = json!({ "access_token": access, "refresh_token": get("refresh_token").unwrap_or_default(), "expires_in": expires_in, "user": user });
        let session = session_from(&value).ok_or_else(|| AuthError::Other("unexpected answer".into()))?;
        save(&session, true);
        crate::services::device::register_in_background();
        Ok((session, kind))
    };
    Some(result.await)
}

/// True for someone who has been here before: signed in, or a profile was chosen on this browser. They go straight to the app;
/// everyone else sees the landing page at the root address.
pub fn is_returning() -> bool {
    session().is_some() || web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten()).is_some()
}
