use leptos::prelude::*;
use crate::models::profile::Profile;
use crate::store::use_profile_store;
use crate::components::profiles::pin_prompt::ProfilePinPrompt;
use crate::components::profiles::add_edit_modal::{AddEditProfileModal, ProfileFormData};
use crate::components::profiles::switch_overlay::{begin_switch, is_compact, ProfileSwitchOverlay};
use crate::components::profiles::phone_picker::PhoneProfilePicker;

fn save_profiles_to_storage(profiles: &[Profile], active_id: Option<&str>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json) = serde_json::to_string(profiles) {
            let _ = storage.set_item(crate::keys::PROFILES, &json);
        }
        if let Some(id) = active_id {
            let _ = storage.set_item(crate::keys::ACTIVE_PROFILE, id);
        }
    }
}

#[component]
pub fn WhosWatchingGate(
    /// The page of Manage profiles (`/profiles/manage`): opens in edit mode, a tap on a profile edits it, Done goes back to the app, and the profile in use stays the one in use.
    #[prop(optional)] manage: bool,
) -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let profile_store = use_profile_store();
    let profiles_signal = profile_store.profiles;
    let active_profile_id = profile_store.active_profile_id;

    let (edit_mode, set_edit_mode) = signal(manage);
    let (pin_prompt_profile, set_pin_prompt_profile) = signal(None::<Profile>);
    let (modal_state, set_modal_state) = signal(None::<(String, Option<Profile>)>); // (mode, initial)
    let (switching_target, set_switching_target) = signal(None::<Profile>);
    let (unlocked_ids, set_unlocked_ids) = signal(Vec::<String>::new());

    // Perform profile switch with overlay animation (Task 103)
    let perform_switch = move |p: Profile| {
        let pid = p.id.clone();
        save_profiles_to_storage(&profiles_signal.get_untracked(), Some(&pid));
        // a phone or a tablet: the switch is drawn at the root of the app and flies the picture to the bar (it outlives this screen); the others use the screen's own overlay
        if is_compact() && !p.is_kids { begin_switch(p); } else { set_switching_target.set(Some(p)); }
    };

    let on_select_tile = move |p: Profile| {
        if edit_mode.get() {
            // In edit mode, clicking a tile opens edit modal (Task 099)
            set_modal_state.set(Some(("edit".to_string(), Some(p))));
        } else {
            // Check if profile has PIN lock (Task 098)
            let is_unlocked = unlocked_ids.get().contains(&p.id);
            if p.pin.is_some() && !is_unlocked {
                set_pin_prompt_profile.set(Some(p));
            } else {
                perform_switch(p);
            }
        }
    };

    let on_unlock_pin = move || {
        if let Some(p) = pin_prompt_profile.get() {
            set_unlocked_ids.update(|ids| ids.push(p.id.clone()));
            set_pin_prompt_profile.set(None);
            perform_switch(p);
        }
    };

    let on_save_modal = move |form: ProfileFormData| {
        if let Some((mode, initial)) = modal_state.get() {
            if mode == "add" {
                let new_id = format!("p_{}", js_sys::Date::now() as u64);
                let new_profile = Profile {
                    id: new_id.clone(),
                    name: form.name,
                    avatar_url: form.avatar_url,
                    is_kids: form.is_kids,
                    is_default: Some(false),
                    pin: form.pin,
                    sort_order: profiles_signal.get().len() as i32,
                };
                profiles_signal.update(|list| list.push(new_profile));
            } else if mode == "edit" {
                if let Some(init) = initial {
                    profiles_signal.update(|list| {
                        if let Some(target) = list.iter_mut().find(|p| p.id == init.id) {
                            target.name = form.name;
                            target.avatar_url = form.avatar_url;
                            target.is_kids = form.is_kids;
                            target.pin = form.pin;
                        }
                    });
                }
            }
            save_profiles_to_storage(&profiles_signal.get(), active_profile_id.get().as_deref());
            if crate::services::account::has_account() { crate::services::sync::push_profiles(profile_store); }
            set_modal_state.set(None);
        }
    };

    let on_delete_profile = move |target_id: String| {
        // the main profile and the last profile stay (the account refuses it too)
        if !crate::models::profile_rules::can_delete(&profiles_signal.get_untracked(), &target_id) {
            set_modal_state.set(None);
            return;
        }
        profiles_signal.update(|list| list.retain(|p| p.id != target_id));
        if active_profile_id.get().as_deref() == Some(&target_id) {
            let next_id = profiles_signal.get().first().map(|p| p.id.clone());
            active_profile_id.set(next_id.clone());
            save_profiles_to_storage(&profiles_signal.get(), next_id.as_deref());
        } else {
            save_profiles_to_storage(&profiles_signal.get(), active_profile_id.get().as_deref());
        }
        if crate::services::account::has_account() { crate::services::sync::push_profiles(profile_store); }
        set_modal_state.set(None);
    };

    view! {
        // the page on Netflix: a dark red glow at the top fading to black, the logo top left, the question centred, tiles 160 px wide (picture 128 px), a pill under them
        // Netflix's Who's watching (measured 2026-10-23, note 113): black with a red glow from above the top, heading 32 px, tiles 168 px apart,
        // 32 px under the heading, names 23 px under the pictures, the button 52 px under the names
        <div class="fixed inset-0 z-(--z-screen) flex select-none flex-col items-center justify-center text-ink" style="background: #000 radial-gradient(2170px at 50% -54dvh, rgba(111, 24, 29, 0.8) 19%, rgba(56, 12, 15, 0.4) 60%, transparent 100%);">
            <PhoneProfilePicker
                profiles=profiles_signal
                editing=edit_mode
                manage=manage
                on_select=Callback::new(move |p| on_select_tile(p))
                on_add=Callback::new(move |_| set_modal_state.set(Some(("add".to_string(), None))))
                on_manage=Callback::new({ let navigate = navigate.clone(); move |_| if manage { navigate("/browse", Default::default()) } else { set_edit_mode.update(|e| *e = !*e) } })
            />
            <div class="hidden md:flex md:h-full md:w-full md:flex-col md:items-center md:justify-center">
            <img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="absolute left-[var(--app-x,56px)] top-[26px] h-[24px] w-auto" />
            // the question, the profiles and the button come in together, as on Netflix: from 90 % and transparent, over 0.45 s (the page itself is there at once)
            <div class="flex flex-col items-center animate-in fade-in zoom-in-90 duration-[450ms]">
            {move || profile_store.notice.get().map(|m| view! { <p role="status" class="mb-8 max-w-[85vw] rounded-control border border-danger/50 bg-danger/10 px-4 py-3 text-center text-body-sm text-danger md:max-w-xl">{m}</p> })}
            <h1 class="text-center text-heading-xl font-bold leading-[38px] text-ink">
                {move || if edit_mode.get() { crate::i18n::t("profile-manage-title") } else { crate::i18n::t("profile-whos-watching") }}
            </h1>

            <div class="mx-auto mt-4 flex max-w-[85vw] flex-wrap items-start justify-center gap-[8px] px-4">
                {move || profiles_signal.get().into_iter().map(|profile| {
                    let p_clone = profile.clone();
                    let name = profile.name.clone();
                    let is_kids = profile.is_kids;
                    let has_pin = profile.pin.is_some();
                    let avatar = profile.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
                    view! {
                        <button type="button" on:click=move |_| on_select_tile(p_clone.clone()) class="group flex w-[160px] cursor-pointer flex-col items-center gap-[23px] p-[16px] focus:outline-none">
                            <div class="relative h-[128px] w-[128px] overflow-hidden rounded-modal outline outline-[3px] outline-transparent transition-[outline-color] group-hover:outline-ink/70 group-focus-visible:outline-ink/70">
                                {if is_kids && p_clone.avatar_url.is_none() {
                                    view! { <crate::components::profiles::kids_avatar::KidsAvatar size=128.0 /> }.into_any()
                                } else {
                                    view! { <img src=avatar class="h-full w-full object-cover" alt=name.clone() /> }.into_any()
                                }}
                                // edit mode: a dark layer with a pencil
                                {move || edit_mode.get().then(|| view! {
                                    <div class="absolute inset-0 flex items-center justify-center bg-void/55"><i class="ph-bold ph-pencil-simple text-display text-ink"></i></div>
                                })}
                            </div>
                            <span class="flex items-center gap-1.5 text-title-sm font-medium leading-[23px] text-ink/70 transition-colors group-hover:text-ink">
                                {name}
                                {(has_pin && !edit_mode.get()).then(|| view! { <i class="ph ph-lock-key text-body-md"></i> })}
                            </span>
                        </button>
                    }
                }).collect::<Vec<_>>()}

                // Add Profile tile
                {move || (profiles_signal.get().len() < 5).then(|| view! {
                    <button type="button" on:click=move |_| set_modal_state.set(Some(("add".to_string(), None))) class="group flex w-[160px] cursor-pointer flex-col items-center gap-[23px] p-[16px] focus:outline-none">
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
                    on:click={ let navigate = navigate.clone(); move |_| if manage { navigate("/browse", Default::default()) } else { set_edit_mode.update(|e| *e = !*e) } }
                    class="btn-pill btn-pill-md bg-void/70 text-ink shadow-[inset_0_0_0_1px_rgb(128_128_128/0.7)] hover:bg-ink hover:text-void"
                >
                    {move || if edit_mode.get() { view! { {crate::i18n::t("common-done")} }.into_any() } else { view! { <i class="ph ph-pencil-simple text-title-sm"></i>{crate::i18n::t("profile-manage")} }.into_any() }}
                </button>
            </div>
            </div>
            </div>

            // PIN prompt modal (Task 098)
            <Show when=move || pin_prompt_profile.get().is_some()>
                {move || pin_prompt_profile.get().map(|p| view! {
                    <ProfilePinPrompt
                        profile=p
                        on_unlock=Callback::new(move |_| on_unlock_pin())
                        on_cancel=Callback::new(move |_| set_pin_prompt_profile.set(None))
                    />
                })}
            </Show>

            // Add/Edit Profile modal (Task 100)
            <Show when=move || modal_state.get().is_some()>
                {move || modal_state.get().map(|(mode, initial)| {
                    let target_id = initial.as_ref().map(|p| p.id.clone());
                    let (can_del, kids_ok) = {
                        let list = profiles_signal.get_untracked();
                        match target_id.as_deref() { Some(id) => (crate::models::profile_rules::can_delete(&list, id), crate::models::profile_rules::can_be_kids(&list, id)), None => (false, true) }
                    };
                    let on_delete_cb = target_id.filter(|_| can_del).map(|tid| Callback::new(move |_| on_delete_profile(tid.clone())));
                    view! {
                        <AddEditProfileModal
                            mode=mode
                            initial=initial
                            on_save=Callback::new(move |form| on_save_modal(form))
                            on_delete=on_delete_cb
                            on_cancel=Callback::new(move |_| set_modal_state.set(None))
                            allow_kids=kids_ok
                        />
                    }
                })}
            </Show>

            // Full-screen Switch Overlay Animation (Task 103)
            <Show when=move || switching_target.get().is_some()>
                {move || switching_target.get().map(|p| {
                    let pid = p.id.clone();
                    view! {
                        <ProfileSwitchOverlay
                            profile=p
                            on_finish=Callback::new(move |_| {
                                // the home page loads afresh for the profile
                                crate::store::activate_profile(&pid);
                                set_switching_target.set(None);
                            })
                        />
                    }
                })}
            </Show>
        </div>
    }
}

/// `/profiles/manage`: the profiles of the account, each with its pencil, an Add Profile tile and Done (what Manage Profiles opens in the app).
#[component]
pub fn ManageProfilesPage() -> impl IntoView {
    view! { <WhosWatchingGate manage=true /> }
}
