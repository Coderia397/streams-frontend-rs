//! The account's security: password, email and phone, one time codes, the devices signed in and the security log.
use super::*;

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

/// Ends the session an older token belongs to, without touching the new one. Every password or code check makes a new sign in
/// at Supabase; without this the old one would stay in the device list for ever. A failure (the old token had expired) is fine.
pub(super) async fn end_old_session(old_token: &str) {
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
    clear_resume();
    clear_session(SessionEnd::Everywhere);
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
