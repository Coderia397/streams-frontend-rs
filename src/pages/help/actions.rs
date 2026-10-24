//! Tools that fix things from the help page itself. An article places one with a line `@@ id` (see markup.rs) and the home page
//! lists them as cards. Each tool acts on this browser only: it reads what is stored here, or tests what this device can reach.
use super::{art, tr, tr_with};
use crate::consent::use_ui;
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// Every tool id an article may use.
pub const TOOLS: &[&str] = &["diagnostics", "data", "profiles", "language", "cookies", "refresh"];

pub fn render(id: &str) -> AnyView {
    match id {
        "diagnostics" => view! { <Diagnostics /> }.into_any(),
        "data" => view! { <DataTools /> }.into_any(),
        "profiles" => view! { <Profiles /> }.into_any(),
        "language" => view! { <LanguageTool /> }.into_any(),
        "cookies" => view! { <CookieTool /> }.into_any(),
        "refresh" => view! { <RefreshTool /> }.into_any(),
        _ => ().into_any(),
    }
}

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn storage() -> Option<web_sys::Storage> {
    window().and_then(|w| w.local_storage().ok().flatten())
}

/// Frame shared by every tool: a bordered card with a short heading.
#[component]
fn Tool(title: String, children: Children) -> impl IntoView {
    view! {
        <section class="mb-6 rounded-card border border-paper-line bg-paper-soft p-5">
            <h3 class="mb-3 text-body-md font-bold text-paper-ink">{title}</h3>
            {children()}
        </section>
    }
}

fn button_class(primary: bool) -> &'static str {
    if primary {
        "rounded-button bg-void px-4 py-2.5 text-body-sm font-bold text-ink hover:bg-brand-deep disabled:opacity-50"
    } else {
        "rounded-button border border-paper-edge bg-paper px-4 py-2.5 text-body-sm font-bold text-paper-ink hover:bg-paper-soft disabled:opacity-50"
    }
}

// ───────────────────────────── device check ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Ok,
    Warn,
    Fail,
}

#[derive(Clone)]
struct Row {
    label: &'static str,
    level: Level,
    detail: String,
    /// Article that explains how to fix a warning or failure.
    fix: Option<&'static str>,
}

