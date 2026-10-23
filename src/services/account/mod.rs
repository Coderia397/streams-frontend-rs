//! Sign up, sign in, password reset and sign out against Supabase Auth over its HTTP interface. The session is kept in the
//! browser (local storage when the person asks to stay signed in, session storage otherwise) and is counted as essential in the
//! cookie manager. Profiles still live in this browser; the database side is in streams-backend-rs/supabase.
use gloo_net::http::Request;

use leptos::prelude::{ArcRwSignal, GetUntracked, Set};

use serde::{Deserialize, Serialize};

use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds when the access token stops working.
    pub expires_at: u64,
    pub user_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub phone: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthError {
    /// Wrong email or password. The same answer is given for an unknown email.
    Invalid,
    /// The address has to be confirmed first.
    Unconfirmed,
    /// The password does not meet the rules; the text says why.
    Weak(String),
    /// Signing up failed in a way that must not reveal whether the address is already registered.
    CouldNotCreate,
    RateLimited,
    /// The database wants a password check made in the last few minutes (error 28000).
    RecentSignIn,
    Network,
    Other(String),
}

/// What signing up leads to.
#[derive(Clone, Debug, PartialEq)]
pub enum SignUp {
    /// The account is ready and signed in.
    SignedIn(Session),
    /// A confirmation email was sent.
    ConfirmEmail,
}

fn url(path: &str) -> String {
    format!("{}{path}", crate::config::SUPABASE_URL)
}

fn storage(persistent: bool) -> Option<web_sys::Storage> {
    let w = web_sys::window()?;
    if persistent { w.local_storage().ok().flatten() } else { w.session_storage().ok().flatten() }
}

fn now_secs() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

fn session_from(v: &Value) -> Option<Session> {
    let meta = &v["user"]["user_metadata"];
    let text = |x: &Value| x.as_str().unwrap_or_default().to_string();
    Some(Session {
        access_token: v["access_token"].as_str()?.to_string(),
        refresh_token: v["refresh_token"].as_str().unwrap_or_default().to_string(),
        expires_at: v["expires_at"].as_u64().unwrap_or_else(|| now_secs() + v["expires_in"].as_u64().unwrap_or(3600)),
        user_id: v["user"]["id"].as_str()?.to_string(),
        email: v["user"]["email"].as_str().unwrap_or_default().to_string(),
        first_name: text(&meta["first_name"]),
        last_name: text(&meta["last_name"]),
        phone: text(&meta["phone"]),
    })
}

/// Maps a Supabase error body to a case the pages can speak about.
pub fn classify(status: u16, body: &Value) -> AuthError {
    let code = body["error_code"].as_str().or_else(|| body["code"].as_str()).unwrap_or_default();
    let msg = body["msg"].as_str().or_else(|| body["message"].as_str()).or_else(|| body["error_description"].as_str()).unwrap_or_default();
    if code == "28000" || msg.contains("recent sign in") {
        return AuthError::RecentSignIn;
    }
    match code {
        "invalid_credentials" | "invalid_grant" | "otp_expired" => AuthError::Invalid,
        "email_not_confirmed" => AuthError::Unconfirmed,
        "weak_password" => AuthError::Weak(msg.to_string()),
        "user_already_exists" | "email_exists" | "signup_disabled" | "validation_failed" | "email_address_invalid" => AuthError::CouldNotCreate,
        "over_request_rate_limit" | "over_email_send_rate_limit" => AuthError::RateLimited,
        _ if status == 429 => AuthError::RateLimited,
        _ if status == 400 && msg.to_lowercase().contains("invalid login") => AuthError::Invalid,
        _ => AuthError::Other(msg.to_string()),
    }
}

async fn post(path: &str, body: &Value, token: Option<&str>) -> Result<(u16, Value), AuthError> {
    let mut req = Request::post(&url(path)).header("apikey", crate::config::SUPABASE_KEY).header("Content-Type", "application/json");
    if let Some(t) = token {
        req = req.header("Authorization", &format!("Bearer {t}"));
    }
    let resp = req.body(body.to_string()).map_err(|_| AuthError::Network)?.send().await.map_err(|_| AuthError::Network)?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    Ok((status, serde_json::from_str(&text).unwrap_or(Value::Null)))
}

