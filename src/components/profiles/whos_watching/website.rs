//! Who's watching on tablets and computers (note 114), as Netflix's website: the question, the profiles 160 px apart with their
//! pictures, Add Profile and the Manage Profiles pill.
use crate::models::profile::Profile;
use leptos::prelude::*;

#[component]
pub fn WebsiteGate(
    #[prop(into)] profiles: Signal<Vec<Profile>>,
    #[prop(into)] editing: Signal<bool>,
    on_select: Callback<Profile>,
    on_add: Callback<()>,
    on_manage: Callback<()>,
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
                        <button type="button" on:click=move |_| on_select.run(p_clone.clone()) class="group flex w-[160px] cursor-pointer flex-col items-center gap-[23px] p-[16px] focus:outline-none">
                            <div class="relative h-[128px] w-[128px] overflow-hidden rounded-modal outline outline-[3px] outline-transparent transition-[outline-color] group-hover:outline-ink/70 group-focus-visible:outline-ink/70">
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

                // Add Profile tile
                {move || (profiles.get().len() < 5).then(|| view! {
                    <button type="button" on:click=move |_| on_add.run(()) class="group flex w-[160px] cursor-pointer flex-col items-center gap-[23px] p-[16px] focus:outline-none">
                        <div class="flex h-[128px] w-[128px] items-center justify-center rounded-modal bg-[#3b2527] outline outline-[3px] outline-transparent transition-[outline-color] group-hover:outline-ink/70">
                            <i class="ph ph-plus text-icon-lg text-ink"></i>
                        </div>
                        <span class="text-title-sm font-medium leading-[23px] text-ink/70 transition-colors group-hover:text-ink">{crate::i18n::t("whos-watching-add-profile")}</span>
                    </button>
                })}
            </div>

            // Manage Profiles opens the editing mode; Done goes back
            <div class="mb-10 mt-[36px]">
                <button
                    type="button"
                    on:click=move |_| on_manage.run(())
                    class="btn-pill btn-pill-md bg-void/70 text-ink shadow-[inset_0_0_0_1px_rgb(128_128_128/0.7)] hover:bg-ink hover:text-void"
                >
                    {move || if editing.get() { view! { {crate::i18n::t("common-done")} }.into_any() } else { view! { <i class="ph ph-pencil-simple text-title-sm"></i>{crate::i18n::t("profile-manage")} }.into_any() }}
                </button>
            </div>
            </div>
            </div>
    }
}