fn get(target: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(target, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

fn browser_name(agent: &str) -> String {
    let find = |marker: &str| agent.split(marker).nth(1).map(|v| v.split(|c: char| !(c.is_ascii_digit() || c == '.')).next().unwrap_or("").split('.').next().unwrap_or("").to_string());
    for (marker, name) in [("Edg/", "Edge"), ("OPR/", "Opera"), ("Firefox/", "Firefox"), ("Chrome/", "Chrome"), ("Version/", "Safari")] {
        if let Some(v) = find(marker) {
            return format!("{name} {v}");
        }
    }
    agent.chars().take(40).collect()
}

/// Asks for a page and reports the status and how long it took. Gives up after `timeout_ms`.
async fn probe(url: &str, timeout_ms: i32) -> Result<(u16, f64), String> {
    let controller = web_sys::AbortController::new().map_err(|_| "abort".to_string())?;
    let stop = controller.clone();
    let on_timeout = Closure::once_into_js(move || stop.abort());
    if let Some(w) = window() {
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(on_timeout.unchecked_ref(), timeout_ms);
    }
    let started = js_sys::Date::now();
    let signal = controller.signal();
    match gloo_net::http::Request::get(url).abort_signal(Some(&signal)).send().await {
        Ok(r) => Ok((r.status(), js_sys::Date::now() - started)),
        Err(_) if signal.aborted() => Err("timeout".into()),
        Err(e) => Err(e.to_string()),
    }
}

async fn service_row(label: &'static str, url: String, fix: &'static str) -> Row {
    match probe(&url, 6000).await {
        Ok((status, ms)) if status < 500 => Row { label, level: Level::Ok, detail: tr_with("help-act-v-reachable", &[("ms", FluentValue::from(ms.round() as i64))]), fix: None },
        Ok(_) => Row { label, level: Level::Fail, detail: tr("help-act-v-unreachable"), fix: Some(fix) },
        Err(e) if e == "timeout" => Row { label, level: Level::Fail, detail: tr_with("help-act-v-timeout", &[("s", FluentValue::from(6))]), fix: Some(fix) },
        Err(_) => Row { label, level: Level::Fail, detail: tr("help-act-v-unreachable"), fix: Some(fix) },
    }
}

async fn run_checks() -> Vec<Row> {
    let mut rows = Vec::new();
    let nav = window().map(|w| w.navigator());
    let nav_value: JsValue = nav.clone().map(JsValue::from).unwrap_or(JsValue::UNDEFINED);

    // connection
    let online = nav.as_ref().map(|n| n.on_line()).unwrap_or(true);
    let connection = get(&nav_value, "connection");
    let (kind, mbps) = if connection.is_undefined() || connection.is_null() { (String::new(), None) } else { (get(&connection, "effectiveType").as_string().unwrap_or_default(), get(&connection, "downlink").as_f64()) };
    let slow = matches!(kind.as_str(), "slow-2g" | "2g" | "3g") || mbps.is_some_and(|m| m < 2.5);
    rows.push(Row {
        label: "help-act-d-connection",
        level: if !online { Level::Fail } else if slow { Level::Warn } else { Level::Ok },
        detail: if !online {
            tr("help-act-v-offline")
        } else if let (false, Some(m)) = (kind.is_empty(), mbps) {
            tr_with("help-act-v-speed", &[("type", FluentValue::from(kind.to_uppercase())), ("mbps", FluentValue::from(format!("{m:.1}")))])
        } else {
            tr("help-act-v-online")
        },
        fix: (!online || slow).then_some("buffering"),
    });

    // browser
    let agent = nav.as_ref().and_then(|n| n.user_agent().ok()).unwrap_or_default();
    let old = agent.contains("MSIE") || agent.contains("Trident/");
    rows.push(Row { label: "help-act-d-browser", level: if old { Level::Fail } else { Level::Ok }, detail: browser_name(&agent), fix: old.then_some("supported-browsers") });

    // video
    let has_mse = window().is_some_and(|w| js_sys::Reflect::has(&w, &JsValue::from_str("MediaSource")).unwrap_or(false) || js_sys::Reflect::has(&w, &JsValue::from_str("ManagedMediaSource")).unwrap_or(false));
    let native_hls = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.create_element("video").ok())
        .and_then(|v| v.dyn_into::<web_sys::HtmlMediaElement>().ok())
        .is_some_and(|v| !v.can_play_type("application/vnd.apple.mpegurl").is_empty());
    rows.push(Row {
        label: "help-act-d-video",
        level: if has_mse || native_hls { Level::Ok } else { Level::Fail },
        detail: if has_mse { tr("help-act-v-mse") } else if native_hls { tr("help-act-v-hls-native") } else { tr("help-act-v-unsupported") },
        fix: (!has_mse && !native_hls).then_some("supported-browsers"),
    });

    // storage
    let stored = storage().is_some_and(|s| s.set_item("streams_probe", "1").is_ok() && s.get_item("streams_probe").ok().flatten().is_some() && s.remove_item("streams_probe").is_ok());
    rows.push(Row { label: "help-act-d-storage", level: if stored { Level::Ok } else { Level::Fail }, detail: if stored { tr("help-act-v-supported") } else { tr("help-act-v-blocked") }, fix: (!stored).then_some("saved-data") });

    // full screen
    let fullscreen = window().and_then(|w| w.document()).map(|d| d.fullscreen_enabled()).unwrap_or(false);
    rows.push(Row { label: "help-act-d-fullscreen", level: if fullscreen { Level::Ok } else { Level::Warn }, detail: if fullscreen { tr("help-act-v-supported") } else { tr("help-act-v-unsupported") }, fix: (!fullscreen).then_some("shortcuts") });

    // our services and the catalog, in parallel
    let engine = service_row("help-act-d-engine", format!("{}/api/health", crate::services::ai_engine::engine_base_url()), "rows-wont-load");
    let sources = service_row("help-act-d-sources", format!("{}/api/providers/health", crate::services::resolver::resolver_base_url()), "playback-unavailable");
    let catalog = service_row("help-act-d-catalog", format!("{}/configuration?api_key={}", crate::config::TMDB_API_URL, crate::config::TMDB_API_KEY), "rows-wont-load");
    let (e, s, c) = futures::join!(engine, sources, catalog);
    rows.extend([e, s, c]);
    rows
}

fn level_icon(level: Level) -> (&'static str, &'static str) {
    match level {
        Level::Ok => ("ph-check-circle", "text-success"),
        Level::Warn => ("ph-warning", "text-accent-alt"),
        Level::Fail => ("ph-x-circle", "text-danger"),
    }
}

#[component]
pub fn Diagnostics() -> impl IntoView {
    let (rows, set_rows) = signal(Vec::<Row>::new());
    let (running, set_running) = signal(false);
    let (copied, set_copied) = signal(false);
    let run = move |_| {
        set_running.set(true);
        set_copied.set(false);
        leptos::task::spawn_local(async move {
            set_rows.set(run_checks().await);
            set_running.set(false);
        });
    };
    let report = move || rows.get().iter().map(|r| format!("{}: {}", tr(r.label), r.detail)).collect::<Vec<_>>().join("\n");
    let copy = move |_| {
        if let Some(w) = window() {
            let _ = w.navigator().clipboard().write_text(&report());
            set_copied.set(true);
        }
    };
    let issues = move || rows.get().iter().filter(|r| r.level != Level::Ok).count();
    view! {
        <Tool title=tr("help-card-diagnostics-title")>
            <p class="mb-4 text-body-sm text-paper-mute">{tr("help-card-diagnostics-body")}</p>
            <div class="flex flex-wrap items-center gap-3">
                <button on:click=run disabled=move || running.get() class=button_class(true)>{move || if running.get() { tr("help-act-running") } else { tr("help-act-run") }}</button>
                <Show when=move || !rows.get().is_empty()>
                    <button on:click=copy class=button_class(false)>{move || if copied.get() { tr("help-act-copied") } else { tr("help-act-copy") }}</button>
                    <a href=move || super::mailto(&tr("help-ui-mail-subject"), &report()) class=button_class(false)>{tr("help-act-email")}</a>
                </Show>
            </div>
            <Show when=move || !rows.get().is_empty()>
                <p role="status" class="mt-5 text-body-sm font-bold text-paper-ink">
                    {move || if issues() == 0 { tr("help-act-summary-ok") } else { tr_with("help-act-summary-issues", &[("count", FluentValue::from(issues()))]) }}
                </p>
                <ul class="mt-3 divide-y divide-paper-line">
                    {move || rows
                        .get()
                        .into_iter()
                        .map(|r| {
                            let (icon, tone) = level_icon(r.level);
                            view! {
                                <li class="flex flex-wrap items-center gap-x-3 gap-y-1 py-2.5 text-body-sm">
                                    <i class=format!("ph-bold {icon} text-body-lg {tone}") aria-hidden="true"></i>
                                    <span class="font-semibold text-paper-ink">{tr(r.label)}</span>
                                    <span class="text-paper-mute">{r.detail.clone()}</span>
                                    {r.fix.map(|slug| view! { <a href=format!("/help/article/{slug}") class="ms-auto text-brand-active underline underline-offset-2">{tr("help-act-fix")}": "{art(slug, "title")}</a> })}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </Show>
        </Tool>
    }
}

// ───────────────────────────── saved data ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    None,
    Progress,
    Recs,
    All,
}

#[component]
pub fn DataTools() -> impl IntoView {
    let profiles = crate::store::use_profile_store();
    let watch = crate::store::use_watch_store();
    let (pending, set_pending) = signal(Pending::None);
    let (message, set_message) = signal(String::new());
    let confirm = move |what: Pending| {
        match what {
            Pending::Progress => {
                watch.clear_all();
                set_message.set(tr("help-act-done"));
            }
            Pending::Recs => {
                if let Some(s) = storage() {
                    let _ = s.remove_item(crate::keys::SESSION_ID);
                    let _ = s.remove_item(crate::keys::USER_AFFINITY);
                }
                set_message.set(tr("help-act-done"));
            }
            Pending::All => {
                if let Some(s) = storage() {
                    let _ = s.clear();
                }
                set_message.set(tr("help-act-reloading"));
                if let Some(w) = window() {
                    let _ = w.location().set_href("/help");
                    let _ = w.location().reload();
                }
            }
            Pending::None => {}
        }
        set_pending.set(Pending::None);
    };
    let ask = move |what: Pending| {
        set_message.set(String::new());
        set_pending.set(what);
    };
    let action = move |what: Pending, label: &'static str| {
        view! { <button on:click=move |_| ask(what) class=button_class(false)>{tr(label)}</button> }
    };
    view! {
        <Tool title=tr("help-act-data-title")>
            <ul class="mb-4 space-y-1 text-body-sm text-paper-mute">
                <li>{move || tr_with("help-act-data-profiles", &[("count", FluentValue::from(profiles.profiles.with(|p| p.len())))])}</li>
                <li>{move || tr_with("help-act-data-progress", &[("count", FluentValue::from(watch.records.with(|r| r.len())))])}</li>
                <li>{move || tr_with("help-act-data-lang", &[("lang", FluentValue::from(crate::i18n::native_name(&crate::i18n::language())))])}</li>
            </ul>
            <div class="flex flex-wrap gap-3">
                {action(Pending::Progress, "help-act-clear-progress")}
                {action(Pending::Recs, "help-act-reset-recs")}
                {action(Pending::All, "help-act-clear-all")}
            </div>
            <Show when=move || pending.get() != Pending::None>
                <div role="alertdialog" class="mt-4 flex flex-wrap items-center gap-3 rounded-control border border-paper-edge bg-paper p-3 text-body-sm text-paper-ink">
                    <span class="font-semibold">{tr("help-act-confirm")}</span>
                    <button on:click=move |_| confirm(pending.get_untracked()) class=button_class(true)>{tr("help-act-yes")}</button>
                    <button on:click=move |_| set_pending.set(Pending::None) class=button_class(false)>{tr("help-act-cancel")}</button>
                </div>
            </Show>
            <p role="status" class="mt-3 text-body-sm font-bold text-paper-ink">{move || message.get()}</p>
        </Tool>
    }
}

// ───────────────────────────── profiles ─────────────────────────────

#[component]
pub fn Profiles() -> impl IntoView {
    let store = crate::store::use_profile_store();
    view! {
        <Tool title=tr("help-act-profiles-title")>
            <ul class="mb-4 divide-y divide-paper-line">
                {move || store
                    .profiles
                    .get()
                    .into_iter()
                    .map(|p| view! {
                        <li class="flex flex-wrap items-center gap-3 py-2.5 text-body-sm">
                            <i class="ph-bold ph-user-circle text-title text-paper-mute" aria-hidden="true"></i>
                            <span class="font-semibold text-paper-ink">{p.name.clone()}</span>
                            {p.is_kids.then(|| view! { <span class="rounded-control bg-brand px-2 py-0.5 text-fine font-bold text-ink">{tr("help-act-profile-kids")}</span> })}
                            {p.pin.is_some().then(|| view! { <span class="rounded-control border border-paper-edge px-2 py-0.5 text-fine font-bold text-paper-ink"><AppIcon glyph=Glyph::Locked class="me-1" />{tr("help-act-profile-pin")}</span> })}
                        </li>
                    })
                    .collect_view()}
            </ul>
            <button on:click=move |_| store.active_profile_id.set(None) class=button_class(true)>{tr("help-act-open-profiles")}</button>
        </Tool>
    }
}

// ───────────────────────────── language ─────────────────────────────

#[component]
pub fn LanguageTool() -> impl IntoView {
    let current = crate::i18n::language();
    let browser = window().and_then(|w| w.navigator().language()).unwrap_or_default();
    let on_change = move |ev: leptos::ev::Event| {
        if let Some(select) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) {
            crate::i18n::set_language(&select.value());
        }
    };
    view! {
        <Tool title=tr("help-card-language-title")>
            <p class="mb-3 text-body-sm text-paper-mute">{tr_with("help-act-lang-current", &[("lang", FluentValue::from(crate::i18n::native_name(&current)))])}</p>
            <div class="flex flex-wrap items-center gap-3">
                <select on:change=on_change aria-label=tr("help-card-language-title") class="min-w-48 cursor-pointer border border-paper-edge bg-paper px-3 py-2.5 text-body-sm text-paper-ink">
                    {crate::i18n::AVAILABLE.iter().map(|code| view! { <option value=*code selected=*code == current>{crate::i18n::native_name(code)}</option> }).collect_view()}
                </select>
                <button on:click=move |_| crate::i18n::use_browser_language() class=button_class(false)>{tr("help-act-lang-use-browser")}</button>
            </div>
            <p class="mt-3 text-fine text-paper-mute">{tr_with("help-act-lang-browser", &[("lang", FluentValue::from(browser))])}</p>
        </Tool>
    }
}

// ───────────────────────────── cookies and refresh ─────────────────────────────

#[component]
pub fn CookieTool() -> impl IntoView {
    let ui = use_ui();
    let status = move || {
        ui.version.track();
        match crate::consent::stored() {
            None => tr("help-act-cookies-none"),
            Some(c) if c.personalization => tr("help-act-cookies-on"),
            Some(_) => tr("help-act-cookies-off"),
        }
    };
    view! {
        <Tool title=tr("help-card-cookies-title")>
            <p class="mb-4 text-body-sm text-paper-mute">{status}</p>
            <button on:click=move |_| ui.open_center() class=button_class(true)>{tr("help-act-cookies-open")}</button>
        </Tool>
    }
}

#[component]
pub fn RefreshTool() -> impl IntoView {
    view! {
        <Tool title=tr("help-card-refresh-title")>
            <p class="mb-4 text-body-sm text-paper-mute">{tr("help-act-refresh-body")}</p>
            <div class="flex flex-wrap items-center gap-3">
                <button on:click=move |_| { if let Some(w) = window() { let _ = w.location().reload(); } } class=button_class(true)>{tr("help-act-refresh-now")}</button>
                <span class="text-fine text-paper-mute">{tr_with("help-act-version", &[("version", FluentValue::from(env!("CARGO_PKG_VERSION")))])}</span>
            </div>
        </Tool>
    }
}
