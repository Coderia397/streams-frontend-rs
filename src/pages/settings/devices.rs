//! Access and devices: the browsers signed in to the account, newest first, with the current one marked. Each card opens to show
//! when it signed in and lets you sign it out. The list comes from the database (`list_my_sessions`); until that is set up the
//! current browser is shown on its own.
use super::account::{error_text, go};
use super::widgets::{local_time, toast, ConfirmDialog, Expandable, Message, OutlineButton, Section, Tone};
use crate::config::{routes, DEVICES_SHOWN, SECURITY_EVENTS_SHOWN};
use crate::data::settings::{BROWSERS, SYSTEMS};
use crate::i18n::{t, t_args};
use crate::services::account::{self, DeviceSession};
use crate::services::device;
use leptos::prelude::*;

/// "Chrome on Mac" from a user agent text, or a general name when nothing is recognised.
pub fn describe_agent(agent: &str) -> String {
    let find = |table: &[(&'static str, &'static str)]| table.iter().find(|(needle, _)| agent.contains(needle)).map(|(_, name)| *name);
    match (find(BROWSERS), find(SYSTEMS)) {
        (Some(browser), Some(system)) => t_args("set-device-name", &[("browser", browser.into()), ("system", system.into())]),
        (Some(browser), None) => browser.to_string(),
        (None, Some(system)) => system.to_string(),
        (None, None) => t("set-device-unknown"),
    }
}

fn current_agent() -> String {
    web_sys::window().and_then(|w| w.navigator().user_agent().ok()).unwrap_or_default()
}

