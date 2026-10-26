//! The profile button of the website's top bar and its menu.
use super::Screen;
use crate::config::routes;
use crate::i18n::t;
use crate::store::use_profile_store;
use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The profile button of the website's top bar: the profile picture and a caret that open a small menu (Back, Account, Manage Profiles,
/// Help Centre, Switch Profile, Sign out), 199 px wide, white, rounded-control 8 px, with a 48 px row for each entry.
#[component]
pub fn AccountMenu(avatar: Memo<String>, signout_open: RwSignal<bool>) -> impl IntoView {
    let open = RwSignal::new(false);
    let root = NodeRef::<leptos::html::Div>::new();
    let profiles = use_profile_store();
    crate::utils::listen::listen_window(leptos::ev::click, move |ev| {
        if !open.get_untracked() {
            return;
        }
        use wasm_bindgen::JsCast;
        let inside = ev.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()).zip(root.get_untracked()).is_some_and(|(n, r)| r.contains(Some(&n)));
        if !inside {
            open.set(false);
        }
    });
    crate::utils::listen::listen_window(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            open.set(false);
        }
    });
    let item = "flex h-12 w-full items-center gap-4 rounded-menu-item px-3 text-start text-body-md font-medium text-ink hover:bg-ink/5";
    let switch = Callback::new(move |_: ()| {
        // Switch Profile brings up the page that asks who is watching over this page, and the pick comes back here (store::activate_profile)
        open.set(false);
        crate::store::ask_who_is_watching(profiles);
    });
    view! {
        <div node_ref=root class="relative">
            <button type="button" aria-haspopup="menu" aria-expanded=move || open.get().to_string() aria-label=move || t("set-nav-profile") on:click=move |_| open.update(|o| *o = !*o) class="flex h-9 w-[68px] items-center justify-center gap-2 rounded-button hover:bg-ink/5">
                <img src=move || avatar.get() alt="" class="h-8 w-8 rounded-control object-cover" />
                <i class=move || if open.get() { "ph-fill ph-caret-up text-fine" } else { "ph-fill ph-caret-down text-fine" } aria-hidden="true"></i>
            </button>
            <Show when=move || open.get()>
                <div role="menu" class="absolute end-0 top-[42px] z-50 w-[199px] rounded-menu bg-page p-[5px] shadow-menu">
                    <a role="menuitem" href=routes::BROWSE class=item><AppIcon glyph=Glyph::Back class="text-heading" />{move || crate::i18n::t_args("set-back-to-app", &[("brand", crate::brand::NAME.into())])}</a>
                    <hr class="mx-3 my-[4px] border-set-line" />
                    <a role="menuitem" href=Screen::Account.path() class=item><i class="ph ph-user text-heading" aria-hidden="true"></i>{t("set-nav-account")}</a>
                    <a role="menuitem" href=Screen::AccountProfiles.path() class=item><i class="ph ph-pencil-simple text-heading" aria-hidden="true"></i>{t("profile-manage")}</a>
                    <a role="menuitem" href=routes::HELP class=item><i class="ph ph-question text-heading" aria-hidden="true"></i><span class="flex-1">{t("set-menu-help")}</span></a>
                    <hr class="mx-3 my-[4px] border-set-line" />
                    <button type="button" role="menuitem" on:click=move |_| switch.run(()) class=item><span class="flex-1">{t("set-menu-switch")}</span><i class="ph ph-caret-right text-body-md rtl:-scale-x-100" aria-hidden="true"></i></button>
                    <button type="button" role="menuitem" on:click=move |_| { open.set(false); signout_open.set(true) } class=item>{t("set-sign-out")}</button>
                </div>
            </Show>
        </div>
    }
}
