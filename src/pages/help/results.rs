use super::search::{self, Filter};
use super::shell::{set_title, HelpShell};
use super::widgets::{highlighted, KindBadge, SearchBox};
use super::{art, encode, tr, tr_with};
use crate::data::help::POPULAR;
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

const PER_PAGE: usize = 6;

/// Keeps the address in step with the box without adding a history entry for every key pressed.
fn replace_address(q: &str) {
    if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
        let url = if q.trim().is_empty() { "/help/search".to_string() } else { format!("/help/search?q={}", encode(q.trim())) };
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

#[component]
pub fn ResultsPage() -> impl IntoView {
    let query = use_query_map();
    let initial = query.with_untracked(|m| m.get("q").unwrap_or_default());
    let (q, set_q) = signal(initial.clone());
    let (page, set_page) = signal(0usize);
    let (filter, set_filter) = signal(Filter::All);
    Effect::new(move |_| set_title(&tr("help-ui-search-label")));
    let on_query = Callback::new(move |value: String| {
        set_q.set(value.clone());
        set_page.set(0);
        replace_address(&value);
    });
    let hits = Memo::new(move |_| search::search(&q.get(), filter.get()));
    let all_hits = Memo::new(move |_| search::search(&q.get(), Filter::All));
    let counts = move || {
        let all = all_hits.get();
        let policies = all.iter().filter(|h| h.target.is_policy()).count();
        (all.len(), all.len() - policies, policies)
    };

    let chip = move |which: Filter, key: &'static str, n: Signal<usize>| {
        view! {
            <button
                on:click=move |_| { set_filter.set(which); set_page.set(0); }
                aria-pressed=move || (filter.get() == which).to_string()
                class="rounded-button border px-4 py-1.5 text-body-sm font-semibold"
                class=("border-paper-ink", move || filter.get() == which)
                class=("bg-void", move || filter.get() == which)
                class=("text-ink", move || filter.get() == which)
                class=("border-paper-line", move || filter.get() != which)
                class=("text-paper-ink", move || filter.get() != which)
            >
                {tr(key)} " " <span class="opacity-70">{move || n.get()}</span>
            </button>
        }
    };

    view! {
        <HelpShell>
            <main class="mx-auto w-full max-w-[720px] px-6 pb-20 pt-8">
                <nav class="mb-5 flex items-center gap-2 text-body-sm text-paper-mute" aria-label=move || crate::i18n::t("help-breadcrumb-label")>
                    <a href="/help" class="underline underline-offset-2 hover:text-brand-active">{tr("help-ui-center")}</a>
                    <i class="ph-bold ph-caret-right text-fine rtl:-scale-x-100" aria-hidden="true"></i>
                    <span>{tr("help-ui-search-label")}</span>
                </nav>
                <SearchBox initial=initial placeholder="help-ui-search-placeholder" on_query=on_query />

                <div class="mt-5 flex flex-wrap gap-2" role="group">
                    {chip(Filter::All, "help-ui-filter-all", Signal::derive(move || counts().0))}
                    {chip(Filter::Articles, "help-ui-filter-articles", Signal::derive(move || counts().1))}
                    {chip(Filter::Policies, "help-ui-filter-policies", Signal::derive(move || counts().2))}
                </div>

                {move || {
                    let words = search::tokens(&q.get());
                    let list = hits.get();
                    if q.get().trim().is_empty() {
                        return view! { <p class="mt-10 text-body-md text-paper-mute">{tr("help-ui-results-empty")}</p> }.into_any();
                    }
                    if list.is_empty() {
                        let fix = search::correction(&q.get());
                        return view! {
                            <div class="mt-10" role="status">
                                <h2 class="text-title font-bold text-paper-ink">{tr_with("help-ui-results-none", &[("query", FluentValue::from(q.get()))])}</h2>
                                {fix.map(|f| {
                                    let label = tr_with("help-ui-did-you-mean", &[("query", FluentValue::from(f.clone()))]);
                                    view! {
                                        <button on:click=move |_| on_query.run(f.clone()) class="mt-3 text-body-md font-semibold text-brand-active underline underline-offset-2 hover:text-brand-deep">{label}</button>
                                    }
                                })}
                                <p class="mt-2 text-body-md text-paper-mute">{tr("help-ui-results-hint")}</p>
                                <h3 class="mb-3 mt-8 text-body-md font-bold text-paper-ink">{tr("help-ui-popular")}</h3>
                                <ul class="space-y-3">
                                    {POPULAR.iter().map(|s| view! { <li><a href=format!("/help/article/{s}") class="text-body-md text-paper-ink underline underline-offset-2 hover:text-brand-active">{art(s, "title")}</a></li> }).collect_view()}
                                </ul>
                            </div>
                        }
                        .into_any();
                    }
                    let best = list[0].clone();
                    let rest: Vec<_> = list[1..].to_vec();
                    let pages = rest.len().div_ceil(PER_PAGE).max(1);
                    let current = page.get().min(pages - 1);
                    let shown: Vec<_> = rest.iter().skip(current * PER_PAGE).take(PER_PAGE).cloned().collect();
                    let count_note = tr_with("help-ui-results-count", &[("count", FluentValue::from(list.len())), ("query", FluentValue::from(q.get()))]);
                    let best_summary = best.target.summary();
                    let best_title = best.target.title();
                    view! {
                        <p class="sr-only" role="status" aria-live="polite">{count_note}</p>
                        {(current == 0).then(|| view! {
                            <section class="mt-8 border-b border-paper-line pb-8">
                                <p class="mb-4 flex items-center gap-3 text-body-sm font-semibold text-paper-ink">{tr("help-ui-recommended")} <KindBadge policy=best.target.is_policy() /></p>
                                <h2 class="mb-3 text-heading-md font-bold text-paper-ink">{best_title}</h2>
                                <p class="mb-6 text-body-md leading-7 text-paper-mute">{highlighted(&best_summary, &words)}</p>
                                <a href=best.target.href() class="inline-block rounded-button bg-void px-5 py-3 text-body-sm font-bold text-ink hover:bg-brand-deep">{tr("help-ui-read")}</a>
                            </section>
                        })}
                        {(!shown.is_empty()).then(|| view! {
                            <section class="mt-8">
                                <h2 class="mb-4 text-body-sm font-semibold text-paper-ink">{tr("help-ui-additional")}</h2>
                                <ul>
                                    {shown.iter().map(|h| {
                                        let summary = h.target.summary();
                                        let title = h.target.title();
                                        view! {
                                            <li class="border-b border-paper-line py-5 first:pt-1">
                                                <div class="flex flex-wrap items-center gap-3">
                                                    <a href=h.target.href() class="text-body-lg font-semibold text-paper-ink underline underline-offset-2 hover:text-brand-active">{title}</a>
                                                    <KindBadge policy=h.target.is_policy() />
                                                </div>
                                                <p class="mt-2 text-body-sm leading-6 text-paper-mute">{highlighted(&summary, &words)}</p>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            </section>
                        })}
                        {(pages > 1).then(|| view! {
                            <nav class="mt-8 flex items-center gap-1" aria-label=move || crate::i18n::t("help-pagination-label")>
                                <button
                                    aria-label=tr("help-ui-page-prev")
                                    disabled=move || page.get() == 0
                                    on:click=move |_| set_page.update(|p| *p = p.saturating_sub(1))
                                    class="flex h-9 w-9 items-center justify-center rounded-button text-paper-ink hover:bg-paper-soft disabled:opacity-30"
                                >
                                    <i class="ph-bold ph-caret-left rtl:-scale-x-100"></i>
                                </button>
                                {(0..pages).map(|n| view! {
                                    <button
                                        aria-label=tr_with("help-ui-page-n", &[("n", FluentValue::from(n + 1))])
                                        aria-current=move || (page.get() == n).then_some("page")
                                        on:click=move |_| set_page.set(n)
                                        class="h-9 min-w-9 rounded-button px-2 text-body-sm text-paper-ink hover:bg-paper-soft"
                                        class=("bg-paper-soft", move || page.get() == n)
                                        class=("font-bold", move || page.get() == n)
                                    >
                                        {n + 1}
                                    </button>
                                }).collect_view()}
                                <button
                                    aria-label=tr("help-ui-page-next")
                                    disabled=move || page.get() + 1 >= pages
                                    on:click=move |_| set_page.update(|p| *p += 1)
                                    class="flex h-9 w-9 items-center justify-center rounded-button text-paper-ink hover:bg-paper-soft disabled:opacity-30"
                                >
                                    <i class="ph-bold ph-caret-right rtl:-scale-x-100"></i>
                                </button>
                            </nav>
                        })}
                    }
                    .into_any()
                }}
            </main>
        </HelpShell>
    }
}
