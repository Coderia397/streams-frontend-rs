use super::search::{self, Filter, Hit};
use super::{encode, recent, tr, tr_with};
use fluent_bundle::FluentValue;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

/// Text with the matching search words in bold.
pub fn highlighted(text: &str, words: &[String]) -> impl IntoView + use<> {
    search::highlight(text, words)
        .into_iter()
        .map(|(piece, hit)| if hit { view! { <b class="font-bold text-paper-ink">{piece}</b> }.into_any() } else { view! { {piece} }.into_any() })
        .collect_view()
}

/// Small label on a result: policies are marked so they are not mistaken for help articles.
#[component]
pub fn KindBadge(policy: bool) -> impl IntoView {
    policy.then(|| view! { <span class="rounded-control border border-paper-edge px-1.5 py-0.5 text-micro font-bold uppercase tracking-wide text-paper-mute">{tr("help-ui-kind-policy")}</span> })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// Gradient frame and a magnifier, used in the home hero.
    Hero,
    /// Plain frame and a magnifier, used on the results page.
    Plain,
    /// Plain frame and an arrow button on the right, used on the contact page.
    Arrow,
}

/// What the drop down below the box lists.
#[derive(Clone, PartialEq)]
enum Panel {
    Closed,
    Recent(Vec<String>),
    Results(Vec<Hit>),
}

