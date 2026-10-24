use leptos::prelude::*;
use crate::components::media::movie_card_rating::{RatingIcon, MovieRating};
use crate::components::media::tooltip_wrapper::TooltipWrapper;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The pill size classes (the .btn-pill presets in style/components.css): sm 32 px, md 40 px, lg 48 px, as Netflix's redesigned pages.
fn pill_size(size: &str) -> &'static str {
    match size {
        "sm" => "btn-pill btn-pill-sm",
        "lg" => "btn-pill btn-pill-lg",
        _ => "btn-pill btn-pill-md",
    }
}

// ── Pill Play Button ──────────────────────────────────────────────────────────
// Used in Hero and InfoModal: Netflix's white Play pill (btn-ink), the icon and the word.
#[component]
pub fn PlayPillButton(
    #[prop(optional, into)] href: Option<Signal<String>>,
    #[prop(optional, into)] on_click: Option<Callback<()>>,
    #[prop(default = Signal::derive(|| false), into)] is_resume: Signal<bool>,
    #[prop(default = "md".to_string(), into)] size: String,
    /// kept for the callers; every Play button now has the same look
    #[prop(default = "pill".to_string(), into)] shape: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let _ = shape;
    let extra_class = class.unwrap_or_default();
    let is_link = href.is_some();
    let base_classes = format!("btn-ink {} {} {}", pill_size(&size), extra_class, if is_link { "no-underline" } else { "" });

    view! {
        {if let Some(url_sig) = href {
            view! {
                <a
                    href=move || url_sig.get()
                    class=base_classes
                    title=move || if is_resume.get() { crate::i18n::t("media-resume") } else { crate::i18n::t("media-play") }
                >
                    <AppIcon glyph=Glyph::Play />
                    <span>{move || if is_resume.get() { crate::i18n::t("media-resume") } else { crate::i18n::t("media-play") }}</span>
                </a>
            }.into_any()
        } else {
            let click_cb = on_click.unwrap_or_else(|| Callback::new(|_| {}));
            view! {
                <button
                    type="button"
                    on:click=move |_| click_cb.run(())
                    class=base_classes
                    title=move || if is_resume.get() { crate::i18n::t("media-resume") } else { crate::i18n::t("media-play") }
                >
                    <AppIcon glyph=Glyph::Play />
                    <span>{move || if is_resume.get() { crate::i18n::t("media-resume") } else { crate::i18n::t("media-play") }}</span>
                </button>
            }.into_any()
        }}
    }
}

// ── Pill More Info Button ─────────────────────────────────────────────────────
// Used in Hero billboard: Netflix's grey see through pill (btn-glass), text only, the same size as the Play button beside it.
#[component]
pub fn MoreInfoPillButton(
    on_click: Callback<()>,
    #[prop(default = crate::i18n::t("action-buttons-more-info"), into)] label: String,
    #[prop(default = "md".to_string(), into)] size: String,
    /// kept for the callers; every More Info button now has the same look
    #[prop(default = "pill".to_string(), into)] shape: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let _ = shape;
    let extra_class = class.unwrap_or_default();
    let label_title = label.clone();
    let base_classes = format!("btn-glass {} {}", pill_size(&size), extra_class);

    view! {
        <button
            type="button"
            on:click=move |_| on_click.run(())
            class=base_classes
            title=label_title
        >
            <span>{label}</span>
        </button>
    }
}

// ── Circular Play Button ──────────────────────────────────────────────────────
// Used in Movie Card hover popups and recommendation overlays.
#[component]
pub fn PlayCircleButton(
    on_click: Callback<()>,
    #[prop(default = Signal::derive(|| false), into)] is_resume: Signal<bool>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (dim_classes, icon_classes, _) = circular_button_specs(&size);

    let extra_class = class.unwrap_or_default();

    view! {
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                on_click.run(());
            }
            class=format!(
                "btn-circle btn-circle-solid shadow-md {} {}",
                dim_classes, extra_class
            )
            title=move || if is_resume.get() { crate::i18n::t("media-resume") } else { crate::i18n::t("media-play") }
        >
            <AppIcon glyph=Glyph::Play class=icon_classes.clone() />
        </button>
    }
}