/// The address emails send people back to, encoded for use in a query.
fn confirm_address() -> String {
    js_sys::encode_uri_component(&format!("{}{}", crate::config::SITE_URL, crate::config::routes::AUTH_CONFIRM)).as_string().unwrap_or_default()
}

fn save(session: &Session, remember: bool) {
    // the other storage must not keep an older copy
    if let Some(other) = storage(!remember) {
        let _ = other.remove_item(crate::keys::SESSION);
    }
    if let (Some(s), Ok(raw)) = (storage(remember), serde_json::to_string(session)) {
        let _ = s.set_item(crate::keys::SESSION, &raw);
    }
    announce();
}

/// Why the session in this tab ended: decides where the tab goes once the account's data is cleared (see `lib.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionEnd {
    /// the person signed out here: the start page
    SignedOut,
    /// signed out of every device (or after a password change): sign in
    Everywhere,
    /// the account refused the session (password changed elsewhere, signed out elsewhere, account gone): sign in, then back to the page
    Refused,
    /// another tab ended it: the start page
    Elsewhere,
}

thread_local! {
    static ENDED_BY: std::cell::Cell<SessionEnd> = const { std::cell::Cell::new(SessionEnd::Elsewhere) };
}

/// The reason the last session ended in this tab, read once (a session ended by another tab reads `Elsewhere`).
pub fn take_session_end() -> SessionEnd {
    ENDED_BY.with(|c| c.replace(SessionEnd::Elsewhere))
}

/// Removes the session from this browser (both storages). Every sign out goes through here, so the signed in signal always follows.
fn clear_session(why: SessionEnd) {
    ENDED_BY.with(|c| c.set(why));
    for persistent in [true, false] {
        if let Some(st) = storage(persistent) {
            let _ = st.remove_item(crate::keys::SESSION);
        }
    }
    announce();
}

thread_local! {
    /// Whether this browser holds an account session, as a signal: the account gate and the pages that differ for members follow it.
    static SIGNED_IN: ArcRwSignal<bool> = ArcRwSignal::new(has_account());
}

/// The signed in signal. It changes when this tab signs in or out (`save`, `clear_session`) and when another tab does (`announce`
/// from the window's storage event, set up in `lib.rs`).
pub fn signed_in() -> ArcRwSignal<bool> {
    SIGNED_IN.with(|s| s.clone())
}

/// Brings the signed in signal up to date with what the browser holds.
pub fn announce() {
    let now = has_account();
    SIGNED_IN.with(|s| {
        if s.get_untracked() != now {
            s.set(now);
        }
    });
}

/// The saved session, if there is one that has not expired.
pub fn session() -> Option<Session> {
    [true, false]
        .into_iter()
        .filter_map(|p| storage(p)?.get_item(crate::keys::SESSION).ok().flatten())
        .filter_map(|raw| serde_json::from_str::<Session>(&raw).ok())
        .find(|s| s.expires_at > now_secs())
}

/// The saved session whether or not its token has expired, and the storage it sits in (a browser that stays signed in, or this tab only).
fn stored_session() -> Option<(Session, bool)> {
    [true, false].into_iter().find_map(|p| {
        let raw = storage(p)?.get_item(crate::keys::SESSION).ok().flatten()?;
        serde_json::from_str::<Session>(&raw).ok().map(|s| (s, p))
    })
}

/// Whether this browser holds an account session (a token that has run out still counts: it is renewed with the refresh token).
pub fn has_account() -> bool {
    stored_session().is_some_and(|(s, _)| s.expires_at > now_secs() || !s.refresh_token.is_empty())
}

/// The session with a token that works: the saved one, or a new one made with the refresh token when the saved token has run out (a token lasts an hour). When the account refuses the
/// refresh token (signed out elsewhere, deleted) the saved session is removed and the person is taken to sign in by the gate.
pub async fn ensure_fresh() -> Option<Session> {
    if let Some(s) = session() {
        // renewed a little before it runs out
        if s.expires_at > now_secs() + 120 || s.refresh_token.is_empty() { return Some(s); }
    }
    let (old, persistent) = stored_session()?;
    if old.refresh_token.is_empty() { return None; }
    match post("/auth/v1/token?grant_type=refresh_token", &json!({ "refresh_token": old.refresh_token }), None).await {
        Ok((200, body)) => {
            let mut fresh = session_from(&body)?;
            if fresh.first_name.is_empty() { fresh.first_name = old.first_name.clone(); fresh.last_name = old.last_name.clone(); }
            save(&fresh, persistent);
            Some(fresh)
        }
        Ok((400..=403, _)) => {
            clear_session(SessionEnd::Refused);
            None
        }
        // no connection: keep what there is, try again later
        _ => session(),
    }
}

