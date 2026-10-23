//! Diagnostics pages: Check network, Playback specification and Internet speed test. Each reads the browser or the network
//! itself; nothing is guessed. Addresses and sizes come from crate::config, probes from crate::data::settings.
use super::widgets::{Message, Section, Tone};
use crate::config::{routes, ENGINE_URL, RESOLVER_URL, SPEED_TEST_BYTES, SPEED_TEST_URL, TMDB_IMAGE_URL};
use crate::data::settings::{CODEC_PROBES, DRM_INIT_DATA, DRM_KEY_SYSTEM, DRM_LEVELS, DRM_NAME, DRM_PROBE_MIME, RESOLUTION_LABELS};
use crate::i18n::{t, t_args};
use gloo_net::http::Request;
use js_sys::{Function, Reflect};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

pub(super) fn window_value(name: &str) -> Option<JsValue> {
    let w = web_sys::window()?;
    Reflect::get(&w, &JsValue::from_str(name)).ok().filter(|v| !v.is_undefined() && !v.is_null())
}

fn call(target: &JsValue, method: &str, args: &[JsValue]) -> Option<JsValue> {
    let f: Function = Reflect::get(target, &JsValue::from_str(method)).ok()?.dyn_into().ok()?;
    let list = js_sys::Array::new();
    for a in args {
        list.push(a);
    }
    f.apply(target, &list).ok()
}

/// True when this browser can play the codec through Media Source (hardware or software, it does not say which).
fn codec_supported(mime: &str) -> bool {
    window_value("MediaSource").and_then(|ms| call(&ms, "isTypeSupported", &[JsValue::from_str(mime)])).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn media_query(query: &str) -> bool {
    web_sys::window().and_then(|w| w.match_media(query).ok().flatten()).is_some_and(|m| m.matches())
}

pub(super) fn browser_name() -> String {
    web_sys::window().and_then(|w| w.navigator().user_agent().ok()).unwrap_or_default()
}

pub(super) fn platform() -> String {
    web_sys::window().and_then(|w| w.navigator().platform().ok()).unwrap_or_default()
}


fn now_ms() -> f64 {
    web_sys::window().and_then(|w| w.performance()).map(|p| p.now()).unwrap_or_else(js_sys::Date::now)
}

/// One line of the network check: what was tested, whether it worked, and how long it took.
#[derive(Clone)]
struct Outcome {
    label_key: &'static str,
    ok: bool,
    ms: Option<u32>,
}

async fn timed(label_key: &'static str, url: String, no_cors: bool) -> Outcome {
    let start = now_ms();
    let mut req = Request::get(&url);
    if no_cors {
        req = req.mode(web_sys::RequestMode::NoCors);
    }
    let ok = req.send().await.is_ok();
    Outcome { label_key, ok, ms: ok.then(|| (now_ms() - start).round() as u32) }
}

async fn run_checks() -> Vec<Outcome> {
    let online = web_sys::window().map(|w| w.navigator().on_line()).unwrap_or(true);
    let mut out = vec![Outcome { label_key: "set-check-internet", ok: online, ms: None }];
    if online {
        out.push(timed("set-check-service", format!("{RESOLVER_URL}/api/providers/health"), false).await);
        out.push(timed("set-check-images", format!("{TMDB_IMAGE_URL}/w92/"), true).await);
        // the recommendation engine only counts when this build points at one that is not on this machine
        if !ENGINE_URL.contains("127.0.0.1") && !ENGINE_URL.contains("localhost") {
            out.push(timed("set-check-engine", format!("{ENGINE_URL}/api/health"), false).await);
        }
    }
    out
}

#[component]
pub fn CheckNetwork() -> impl IntoView {
    let results = RwSignal::new(None::<Vec<Outcome>>);
    let busy = RwSignal::new(false);
    let start = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        results.set(None);
        leptos::task::spawn_local(async move {
            results.set(Some(run_checks().await));
            busy.set(false);
        });
    };
    view! {
        <div class="max-w-xl">
            <p class="mb-6 text-body-md text-ink/70">{move || t_args("set-check-intro", &[("brand", crate::brand::NAME.into())])}</p>
            <button type="button" on:click=start disabled=move || busy.get() class="btn-ink mb-6 px-8 py-3 disabled:opacity-50">{move || if busy.get() { t("set-check-running") } else { t("set-check-start") }}</button>
            {move || results.get().map(|list| {
                let all_ok = list.iter().all(|o| o.ok);
                view! {
                    <Section>
                        {list.into_iter().map(|o| view! {
                            <div class="set-row flex min-h-settings-row w-full items-center gap-3 border-b border-set-line px-1 py-3 last:border-b-0">
                                <i class=if o.ok { "ph-bold ph-check-circle text-2xl text-success" } else { "ph-bold ph-x-circle text-2xl text-danger" } aria-hidden="true"></i>
                                <span class="flex-1 text-title-sm font-semibold leading-row">{t(o.label_key)}</span>
                                <span class="text-body-sm text-ink/70">{o.ms.map(|m| t_args("set-check-ms", &[("ms", m.into())])).unwrap_or_default()}</span>
                            </div>
                        }).collect_view()}
                    </Section>
                    <Message text=RwSignal::new(Some((if all_ok { Tone::Done } else { Tone::Error }, t(if all_ok { "set-check-all-ok" } else { "set-check-some-failed" }))))/>
                }
            })}
        </div>
    }
}

/// What this browser says about playback.
#[derive(Clone)]
struct Spec {
    widevine: Option<&'static str>,
    codecs: Vec<&'static str>,
    hdr: bool,
    width: u32,
    height: u32,
}

