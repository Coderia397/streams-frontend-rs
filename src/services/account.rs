//! Sign up, sign in, password reset and sign out against Supabase Auth over its HTTP interface. The session is kept in the
//! browser (local storage when the person asks to stay signed in, session storage otherwise) and is counted as essential in the
//! cookie manager. Profiles still live in this browser; the database side is in streams-backend-rs/supabase.
use gloo_net::http::Request;
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
}

/// The saved session, if there is one that has not expired.
pub fn session() -> Option<Session> {
    [true, false]
        .into_iter()
        .filter_map(|p| storage(p)?.get_item(crate::keys::SESSION).ok().flatten())
        .filter_map(|raw| serde_json::from_str::<Session>(&raw).ok())
        .find(|s| s.expires_at > now_secs())
}

pub async fn sign_in(email: &str, password: &str, remember: bool) -> Result<Session, AuthError> {
    let previous = session().map(|s| s.access_token);
    let (status, body) = post("/auth/v1/token?grant_type=password", &json!({ "email": email.trim(), "password": password }), None).await?;
    if status == 200 {
        let session = session_from(&body).ok_or_else(|| AuthError::Other("unexpected answer".into()))?;
        save(&session, remember);
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
    if let Some(s) = session() {
        leptos::task::spawn_local(async move {
            let _ = post("/auth/v1/logout", &json!({}), Some(&s.access_token)).await;
        });
    }
    for persistent in [true, false] {
        if let Some(st) = storage(persistent) {
            let _ = st.remove_item(crate::keys::SESSION);
        }
    }
}

async fn put_user(body: &Value, token: &str) -> Result<(), AuthError> {
    let resp = Request::put(&url("/auth/v1/user"))
        .header("apikey", crate::config::SUPABASE_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .body(body.to_string())
        .map_err(|_| AuthError::Network)?
        .send()
        .await
        .map_err(|_| AuthError::Network)?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if status == 200 { Ok(()) } else { Err(classify(status, &serde_json::from_str(&text).unwrap_or(Value::Null))) }
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
            for p in [true, false] { if let Some(st) = storage(p) { let _ = st.remove_item(crate::keys::SESSION); } }
            None
        }
        // no connection: keep what there is, try again later
        _ => session(),
    }
}

/// Keeps the session fresh while the page is open (every five minutes).
pub fn keep_session_fresh() {
    leptos::task::spawn_local(async { let _ = ensure_fresh().await; });
    leptos::leptos_dom::helpers::set_interval(|| { leptos::task::spawn_local(async { let _ = ensure_fresh().await; }); }, std::time::Duration::from_secs(300));
}

pub(crate) async fn token() -> Result<String, AuthError> {
    ensure_fresh().await.map(|s| s.access_token).ok_or_else(|| AuthError::Other("not signed in".into()))
}

/// Ends the session an older token belongs to, without touching the new one. Every password or code check makes a new sign in
/// at Supabase; without this the old one would stay in the device list for ever. A failure (the old token had expired) is fine.
async fn end_old_session(old_token: &str) {
    let _ = post("/auth/v1/logout?scope=local", &json!({}), Some(old_token)).await;
}

/// Checks the password of the signed in account for the step up check. The sign in it creates replaces the saved session (in the
/// same storage), because the database decides "recent sign in" from the time the password was last used, and that time is in the
/// new token.
pub async fn verify_password(password: &str) -> Result<(), AuthError> {
    let current = session().ok_or_else(|| AuthError::Other("not signed in".into()))?;
    let remember = storage(true).and_then(|s| s.get_item(crate::keys::SESSION).ok().flatten()).is_some();
    let (status, body) = post("/auth/v1/token?grant_type=password", &json!({ "email": current.email, "password": password }), None).await?;
    if status == 200 {
        if let Some(fresh) = session_from(&body) {
            save(&fresh, remember);
            end_old_session(&current.access_token).await;
            crate::services::device::register_in_background();
        }
        return Ok(());
    }
    Err(classify(status, &body))
}

pub async fn change_password(new_password: &str) -> Result<(), AuthError> {
    put_user(&json!({ "password": new_password }), &token().await?).await
}

