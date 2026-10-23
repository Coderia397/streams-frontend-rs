//! The account's own data: names, switches, the export, deleting the account, and profile PINs.
use super::*;

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

/// Saves one of the account switches (`test_participation` or `profile_transfer`) through `set_account_switch`.
pub async fn set_switch(name: &str, value: bool) -> Result<(), AuthError> {
    rpc("set_account_switch", &json!({ "p_name": name, "p_value": value })).await.map(|_| ())
}

/// Everything held about the signed in person (`export_my_data`), as text ready to save as a file.
pub async fn export_data() -> Result<String, AuthError> {
    let body = rpc("export_my_data", &json!({})).await?;
    serde_json::to_string_pretty(&body).map_err(|_| AuthError::Other("unexpected answer".into()))
}

/// Deletes the signed in account and everything stored for it, through the `delete_my_account` database function. The caller
/// clears the data kept in this browser afterwards.
pub async fn delete_account() -> Result<(), AuthError> {
    rpc("delete_my_account", &json!({})).await.map(|_| ())
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
