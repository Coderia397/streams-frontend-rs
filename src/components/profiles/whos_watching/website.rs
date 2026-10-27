//! Who's watching on tablets and computers (note 114), as Netflix's website: the question, the profiles 160 px apart with their
//! pictures, Add Profile and the Manage Profiles pill. Under the pointer a tile gets a panel, white at 8.5 % over the page and as large as
//! the tile (160 x 205), and its name turns white (measured on the owner's recording of netflix.com, 2026-10-26, note 132). Asked by a
//! Kids profile, the screen is Netflix's Kids form: no Add Profile, no Manage Profiles, and the kids mark in the corner.
use crate::models::profile::Profile;
use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

#[component]
pub fn WebsiteGate(
    #[prop(into)] profiles: Signal<Vec<Profile>>,
    #[prop(into)] editing: Signal<bool>,
    on_select: Callback<Profile>,
    on_add: Callback<()>,
    on_manage: Callback<()>,
    /// the Kids form (store::kids_gate)
    #[prop(optional)]
    kids: bool,
) -> impl IntoView {
    let profile_store = crate::store::use_profile_store();
    view! {
            <div class="flex h-full w-full flex-col items-center justify-center">
            <img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="absolute left-[var(--app-x,56px)] top-[26px] h-[24px] w-auto" />
            // the question, the profiles and the button come in together, as on Netflix: from 90 % and transparent, over 0.45 s (the page itself is there at once)
            <div class="flex flex-col items-center animate-in fade-in zoom-in-90 duration-[450ms]">
            {move || profile_store.notice.get().map(|m| view! { <p role="status" class="mb-8 max-w-[85vw] rounded-control border border-danger/50 bg-danger/10 px-4 py-3 text-center text-body-sm text-danger md:max-w-xl">{m}</p> })}
            <h1 class="text-center text-heading-xl font-bold leading-[38px] text-ink">
                {move || if editing.get() { crate::i18n::t("profile-manage-title") } else { crate::i18n::t("profile-whos-watching") }}
            </h1>

            <div class="mx-auto mt-4 flex max-w-[85vw] flex-wrap items-start justify-center gap-[8px] px-4">
                {move || profiles.get().into_iter().map(|profile| {
                    let p_clone = profile.clone();
                    let name = profile.name.clone();
                    let is_kids = profile.is_kids;
                    let has_pin = profile.pin.is_some();
                    let avatar = profile.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
                    view! {
                        <button type="button" on:click=move |_| on_select.run(p_clone.clone()) class=TILE>
                            <div class="relative h-[128px] w-[128px] overflow-hidden rounded-modal">
                                {if is_kids && p_clone.avatar_url.is_none() {
                                    view! { <crate::components::profiles::kids_avatar::KidsAvatar size=128.0 /> }.into_any()
                                } else {
                                    view! { <img src=avatar class="h-full w-full object-cover" alt=name.clone() /> }.into_any()
                                }}
                                // edit mode: a dark layer with a pencil
                                {move || editing.get().then(|| view! {
                                    <div class="absolute inset-0 flex items-center justify-center bg-void/55"><i class="ph-bold ph-pencil-simple text-display text-ink"></i></div>
                                })}
                            </div>
                            <span class="flex items-center gap-1.5 text-title-sm font-medium leading-[23px] text-ink/70 transition-colors group-hover:text-ink">
                                {name}
                                {(has_pin && !editing.get()).then(|| view! { <i class="ph ph-lock-key text-body-md"></i> })}
                            </span>
                        </button>
                    }
                }).collect::<Vec<_>>()}

                // Add Profile tile (not in the Kids form)
                {move || (!kids && profiles.get().len() < crate::config::MAX_PROFILES).then(|| view! {
                    <button type="button" on:click=move |_| on_add.run(()) class=TILE>
                        <div class="flex h-[128px] w-[128px] items-center justify-center rounded-modal bg-gate-tile">
                            <AppIcon glyph=Glyph::Plus class="text-icon-lg text-ink" />
                        </div>
                        <span class="text-title-sm font-medium leading-[23px] text-ink/70 transition-colors group-hover:text-ink">{crate::i18n::t("whos-watching-add-profile")}</span>
                    </button>
                })}
            </div>

            // Manage Profiles opens the editing mode; Done goes back (not in the Kids form)
            <div class="mb-10 mt-[36px]" class:hidden=kids>
                <button
                    type="button"
                    on:click=move |_| on_manage.run(())
                    class="btn-pill btn-pill-md bg-void/70 text-ink shadow-[inset_0_0_0_1px_rgb(128_128_128/0.7)] hover:bg-ink hover:text-void"
                >
                    {move || if editing.get() { view! { {crate::i18n::t("common-done")} }.into_any() } else { view! { <i class="ph ph-pencil-simple text-title-sm"></i>{crate::i18n::t("profile-manage")} }.into_any() }}
                </button>
            </div>
            </div>
            // the Kids form: the mark that stays in the corner of a Kids profile's pages (layout/nav/website)
            {kids.then(|| view! {
                <div class="pointer-events-none fixed bottom-4 right-[var(--app-x)]"><crate::components::profiles::kids_splash::KidsWordmark class="h-8 w-auto" /></div>
            })}
            </div>
    }
}

/// A profile's tile, and Add Profile's: the panel under the pointer or the keyboard's focus.
const TILE: &str = "group flex w-[160px] cursor-pointer flex-col items-center gap-[23px] rounded-modal p-[16px] transition-colors duration-(--motion-quick) hover:bg-ink/[0.085] focus:outline-none focus-visible:bg-ink/[0.085]";