// ── Shared Circular Button Styling ───────────────────────────────────────────
// Guarantees that MyList, Rating, Mute/Replay, Close, and Caret buttons share 100%
// identical geometry, background, border, hover states, and animations across all surfaces.
/// The round button sizes (the .btn-circle presets in style/components.css): the size class, an icon class (empty: the icon takes the
/// preset's size, half the disc) and the icon size in pixels for icons drawn as SVG.
fn circular_button_specs(size: &str) -> (&'static str, &'static str, u32) {
    match size {
        "xs" | "sm" => ("btn-circle-sm", "", 16),
        "lg" => ("btn-circle-lg", "", 24),
        _ => ("btn-circle-md", "", 20),
    }
}

fn circular_button_classes(dim_classes: &str, extra_class: &str) -> String {
    format!("btn-circle {} {}", dim_classes, extra_class)
}

// ── Round icon button ─────────────────────────────────────────────────────────
// The plain round button of the action rows (mark as watched, remove): an icon and a tooltip text.
#[component]
pub fn CircleIconButton(
    icon: &'static str,
    on_click: Callback<()>,
    #[prop(into)] title: String,
    #[prop(default = "md".to_string(), into)] size: String,
) -> impl IntoView {
    let (dim_classes, icon_classes, _) = circular_button_specs(&size);
    let btn_classes = circular_button_classes(dim_classes, "");
    view! {
        <button type="button" on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); on_click.run(()); } class=btn_classes title=title.clone() aria-label=title>
            <i class=format!("ph-bold {} {}", icon, icon_classes)></i>
        </button>
    }
}

// ── My List Button ────────────────────────────────────────────────────────────
// Used across InfoModal, MovieCardPopup, InfoModalRecommendations.
#[component]
pub fn MyListButton(
    #[prop(into)] is_in_list: Signal<bool>,
    on_toggle: Callback<()>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(default = false)] show_tooltip: bool,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (dim_classes, icon_classes, _) = circular_button_specs(&size);
    let extra_class = class.unwrap_or_default();
    let btn_classes = circular_button_classes(dim_classes, &extra_class);

    let title_text = move || if is_in_list.get() { crate::i18n::t("media-remove-from-list") } else { crate::i18n::t("media-add-to-list") };

    let btn = view! {
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                on_toggle.run(());
            }
            class=btn_classes.clone()
            title=title_text
        >
            {move || if is_in_list.get() {
                view! { <AppIcon glyph=Glyph::Check class=format!("text-success {}", icon_classes) /> }.into_any()
            } else {
                view! { <AppIcon glyph=Glyph::Plus class=icon_classes.clone() /> }.into_any()
            }}
        </button>
    };

    view! {
        {if show_tooltip {
            view! {
                <TooltipWrapper label=crate::i18n::t("action-buttons-add-to-my-list").to_string()>
                    {btn}
                </TooltipWrapper>
            }.into_any()
        } else {
            btn.into_any()
        }}
    }
}

// ── Rating Button ("I like this") ─────────────────────────────────────────────
// Used in InfoModal and MovieCardPopup.
#[component]
pub fn RatingButton(
    #[prop(into)] rating: Signal<Option<MovieRating>>,
    on_rate: Callback<()>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(default = false)] show_tooltip: bool,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (dim_classes, _, icon_pixel_size) = circular_button_specs(&size);
    let extra_class = class.unwrap_or_default();
    let btn_classes = circular_button_classes(dim_classes, &extra_class);

    let btn = view! {
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                on_rate.run(());
            }
            class=btn_classes.clone()
            title=crate::i18n::t("action-buttons-rate")
        >
            <RatingIcon rating=rating.get() size=icon_pixel_size />
        </button>
    };

    view! {
        {if show_tooltip {
            view! {
                <TooltipWrapper label=crate::i18n::t("action-buttons-i-like-this").to_string()>
                    {btn}
                </TooltipWrapper>
            }.into_any()
        } else {
            btn.into_any()
        }}
    }
}