#[component]
pub fn Access() -> impl IntoView {
    let list = RwSignal::new(None::<Vec<DeviceSession>>);
    let failed = RwSignal::new(false);
    let note = RwSignal::new(None::<(Tone, String)>);
    let shown = RwSignal::new(DEVICES_SHOWN);
    let confirm_all = RwSignal::new(false);
    let revoke_target = RwSignal::new(None::<String>);
    let confirm_one = RwSignal::new(false);
    let events = RwSignal::new(Vec::<(String, String)>::new());

    let load = move || {
        leptos::task::spawn_local(async move {
            match account::sessions().await {
                Ok(l) => {
                    failed.set(false);
                    list.set(Some(l));
                }
                Err(_) => failed.set(true),
            }
        });
    };
    load();
    leptos::task::spawn_local(async move {
        if let Ok(l) = account::security_events(SECURITY_EVENTS_SHOWN).await {
            events.set(l);
        }
    });

    let sign_out_all = Callback::new(move |_| {
        leptos::task::spawn_local(async move {
            match account::sign_out_everywhere().await {
                Ok(()) => go(routes::LOGIN),
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
        });
    });
    let sign_out_one = Callback::new(move |_| {
        let Some(id) = revoke_target.get_untracked() else { return };
        leptos::task::spawn_local(async move {
            match account::revoke_session(&id).await {
                Ok(()) => {
                    toast(t("set-device-signed-out"));
                    load();
                }
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
        });
    });

    view! {
        <div>
            <p class="mb-6 text-body-md text-ink/60">{t("set-devices-hint")}</p>
            <Message text=note />
            {move || {
                let sessions = list.get();
                let cards = match sessions {
                    Some(l) => l,
                    // not set up yet, or not reachable: the current browser on its own
                    None if failed.get() => vec![DeviceSession { user_agent: Some(current_agent()), is_current: true, ..Default::default() }],
                    None => return view! { <p class="mb-6 text-body-md text-ink/60">{t("set-check-running")}</p> }.into_any(),
                };
                let total = cards.len();
                view! {
                    <div class="mb-6 space-y-3">
                        {cards.into_iter().take(shown.get()).map(|d| {
                            let agent = d.user_agent.clone().unwrap_or_default();
                            // the name comes from what the browser reported; a session with no report falls back to its user agent text
                            let name = match (d.browser.as_deref(), d.os.as_deref()) {
                                (Some(b), Some(o)) => t_args("set-device-name", &[("browser", match d.browser_major { Some(v) => format!("{b} {v}"), None => b.to_string() }.into()), ("system", o.into())]),
                                _ => describe_agent(&agent),
                            };
                            let id = d.id.clone();
                            let current = d.is_current;
                            let country = d.country.as_deref().map(device::region_name);
                            let last = if d.last_seen.is_empty() { None } else { Some(t_args("set-device-last", &[("time", local_time(&d.last_seen).into())])) };
                            let meta = match (last, country.clone()) {
                                (Some(l), Some(c)) => format!("{l} · {c}"),
                                (Some(l), None) => l,
                                (None, Some(c)) => c,
                                (None, None) => String::new(),
                            };
                            let kind = d.device_type.clone().filter(|k| k != "other");
                            let icon = match kind.as_deref() { Some("mobile") => "device-mobile", Some("tablet") => "device-tablet", _ => "desktop" };
                            let several = d.countries.as_ref().is_some_and(|c| c.len() > 1);
                            let seen_in = d.countries.clone().unwrap_or_default().iter().map(|c| device::region_name(c)).collect::<Vec<_>>().join(", ");
                            let rows: Vec<(String, String)> = [
                                (t("set-device-kind"), kind.as_ref().map(|k| t(&format!("set-device-type-{k}")))),
                                (t("set-device-first-seen"), d.device_first_seen.as_deref().or(Some(d.created_at.as_str())).filter(|s| !s.is_empty()).map(local_time)),
                                (t("set-device-timezone"), d.timezone.clone()),
                                (t("set-device-screen"), d.screen.clone()),
                                (t("set-device-language"), d.language.clone()),
                            ]
                            .into_iter()
                            .filter_map(|(label, value)| value.map(|v| (label, v)))
                            .collect();
                            let rows = StoredValue::new(rows);
                            view! {
                                <Expandable icon=icon title=name meta=meta badge=if current { t("set-devices-current") } else { String::new() }>
                                    <dl class="mb-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
                                        {rows.get_value().into_iter().map(|(label, value)| view! { <dt class="text-ink/60">{label}</dt><dd class="break-words">{value}</dd> }).collect_view()}
                                    </dl>
                                    {several.then(|| view! { <p class="mb-3 rounded-control bg-ink/5 p-3">{t_args("set-device-countries-note", &[("countries", seen_in.clone().into())])}</p> })}
                                    <p class="mb-3 break-words text-ink/50">{agent.clone()}</p>
                                    {(!current && !id.is_empty()).then(|| { let id = id.clone(); view! { <OutlineButton label=t("set-device-sign-out") danger=true on_click=Callback::new(move |_| { revoke_target.set(Some(id.clone())); confirm_one.set(true); }) /> } })}
                                </Expandable>
                            }
                        }).collect_view()}
                    </div>
                    {(total > shown.get()).then(|| view! {
                        <button type="button" on:click=move |_| shown.update(|n| *n += DEVICES_SHOWN) class="mb-6 w-full rounded-button bg-ink/10 px-6 py-3 text-body-md font-bold hover:bg-ink/15">{t("set-show-more")}</button>
                    })}
                }
                .into_any()
            }}
            <p class="mb-6 text-label text-ink/50">{move || if failed.get() { t("set-devices-note") } else { t("set-devices-list-note") }}</p>
            <Show when=move || !events.get().is_empty()>
                <Section title=t("set-events-title")>
                    {move || events.get().into_iter().map(|(ts, event)| view! {
                        <div class="set-row flex min-h-settings-row w-full items-center justify-between gap-4 border-b border-set-line px-1 py-3 last:border-b-0">
                            <span class="text-body-md">{crate::i18n::t_or(&format!("set-event-{event}"), &event)}</span>
                            <span class="shrink-0 text-label text-ink/50">{local_time(&ts)}</span>
                        </div>
                    }).collect_view()}
                </Section>
            </Show>
            <OutlineButton label=t("set-devices-signout-all") danger=true on_click=Callback::new(move |_| confirm_all.set(true)) />
            <ConfirmDialog open=confirm_all title=t("set-devices-signout-all") body=t("set-devices-signout-body") confirm_label=t("set-sign-out") danger=true on_confirm=sign_out_all />
            <ConfirmDialog open=confirm_one title=t("set-device-sign-out") body=t("set-device-sign-out-body") confirm_label=t("set-sign-out") danger=true on_confirm=sign_out_one />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::data::settings::{BROWSERS, SYSTEMS};

    #[test]
    fn common_agents_are_recognised() {
        let find = |agent: &str, table: &[(&'static str, &'static str)]| table.iter().find(|(n, _)| agent.contains(n)).map(|(_, v)| *v);
        let mac_chrome = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36";
        assert_eq!((find(mac_chrome, BROWSERS), find(mac_chrome, SYSTEMS)), (Some("Chrome"), Some("Mac")));
        let edge = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/150.0 Safari/537.36 Edg/150.0";
        assert_eq!((find(edge, BROWSERS), find(edge, SYSTEMS)), (Some("Edge"), Some("Windows")));
        let phone = "Mozilla/5.0 (Linux; Android 16; Pixel 8) AppleWebKit/537.36 Chrome/150.0 Mobile Safari/537.36";
        assert_eq!(find(phone, SYSTEMS), Some("Android"));
    }
}
