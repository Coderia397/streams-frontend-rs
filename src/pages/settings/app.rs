//! App settings (per device): data use, notifications, about, diagnostics and the legal pages. Everything runs inside the app,
//! the tests read the browser itself.
use super::prefs::use_prefs;
use super::widgets::{ActionRow, ChoiceList, LinkRow, Section, SwitchRow};
use super::diagnostics::{browser_name, platform};
use super::notifications::{push_permission, request_push_permission};
use super::widgets::{Message, Tone};
use super::Screen;
use crate::data::settings::{licences, DATA_USAGE, LEGAL_LINKS};
use crate::components::media::icons::app_icons::{AppIcon, Glyph};
use crate::i18n::t;
use leptos::prelude::*;

#[component]
pub fn AppSettings() -> impl IntoView {
    let prefs = use_prefs();
    let consent = crate::consent::use_ui();
    let data_open = RwSignal::new(false);
    let note = RwSignal::new(None::<(Tone, String)>);

    let data_label = |v: &str| DATA_USAGE.iter().find(|(value, _)| *value == v).map(|(_, key)| t(key)).unwrap_or_default();
    let data_value = Signal::derive(move || data_label(&prefs.app.get().data_usage));
    let options = DATA_USAGE.iter().map(|(v, key)| (v.to_string(), t(key))).collect::<Vec<_>>();
    // turning notifications on asks the browser; a refusal leaves the switch off and says why
    let set_notifications = Callback::new(move |on: bool| {
        if !on {
            note.set(None);
            prefs.app.update(|a| a.notifications = false);
            return;
        }
        request_push_permission(Callback::new(move |granted| {
            if granted {
                note.set(None);
                prefs.app.update(|a| a.notifications = true);
            } else {
                note.set(Some((Tone::Error, t(if push_permission() == "unsupported" { "set-push-unsupported" } else { "set-push-denied" }))));
            }
        }));
    });

    view! {
        <div class="set-bands">
            <Section title=t("set-app-playback")>
                <ActionRow icon="cell-signal-full" title=t("set-data-usage") value=data_value chevron=true on_click=Callback::new(move |_| data_open.update(|o| *o = !*o)) />
            </Section>
            <Show when=move || data_open.get()>
                <div class="-mt-4 mb-8">
                    <ChoiceList
                        options=options.clone()
                        selected=Signal::derive(move || prefs.app.get().data_usage)
                        on_pick=Callback::new(move |v: String| prefs.app.update(|a| a.data_usage = v))
                    />
                </div>
            </Show>

            <Section title=t("set-app-notifications")>
                <SwitchRow
                    icon="bell"
                    title=t("set-allow-notifications")
                    hint=t("set-allow-notifications-hint")
                    checked=Signal::derive(move || prefs.app.get().notifications)
                    on_change=set_notifications
                />
            </Section>
            <Message text=note />

            <Section title=t("set-app-about")>
                <div class="set-row px-1 py-3">
                    <p class="text-title-sm font-semibold leading-row">{t("set-app-device")}</p>
                    <p class="mt-1 break-words text-body-sm text-ink/70">{crate::i18n::t_args("set-device-line", &[("platform", platform().into()), ("browser", browser_name().into())])}</p>
                </div>
                <div class="set-row px-1 py-3">
                    <p class="text-title-sm font-semibold leading-row">{t("set-app-version")}</p>
                    <p class="mt-1 text-body-sm text-ink/70">{env!("CARGO_PKG_VERSION")}</p>
                </div>
            </Section>

            <Section title=t("set-app-diagnostics")>
                <LinkRow icon="wifi-high" title=t("set-check-network") hint=t("set-check-network-hint") href=Screen::CheckNetwork.path() />
                <LinkRow icon="film-reel" title=t("set-playback-spec") href=Screen::PlaybackSpec.path() />
                <LinkRow icon="gauge" title=t("set-title-speed-test") href=Screen::SpeedTest.path() />
            </Section>

            <Section title=t("set-app-legal")>
                {LEGAL_LINKS
                    .iter()
                    .map(|(icon, key, href)| {
                        if href.is_empty() {
                            view! { <ActionRow icon=*icon title=t(key) on_click=Callback::new(move |_| consent.open_center()) /> }.into_any()
                        } else {
                            view! { <LinkRow icon=*icon title=t(key) href=*href /> }.into_any()
                        }
                    })
                    .collect_view()}
            </Section>
        </div>
    }
}

/// Every package the app is built from, with its licence, searchable. The list is generated from Cargo and npm
/// (scripts/make_licences.py), so it is the real one.
#[component]
pub fn Licences() -> impl IntoView {
    let all = licences();
    let query = RwSignal::new(String::new());
    let shown = RwSignal::new(crate::config::LICENCES_PAGE);
    let count = all.len();
    let filtered = Memo::new(move |_| {
        let q = query.get().trim().to_lowercase();
        all.iter().filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q) || p.license.to_lowercase().contains(&q)).cloned().collect::<Vec<_>>()
    });
    view! {
        <div>
            <p class="mb-4 text-body-md text-ink/60">{crate::i18n::t_args("set-licences-hint", &[("count", count.into())])}</p>
            <label class="search-field search-field-md mb-5">
                <AppIcon glyph=Glyph::Search class="text-title" />
                <input
                    type="search"
                    placeholder=move || t("set-licences-search")
                    aria-label=move || t("set-licences-search")
                    prop:value=move || query.get()
                    on:input=move |ev| { query.set(event_target_value(&ev)); shown.set(crate::config::LICENCES_PAGE); }
                    class="text-body-md"
                />
            </label>
            <Section>
                {move || filtered.get().into_iter().take(shown.get()).map(|p| view! {
                    <div class="set-row flex min-h-settings-row w-full items-center justify-between gap-4 border-b border-set-line px-1 py-3 last:border-b-0">
                        <span class="min-w-0"><span class="block truncate text-body-md font-semibold">{p.name.clone()}</span><span class="block text-label text-ink/50">{p.version.clone()}</span></span>
                        <span class="shrink-0 text-label text-ink/70">{p.license.clone()}</span>
                    </div>
                }).collect_view()}
            </Section>
            {move || (filtered.with(|f| f.len()) > shown.get()).then(|| view! {
                <button type="button" on:click=move |_| shown.update(|n| *n += crate::config::LICENCES_PAGE) class="mb-6 w-full rounded-button bg-ink/10 px-6 py-3 text-body-md font-bold hover:bg-ink/15">{t("set-show-more")}</button>
            })}
            <p class="text-label text-ink/50">{t("set-licences-tmdb")}</p>
        </div>
    }
}
