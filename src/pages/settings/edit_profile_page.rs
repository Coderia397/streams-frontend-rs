//! The website's Edit Profile page: a page of its own (only the logo above it), the picture with a pencil, the name, the gender, the game handle, the contact info of the
//! account (name, email, mobile phone), Save, Cancel and Delete Profile. Measured on netflix.com/settings/profile/edit/<id> at 1851 px wide: a 690 px column. As there
//! (read live on 2026-10-27, note 138) the picture's pencil opens "Choose a profile icon" in place, and the game handle is a field of this page, saved with it.
use super::profile::{remove_profile, valid_handle, IconPanel};
use super::prefs::use_prefs;
use super::widgets::{toast, ConfirmDialog, FloatField};
use super::Screen;
use crate::config::{routes, GAME_HANDLE_MAX, GAME_HANDLE_MIN, PROFILE_NAME_MAX};
use crate::i18n::t;
use crate::models::profile_rules::can_delete;
use crate::services::account;
use crate::store::use_profile_store;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

const GENDERS: [(&str, &str); 3] = [("woman", "set-gender-woman"), ("man", "set-gender-man"), ("none", "set-gender-none")];

#[component]
pub fn EditProfilePage(id: String) -> impl IntoView {
    let store = use_profile_store();
    let prefs = use_prefs();
    let navigate = use_navigate();
    let id = StoredValue::new(id);
    let profile = Memo::new(move |_| store.profiles.with(|l| l.iter().find(|p| p.id == id.get_value()).cloned()));
    let name = RwSignal::new(profile.get_untracked().map(|p| p.name).unwrap_or_default());
    let gender = RwSignal::new(prefs.profile(&id.get_value()).gender.unwrap_or_default());
    let gender_open = RwSignal::new(false);
    let handle = RwSignal::new(prefs.profile(&id.get_value()).game_handle.unwrap_or_default());
    let choosing_icon = RwSignal::new(false);
    let confirm = RwSignal::new(false);
    let note = RwSignal::new(None::<String>);
    let session = account::session();
    let (email, phone) = (session.as_ref().map(|s| s.email.clone()).unwrap_or_default(), session.as_ref().map(|s| s.phone.clone()).unwrap_or_default());
    let full_name = session.as_ref().map(|s| format!("{} {}", s.first_name, s.last_name).trim().to_string()).unwrap_or_default();

    let back = move || Screen::Profile(id.get_value()).path();
    let nav_save = navigate.clone();
    let save = move |_| {
        let value = name.get_untracked().trim().chars().take(PROFILE_NAME_MAX).collect::<String>();
        if value.is_empty() {
            note.set(Some(t("set-profile-name-required")));
            return;
        }
        if store.profiles.with_untracked(|l| l.iter().any(|p| p.id != id.get_value() && p.name.trim().to_lowercase() == value.to_lowercase())) {
            note.set(Some(t("set-name-taken")));
            return;
        }
        let h = handle.get_untracked().trim().to_string();
        let held = prefs.profile(&id.get_value()).game_handle.unwrap_or_default();
        if h != held && !valid_handle(&h) {
            note.set(Some(crate::i18n::t_args("set-handle-invalid", &[("min", GAME_HANDLE_MIN.into()), ("max", GAME_HANDLE_MAX.into())])));
            return;
        }
        store.profiles.update(|l| { if let Some(p) = l.iter_mut().find(|p| p.id == id.get_value()) { p.name = value; } });
        let g = gender.get_untracked();
        prefs.update_profile(&id.get_value(), |p| {
            p.gender = (!g.is_empty()).then(|| g.clone());
            if h != held { p.game_handle = Some(h.clone()); }
        });
        super::hub::persist_profiles(store);
        toast(t("set-saved"));
        nav_save(&back(), Default::default());
    };
    let nav_cancel = navigate.clone();
    let delete = Callback::new({
        let navigate = navigate.clone();
        move |_| {
            remove_profile(&id.get_value());
            navigate(&Screen::AccountProfiles.path(), Default::default());
        }
    });
    let deletable = move || store.profiles.with(|l| can_delete(l, &id.get_value()));
    let field_row = |icon: &'static str, title: String, value: String, empty: String, href: String| {
        let shown = if value.is_empty() { empty } else { value.clone() };
        let warn = value.is_empty();
        view! {
            <a href=href class="flex items-center gap-4 px-4 py-[14px] hover:bg-chrome/5">
                <i class=format!("ph ph-{icon} shrink-0 text-heading text-ink/70") aria-hidden="true"></i>
                <span class="min-w-0 flex-1">
                    <span class="block text-body-md font-medium leading-[24px] text-ink">{title}</span>
                    <span class="block text-fine leading-[22px]" class=("text-warning", warn) class=("text-ink/70", !warn)>{shown}</span>
                </span>
                <i class="ph ph-caret-right shrink-0 text-body-md text-ink" aria-hidden="true"></i>
            </a>
        }
    };
    view! {
        <div class="lg:fixed lg:inset-0 lg:z-(--z-screen) lg:overflow-auto lg:bg-page">
            <header class="hidden h-settings-header items-center border-b border-chrome/10 set-gutter lg:flex">
                <a href=routes::BROWSE aria-label=crate::brand::NAME><img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="h-7 w-auto" /></a>
            </header>
            <div class="mx-auto w-full max-w-[690px] pb-16 pt-[24px] max-lg:max-w-md">
                <h1 class="mb-[33px] text-display font-bold leading-[51px] text-ink">{t("set-edit-profile-page")}</h1>
                {move || note.get().map(|m| view! { <p role="status" class="mb-4 rounded-control border border-danger/50 bg-danger/10 px-4 py-3 text-body-sm text-danger">{m}</p> })}
                <div class="mb-[24px] flex items-start gap-[20px]">
                    // radius: the avatar tile
                    <button type="button" on:click=move |_| choosing_icon.set(true) class="group relative -ms-[4px] block h-[128px] w-[128px] shrink-0 overflow-hidden rounded-control" aria-label=t("set-title-avatar")>
                        {move || profile.get().map(|p| view! { <img src=p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string()) alt="" class="h-full w-full object-cover" /> })}
                        <span class="absolute left-1/2 top-1/2 flex h-12 w-12 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full bg-chrome/50"><i class="ph ph-pencil-simple text-heading text-chrome-ink" aria-hidden="true"></i></span>
                    </button>
                    <div class="flex min-w-0 flex-1 flex-col gap-[8px]">
                        <FloatField label=t("set-profile-name") value=name maxlength=PROFILE_NAME_MAX as u32 />
                        <div class="relative">
                            <button type="button" on:click=move |_| gender_open.update(|o| *o = !*o) aria-haspopup="listbox" aria-expanded=move || gender_open.get().to_string() class="flex h-[56px] w-full items-center justify-between rounded-button border border-scrim-edge bg-page px-4 text-start">
                                <span class="flex flex-col"><span class="text-fine leading-[14px] text-ink/70">{t("set-gender")}</span><span class="text-body-md leading-[22px] text-ink">{move || GENDERS.iter().find(|(k, _)| *k == gender.get()).map(|(_, l)| t(l)).unwrap_or_default()}</span></span>
                                <i class="ph-fill ph-caret-down text-fine" aria-hidden="true"></i>
                            </button>
                            <Show when=move || gender_open.get()>
                                <ul role="listbox" class="absolute inset-x-0 top-[calc(100%+3px)] z-10 rounded-menu border border-scrim-edge bg-page py-2">
                                    {GENDERS.iter().map(|(k, l)| { let k = *k; view! { <li role="option"><button type="button" on:click=move |_| { gender.set(k.to_string()); gender_open.set(false); } class="block w-full px-4 py-4 text-start text-body-md font-medium hover:bg-chrome/5">{t(l)}</button></li> } }).collect_view()}
                                </ul>
                            </Show>
                        </div>
                    </div>
                </div>

                <h2 class="text-heading font-medium leading-[30px] text-ink">{t("set-title-game-handle")}</h2>
                <p class="mb-4 mt-4 text-body-md leading-6 text-ink/70">{crate::i18n::t_args("set-handle-lead", &[("brand", crate::brand::NAME.into())])}" "<a href=routes::HELP class="underline underline-offset-2">{t("set-learn-more")}</a></p>
                <label class="relative flex h-[56px] w-full items-center gap-4 rounded-control border border-scrim-edge bg-page px-4 focus-within:border-ink">
                    <i class="ph ph-game-controller shrink-0 text-heading" aria-hidden="true"></i>
                    <span class="flex min-w-0 flex-1 flex-col">
                        <span class="text-fine leading-[14px] text-ink/70">{t("set-title-game-handle")}</span>
                        <input type="text" maxlength=GAME_HANDLE_MAX as u32 autocomplete="off" spellcheck="false" placeholder=t("set-handle-create") prop:value=move || handle.get() on:input=move |e| handle.set(event_target_value(&e)) class="w-full bg-transparent text-body-md leading-[22px] text-ink outline-none placeholder:text-ink/70" />
                    </span>
                </label>

                <hr class="my-[32px] border-chrome/20" />
                <h2 class="mb-[16px] text-title-sm leading-[27px] text-line">{t("set-contact-info")}</h2>
                <div class="divide-y divide-chrome/10 rounded-card border border-chrome/10 px-[4px]">
                    {field_row("user", t("set-title-name"), full_name, t("set-add-your-name"), Screen::Name(id.get_value()).path())}
                    {field_row("envelope-simple", t("set-title-email"), email, String::new(), Screen::Email.path())}
                    {field_row("device-mobile", t("set-title-phone"), phone, t("set-add-phone"), Screen::Phone.path())}
                </div>

                <div class="mt-[72px] flex flex-col gap-2">
                    <button type="button" on:click=save class="h-10 w-full rounded-button bg-chrome text-body-md font-medium text-chrome-ink">{t("set-save")}</button>
                    <button type="button" on:click=move |_| nav_cancel(&Screen::Profile(id.get_value()).path(), Default::default()) class="h-10 w-full rounded-button text-body-md font-medium text-ink">{t("set-cancel")}</button>
                </div>
                <hr class="mb-[24px] mt-[32px] border-chrome/20" />
                <button type="button" disabled=move || !deletable() on:click=move |_| confirm.set(true) class="h-10 w-full rounded-button text-body-md font-medium text-danger disabled:text-chrome/40">{t("set-po-delete-profile")}</button>
                <Show when=move || !deletable()><p class="mt-[16px] text-body-sm text-chrome/70">{t("set-primary-cannot-delete")}</p></Show>
            </div>
            <ConfirmDialog open=confirm title=t("set-delete-profile") body=t("set-delete-profile-body") confirm_label=t("set-delete") danger=true on_confirm=delete />
            <Show when=move || choosing_icon.get()>
                <IconPanel id=id.get_value() on_done=Callback::new(move |_| choosing_icon.set(false)) />
            </Show>
        </div>
    }
}