/// Search field. Typing shows matching articles and policy sections at once; with an empty box it lists recent searches.
/// Enter opens the results page (or the highlighted suggestion). On the results page `on_query` is given, and the page updates
/// as you type instead of leaving.
#[component]
pub fn SearchBox(
    #[prop(into)] initial: String,
    placeholder: &'static str,
    #[prop(default = Variant::Plain)] variant: Variant,
    #[prop(optional)] on_query: Option<Callback<String>>,
) -> impl IntoView {
    let navigate = use_navigate();
    let (value, set_value) = signal(initial);
    let (open, set_open) = signal(false);
    let (active, set_active) = signal(None::<usize>);
    let (recents, set_recents) = signal(recent::searches());
    let root = NodeRef::<leptos::html::Div>::new();

    let panel = Memo::new(move |_| -> Panel {
        let q = value.get();
        if q.trim().is_empty() {
            let list = recents.get();
            if list.is_empty() { Panel::Closed } else { Panel::Recent(list) }
        } else {
            let hits: Vec<Hit> = search::search(&q, Filter::All).into_iter().take(6).collect();
            if hits.is_empty() { Panel::Closed } else { Panel::Results(hits) }
        }
    });
    let count = move || match panel.get() {
        Panel::Closed => 0,
        Panel::Recent(l) => l.len(),
        Panel::Results(l) => l.len(),
    };
    let results_url = move |q: &str| format!("/help/search?q={}", encode(q.trim()));

    let nav_submit = navigate.clone();
    let submit = Callback::new(move |q: String| {
        if q.trim().is_empty() {
            return;
        }
        recent::remember_search(&q);
        set_recents.set(recent::searches());
        set_open.set(false);
        match on_query {
            Some(cb) => cb.run(q),
            None => nav_submit(&results_url(&q), Default::default()),
        }
    });
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        submit.run(value.get_untracked());
    };
    let nav_key = navigate.clone();
    let on_key = move |ev: leptos::ev::KeyboardEvent| {
        let n = count();
        match ev.key().as_str() {
            "ArrowDown" if n > 0 => {
                ev.prevent_default();
                set_open.set(true);
                set_active.update(|a| *a = Some(a.map_or(0, |i| (i + 1) % n)));
            }
            "ArrowUp" if n > 0 => {
                ev.prevent_default();
                set_active.update(|a| *a = Some(a.map_or(n - 1, |i| (i + n - 1) % n)));
            }
            "Escape" => set_open.set(false),
            "Enter" => {
                if let Some(i) = active.get_untracked() {
                    match panel.get_untracked() {
                        Panel::Results(list) => {
                            if let Some(hit) = list.get(i) {
                                ev.prevent_default();
                                set_open.set(false);
                                nav_key(&hit.target.href(), Default::default());
                            }
                        }
                        Panel::Recent(list) => {
                            if let Some(q) = list.get(i) {
                                ev.prevent_default();
                                set_value.set(q.clone());
                                submit.run(q.clone());
                            }
                        }
                        Panel::Closed => {}
                    }
                }
            }
            _ => {}
        }
    };
    let on_focusout = move |ev: leptos::ev::FocusEvent| {
        let inside = ev.related_target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()).zip(root.get_untracked()).is_some_and(|(n, r)| r.contains(Some(&n)));
        if !inside {
            set_open.set(false);
        }
    };

    let frame = match variant {
        Variant::Hero => "rounded-input p-0.5 bg-linear-to-r from-accent via-brand-glow to-brand shadow-raise",
        Variant::Plain => "rounded-input border border-paper-edge focus-within:border-paper-ink",
        Variant::Arrow => "rounded-input border border-paper-line focus-within:border-paper-ink",
    };
    let arrow = variant == Variant::Arrow;
    view! {
        <div node_ref=root class="relative w-full" on:focusout=on_focusout>
            <form role="search" on:submit=on_submit class=frame>
                <div class="flex items-center rounded-field bg-paper">
                    {(!arrow).then(|| view! { <i class="ph-bold ph-magnifying-glass ps-4 text-title text-paper-ink" aria-hidden="true"></i> })}
                    <input
                        type="search"
                        autocomplete="off"
                        enterkeyhint="search"
                        aria-label=move || tr("help-ui-search-label")
                        placeholder=move || tr(placeholder)
                        prop:value=value
                        on:input=move |ev| {
                            let q = event_target_value(&ev);
                            set_value.set(q.clone());
                            set_active.set(None);
                            set_open.set(true);
                            if let Some(cb) = on_query {
                                cb.run(q);
                            }
                        }
                        on:focus=move |_| set_open.set(true)
                        on:keydown=on_key
                        class="h-12 w-full min-w-0 bg-transparent px-3 text-body-md text-paper-ink placeholder:text-paper-mute focus:outline-none"
                    />
                    {arrow.then(|| view! {
                        <button type="submit" aria-label=move || tr("help-ui-search-label") class="flex h-12 w-12 shrink-0 items-center justify-center text-title text-paper-mute hover:text-paper-ink">
                            <i class="ph-bold ph-arrow-right rtl:-scale-x-100" aria-hidden="true"></i>
                        </button>
                    })}
                </div>
            </form>
            {move || {
                let current = panel.get();
                (open.get() && current != Panel::Closed).then(|| {
                    let words = search::tokens(&value.get());
                    let body = match current {
                        Panel::Results(list) => view! {
                            {list
                                .into_iter()
                                .enumerate()
                                .map(|(i, hit)| {
                                    let title = hit.target.title();
                                    view! {
                                        <li role="option" aria-selected=move || (active.get() == Some(i)).to_string()>
                                            <a
                                                href=hit.target.href()
                                                class="flex items-center gap-3 px-4 py-3 text-body-sm text-paper-ink hover:bg-paper-soft"
                                                class=("bg-paper-soft", move || active.get() == Some(i))
                                            >
                                                <i class=if hit.target.is_policy() { "ph-bold ph-scales text-paper-mute" } else { "ph-bold ph-article text-paper-mute" }></i>
                                                <span class="min-w-0 flex-1">{highlighted(&title, &words)}</span>
                                                <KindBadge policy=hit.target.is_policy() />
                                            </a>
                                        </li>
                                    }
                                })
                                .collect_view()}
                            <li>
                                <a href=results_url(&value.get()) class="flex items-center gap-3 border-t border-paper-line px-4 py-3 text-body-sm font-semibold text-paper-ink hover:bg-paper-soft">
                                    <i class="ph-bold ph-magnifying-glass text-paper-mute"></i>
                                    {tr_with("help-ui-results-count-link", &[("query", FluentValue::from(value.get().trim().to_string()))])}
                                </a>
                            </li>
                        }
                        .into_any(),
                        Panel::Recent(list) => view! {
                            <li class="flex items-center justify-between px-4 pb-1 pt-3 text-fine font-bold text-paper-mute">
                                <span>{tr("help-ui-recent")}</span>
                                <button type="button" on:click=move |_| { recent::clear_searches(); set_recents.set(Vec::new()); } class="underline underline-offset-2 hover:text-paper-ink">{tr("help-ui-clear")}</button>
                            </li>
                            {list
                                .into_iter()
                                .enumerate()
                                .map(|(i, q)| {
                                    let pick = q.clone();
                                    view! {
                                        <li role="option" aria-selected=move || (active.get() == Some(i)).to_string()>
                                            <button
                                                type="button"
                                                on:click=move |_| { set_value.set(pick.clone()); submit.run(pick.clone()); }
                                                class="flex w-full items-center gap-3 px-4 py-2.5 text-start text-body-sm text-paper-ink hover:bg-paper-soft"
                                                class=("bg-paper-soft", move || active.get() == Some(i))
                                            >
                                                <i class="ph-bold ph-clock-counter-clockwise text-paper-mute"></i>
                                                <span>{q}</span>
                                            </button>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        }
                        .into_any(),
                        Panel::Closed => ().into_any(),
                    };
                    view! {
                        <ul
                            role="listbox"
                            aria-label=move || tr("help-ui-suggestions")
                            on:mousedown=|ev| ev.prevent_default()
                            class="absolute inset-x-0 top-full z-(--z-dialog) mt-2 overflow-hidden rounded-menu border border-paper-line bg-paper text-start shadow-popover"
                        >
                            {body}
                        </ul>
                    }
                })
            }}
        </div>
    }
}
