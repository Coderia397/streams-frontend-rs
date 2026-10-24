//! colour: art. A developer page that previews badges on light and dark.
//! Developer preview of the age badges (route /dev/badges, debug builds only): both rating systems, every label, every size, on light, dark and
//! picture like backgrounds. Used to look at the badges, not part of the product.
use crate::components::media::movie_card_badges::AgeBadge;
use crate::data::ratings::{age_label, RatingSystem};
use leptos::prelude::*;

const SAMPLES: &[(&str, Option<&str>, Option<&str>)] = &[
    ("U / G", Some("U"), Some("G")),
    ("PG", Some("PG"), Some("PG")),
    ("12A / PG-13", Some("12A"), Some("PG-13")),
    ("15 / R", Some("15"), Some("R")),
    ("18 / NC-17", Some("18"), Some("NC-17")),
    ("TV-Y7", None, Some("TV-Y7")),
    ("TV-14", None, Some("TV-14")),
    ("TV-MA", None, Some("TV-MA")),
    ("only British 12", Some("12"), None),
    ("R18", Some("R18"), None),
    ("only American R", None, Some("R")),
    ("none", None, None),
];

const BACKGROUNDS: &[(&str, &str)] = &[
    ("white", "background:#ffffff"),
    ("black", "background:#000000"),
    ("picture", "background:linear-gradient(120deg,#f4d35e,#ee964b 35%,#3a86ff 65%,#1b1b3a)"),
];

#[component]
pub fn DevBadgesPage() -> impl IntoView {
    if !cfg!(debug_assertions) {
        return view! { <leptos_router::components::Redirect path="/" /> }.into_any();
    }
    view! {
        <div class="p-6 text-ink" style="background:#111">
            {[RatingSystem::Uk, RatingSystem::Us].into_iter().map(|system| view! {
                <h2 class="mt-6 mb-2 text-xl font-bold">{system.key().to_uppercase()}</h2>
                {BACKGROUNDS.iter().map(|(bg_name, bg)| view! {
                    <div class="mb-3 rounded-card p-4" style=*bg>
                        <p class="mb-2 text-sm font-bold" style="mix-blend-mode:difference;color:#fff">{*bg_name}</p>
                        {["xs", "sm", "md", "lg"].into_iter().map(|size| view! {
                            <div class="mb-2 flex flex-wrap items-center gap-3">
                                {SAMPLES.iter().map(|(_, gb, us)| view! { <AgeBadge label=age_label(system, *gb, *us) system=system size=size.to_string() /> }).collect_view()}
                            </div>
                        }).collect_view()}
                    </div>
                }).collect_view()}
            }).collect_view()}
        </div>
    }
    .into_any()
}
