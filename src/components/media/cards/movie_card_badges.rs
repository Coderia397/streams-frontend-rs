use crate::data::ratings::{age_label, colors_for, AgeLabel, RatingSystem, Say};
use crate::pages::settings::prefs::use_prefs;
use leptos::prelude::*;

/// A badge for an age label: one look for both rating systems, a ticket (a solid colour by rank with a half round notch cut from each short
/// side, so it reads on any poster and cannot be mistaken for an official classification symbol). `size` is "xs", "sm", "md" or "lg"; the badge is drawn in em, so it also follows the text around it.
#[component]
pub fn AgeBadge(label: AgeLabel, #[prop(default = RatingSystem::Uk)] system: RatingSystem, #[prop(default = "sm".to_string())] size: String) -> impl IntoView {
    let (bg, fg) = colors_for(system, &label);
    let px = match size.as_str() {
        "xs" => 11,
        "md" => 16,
        "lg" => 20,
        _ => 13,
    };
    let (shown, aria) = match &label.say {
        Say::Rated(text) => (text.clone(), crate::i18n::t_args("media-rating-aria", &[("rating", fluent_bundle::FluentValue::from(text.clone()))])),
        Say::All => (crate::i18n::t("age-badge-all"), crate::i18n::t("age-aria-all")),
        Say::Plus(n) => (format!("{n}+"), crate::i18n::t_args("age-aria-plus", &[("age", fluent_bundle::FluentValue::from(*n as i64))])),
        Say::NotRated => ("NR".to_string(), crate::i18n::t("media-rating-nr")),
    };
    view! {
        <span class="age-badge-wrap" role="img" aria-label=aria style=format!("font-size:{px}px")>
            <span class="age-badge" aria-hidden="true" style=format!("--age-bg:{bg};--age-fg:{fg}")>{shown}</span>
        </span>
    }
}

/// The badge for a title: shows the certificates the caller already has, otherwise asks the engine (NR until it answers or when the title
/// has none). The label follows the rating system the person reads (see `Prefs::rating_system`).
#[component]
pub fn RatingBadge(
    id: u32,
    #[prop(default = false)] is_tv: bool,
    #[prop(default = "sm".to_string())] size: String,
    #[prop(default = None)] gb: Option<String>,
    #[prop(default = None)] us: Option<String>,
) -> impl IntoView {
    let prefs = use_prefs();
    // what is already known is drawn at once (the caller's certificates, or an earlier answer): the badge never appears as "NR" first and then changes, which showed as a blink on hover
    let known = gb.is_some() || us.is_some();
    let initial = if known {
        let info = crate::services::rating::RatingInfo { gb: gb.clone(), us: us.clone() };
        crate::services::rating::remember(id, is_tv, &info);
        Some(info)
    } else {
        crate::services::rating::peek(id, is_tv)
    };
    let info = RwSignal::new(initial);
    if info.get_untracked().is_none() {
        leptos::task::spawn_local(async move { info.set(Some(crate::services::rating::fetch(id, is_tv).await)); });
    }
    view! {
        {move || {
            let system = prefs.rating_system();
            match info.get() {
                Some(i) => {
                    let label = age_label(system, i.gb.as_deref(), i.us.as_deref());
                    view! { <AgeBadge label=label system=system size=size.clone() /> }.into_any()
                }
                // until the answer comes the place of the badge is held empty (no "NR" that then changes)
                None => view! { <span class="inline-block invisible"><AgeBadge label=age_label(system, None, None) system=system size=size.clone() /></span> }.into_any(),
            }
        }}
    }
}
