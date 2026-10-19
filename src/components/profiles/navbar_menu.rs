use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::store::use_profile_store;

#[component]
pub fn NavbarProfileMenu() -> impl IntoView {
    let profile_store = use_profile_store();
    let profiles = profile_store.profiles;
    let active_profile_id = profile_store.active_profile_id;
    let (is_open, set_is_open) = signal(false);
    let navigate = use_navigate();

    let active_profile = move || {
        let id = active_profile_id.get()?;
        profiles.get().into_iter().find(|p| p.id == id)
    };

    let is_kids = Memo::new(move |_| active_profile().is_some_and(|p| p.is_kids));
    // Switch profile goes back to the page that asks who is watching
    let nav_to_gate = navigate.clone();
    let switch_gate = Callback::new(move |_: ()| {
        set_is_open.set(false);
        active_profile_id.set(None);
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item(crate::keys::ACTIVE_PROFILE);
        }
        nav_to_gate(crate::config::routes::BROWSE, Default::default());
    });

    let other_profiles = move || {
        let id = active_profile_id.get().unwrap_or_default();
        profiles.get().into_iter().filter(|p| p.id != id).collect::<Vec<_>>()
    };

    let logout = move |_| {
        set_is_open.set(false);
        // the same sign out as the settings page (the session is kept for a day, sealed, when "Remember me" was on)
        crate::pages::settings::hub::sign_out_now();
    };

    let switch_profile = move |id: String| {
        set_is_open.set(false);
        match profiles.get_untracked().into_iter().find(|p| p.id == id) {
            Some(p) => crate::components::profiles::switch_overlay::begin_switch(p),
            None => crate::store::activate_profile(&id),
        }
    };

    // Manage Profiles opens the account Profiles page (parental controls, transfer, one row per profile, Add Profile), as on the Netflix website.
    // The page of profiles with pencils (`/profiles/manage`) is the "Manage Profiles" button of Who's watching.
    let nav_to_manage = navigate.clone();
    let manage_profiles = move |_| {
        set_is_open.set(false);
        nav_to_manage(&crate::pages::settings::Screen::AccountProfiles.path(), Default::default());
    };

    let nav_to_account = navigate.clone();
    let nav_to_transfer = navigate.clone();
    let nav_to_help = navigate.clone();

    // The menu opens and closes on click, and closes on a click elsewhere or Escape.
    let root = NodeRef::<leptos::html::Div>::new();
    crate::utils::listen::listen_window(leptos::ev::click, move |ev| {
        if !is_open.get_untracked() {
            return;
        }
        use wasm_bindgen::JsCast;
        let inside = ev.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()).zip(root.get_untracked()).is_some_and(|(n, r)| r.contains(Some(&n)));
        if !inside {
            set_is_open.set(false);
        }
    });
    crate::utils::listen::listen_window(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            set_is_open.set(false);
        }
    });

    view! {
        <div
            node_ref=root
            class="relative flex items-center group/menu select-none"
        >
            <button
                type="button"
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| set_is_open.update(|o| *o = !*o)
                class="flex cursor-pointer items-center"
            >
                <div data-profile-avatar="1" class="w-8 h-8 rounded-control overflow-hidden">
                    {move || {
                        let p = active_profile();
                        if p.as_ref().map(|p| p.is_kids && p.avatar_url.is_none()).unwrap_or(false) {
                            view! { <crate::components::profiles::kids_avatar::KidsAvatar size=32.0 /> }.into_any()
                        } else {
                            let url = p.and_then(|p| p.avatar_url).unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
                            view! { <img src=url class="w-full h-full object-cover" alt=crate::i18n::t("navbar-menu-current-profile") /> }.into_any()
                        }
                    }}
                </div>
                <i class="ph-fill ph-caret-down text-ink-silver text-fine ms-3 me-1 transition-transform duration-(--motion-base)"
                   class=("rotate-180", move || is_open.get())></i>
            </button>

            // Dropdown menu, laid out like the Netflix one: 250px wide, 52px rows, 32px avatars and icons, a line above Sign out
            <div
                role="menu"
                class=move || format!(
                    "absolute top-[calc(100%+8px)] right-0 w-[250px] origin-top-right rounded-modal bg-field py-1 shadow-popover transition-all duration-(--motion-base) z-50 {}",
                    if is_open.get() { "opacity-100 scale-100 pointer-events-auto" } else { "opacity-0 scale-95 pointer-events-none" }
                )
            >
                {move || is_kids.get().then(|| view! {
                    // a Kids profile has a short menu: Switch profile, Help Centre, Sign out
                    <button type="button" role="menuitem" on:click=move |_| switch_gate.run(()) class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline">
                        <span class="text-body-lg text-ink">{crate::i18n::t("set-menu-switch")}</span>
                    </button>
                    <a role="menuitem" href="/help" class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline">
                        <i class="ph ph-question flex h-8 w-8 shrink-0 items-center justify-center text-heading-lg text-ink-silver" aria-hidden="true"></i>
                        <span class="text-body-lg text-ink">{crate::i18n::t("footer-help")}</span>
                    </a>
                })}
                <div class=move || if is_kids.get() { "hidden" } else { "contents" }>
                // Other profiles
                {move || other_profiles().into_iter().map(|p| {
                    let id = p.id.clone();
                    let name = p.name.clone();
                    let is_kids = p.is_kids;
                    let has_custom_avatar = p.avatar_url.is_some();
                    let avatar = p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
                    view! {
                        <button
                            type="button"
                            role="menuitem"
                            class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline"
                            on:click=move |_| switch_profile(id.clone())
                        >
                            <span class="h-8 w-8 shrink-0 overflow-hidden rounded-control">
                                {if is_kids && !has_custom_avatar {
                                    view! { <crate::components::profiles::kids_avatar::KidsAvatar size=32.0 /> }.into_any()
                                } else {
                                    view! { <img src=avatar class="h-full w-full object-cover" alt=name.clone() /> }.into_any()
                                }}
                            </span>
                            <span class="truncate text-body-lg text-ink">{name}</span>
                        </button>
                    }
                }).collect::<Vec<_>>()}

                <button type="button" role="menuitem" on:click=manage_profiles class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline">
                    <i class="ph ph-pencil-simple flex h-8 w-8 shrink-0 items-center justify-center text-heading-lg text-ink-silver" aria-hidden="true"></i>
                    <span class="text-body-lg text-ink">{crate::i18n::t("profile-manage")}</span>
                </button>

                <button type="button" role="menuitem" on:click=move |_| {
                        set_is_open.set(false);
                        // the transfer screen of the profile in use (the same page the profile settings open)
                        let id = active_profile_id.get_untracked().unwrap_or_default();
                        nav_to_transfer(&format!("/settings/profile/{id}/transfer"), Default::default());
                    } class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline">
                    <i class="ph ph-user-switch flex h-8 w-8 shrink-0 items-center justify-center text-heading-lg text-ink-silver" aria-hidden="true"></i>
                    <span class="text-body-lg text-ink">{crate::i18n::t("profile-transfer")}</span>
                </button>

                <button
                    type="button"
                    role="menuitem"
                    on:click=move |_| {
                        set_is_open.set(false);
                        nav_to_account("/settings/account", Default::default());
                    }
                    class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline"
                >
                    <i class="ph ph-user flex h-8 w-8 shrink-0 items-center justify-center text-heading-lg text-ink-silver" aria-hidden="true"></i>
                    <span class="text-body-lg text-ink">{crate::i18n::t("help-ui-account")}</span>
                </button>

                <button
                    type="button"
                    role="menuitem"
                    on:click=move |_| {
                        set_is_open.set(false);
                        nav_to_help("/help", Default::default());
                    }
                    class="flex w-full cursor-pointer items-center gap-3 px-4 py-2.5 text-start hover:underline"
                >
                    <i class="ph ph-question flex h-8 w-8 shrink-0 items-center justify-center text-heading-lg text-ink-silver" aria-hidden="true"></i>
                    <span class="text-body-lg text-ink">{crate::i18n::t("footer-help")}</span>
                </button>

                </div>

                // Sign out
                <div class="mt-1 border-t border-ink/15">
                    <button type="button" role="menuitem" on:click=logout class="w-full cursor-pointer px-4 py-4 text-start text-body-lg text-ink hover:underline">
                        {crate::i18n::t_args("profile-sign-out", &[("brand", fluent_bundle::FluentValue::from(crate::brand::NAME))])}
                    </button>
                </div>
            </div>
        </div>
    }
}