/// Keeps the session fresh while the page is open (every five minutes).
pub fn keep_session_fresh() {
    end_expired_resume();
    leptos::task::spawn_local(async { let _ = ensure_fresh().await; });
    leptos::leptos_dom::helpers::set_interval(|| { leptos::task::spawn_local(async { let _ = ensure_fresh().await; }); }, std::time::Duration::from_secs(300));
}

pub(crate) async fn token() -> Result<String, AuthError> {
    ensure_fresh().await.map(|s| s.access_token).ok_or_else(|| AuthError::Other("not signed in".into()))
}

/// A call to the account database's table interface (PostgREST) with the person's token: `method` is GET, POST, PATCH or DELETE, `path` starts with `/rest/v1/`, `prefer` is the Prefer header
/// (`return=representation` to get the written rows back). Returns the answer's JSON (Null for no content).
pub(crate) async fn rest(method: &str, path: &str, body: Option<&Value>, prefer: &str) -> Result<Value, AuthError> {
    let t = token().await?;
    let builder = match method {
        "POST" => Request::post(&url(path)),
        "PATCH" => Request::patch(&url(path)),
        "DELETE" => Request::delete(&url(path)),
        _ => Request::get(&url(path)),
    }
    .header("apikey", crate::config::SUPABASE_KEY)
    .header("Authorization", &format!("Bearer {t}"))
    .header("Content-Type", "application/json");
    let builder = if prefer.is_empty() { builder } else { builder.header("Prefer", prefer) };
    let resp = match body {
        Some(b) => builder.body(b.to_string()).map_err(|_| AuthError::Network)?.send().await,
        None => builder.send().await,
    }
    .map_err(|_| AuthError::Network)?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if (200..300).contains(&status) { Ok(value) } else { Err(classify(status, &value)) }
}

pub(crate) async fn rpc(name: &str, args: &Value) -> Result<Value, AuthError> {
    let t = token().await?;
    let resp = Request::post(&url(&format!("/rest/v1/rpc/{name}")))
        .header("apikey", crate::config::SUPABASE_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {t}"))
        .body(args.to_string())
        .map_err(|_| AuthError::Network)?
        .send()
        .await
        .map_err(|_| AuthError::Network)?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let body: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if (200..300).contains(&status) { Ok(body) } else { Err(classify(status, &body)) }
}

pub fn valid_email(email: &str) -> bool {
    let e = email.trim();
    match e.split_once('@') {
        Some((local, domain)) => !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.') && !e.contains(' '),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_shape() {
        assert!(valid_email("a@b.co"));
        assert!(valid_email("  first.last@mail.example.org "));
        assert!(!valid_email("a@b"));
        assert!(!valid_email("@b.co"));
        assert!(!valid_email("a b@c.de"));
        assert!(!valid_email("plain"));
    }

    #[test]
    fn supabase_errors_map_to_cases() {
        assert_eq!(classify(400, &json!({"error_code": "invalid_credentials", "msg": "Invalid login credentials"})), AuthError::Invalid);
        assert_eq!(classify(400, &json!({"error_code": "email_not_confirmed"})), AuthError::Unconfirmed);
        assert_eq!(classify(422, &json!({"error_code": "user_already_exists"})), AuthError::CouldNotCreate);
        assert_eq!(classify(422, &json!({"error_code": "weak_password", "msg": "Password should be at least 8 characters"})), AuthError::Weak("Password should be at least 8 characters".into()));
        assert_eq!(classify(429, &Value::Null), AuthError::RateLimited);
    }

    #[test]
    fn a_session_is_read_from_the_token_answer() {
        let body = json!({"access_token": "a", "refresh_token": "r", "expires_at": 4102444800u64, "user": {"id": "u1", "email": "x@y.zz", "user_metadata": {"first_name": "Ada", "last_name": "Lovelace"}}});
        let s = session_from(&body).unwrap();
        assert_eq!((s.first_name.as_str(), s.last_name.as_str(), s.user_id.as_str()), ("Ada", "Lovelace", "u1"));
    }
}

mod profile_data;
mod security;
mod sign_in;

pub use profile_data::*;
pub use security::*;
pub use sign_in::*;