// ── Rating pill ───────────────────────────────────────────────────────────────
// The rating control of the app: a round button showing the rating that is set; while the pointer rests on it, it opens into a dark pill of three round options (Not for me, I like this,
// Love this!) laid over its neighbours, each with a light bubble above it saying what it does ("Rated" over the one that is set). The set option is solid, the others are outlines;
// choosing the set option again takes the rating back.
#[component]
pub fn RatingPill(
    #[prop(into)] rating: Signal<Option<MovieRating>>,
    on_set: Callback<Option<MovieRating>>,
    #[prop(default = "md".to_string(), into)] size: String,
) -> impl IntoView {
    let (dim_classes, _, icon_px) = circular_button_specs(&size);
    let options = [
        (MovieRating::Dislike, "rating-not-for-me"),
        (MovieRating::Like, "rating-like"),
        (MovieRating::Love, "rating-love"),
    ];
    view! {
        <div class="rating-pill-wrap group/pill relative inline-flex shrink-0 hover:z-50">
            // the closed button: the rating that is set, or the outline thumb
            <button type="button" aria-label=crate::i18n::t("action-buttons-rate") class=circular_button_classes(dim_classes, "")>
                {move || match rating.get() {
                    Some(MovieRating::Love) => view! { <crate::components::media::movie_card_rating::DoubleThumbsUpIcon size=icon_px filled=true /> }.into_any(),
                    Some(MovieRating::Dislike) => view! { <AppIcon glyph=Glyph::ThumbDownFilled style=format!("font-size: {icon_px}px;") /> }.into_any(),
                    Some(MovieRating::Like) => view! { <AppIcon glyph=Glyph::ThumbUpFilled style=format!("font-size: {icon_px}px;") /> }.into_any(),
                    None => view! { <AppIcon glyph=Glyph::ThumbUp style=format!("font-size: {icon_px}px;") /> }.into_any(),
                }}
            </button>
            // the opened pill, centred on the button
            <div class="rating-pill absolute left-1/2 top-1/2 z-50 flex items-center gap-1 rounded-full bg-field p-[3px] shadow-rating">
                {options.into_iter().map(|(option, label_key)| {
                    let is_set = move || rating.get() == Some(option);
                    view! {
                        <div class="group/opt relative">
                            <button
                                type="button"
                                on:click=move |e: leptos::ev::MouseEvent| { e.stop_propagation(); on_set.run(if rating.get_untracked() == Some(option) { None } else { Some(option) }); }
                                aria-label=crate::i18n::t(label_key)
                                class=format!("flex items-center justify-center rounded-full text-ink transition-colors hover:bg-lift cursor-pointer {dim_classes}")
                            >
                                {move || {
                                    let on = is_set();
                                    match option {
                                        MovieRating::Love => view! { <crate::components::media::movie_card_rating::DoubleThumbsUpIcon size=icon_px filled=on /> }.into_any(),
                                        MovieRating::Dislike => view! { <AppIcon glyph=if on { Glyph::ThumbDownFilled } else { Glyph::ThumbDown } style=format!("font-size: {icon_px}px;") /> }.into_any(),
                                        MovieRating::Like => view! { <AppIcon glyph=if on { Glyph::ThumbUpFilled } else { Glyph::ThumbUp } style=format!("font-size: {icon_px}px;") /> }.into_any(),
                                    }
                                }}
                            </button>
                            // the bubble over the option
                            <span class="rating-bubble pointer-events-none absolute bottom-[calc(100%+10px)] left-1/2 -translate-x-1/2 whitespace-nowrap rounded-control bg-ink-soft px-3 py-1.5 text-body font-semibold leading-none text-chrome opacity-0 transition-opacity duration-(--motion-fast) group-hover/opt:opacity-100">
                                {move || if is_set() { crate::i18n::t("rating-rated") } else { crate::i18n::t(label_key) }}
                                <span class="absolute left-1/2 top-full h-0 w-0 -translate-x-1/2 border-x-[6px] border-t-[6px] border-x-transparent border-t-ink-soft"></span>
                            </span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

// ── Mute / Replay Button ──────────────────────────────────────────────────────
// "mute/add mute that doubles as a reload button"
// Used in Hero, InfoModal, MovieCardPopup.
#[component]
pub fn MuteReplayButton(
    #[prop(into)] is_muted: Signal<bool>,
    #[prop(into)] is_ended: Signal<bool>,
    on_toggle: Callback<()>,
    #[prop(optional, into)] on_force_replay: Option<Callback<()>>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (dim_classes, icon_classes, _) = circular_button_specs(&size);
    let extra_class = class.unwrap_or_default();
    let btn_classes = circular_button_classes(dim_classes, &extra_class);

    let title_text = move || {
        if is_ended.get() {
            crate::i18n::t("action-buttons-replay-trailer")
        } else if is_muted.get() {
            crate::i18n::t("action-buttons-unmute-trailer")
        } else {
            crate::i18n::t("action-buttons-mute-trailer")
        }
    };

    view! {
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                on_toggle.run(());
            }
            on:dblclick=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                if let Some(cb) = on_force_replay {
                    cb.run(());
                }
            }
            class=btn_classes
            title=title_text
            aria-label=title_text
        >
            {move || {
                if is_ended.get() {
                    view! { <i class=format!("ph ph-arrow-counter-clockwise text-ink {}", icon_classes)></i> }.into_any()
                } else if is_muted.get() {
                    view! { <AppIcon glyph=Glyph::Muted class=format!("text-ink {}", icon_classes) /> }.into_any()
                } else {
                    view! { <AppIcon glyph=Glyph::Speaker class=format!("text-ink {}", icon_classes) /> }.into_any()
                }
            }}
        </button>
    }
}

// ── Close Button ──────────────────────────────────────────────────────────────
// Modal ✕ button matching Like, Add to My List, and Mute/Replay in size and color.
#[component]
pub fn CloseButton(
    on_close: Callback<()>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (dim_classes, icon_classes, _) = circular_button_specs(&size);
    let extra_class = class.unwrap_or_default();
    let btn_classes = circular_button_classes(dim_classes, &extra_class);

    view! {
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                on_close.run(());
            }
            class=btn_classes
            title=crate::i18n::t("action-buttons-close")
            aria-label=crate::i18n::t("action-buttons-close")
        >
            <i class=format!("ph-bold ph-x {}", icon_classes)></i>
        </button>
    }
}

// ── More Info Caret Button ────────────────────────────────────────────────────
// Caret down circular button on movie card popups.
#[component]
pub fn MoreInfoCaretButton(
    on_open: Callback<()>,
    #[prop(default = "md".to_string(), into)] size: String,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let (dim_classes, icon_classes, _) = circular_button_specs(&size);
    let extra_class = class.unwrap_or_default();
    let btn_classes = circular_button_classes(dim_classes, &extra_class);

    view! {
        <button
            type="button"
            on:click=move |e: leptos::ev::MouseEvent| {
                e.stop_propagation();
                on_open.run(());
            }
            class=btn_classes
            title=crate::i18n::t("action-buttons-more-info")
            aria-label=crate::i18n::t("action-buttons-more-info")
        >
            <i class=format!("ph-bold ph-caret-down {}", icon_classes)></i>
        </button>
    }
}
