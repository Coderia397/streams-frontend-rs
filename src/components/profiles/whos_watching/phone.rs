//! "Choose your profile" on a phone, as the Netflix app draws it (screenshots of the app): a title picture over the whole screen with its logo and one line about it (a chart place, a date) at the
//! foot, changing every few seconds; below it a dark panel with a curved top, tinted by the picture, the words "Choose your profile" and the profiles three in a row, the rest centred under them.
use leptos::prelude::*;
use crate::models::profile::Profile;
use crate::services::ai_engine::fetch_hero_feed;
use crate::services::tmdb::{fetch_movie_logo, MediaItem};

/// How long each title stays, in milliseconds.
const ROTATE_MS: u64 = 5200;

#[component]
pub fn PhoneProfilePicker(
    #[prop(into)] profiles: Signal<Vec<Profile>>,
    #[prop(into)] editing: Signal<bool>,
    /// Whether the page is the Manage profiles page (the pill says Done and goes back)
    manage: bool,
    on_select: Callback<Profile>,
    on_add: Callback<()>,
    on_manage: Callback<()>,
) -> impl IntoView {
    let heroes = LocalResource::new(|| async { fetch_hero_feed("all", None).await.unwrap_or_default().into_iter().filter(|i| i.poster_path.is_some()).take(6).collect::<Vec<MediaItem>>() });
    let idx = RwSignal::new(0usize);
    Effect::new(move |_| {
        let handle = leptos::leptos_dom::helpers::set_interval_with_handle(
            move || {
                let n = heroes.get_untracked().map(|h| h.len()).unwrap_or(0);
                if n > 1 { idx.update(|i| *i = (*i + 1) % n); }
            },
            std::time::Duration::from_millis(ROTATE_MS),
        );
        if let Ok(h) = handle { on_cleanup(move || h.clear()); }
    });
    // the logo and the line of the title in view
    let facts = LocalResource::new(move || {
        let item = heroes.get().and_then(|h| h.get(idx.get()).cloned());
        async move {
            let Some(it) = item else { return (None::<String>, None::<String>) };
            let logo = fetch_movie_logo(it.id, it.is_tv()).await.ok().flatten();
            // the first of the lines the hero feed chose for this title (the billboard shows the same ones)
            let hook = it.hooks.first().map(|h| crate::services::ai_engine::engine_text(&h.key, Some(&h.args)).unwrap_or_else(|| h.text.clone()));
            (logo, hook)
        }
    });
    let title_now = move || heroes.get().and_then(|h| h.get(idx.get()).map(|i| i.display_title().to_string())).unwrap_or_default();
    let picture = move |i: usize| heroes.get().and_then(|h| h.get(i).and_then(|it| it.poster_path.clone())).map(|p| format!("{}/w780{}", crate::config::TMDB_IMAGE_URL, p)).unwrap_or_default();
    let count = move || heroes.get().map(|h| h.len()).unwrap_or(0);

    view! {
        <div class="absolute inset-0 flex flex-col overflow-hidden bg-gate-phone">
            // the picture
            <div class="relative min-h-0 flex-1 overflow-hidden">
                {move || (0..count()).map(|i| view! {
                    <img src=picture(i) alt="" class="absolute inset-0 h-full w-full object-cover object-top transition-opacity duration-[900ms]" style=move || format!("opacity: {};", if idx.get() == i { 1 } else { 0 }) />
                }).collect::<Vec<_>>()}
                <div class="absolute inset-0" style="background: linear-gradient(to bottom, rgba(0,0,0,0.35) 0%, rgba(0,0,0,0) 22%, rgba(0,0,0,0) 52%, rgba(0,0,0,0.92) 100%);"></div>
                <div class="absolute inset-x-0 bottom-[44px] flex flex-col items-center gap-3 px-8 text-center">
                    {move || match facts.get() {
                        Some((Some(logo), _)) => view! { <img src=logo alt=title_now() class="max-h-[84px] max-w-[72%] object-contain drop-shadow-[0_2px_8px_rgba(0,0,0,0.6)]" /> }.into_any(),
                        _ => view! { <p class="text-heading font-bold leading-tight text-ink drop-shadow-[0_2px_8px_rgba(0,0,0,0.6)]">{title_now()}</p> }.into_any(),
                    }}
                    {move || facts.get().and_then(|(_, hook)| hook).map(|h| {
                        let ranked = h.starts_with("No.");
                        view! { <p class="flex items-center gap-2 text-title-sm font-semibold text-ink">{ranked.then(|| view! { <span class="rounded-[3px] bg-brand px-1 text-fine font-extrabold leading-4 text-ink">"TOP 10"</span> })}{h}</p> }
                    })}
                </div>
            </div>
            // the panel: a curved top, the colour of the picture
            <div class="relative z-10 -mt-[30px] overflow-hidden pb-[max(20px,env(safe-area-inset-bottom))] pt-[34px]" style="margin-inline: -12%; padding-inline: 12%; border-radius: 50% 50% 0 0 / 56px 56px 0 0; background: var(--color-gate-phone-sheet);">
                <div class="absolute inset-0 -z-10" aria-hidden="true">
                    {move || (0..count()).map(|i| view! {
                        <img src=picture(i) alt="" class="absolute inset-0 h-full w-full scale-150 object-cover object-bottom blur-[38px] transition-opacity duration-[900ms]" style=move || format!("opacity: {};", if idx.get() == i { 0.55 } else { 0.0 }) />
                    }).collect::<Vec<_>>()}
                    <div class="absolute inset-0 bg-void/45"></div>
                </div>
                <p class="mb-5 text-center text-body-lg text-ink/70">
                    {move || if editing.get() { crate::i18n::t("profile-manage-title") } else { crate::i18n::t("whos-watching-choose") }}
                </p>
                // the app's grid (measured on a Pixel 8, note 116): 100 px tiles with 72 px pictures, 12 px apart, rows 16 px apart, three to a row
                // centred; the last tile is Edit
                <div class="mx-auto flex max-w-[324px] flex-wrap justify-center gap-x-3 gap-y-4">
                    {move || profiles.get().into_iter().map(|profile| {
                        let p = profile.clone();
                        let name = profile.name.clone();
                        let avatar = profile.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
                        let kids_default = profile.is_kids && profile.avatar_url.is_none();
                        let locked = profile.pin.is_some();
                        view! {
                            <button type="button" on:click=move |_| on_select.run(p.clone()) class="flex w-[100px] cursor-pointer flex-col items-center gap-[13px] focus:outline-none active:scale-95 transition-transform">
                                <span class="relative block h-[72px] w-[72px] overflow-hidden rounded-[8px] shadow-[0_6px_14px_rgba(0,0,0,0.45)]">
                                    {if kids_default {
                                        view! { <crate::components::profiles::kids_avatar::KidsAvatar size=72.0 /> }.into_any()
                                    } else {
                                        view! { <img src=avatar alt=name.clone() class="h-full w-full object-cover" /> }.into_any()
                                    }}
                                    {move || editing.get().then(|| view! { <span class="absolute inset-0 flex items-center justify-center bg-void/55"><i class="ph-bold ph-pencil-simple text-heading text-ink"></i></span> })}
                                </span>
                                <span class="flex max-w-full items-center gap-1 text-body-lg font-bold leading-tight text-ink">
                                    <span class="truncate">{name}</span>
                                    {(locked && !editing.get_untracked()).then(|| view! { <i class="ph ph-lock-key text-body-md"></i> })}
                                </span>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                    {move || (profiles.get().len() < 5).then(|| view! {
                        <button type="button" on:click=move |_| on_add.run(()) class="flex w-[100px] cursor-pointer flex-col items-center gap-[13px] focus:outline-none active:scale-95 transition-transform">
                            <span class="flex h-[72px] w-[72px] items-center justify-center rounded-[8px] bg-ink/15"><i class="ph ph-plus text-heading text-ink"></i></span>
                            <span class="text-body-lg font-normal leading-tight text-ink">{crate::i18n::t("whos-watching-add-profile")}</span>
                        </button>
                    })}
                    // Edit: a pencil on a 15 % white square (it takes the sheet's tint, as the app's), the label in regular weight
                    <Show when=move || !editing.get() && !manage>
                        <button type="button" on:click=move |_| on_manage.run(()) class="flex w-[100px] cursor-pointer flex-col items-center gap-[13px] focus:outline-none active:scale-95 transition-transform">
                            <span class="flex h-[72px] w-[72px] items-center justify-center rounded-[8px] bg-ink/15"><i class="ph ph-pencil-simple text-heading text-ink"></i></span>
                            <span class="text-body-lg font-normal leading-tight text-ink">{crate::i18n::t("whos-watching-edit")}</span>
                        </button>
                    </Show>
                </div>
                // while editing (or on Manage profiles), Done under the grid
                <Show when=move || editing.get() || manage>
                    <div class="mt-6 flex justify-center">
                        <button type="button" on:click=move |_| on_manage.run(()) class="flex h-10 cursor-pointer items-center gap-2 rounded-button border border-ink/80 bg-void/60 px-5 text-body-md font-medium text-ink active:scale-95 transition-transform">
                            {crate::i18n::t("common-done")}
                        </button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
