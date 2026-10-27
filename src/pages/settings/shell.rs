//! The frame of every account and settings page: the website's top bar, the side menu and the page heading with its back arrow.
//! Phones and tablets get the app's frame (a black bar with the logo, a back arrow that climbs the phone tree); from 1024 px wide
//! the website's frame is used, measured on netflix.com (see notes/76 and style/components.css `.set-gutter`).
use super::account_menu::AccountMenu;
use super::{sidebar, Screen};
use crate::config::routes;
use crate::i18n::t;
use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// The black bar of a phone with the logo and a back arrow.
#[component]
pub fn PhoneBar(screen: Memo<Screen>) -> impl IntoView {
    view! {
        <div class="relative flex h-settings-header items-center bg-chrome px-2 lg:hidden">
            <a href=move || screen.get().parent() aria-label=move || t("set-back") class="flex h-10 w-10 items-center justify-center text-chrome-ink">
                <AppIcon glyph=Glyph::Back class="text-2xl rtl:-scale-x-100" />
            </a>
            <img src=crate::brand::LOGO_WORDMARK alt=crate::brand::NAME class="absolute inset-x-0 mx-auto h-6 w-auto" />
        </div>
    }
}

/// The website's top bar: the logo on the left, the profile button and its menu on the right.
#[component]
pub fn TopBar(avatar: Memo<String>, signout_open: RwSignal<bool>) -> impl IntoView {
    view! {
        <header class="mx-auto hidden h-settings-header items-center justify-between lg:flex set-gutter">
            <a href=routes::BROWSE aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="h-7 w-auto" /></a>
            <AccountMenu avatar=avatar signout_open=signout_open />
        </header>
    }
}

/// The menu on the left of the website pages: Back, then Overview, Security, Devices, Profiles and the app's own entries.
#[component]
pub fn SideMenu(screen: Memo<Screen>, signout_open: RwSignal<bool>) -> impl IntoView {
    let row = "flex h-settings-menu-item items-center gap-3 rounded-control px-3 text-body-md font-medium hover:bg-ink/5 lg:px-4";
    view! {
        <aside class="hidden shrink-0 lg:block lg:w-[27.55%]">
            <a href=routes::BROWSE class="mb-[33px] flex h-settings-menu-item items-center gap-2 rounded-button px-4 text-body-sm hover:bg-ink/5">
                <AppIcon glyph=Glyph::Back class="text-xl rtl:-scale-x-100" />
                {move || crate::i18n::t_args("set-back-to-app", &[("brand", crate::brand::NAME.into())])}
            </a>
            <nav class="flex flex-col gap-2" aria-label=move || t("set-title")>
                {sidebar()
                    .into_iter()
                    .map(|(icon, key, target, section)| view! {
                        <a
                            href=target.path()
                            class=row
                            class=("text-ink", move || screen.get().section() == section)
                            class=("text-ink/70", move || screen.get().section() != section)
                            aria-current=move || (screen.get().section() == section).then_some("page")
                        >
                            <i class=move || format!("{} ph-{icon} text-2xl", if screen.get().section() == section { "ph-fill" } else { "ph" }) aria-hidden="true"></i>
                            {move || t(key)}
                        </a>
                    })
                    .collect_view()}
                <a href=routes::HELP class=format!("{row} text-ink/70")>
                    <i class="ph ph-question text-2xl" aria-hidden="true"></i>
                    {move || t("set-nav-help")}
                </a>
                <button type="button" on:click=move |_| signout_open.set(true) class=format!("{row} text-start text-ink/70")>
                    <i class="ph ph-sign-out text-2xl" aria-hidden="true"></i>
                    {move || t("set-sign-out")}
                </button>
            </nav>
        </aside>
    }
}