/// Starts an email change; the address in use changes once the new one is confirmed.
pub async fn change_email(new_email: &str) -> Result<(), AuthError> {
    put_user(&json!({ "email": new_email.trim() }), &token().await?).await
}

/// Keeps the mobile number in the account's profile data (no text message is sent for it).
pub async fn save_phone(phone: &str) -> Result<(), AuthError> {
    put_user(&json!({ "data": { "phone": phone.trim() } }), &token().await?).await?;
    // the saved session keeps the number so the Account screen shows it without asking again
    for persistent in [true, false] {
        if let Some(st) = storage(persistent) {
            if let Some(mut saved) = st.get_item(crate::keys::SESSION).ok().flatten().and_then(|raw| serde_json::from_str::<Session>(&raw).ok()) {
                saved.phone = phone.trim().to_string();
                if let Ok(raw) = serde_json::to_string(&saved) {
                    let _ = st.set_item(crate::keys::SESSION, &raw);
                }
            }
        }
    }
    Ok(())
}

/// Signs this browser and every other browser out of the account.
pub async fn sign_out_everywhere() -> Result<(), AuthError> {
    let t = token().await?;
    let (status, body) = post("/auth/v1/logout?scope=global", &json!({}), Some(&t)).await?;
    if status != 204 && status != 200 {
        return Err(classify(status, &body));
    }
    for persistent in [true, false] {
        if let Some(st) = storage(persistent) {
            let _ = st.remove_item(crate::keys::SESSION);
        }
    }
    Ok(())
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

/// Changes the first and last name through `update_account_names` (the main profile follows its first name there). The names are
/// also kept in the session so the screens show them at once.
pub async fn update_names(first: &str, last: &str) -> Result<(), AuthError> {
    rpc("update_account_names", &json!({ "p_first": first.trim(), "p_last": last.trim() })).await?;
    for persistent in [true, false] {
        if let Some(st) = storage(persistent) {
            if let Some(mut saved) = st.get_item(crate::keys::SESSION).ok().flatten().and_then(|raw| serde_json::from_str::<Session>(&raw).ok()) {
                saved.first_name = first.trim().to_string();
                saved.last_name = last.trim().to_string();
                if let Ok(raw) = serde_json::to_string(&saved) {
                    let _ = st.set_item(crate::keys::SESSION, &raw);
                }
            }
        }
    }
    Ok(())
}

/// Sends a one time code to the account's email address, for the password check. The same answer is given whatever the address.
pub async fn send_code() -> Result<(), AuthError> {
    let email = session().ok_or_else(|| AuthError::Other("not signed in".into()))?.email;
    let (status, body) = post("/auth/v1/otp", &json!({ "email": email, "create_user": false }), None).await?;
    if status == 200 { Ok(()) } else { Err(classify(status, &body)) }
}

/// Checks the code from `send_code`. Like `verify_password` the sign in it creates replaces the saved session.
pub async fn verify_code(code: &str) -> Result<(), AuthError> {
    let current = session().ok_or_else(|| AuthError::Other("not signed in".into()))?;
    let remember = storage(true).and_then(|s| s.get_item(crate::keys::SESSION).ok().flatten()).is_some();
    let (status, body) = post("/auth/v1/verify", &json!({ "type": "email", "email": current.email, "token": code.trim() }), None).await?;
    if status == 200 {
        if let Some(fresh) = session_from(&body) {
            save(&fresh, remember);
            end_old_session(&current.access_token).await;
            crate::services::device::register_in_background();
        }
        return Ok(());
    }
    Err(classify(status, &body))
}

/// One signed in browser of the account (`list_my_sessions`).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DeviceSession {
    pub id: String,
    pub created_at: String,
    pub last_seen: String,
    #[serde(default)]
    pub user_agent: Option<String>,
    pub is_current: bool,
    // what the device signature added (empty for a session that signed in before it existed)
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub browser: Option<String>,
    #[serde(default)]
    pub browser_major: Option<u32>,
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub screen: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub countries: Option<Vec<String>>,
    #[serde(default)]
    pub device_first_seen: Option<String>,
}

