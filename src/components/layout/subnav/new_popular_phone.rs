use leptos::prelude::*;
use crate::components::media::chip_icons::{Controller, Flame, Popcorn, Top10};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewPopularTab {
    Watching,
    JustLanded,
    Games,
    Top10Movies,
    Top10Series,
    ComingSoon,
}

fn pill_id(tab: NewPopularTab) -> &'static str {
    match tab {
        NewPopularTab::ComingSoon => "coming-soon",
        NewPopularTab::Watching => "watching",
        NewPopularTab::JustLanded => "just-landed",
        NewPopularTab::Games => "games",
        NewPopularTab::Top10Series => "top10-series",
        NewPopularTab::Top10Movies => "top10-films",
    }
}

#[derive(Clone, Copy)]
enum Glyph {
    Popcorn,
    Flame,
    Games,
    Top10,
}

#[component]
pub fn NewPopularSubNavMobile(
    active_tab: ReadSignal<NewPopularTab>,
    on_tab_change: Callback<NewPopularTab>,
) -> impl IntoView {
    let features = LocalResource::new(|| async { crate::services::ai_engine::fetch_features().await });
    let games_on = move || features.get().is_some_and(|f| f.get("games") == Some(&true));
    // The pills of the app: Coming Soon, Everyone's Watching, Games, Top 10 Series, Top 10 Films, each with its own glyph
    let tabs = vec![
        (Some(NewPopularTab::ComingSoon), "tabs-coming-soon", Glyph::Popcorn),
        (Some(NewPopularTab::Watching), "tabs-everyone-watching", Glyph::Flame),
        (Some(NewPopularTab::Games), "nav-games", Glyph::Games),
        (Some(NewPopularTab::Top10Series), "tabs-top10-series", Glyph::Top10),
        (Some(NewPopularTab::Top10Movies), "tabs-top10-films", Glyph::Top10),
    ];

    // the row scrolls along with the page: the pill in use is brought to the middle of the row, so it is never out of sight
    Effect::new(move |_| {
        let tab = active_tab.get();
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
        let (Some(row), Some(pill)) = (doc.get_element_by_id("nh-pills"), doc.get_element_by_id(&format!("nh-pill-{}", pill_id(tab)))) else { return };
        let pill: web_sys::HtmlElement = wasm_bindgen::JsCast::unchecked_into(pill);
        let target = pill.offset_left() as f64 - (row.client_width() as f64 - pill.offset_width() as f64) / 2.0;
        let opts = web_sys::ScrollToOptions::new();
        opts.set_left(target.max(0.0));
        opts.set_behavior(web_sys::ScrollBehavior::Smooth);
        row.scroll_to_with_scroll_to_options(&opts);
    });

    view! {
        <div class="relative w-full overflow-hidden select-none">
            <div id="nh-pills" class="pt-2 pb-3 px-3 flex items-center justify-start overflow-x-auto scrollbar-hide max-w-full">
                <div class="flex items-center gap-2 shrink-0">
                    {tabs.into_iter().map(|(tab, label, glyph)| {
                        let is_active = move || tab.is_some_and(|t| active_tab.get() == t);
                        let on_change = on_tab_change;
                        let is_games = tab == Some(NewPopularTab::Games);
                        view! {
                            <button
                                id=tab.map(|t| format!("nh-pill-{}", pill_id(t)))
                                class:hidden=move || is_games && !games_on()
                                on:click=move |_| if let Some(t) = tab { on_change.run(t) }
                                class=move || {
                                    let look = if is_active() {
                                        "bg-chrome-ink text-chrome border-chrome-ink"
                                    } else {
                                        "bg-transparent text-ink border-ink/35"
                                    };
                                    format!("flex items-center justify-center gap-2 h-[33px] ps-2.5 pe-3.5 rounded-button border-[1.5px] text-pill font-semibold whitespace-nowrap active:scale-95 transition-all leading-none shrink-0 cursor-pointer {look}")
                                }
                            >
                                {match glyph {
                                    Glyph::Popcorn => view! { <Popcorn size=22 /> }.into_any(),
                                    Glyph::Flame => view! { <Flame size=22 /> }.into_any(),
                                    Glyph::Games => view! { <Controller size=22 /> }.into_any(),
                                    Glyph::Top10 => view! { <Top10 size=20 /> }.into_any(),
                                }}
                                {crate::i18n::t(label)}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}
