//! One row of profile pictures, built like the title rows of the browse page: the same endless paging (a copy of the last pictures in front and of
//! the first behind), the same edge buttons that appear when the row is hovered, and the same page dashes over the row.
use crate::components::media::row_scroll::{page_position, page_target, turn_page, Edge, EdgeButton};
use leptos::prelude::*;

/// A picture a person can pick.
#[derive(Clone, PartialEq)]
pub struct IconChoice {
    pub name: String,
    pub url: String,
}

/// How many pictures at each end are copied so the row can page round (none for a short row, which just ends).
fn copies(len: usize) -> usize {
    if len > 7 { len.min(12) } else { 0 }
}

#[component]
pub fn IconRow(
    #[prop(into)] title: String,
    items: Vec<IconChoice>,
    /// The address of the picture the profile has now, marked with a ring.
    #[prop(into)] current: Signal<Option<String>>,
    on_pick: Callback<String>,
) -> impl IntoView {
    let strip = NodeRef::<leptos::html::Div>::new();
    let (can_left, set_can_left) = signal(false);
    let (can_right, set_can_right) = signal(true);
    let (looping, set_looping) = signal(false);
    let (position, set_position) = signal(None::<(usize, usize)>);
    let update = Callback::new(move |_: ()| {
        if let Some(el) = strip.get() {
            set_can_left.set(el.scroll_left() > 10);
            set_can_right.set(el.scroll_width() > el.client_width() + 10 && el.scroll_width() - (el.scroll_left() + el.client_width()) > 10);
            set_position.set(page_position(&el, 0.0));
        }
    });
    let page = Callback::new(move |forward: bool| {
        if let Some(el) = strip.get() {
            set_looping.set(true);
            if !turn_page(&el, forward, 0.0) {
                let (client, cur, total) = (el.client_width() as f64, el.scroll_left() as f64, el.scroll_width() as f64);
                el.scroll_to_with_x_and_y(page_target(forward, cur, client, total, (client * 0.9).max(100.0)), 0.0);
            }
            leptos::leptos_dom::helpers::set_timeout(move || update.run(()), std::time::Duration::from_millis(600));
        }
    });
    // the pictures, with a copy of the last ones in front and of the first ones behind
    let tile = move |item: IconChoice, role: u8| {
        let url = item.url.clone();
        let pick_url = item.url.clone();
        let mark = Signal::derive({ let url = url.clone(); move || current.get().as_deref() == Some(url.as_str()) });
        view! {
            <button
                type="button"
                aria-label=item.name.clone()
                aria-pressed=move || mark.get().to_string()
                on:click=move |_| on_pick.run(pick_url.clone())
                data-loop-first=(role == 1).then_some("1")
                data-loop-clone=(role == 2).then_some("1")
                class=format!("relative block h-[128px] w-[128px] flex-none cursor-pointer overflow-hidden rounded-control outline-none transition-[outline-color] focus-visible:outline-[3px] focus-visible:outline-ink hover:outline-[3px] hover:outline-ink outline outline-[3px] outline-transparent{}", if role >= 2 { " max-md:hidden" } else { "" })
                class=("!outline-ink", move || mark.get())
            >
                <img src=url alt="" loading="lazy" class="h-full w-full object-cover" />
            </button>
        }
    };
    let n = items.len();
    let k = copies(n);
    let lead: Vec<_> = items[n - k..].iter().cloned().map(|it| tile(it, 3)).collect();
    let behind: Vec<IconChoice> = items[..k].to_vec();
    let mut cards: Vec<_> = items.into_iter().enumerate().map(|(i, it)| tile(it, if i == 0 { 1 } else { 0 })).collect();
    cards.extend(behind.into_iter().enumerate().map(|(i, it)| tile(it, if i == 0 { 2 } else { 3 })));

    let aria = title.clone();
    view! {
        <section class="group relative my-6" aria-label=aria>
            <div class="flex items-center justify-between px-[var(--app-x)]">
                <h2 class="text-[24px] font-bold leading-[31px] text-ink">{title}</h2>
                {move || position.get().map(|(index, pages)| view! {
                    <div class="hidden items-center gap-[3px] opacity-0 transition-opacity duration-(--motion-slow) group-hover:opacity-100 md:flex" aria-hidden="true">
                        {(0..pages).map(|i| view! { <div class=if i == index { "h-[2px] w-3 rounded-full bg-ink" } else { "h-[2px] w-3 rounded-full bg-ink/20" } /> }).collect_view()}
                    </div>
                })}
            </div>
            <div class="group/row relative mt-3">
                <div node_ref=strip on:scroll=move |_| update.run(()) class="flex gap-[16px] overflow-x-auto py-[3px] ps-[calc(var(--app-x)-16px)] scrollbar-hide" style="scroll-behavior: smooth;">
                    <div data-loop-lead="1" class="flex flex-none gap-[16px]" style="display: none;">{lead}</div>
                    <div data-loop-spacer="1" class="w-0 flex-none"></div>
                    {cards}
                    <div class="w-[var(--app-x)] flex-none"></div>
                </div>
                <EdgeButton edge=Edge::Left visible=Signal::derive(move || can_left.get() || looping.get()) on_click=Callback::new(move |_| page.run(false)) label=crate::i18n::t("vibe-row-scroll-left") inset="-top-1 -bottom-1" />
                <EdgeButton edge=Edge::Right visible=can_right on_click=Callback::new(move |_| page.run(true)) label=crate::i18n::t("vibe-row-scroll-right") inset="-top-1 -bottom-1" />
            </div>
        </section>
    }
}
