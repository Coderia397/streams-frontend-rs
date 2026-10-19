use super::shell::{set_title, HelpShell, Look};
use super::widgets::{SearchBox, Variant};
use super::actions::{self, TOOLS};
use super::{art, legal_text, recent, tr};
use crate::data::help::{legal_doc, Group, Topic, DOC_PREFIX, POPULAR, QUICK_LINKS, TOPICS};
use leptos::prelude::*;

/// Id of the group that is open. Only one group is open at a time on the whole page, like the Netflix help home.
type OpenGroup = RwSignal<Option<&'static str>>;

#[component]
fn GroupView(group: &'static Group) -> impl IntoView {
    let open = use_context::<OpenGroup>().expect("OpenGroup provided by HelpPage");
    let is_open = move || open.get() == Some(group.id);
    let panel_id = format!("help-group-{}", group.id);
    view! {
        <div class="border-t border-paper-line first:border-t-0">
            <button
                aria-expanded=move || is_open().to_string()
                aria-controls=panel_id.clone()
                on:click=move |_| open.update(|o| *o = if *o == Some(group.id) { None } else { Some(group.id) })
                class="flex w-full cursor-pointer items-center justify-between gap-4 rounded-control border-2 border-transparent py-3.5 text-left text-body-md font-semibold text-paper-ink focus-visible:border-paper-ink"
            >
                <span>{tr(&format!("help-group-{}", group.id))}</span>
                <i class="ph-bold text-body-lg" class=("ph-caret-down", move || !is_open()) class=("ph-caret-up", is_open) aria-hidden="true"></i>
            </button>
            <Show when=is_open>
                <ul id=panel_id.clone() class="space-y-4 pb-5 pt-1">
                    {group
                        .articles
                        .iter()
                        .map(|slug| {
                            // entries starting with doc: are policies, which open their own page
                            let (href, icon, label) = match slug.strip_prefix(DOC_PREFIX).and_then(legal_doc) {
                                Some(d) => (d.path.to_string(), "ph-scales", legal_text(&format!("legal-{}-title", d.id))),
                                None => (format!("/help/article/{slug}"), "ph-article", art(slug, "title")),
                            };
                            view! {
                                <li class="flex gap-3">
                                    <i class=format!("ph-bold {icon} mt-0.5 text-body-md text-paper-ink") aria-hidden="true"></i>
                                    <a href=href class="text-body-md text-paper-ink underline underline-offset-2 hover:text-brand-active">{label}</a>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </Show>
        </div>
    }
}

#[component]
fn TopicCard(topic: &'static Topic) -> impl IntoView {
    view! {
        <section class="rounded-card border border-paper-line bg-paper px-3 pb-1 pt-3 md:px-3.5">
            <h3 class="flex items-center gap-4 border-b border-paper-line px-1 pb-4 pt-1 text-body-lg font-semibold text-paper-ink">
                <i class=format!("ph-bold {} text-heading {}", topic.icon, topic.tone) aria-hidden="true"></i>
                {tr(&format!("help-topic-{}", topic.id))}
            </h3>
            {topic.groups.iter().map(|g| view! { <GroupView group=g /> }).collect_view()}
        </section>
    }
}

/// The articles this browser opened last, so a visitor can get back to them.
#[component]
fn RecentlyViewed() -> impl IntoView {
    let slugs: Vec<String> = recent::viewed().into_iter().filter(|s| crate::data::help::article(s).is_some()).collect();
    (!slugs.is_empty()).then(|| view! {
        <section aria-labelledby="viewed-title">
            <h2 id="viewed-title" class="mb-4 text-title font-semibold text-paper-ink">{tr("help-ui-viewed")}</h2>
            <ul class="space-y-3">
                {slugs.into_iter().map(|slug| view! {
                    <li class="flex gap-3">
                        <i class="ph-bold ph-clock-counter-clockwise mt-0.5 text-body-md text-paper-mute" aria-hidden="true"></i>
                        <a href=format!("/help/article/{slug}") class="text-body-md text-paper-ink underline underline-offset-2 hover:text-brand-active">{art(&slug, "title")}</a>
                    </li>
                }).collect_view()}
            </ul>
        </section>
    })
}

/// Icon for each self-service tool card.
fn tool_icon(id: &str) -> &'static str {
    match id {
        "diagnostics" => "ph-stethoscope",
        "profiles" => "ph-users",
        "data" => "ph-database",
        "language" => "ph-translate",
        "cookies" => "ph-cookie",
        _ => "ph-arrow-clockwise",
    }
}

/// Cards that open a tool in place, so a visitor can fix something on this device without leaving the page.
#[component]
fn ForYou() -> impl IntoView {
    let (open, set_open) = signal(None::<&'static str>);
    view! {
        <section aria-labelledby="foryou-title">
            <h2 id="foryou-title" class="mb-4 text-title font-semibold text-paper-ink">{tr("help-ui-foryou")}</h2>
            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                {TOOLS
                    .iter()
                    .map(|id| {
                        let id: &'static str = id;
                        view! {
                            <button
                                on:click=move |_| set_open.update(|o| *o = if *o == Some(id) { None } else { Some(id) })
                                aria-expanded=move || (open.get() == Some(id)).to_string()
                                class="flex items-start gap-3 rounded-card border border-paper-line bg-paper p-4 text-start hover:bg-paper-soft"
                                class=("border-paper-ink", move || open.get() == Some(id))
                            >
                                <i class=format!("ph-bold {} mt-0.5 text-heading text-brand", tool_icon(id)) aria-hidden="true"></i>
                                <span>
                                    <span class="block text-body-md font-semibold text-paper-ink">{tr(&format!("help-card-{id}-title"))}</span>
                                    <span class="mt-0.5 block text-body-sm text-paper-mute">{tr(&format!("help-card-{id}-body"))}</span>
                                </span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="mt-4">{move || open.get().map(actions::render)}</div>
        </section>
    }
}

#[component]
pub fn QuickLinks() -> impl IntoView {
    view! {
        <section>
            <h2 class="border-b border-paper-line pb-4 text-title font-semibold text-paper-ink">{tr("help-ui-quick-links")}</h2>
            <ul>
                {QUICK_LINKS
                    .iter()
                    .map(|(id, icon, href)| {
                        let label = super::tr_with(&format!("help-ui-ql-{id}"), &[]);
                        // the email link carries the visitor's browser details, so it is built when the page is shown
                        let href = if *href == "mailto" { super::mailto(&tr("help-ui-mail-subject"), &tr("help-ui-mail-intro")) } else { href.to_string() };
                        view! {
                            <li class="border-b border-paper-line">
                                <a href=href class="group/ql flex items-center gap-3 py-3.5 text-body-md font-semibold text-paper-ink hover:text-brand-active">
                                    <i class=format!("ph-bold {icon} text-body-lg text-paper-ink") aria-hidden="true"></i>
                                    <span class="underline underline-offset-2">{label}</span>
                                </a>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}

#[component]
pub fn HelpPage() -> impl IntoView {
    Effect::new(move |_| set_title(""));
    provide_context::<OpenGroup>(RwSignal::new(None));

    // The search box stays pinned to the top once the page is scrolled past it, like the Netflix help home.
    let (stuck, set_stuck) = signal(false);
    let pin = NodeRef::<leptos::html::Div>::new();
    crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
        if let Some(el) = pin.get_untracked() {
            set_stuck.set(el.get_bounding_client_rect().top() <= 0.5);
        }
    });

    // Articles chosen from what this browser holds, then the popular ones to fill the line.
    let profiles = crate::store::use_profile_store();
    let watch = crate::store::use_watch_store();
    let mut picks: Vec<&'static str> = Vec::new();
    if web_sys::window().is_some_and(|w| !w.navigator().on_line()) {
        picks.push("buffering");
    }
    if profiles.profiles.with_untracked(|p| p.iter().any(|p| p.pin.is_some())) {
        picks.push("forgot-pin");
    }
    if profiles.profiles.with_untracked(|p| p.iter().any(|p| p.is_kids)) {
        picks.push("kids-profile");
    }
    if watch.records.with_untracked(|r| !r.is_empty()) {
        picks.push("continue-watching");
    }
    if crate::i18n::language() != "en" {
        picks.push("language");
    }
    let personal = !picks.is_empty();
    for slug in POPULAR {
        if picks.len() >= 5 {
            break;
        }
        if !picks.contains(slug) {
            picks.push(slug);
        }
    }
    let popular = picks
        .iter()
        .enumerate()
        .map(|(i, slug)| {
            view! {
                {(i > 0).then_some(", ")}
                <a href=format!("/help/article/{slug}") class="underline underline-offset-2 hover:text-brand-active">{art(slug, "title")}</a>
            }
        })
        .collect_view();
    let label = if personal { tr("help-ui-recommended-inline") } else { tr("help-ui-popular") };
    view! {
        <HelpShell look=Look::Light>
            <div class="bg-linear-to-b from-paper-tint to-paper bg-size-[100%_calc(100dvh-4rem)] bg-no-repeat">
                <div class="flex h-[calc((100dvh-4rem)/2-1.75rem)] items-end justify-center px-6 pb-6">
                    <h1 class="text-center text-heading-xl font-black text-paper-ink md:text-display-sm">{tr("help-ui-hero-title")}</h1>
                </div>
                <div
                    node_ref=pin
                    class="sticky top-0 z-(--z-nav) px-6 py-3.5 transition-colors duration-(--motion-base)"
                    class=("bg-paper", move || stuck.get())
                    class=("border-b", move || stuck.get())
                    class=("border-paper-line", move || stuck.get())
                >
                    <div class="mx-auto w-full max-w-[670px]">
                        <SearchBox initial="" placeholder="help-ui-search-placeholder" variant=Variant::Hero />
                    </div>
                </div>
                <div class="mx-auto flex h-[calc((100dvh-4rem)/2-1.75rem-1px)] max-w-[670px] flex-col items-center justify-between px-6 pt-1">
                    <p class="text-center text-body-sm leading-6 text-paper-mute">
                        <b class="font-bold text-paper-ink">{label}</b>
                        ": "
                        {popular}
                    </p>
                    <a href="#topics" class="flex flex-col items-center gap-2 pb-8 text-body-sm font-bold text-paper-ink">
                        {tr("help-ui-explore")}
                        <i class="ph-bold ph-arrow-down text-title" aria-hidden="true"></i>
                    </a>
                </div>

                <div id="topics" class="mx-auto flex w-full max-w-[670px] scroll-mt-20 flex-col gap-5 px-6 pb-14 pt-10">
                    <RecentlyViewed />
                    <ForYou />
                    {TOPICS.iter().map(|t| view! { <TopicCard topic=t /> }).collect_view()}
                    <div class="pt-6">
                        <QuickLinks />
                    </div>
                </div>
            </div>
        </HelpShell>
    }
}
