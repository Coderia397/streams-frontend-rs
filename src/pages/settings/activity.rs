//! Viewing activity: what was watched, newest first, with the controls the Netflix page has: Report a problem, hide a title,
//! Show more, Hide all and Download all (a CSV file). Watch records are kept in this browser for now (store::watch_store);
//! later they come from public.watch_progress, where `hidden` removes a row from this list.
use super::widgets::{local_date, save_as_file, toast, ConfirmDialog, OutlineButton, Section};
use crate::config::{routes, VIEWING_PAGE};
use crate::i18n::{t, t_args};
use crate::store::{use_profile_store, use_watch_store, WatchRecord};
use leptos::prelude::*;

/// One CSV field: quoted when it holds a comma, a quote or a line break.
fn csv_field(text: &str) -> String {
    if text.contains([',', '"', '\n']) { format!("\"{}\"", text.replace('"', "\"\"")) } else { text.to_string() }
}

/// The history as CSV text: title, season, episode, date. The header is translated.
fn to_csv(rows: &[WatchRecord]) -> String {
    let mut out = format!("{},{},{},{}\n", t("set-csv-title"), t("set-csv-season"), t("set-csv-episode"), t("set-csv-date"));
    for r in rows {
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(r.updated_at as f64)).to_iso_string().as_string().unwrap_or_default();
        out.push_str(&format!("{},{},{},{}\n", csv_field(&r.title), r.season.map(|s| s.to_string()).unwrap_or_default(), r.episode.map(|e| e.to_string()).unwrap_or_default(), date));
    }
    out
}

fn title_path(r: &WatchRecord) -> String {
    format!("{}/{}/{}", routes::WATCH, if r.is_tv { "tv" } else { "movie" }, r.media_id)
}

#[component]
pub fn Activity() -> impl IntoView {
    let watch = use_watch_store();
    let profiles = use_profile_store();
    let shown = RwSignal::new(VIEWING_PAGE);
    let confirm = RwSignal::new(false);
    let name = Memo::new(move |_| profiles.profiles.with(|l| profiles.active_profile_id.with(|id| l.iter().find(|p| Some(&p.id) == id.as_ref()).or(l.first()).map(|p| p.name.clone()).unwrap_or_default())));
    let rows = Memo::new(move |_| {
        let mut list: Vec<WatchRecord> = watch.records.with(|m| m.values().cloned().collect());
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        list
    });
    let download = Callback::new(move |_| {
        save_as_file(&crate::data::settings::export_file_name_for("viewing-activity", "csv"), &to_csv(&rows.get_untracked()), "text/csv");
    });
    view! {
        <div>
            <p class="mb-5 text-body-md text-ink/60">{move || t_args("set-activity-for", &[("name", name.get().into())])}</p>
            {move || {
                let list = rows.get();
                if list.is_empty() {
                    return view! { <p class="rounded-card border border-set-edge bg-set-card p-6 text-center text-body-md text-ink/60">{t("settings-no-history")}</p> }.into_any();
                }
                let total = list.len();
                let visible: Vec<WatchRecord> = list.into_iter().take(shown.get()).collect();
                view! {
                    <Section>
                        {visible.into_iter().map(|r| {
                            let (id, tv) = (r.media_id, r.is_tv);
                            let episode = match (r.season, r.episode) {
                                (Some(s), Some(e)) => Some(t_args("set-activity-episode", &[("season", s.into()), ("episode", e.into())])),
                                _ => None,
                            };
                            let title = match episode {
                                Some(e) => t_args("set-activity-title", &[("title", r.title.clone().into()), ("episode", e.into())]),
                                None => r.title.clone(),
                            };
                            view! {
                                <div class="set-row flex min-h-settings-row w-full items-center gap-4 border-b border-set-line px-1 py-3 last:border-b-0">
                                    <span class="w-24 shrink-0 text-body-sm text-ink/70">{local_date(r.updated_at as f64)}</span>
                                    <span class="min-w-0 flex-1">
                                        <a href=title_path(&r) class="block truncate text-body-md font-semibold text-ink hover:underline">{title}</a>
                                        <a href=routes::HELP_CONTACT class="text-body-sm text-ink/70 underline underline-offset-2 hover:text-ink">{t("set-activity-report")}</a>
                                    </span>
                                    <button type="button" aria-label=t("set-activity-hide") on:click=move |_| { watch.remove_record(id, tv); toast(t("set-activity-hidden")); } class="btn-circle btn-circle-clear btn-circle-md text-ink/70">
                                        <i class="ph-bold ph-prohibit text-xl" aria-hidden="true"></i>
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    </Section>
                    {(total > shown.get()).then(|| view! {
                        <button type="button" on:click=move |_| shown.update(|n| *n += VIEWING_PAGE) class="mb-8 w-full rounded-button bg-ink/10 px-6 py-3 text-body-md font-bold hover:bg-ink/15">{t("set-show-more")}</button>
                    })}
                    <div class="flex flex-wrap gap-3">
                        <button type="button" on:click=move |_| download.run(()) class="rounded-button border border-ink/40 px-6 py-3 text-body-md font-bold hover:bg-ink/10">{t("set-activity-download")}</button>
                        <OutlineButton label=t("set-activity-hide-all") danger=true on_click=Callback::new(move |_| confirm.set(true)) />
                    </div>
                }
                .into_any()
            }}
            <ConfirmDialog open=confirm title=t("set-activity-hide-all") body=t("set-activity-clear-body") confirm_label=t("set-delete") danger=true on_confirm=Callback::new(move |_| { watch.clear_all(); toast(t("set-activity-hidden")); }) />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::csv_field;

    #[test]
    fn csv_fields_are_quoted_when_needed() {
        assert_eq!(csv_field("Plain"), "Plain");
        assert_eq!(csv_field("A, B"), "\"A, B\"");
        assert_eq!(csv_field("Say \"hi\""), "\"Say \"\"hi\"\"\"");
    }
}