/// Asks the browser for a Widevine session at the highest robustness it will grant.
async fn widevine_level() -> Option<&'static str> {
    let navigator = window_value("navigator")?;
    let make = |robustness: &str| {
        let c = js_sys::Object::new();
        let _ = Reflect::set(&c, &"contentType".into(), &DRM_PROBE_MIME.into());
        let _ = Reflect::set(&c, &"robustness".into(), &robustness.into());
        c
    };
    for (robustness, label) in DRM_LEVELS.iter().copied() {
        let config = js_sys::Object::new();
        let _ = Reflect::set(&config, &"videoCapabilities".into(), &js_sys::Array::of1(&make(robustness).into()));
        let _ = Reflect::set(&config, &"initDataTypes".into(), &js_sys::Array::of1(&DRM_INIT_DATA.into()));
        if let Some(promise) = call(&navigator, "requestMediaKeySystemAccess", &[DRM_KEY_SYSTEM.into(), js_sys::Array::of1(&config).into()]) {
            if let Ok(p) = promise.dyn_into::<js_sys::Promise>() {
                if wasm_bindgen_futures::JsFuture::from(p).await.is_ok() {
                    return Some(label);
                }
            }
        }
    }
    None
}

async fn read_spec() -> Spec {
    let codecs = CODEC_PROBES.iter().filter(|(_, mime)| codec_supported(mime)).map(|(name, _)| *name).collect();
    let (width, height) = web_sys::window()
        .and_then(|w| {
            let s = w.screen().ok()?;
            let ratio = w.device_pixel_ratio();
            Some(((s.width().ok()? as f64 * ratio).round() as u32, (s.height().ok()? as f64 * ratio).round() as u32))
        })
        .unwrap_or_default();
    Spec { widevine: widevine_level().await, codecs, hdr: media_query("(dynamic-range: high)"), width, height }
}

fn resolution_label(height: u32, width: u32) -> String {
    let long = height.max(width);
    RESOLUTION_LABELS.iter().find(|(min, _)| long >= *min).map(|(_, name)| name.to_string()).unwrap_or_else(|| format!("{width} x {height}"))
}

#[component]
fn SpecRow(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="set-row flex w-full flex-col border-b border-set-line px-1 py-3 last:border-b-0">
            <span class="text-title-sm font-semibold leading-row">{label}</span>
            <span class="text-body-md text-ink/70">{value}</span>
        </div>
    }
}

#[component]
pub fn PlaybackSpec() -> impl IntoView {
    let spec = RwSignal::new(None::<Spec>);
    leptos::task::spawn_local(async move { spec.set(Some(read_spec().await)) });
    view! {
        <div class="max-w-xl">
            {move || match spec.get() {
                None => view! { <p class="text-body-md text-ink/60">{t("set-check-running")}</p> }.into_any(),
                Some(s) => {
                    let yes_no = |b: bool| t(if b { "set-spec-yes" } else { "set-spec-no" });
                    view! {
                        <Section title=t("set-spec-drm-title")>
                            <SpecRow label=t("set-spec-drm") value=s.widevine.map(|l| format!("{DRM_NAME} {l}")).unwrap_or_else(|| t("set-spec-none")) />
                        </Section>
                        <Section title=t("set-spec-playback-title")>
                            <SpecRow label=t("set-spec-resolution") value=resolution_label(s.height, s.width) />
                            <SpecRow label=t("set-spec-screen") value=format!("{} x {}", s.width, s.height) />
                            <SpecRow label=t("set-spec-codecs") value=if s.codecs.is_empty() { t("set-spec-none") } else { s.codecs.join(", ") } />
                            <SpecRow label=t("set-spec-hdr") value=yes_no(s.hdr) />
                        </Section>
                        <a href=routes::HELP class="inline-block rounded-button border border-ink/40 px-6 py-3 text-body-md font-bold hover:bg-ink/10">{t("set-spec-help")}</a>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

/// Internet speed test: downloads a known number of bytes from the test address and divides by the time it took.
#[component]
pub fn SpeedTest() -> impl IntoView {
    let mbps = RwSignal::new(None::<f64>);
    let busy = RwSignal::new(false);
    let failed = RwSignal::new(false);
    let start = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        failed.set(false);
        mbps.set(None);
        leptos::task::spawn_local(async move {
            let started = now_ms();
            let outcome = match Request::get(&format!("{SPEED_TEST_URL}?bytes={SPEED_TEST_BYTES}&cache={}", js_sys::Date::now() as u64)).send().await {
                Ok(resp) if resp.ok() => resp.binary().await.ok(),
                _ => None,
            };
            match outcome {
                Some(bytes) => {
                    let seconds = ((now_ms() - started) / 1000.0).max(0.001);
                    mbps.set(Some(bytes.len() as f64 * 8.0 / seconds / 1_000_000.0));
                }
                None => failed.set(true),
            }
            busy.set(false);
        });
    };
    view! {
        <div class="max-w-xl text-center">
            <p class="mb-8 text-body-md text-ink/70">{t("set-speed-intro")}</p>
            <div class="mb-8 flex items-baseline justify-center gap-3" role="status" aria-live="polite">
                <span class="text-display-sm font-bold">{move || mbps.get().map(|v| format!("{v:.1}")).unwrap_or_else(|| "–".to_string())}</span>
                <span class="text-title text-ink/70">{t("set-speed-unit")}</span>
            </div>
            <Show when=move || failed.get()><p class="mb-6 text-body-md text-danger">{t("set-speed-failed")}</p></Show>
            <button type="button" on:click=start disabled=move || busy.get() class="btn-ink px-8 py-3 disabled:opacity-50">{move || if busy.get() { t("set-speed-running") } else if mbps.get().is_some() { t("set-speed-again") } else { t("set-check-start") }}</button>
        </div>
    }
}
