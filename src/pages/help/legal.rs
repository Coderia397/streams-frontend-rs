//! The policies (privacy, terms, cookies, DMCA, disclaimer) inside the help center: same header, footer, search and look as the
//! articles, with a table of contents. Structure in data/help.rs, text in i18n/en.ftl.
use super::shell::{set_title, HelpShell};
use super::toc::Toc;
use super::{legal_text, tr};
use crate::data::help::{legal_doc, LEGAL};
use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[component]
pub fn LegalPage(doc: &'static str) -> impl IntoView {
    let d = legal_doc(doc).expect("policy listed in data/help.rs");
    let title = legal_text(&format!("legal-{doc}-title"));
    {
        let t = title.clone();
        Effect::new(move |_| {
            set_title(&t);
            super::shell::set_description(&legal_text(&format!("legal-{doc}-1-body")).chars().take(160).collect::<String>());
        });
    }
    let toc: Vec<(String, String)> = (1..=d.sections).map(|n| (format!("s-{n}"), legal_text(&format!("legal-{doc}-{n}-title")))).collect();
    let sections = (1..=d.sections)
        .map(|n| {
            let mut bullets = Vec::new();
            let mut k = 1;
            while crate::i18n::has(&format!("legal-{doc}-{n}-b{k}")) {
                bullets.push(legal_text(&format!("legal-{doc}-{n}-b{k}")));
                k += 1;
            }
            view! {
                <section id=format!("s-{n}") class="group scroll-mt-6 space-y-4">
                    <h2 class="flex items-center gap-2 text-title font-bold text-paper-ink">
                        {legal_text(&format!("legal-{doc}-{n}-title"))}
                        <a href=format!("#s-{n}") aria-label=tr("help-ui-section-link") class="text-body-md text-paper-edge opacity-0 transition-opacity focus:opacity-100 group-hover:opacity-100">
                            <i class="ph-bold ph-link-simple" aria-hidden="true"></i>
                        </a>
                    </h2>
                    <p class="text-body-md leading-7 text-paper-ink" inner_html=legal_text(&format!("legal-{doc}-{n}-body")) />
                    {(!bullets.is_empty()).then(|| view! {
                        <ul class="list-disc space-y-3 ps-6 text-body-md leading-7 text-paper-ink">
                            {bullets.into_iter().map(|b| view! { <li inner_html=b /> }).collect_view()}
                        </ul>
                    })}
                </section>
            }
        })
        .collect_view();
    let others = LEGAL.iter().filter(|x| x.id != doc).map(|x| view! {
        <li><a href=x.path class="text-body-sm text-paper-ink underline underline-offset-2 hover:text-brand-active">{legal_text(&format!("legal-{}-title", x.id))}</a></li>
    }).collect_view();
    view! {
        <HelpShell>
            <main class="mx-auto w-full max-w-[1100px] px-6 pb-20 pt-6 md:px-12">
                <a href="/help" class="mb-6 inline-flex items-center gap-2 text-body-sm font-semibold text-paper-ink hover:text-brand-active">
                    <AppIcon glyph=Glyph::Back class="rtl:-scale-x-100" />
                    {tr("help-ui-back")}
                </a>
                <div class="grid gap-10 lg:grid-cols-[230px_minmax(0,1fr)]">
                    <aside class="lg:sticky lg:top-6 lg:self-start">
                        <Toc items=toc title=tr("help-ui-on-this-page") />
                    </aside>
                    <article class="min-w-0 max-w-[720px]">
                        <h1 class="break-words hyphens-auto text-heading-xl font-extrabold leading-tight text-paper-ink md:text-display-sm">{title}</h1>
                        <p class="mb-10 mt-3 text-body-md font-semibold text-paper-mute">{legal_text(&format!("legal-{doc}-updated"))}</p>
                        <div class="space-y-10">{sections}</div>
                        <div class="mt-14 border-t border-paper-line pt-6">
                            <h2 class="mb-3 text-body-md font-bold text-paper-ink">{tr("help-ui-other-policies")}</h2>
                            <ul class="space-y-2">{others}</ul>
                            <p class="mt-6 text-body-sm text-paper-mute">
                                {tr("help-ui-policy-questions")}" "
                                <a href="/help/contact" class="text-brand-active underline underline-offset-2 hover:text-brand-deep">{tr("help-ui-contact-us")}</a>
                            </p>
                        </div>
                    </article>
                </div>
            </main>
        </HelpShell>
    }
}
