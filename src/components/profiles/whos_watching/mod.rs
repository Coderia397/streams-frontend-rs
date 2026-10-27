use leptos::prelude::*;
use crate::models::profile::Profile;
use crate::store::use_profile_store;
use crate::components::profiles::pin_prompt::ProfilePinPrompt;
use crate::components::profiles::add_edit_modal::{AddEditProfileModal, ProfileFormData};
use crate::components::profiles::switch_overlay::{begin_switch, is_app_switch, ProfileSwitchOverlay};
pub mod phone;
pub mod website;


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
    // what the PIN opens: the profile (false) or, in Manage Profiles, its edit window (true)
    let (pin_then_edit, set_pin_then_edit) = signal(false);
    let (modal_state, set_modal_state) = signal(None::<(String, Option<Profile>)>); // (mode, initial)
    let (switching_target, set_switching_target) = signal(None::<Profile>);
    let (unlocked_ids, set_unlocked_ids) = signal(Vec::<String>::new());

    // Perform profile switch with overlay animation (Task 103)
    let perform_switch = move |p: Profile| {
        let pid = p.id.clone();
        save_profiles_to_storage(&profiles_signal.get_untracked(), Some(&pid));
        // the phone form: the switch is drawn at the root of the app and flies the picture to the bar (it outlives this screen); the website uses the screen's own overlay
        if is_app_switch() && !p.is_kids { begin_switch(p); } else { set_switching_target.set(Some(p)); }
    };

    // a profile chosen for editing: on the website its settings page, Netflix's "Manage profile and preferences", whose back arrow comes
    // back to Manage Profiles (note 132); on the phone the editing window
    let open_editor = Callback::new({
        let navigate = navigate.clone();
        move |p: Profile| {
            if crate::form::current().is_phone() {
                set_modal_state.set(Some(("edit".to_string(), Some(p))));
            } else {
                crate::store::open_from_manage(&p.id);
                navigate(&crate::pages::settings::Screen::Profile(p.id.clone()).path(), Default::default());
            }
        }
    });
    let on_select_tile = move |p: Profile| {
        if edit_mode.get() {
            // a locked profile is edited only after its PIN, as it is opened only after it: Manage Profiles was a way round the lock
            if p.pin.is_some() && !unlocked_ids.get().contains(&p.id) {
                set_pin_then_edit.set(true);
                set_pin_prompt_profile.set(Some(p));
            } else {
                open_editor.run(p);
            }
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
            if pin_then_edit.get_untracked() {
                set_pin_then_edit.set(false);
                open_editor.run(p);
            } else {
                perform_switch(p);
            }
        }
    };

    let on_save_modal = move |form: ProfileFormData| {
        if let Some((mode, initial)) = modal_state.get() {
            if mode == "add" {
                let new_id = format!("p_{}", js_sys::Date::now() as u64);
                crate::components::layout::tour::mark_new(&new_id);
                let new_profile = Profile {
                    id: new_id.clone(),
                    name: form.name,
                    avatar_url: form.avatar_url,
                    is_kids: form.is_kids,
                    is_default: Some(false),
                    pin: None,
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

    // Manage Profiles: on the website its own page (/profiles/manage, Netflix's ManageProfiles), whose Done goes back to Who's watching over
    // the page it was asked on (or Home when a profile is in use); on the phone the editing mode in place (note 132)
    let on_manage = Callback::new({
        let navigate = navigate.clone();
        move |_| {
            if manage {
                let back = if active_profile_id.get_untracked().is_none() { crate::store::gate_page() } else { crate::config::routes::BROWSE.to_string() };
                navigate(&back, Default::default());
            } else if crate::form::current().is_phone() {
                set_edit_mode.update(|e| *e = !*e);
            } else {
                if let Some(loc) = web_sys::window().map(|w| w.location()) {
                    crate::store::remember_gate_page(&format!("{}{}", loc.pathname().unwrap_or_default(), loc.search().unwrap_or_default()));
                }
                navigate("/profiles/manage", Default::default());
            }
        }
    });

    view! {
        // the page on Netflix: a dark red glow at the top fading to black, the logo top left, the question centred, tiles 160 px wide (picture 128 px), a pill under them
        // Netflix's Who's watching (measured 2026-10-23, note 113): black with a red glow from above the top, heading 32 px, tiles 168 px apart,
        // 32 px under the heading, names 23 px under the pictures, the button 52 px under the names
        <div class="fixed inset-0 z-(--z-screen) flex select-none flex-col items-center justify-center text-ink" style="background: var(--color-void) radial-gradient(2170px at 50% -54dvh, var(--color-gate-glow) 19%, var(--color-gate-glow-far) 60%, transparent 100%);">
            // the profiles themselves, one form at a time (note 114): the website's screen or the phone's picker
            <crate::form::ByForm
                phone=move || view! {
                    <phone::PhoneProfilePicker
                        profiles=profiles_signal
                        editing=edit_mode
                        manage=manage
                        on_select=Callback::new(move |p| on_select_tile(p))
                        on_add=Callback::new(move |_| set_modal_state.set(Some(("add".to_string(), None))))
                        on_manage=on_manage
                    />
                }
                desktop=move || view! {
                    <website::WebsiteGate
                        profiles=profiles_signal
                        editing=edit_mode
                        on_select=Callback::new(move |p| on_select_tile(p))
                        on_add=Callback::new(move |_| set_modal_state.set(Some(("add".to_string(), None))))
                        on_manage=on_manage
                        kids=!manage && crate::store::kids_gate()
                    />
                }
            />

            // PIN prompt modal (Task 098)
            <Show when=move || pin_prompt_profile.get().is_some()>
                {move || pin_prompt_profile.get().map(|p| view! {
                    <ProfilePinPrompt
                        profile=p
                        on_unlock=Callback::new(move |_| on_unlock_pin())
                        on_cancel=Callback::new(move |_| { set_pin_prompt_profile.set(None); set_pin_then_edit.set(false); })
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