pub async fn sessions() -> Result<Vec<DeviceSession>, AuthError> {
    let body = rpc("list_my_sessions", &json!({})).await?;
    serde_json::from_value(body).map_err(|_| AuthError::Other("unexpected answer".into()))
}

/// Signs one other browser out (`revoke_my_session`), after a recent password check.
pub async fn revoke_session(id: &str) -> Result<(), AuthError> {
    rpc("revoke_my_session", &json!({ "p_id": id })).await.map(|_| ())
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

/// Saves one of the account switches (`test_participation` or `profile_transfer`) through `set_account_switch`.
pub async fn set_switch(name: &str, value: bool) -> Result<(), AuthError> {
    rpc("set_account_switch", &json!({ "p_name": name, "p_value": value })).await.map(|_| ())
}

/// Everything held about the signed in person (`export_my_data`), as text ready to save as a file.
pub async fn export_data() -> Result<String, AuthError> {
    let body = rpc("export_my_data", &json!({})).await?;
    serde_json::to_string_pretty(&body).map_err(|_| AuthError::Other("unexpected answer".into()))
}

/// The newest security events of the account (sign ins that changed things, PIN changes, profile changes): time and event name.
pub async fn security_events(limit: u32) -> Result<Vec<(String, String)>, AuthError> {
    let t = token().await?;
    let resp = Request::get(&url(&format!("/rest/v1/account_events?select=ts,event&order=ts.desc&limit={limit}")))
        .header("apikey", crate::config::SUPABASE_KEY)
        .header("Authorization", &format!("Bearer {t}"))
        .send()
        .await
        .map_err(|_| AuthError::Network)?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let body: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if status != 200 {
        return Err(classify(status, &body));
    }
    Ok(body.as_array().map(|rows| rows.iter().filter_map(|r| Some((r["ts"].as_str()?.to_string(), r["event"].as_str()?.to_string()))).collect()).unwrap_or_default())
}

/// Deletes the signed in account and everything stored for it, through the `delete_my_account` database function. The caller
/// clears the data kept in this browser afterwards.
pub async fn delete_account() -> Result<(), AuthError> {
    rpc("delete_my_account", &json!({})).await.map(|_| ())
}

/// True for someone who has been here before: signed in, or a profile was chosen on this browser. They go straight to the app;
/// everyone else sees the landing page at the root address.
pub fn is_returning() -> bool {
    session().is_some() || web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(crate::keys::ACTIVE_PROFILE).ok().flatten()).is_some()
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

// ── profile lock (PIN), through the database functions: the PIN is hashed there and never leaves it ────────────────────────────

fn answer_bool(v: Value) -> bool {
    v.as_bool().unwrap_or(false)
}

/// Sets or changes the PIN of a profile (`set_profile_pin`). `current` is needed when the profile already has one. False when the current PIN was wrong.
pub async fn pin_set(profile: &str, pin: &str, current: Option<&str>) -> Result<bool, AuthError> {
    rpc("set_profile_pin", &json!({ "p_profile": profile, "p_pin": pin, "p_current": current })).await.map(answer_bool)
}

/// Removes the PIN (`clear_profile_pin`). False when the current PIN was wrong.
pub async fn pin_clear(profile: &str, current: &str) -> Result<bool, AuthError> {
    rpc("clear_profile_pin", &json!({ "p_profile": profile, "p_current": current })).await.map(answer_bool)
}

/// Checks a PIN (`verify_profile_pin`; the database counts the wrong tries and locks the profile for a while after too many).
pub async fn pin_verify(profile: &str, pin: &str) -> Result<bool, AuthError> {
    rpc("verify_profile_pin", &json!({ "p_profile": profile, "p_pin": pin })).await.map(answer_bool)
}

/// Removes the PIN without asking for it (`reset_profile_pin`; the screen asks for the password first).
pub async fn pin_reset(profile: &str) -> Result<(), AuthError> {
    rpc("reset_profile_pin", &json!({ "p_profile": profile })).await.map(|_| ())
}
