use super::shell::{set_title, HelpShell};
use super::toc::Toc;
use super::{art, mailto, markup, recent, tr, tr_with};
use crate::data::help::{self, Article};
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use std::collections::HashMap;

fn read_votes() -> HashMap<String, bool> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(crate::keys::HELP_VOTES).ok().flatten())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_vote(slug: &str, helpful: bool) {
    let mut votes = read_votes();
    votes.insert(slug.to_string(), helpful);
    if let (Some(s), Ok(raw)) = (web_sys::window().and_then(|w| w.local_storage().ok().flatten()), serde_json::to_string(&votes)) {
        let _ = s.set_item(crate::keys::HELP_VOTES, &raw);
    }
}

#[component]
fn Feedback(slug: &'static str) -> impl IntoView {
    let (vote, set_vote) = signal(read_votes().get(slug).copied());
    let cast = move |helpful: bool| {
        save_vote(slug, helpful);
        set_vote.set(Some(helpful));
    };
    let missing = move || mailto(&tr_with("help-ui-mail-feedback", &[("title", FluentValue::from(art(slug, "title")))]), &tr("help-ui-tell-more"));
    view! {
        <div class="mt-12 border-t border-paper-line pt-5 text-body-sm" role="group" aria-label=tr("help-ui-helpful")>
            {move || match vote.get() {
                None => view! {
                    <div class="flex items-center gap-4">
                        <span class="font-bold text-paper-ink">{tr("help-ui-helpful")}</span>
                        <button on:click=move |_| cast(true) class="text-paper-ink underline underline-offset-2 hover:text-brand-active">{tr("help-ui-yes")}</button>
                        <button on:click=move |_| cast(false) class="text-paper-ink underline underline-offset-2 hover:text-brand-active">{tr("help-ui-no")}</button>
                    </div>
                }
                .into_any(),
                Some(helpful) => view! {
                    <p role="status" class="text-paper-ink">
                        <span class="font-bold">{tr("help-ui-thanks")}</span>
                        {(!helpful).then(|| view! { " " <a href=missing() class="underline underline-offset-2 hover:text-brand-active">{tr("help-ui-tell-more")}</a> })}
                    </p>
                }
                .into_any(),
            }}
        </div>
    }
}

fn related(a: &'static Article) -> impl IntoView {
    view! {
        <aside class="border-t-4 border-brand bg-paper px-5 pb-5 pt-4 shadow-raise/10 ring-1 ring-paper-line" aria-labelledby="related-title">
            <h2 id="related-title" class="mb-4 text-body-md font-bold text-paper-ink">{tr("help-ui-related")}</h2>
            <ul class="space-y-3">
                {a.related
                    .iter()
                    .map(|slug| view! {
                        <li class="flex gap-3">
                            <i class="ph-bold ph-article mt-0.5 text-body-md text-paper-ink" aria-hidden="true"></i>
                            <a href=format!("/help/article/{slug}") class="text-body-sm text-paper-ink underline underline-offset-2 hover:text-brand-active">{art(slug, "title")}</a>
                        </li>
                    })
                    .collect_view()}
            </ul>
        </aside>
    }
}

fn article_view(a: &'static Article) -> impl IntoView {
    let title = art(a.slug, "title");
    let parsed = markup::parse(&art(a.slug, "body"));
    let toc = markup::headings(&parsed);
    let body = markup::blocks(parsed);
    set_title(&title);
    super::shell::set_description(&art(a.slug, "summary"));
    recent::remember_viewed(a.slug);
    view! {
        <div class="grid gap-10 lg:grid-cols-[minmax(0,1fr)_320px]">
            <article class="min-w-0 max-w-[680px]">
                <h1 class="mb-6 break-words hyphens-auto text-heading-xl font-extrabold leading-tight text-paper-ink md:text-display-sm">{title}</h1>
                {body}
                <Feedback slug=a.slug />
            </article>
            <div class="space-y-8 lg:sticky lg:top-6 lg:self-start lg:pt-2">
                {(toc.len() >= 2).then(|| view! { <div class="hidden lg:block"><Toc items=toc title=tr("help-ui-on-this-page") /></div> })}
                {related(a)}
            </div>
        </div>
    }
}

#[component]
pub fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let slug = Memo::new(move |_| params.read().get("slug").unwrap_or_default());
    view! {
        <HelpShell>
            <main class="mx-auto w-full max-w-[1100px] px-6 pb-20 pt-6 md:px-12">
                <a href="/help" class="mb-6 inline-flex items-center gap-2 text-body-sm font-semibold text-paper-ink hover:text-brand-active">
                    <i class="ph-bold ph-arrow-left rtl:-scale-x-100" aria-hidden="true"></i>
                    {tr("help-ui-back")}
                </a>
                {move || match help::article(&slug.get()) {
                    Some(a) => article_view(a).into_any(),
                    None => {
                        set_title(&tr("help-ui-not-found-title"));
                        view! {
                            <div class="max-w-[680px]">
                                <h1 class="mb-4 text-heading-xl font-extrabold text-paper-ink">{tr("help-ui-not-found-title")}</h1>
                                <p class="mb-6 text-body-md leading-7 text-paper-mute">{tr("help-ui-not-found-body")}</p>
                                <a href="/help" class="inline-block rounded-button bg-void px-5 py-3 text-body-sm font-bold text-ink hover:bg-brand-deep">{tr("help-ui-back")}</a>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </main>
        </HelpShell>
    }
}