/// The heading of a page: a back arrow (up one level of the phone tree on a phone, up one level of the website tree from 1024 px)
/// and the title. The title steps aside while the check of who is asking is showing, and on the Profile transfer page.
#[component]
pub fn PageHeading(screen: Memo<Screen>, gate_active: RwSignal<bool>, light_phone: Signal<bool>) -> impl IntoView {
    // a profile's page opened from Manage Profiles while no profile is in use goes back there, as Netflix's "Manage profile and
    // preferences" goes back to its ManageProfiles (note 132)
    let profiles = crate::store::use_profile_store();
    let wide_back = move || match screen.get() {
        Screen::Profile(id) if profiles.active_profile_id.get().is_none() && crate::store::manage_edit().as_deref() == Some(id.as_str()) => "/profiles/manage".to_string(),
        other => other.parent_wide(),
    };
    view! {
        // Manage profile and preferences has no line under its title: the card keeps Netflix's 20 px from it (owner, 2026-10-26)
        <header class="flex items-center gap-3 max-lg:mb-1" class=("max-lg:mb-4", move || light_phone.get()) class=("lg:mb-0", move || !matches!(screen.get(), Screen::Profile(_))) class=("lg:mb-5", move || matches!(screen.get(), Screen::Profile(_)))>
            <a
                href=move || screen.get().parent()
                aria-label=move || t("set-back")
                class="btn-circle btn-circle-clear btn-circle-md lg:hidden"
                class=("max-lg:hidden", move || light_phone.get())
            >
                <AppIcon glyph=Glyph::Back class="text-2xl rtl:-scale-x-100" />
            </a>
            <a
                href=wide_back
                aria-label=move || t("set-back")
                class="hidden btn-circle btn-circle-clear btn-circle-md lg:flex"
                class=("lg:hidden", move || matches!(screen.get(), Screen::Account | Screen::AccountSecurity | Screen::AccountDevices | Screen::AccountProfiles))
                class=("lg:absolute", move || screen.get().standalone())
                class=("lg:left-[210px]", move || screen.get().standalone())
                class=("lg:top-[97px]", move || screen.get().standalone())
            >
                <AppIcon glyph=Glyph::Back class="text-2xl rtl:-scale-x-100" />
            </a>
            <h1 class="text-title font-bold lg:w-full lg:flex-1 lg:truncate lg:text-account-title" class=("max-lg:text-account-title-sm", move || light_phone.get()) class=("lg:text-heading-xl!", move || screen.get().standalone()) class=("lg:leading-[40px]!", move || screen.get().standalone()) class=("lg:text-line!", move || screen.get().standalone()) class=("hidden", move || gate_active.get() || screen.get() == Screen::TransferSetting)>
                <span class=("lg:hidden", move || matches!(screen.get(), Screen::Profile(_)))>{move || t(screen.get().title_key())}</span>
                <span class="hidden" class=("lg:inline", move || matches!(screen.get(), Screen::Profile(_)))>{t("set-title-manage-profile")}</span>
            </h1>
        </header>
    }
}

/// The bar above a standalone page: the logo on the left and the profile menu on the right, from 1024 px wide.
#[component]
pub fn StandaloneBar(screen: Memo<Screen>, avatar: Memo<String>, signout_open: RwSignal<bool>) -> impl IntoView {
    view! {
        <header class=move || if screen.get().standalone() { "hidden h-settings-header items-center justify-between border-b border-chrome/10 set-gutter lg:flex" } else { "hidden" }>
            <a href=routes::BROWSE aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="h-7 w-auto" /></a>
            <AccountMenu avatar=avatar signout_open=signout_open />
        </header>
    }
}

/// "For <name>" and the profile picture under the title of a standalone page.
#[component]
pub fn ForProfile(screen: Memo<Screen>) -> impl IntoView {
    let store = crate::store::use_profile_store();
    move || {
        let s = screen.get();
        if !s.standalone() {
            return None;
        }
        let id = s.profile_of().or_else(|| store.active_profile_id.get());
        let p = store.profiles.with(|l| id.and_then(|id| l.iter().find(|p| p.id == id).cloned()).or_else(|| l.first().cloned()))?;
        let pic = p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
        Some(view! {
            <p class="mb-[44px] mt-[10px] hidden items-center gap-3 text-title-sm leading-[27px] text-chrome/70 lg:flex">
                {crate::i18n::t_args("set-for-profile", &[("name", p.name.clone().into())])}
                <img src=pic alt="" class="h-8 w-8 rounded-full object-cover" />
            </p>
        })
    }
}
