//! The page behind "Delete account": a short explanation, one card to open, a box to tick and the red button. On the website it is a page of its
//! own (no side menu), measured on netflix.com/account/delete/confirm after the check of who is asking.
use super::hub::{persist_profiles, sign_out_now};
use super::prefs::use_prefs;
use super::widgets::{CollapsibleCard, Message, Tone};
use super::account::error_text;
use super::Screen;
use crate::config::routes;
use crate::i18n::t;
use crate::services::account;
use crate::store::use_profile_store;
use leptos::prelude::*;

#[component]
pub fn DeleteAccount() -> impl IntoView {
    let note = RwSignal::new(None::<(Tone, String)>);
    let profiles = use_profile_store();
    let prefs = use_prefs();
    let delete = Callback::new(move |_| {
        leptos::task::spawn_local(async move {
            match account::delete_account().await {
                Ok(()) => {
                    profiles.profiles.set(Vec::new());
                    persist_profiles(profiles);
                    prefs.forget_all();
                    sign_out_now();
                }
                Err(e) => note.set(Some((Tone::Error, error_text(&e)))),
            }
        });
    });
    let agreed = RwSignal::new(false);
    let brand = crate::brand::NAME;
    // from 1024 px this is a page of its own like the website's: no side menu, a back arrow, one 690 px column
    view! {
        <div class="lg:fixed lg:inset-x-0 lg:bottom-0 lg:top-settings-header lg:z-(--z-screen) lg:overflow-auto lg:bg-page">
            <div class="relative w-full max-w-[690px] max-lg:max-w-md lg:mx-auto lg:pt-[19px]">
                <a href=Screen::AccountSecurity.path() aria-label=t("set-back") class="hidden h-10 w-10 items-center justify-center rounded-full text-ink hover:bg-ink/10 lg:absolute lg:-left-[266px] lg:top-[19px] lg:flex">
                    <i class="ph-bold ph-arrow-left text-2xl rtl:-scale-x-100" aria-hidden="true"></i>
                </a>
                <h1 class="mb-2 hidden text-heading-xl font-bold leading-[41px] text-ink/80 lg:block">{t("set-title-delete")}</h1>
                <p class="mb-8 text-title-sm leading-[27px] text-ink/80">{crate::i18n::t_args("set-del-intro", &[("brand", brand.into())])}</p>
                <Message text=note />
                <CollapsibleCard title=t("set-del-card") show_label=t("set-del-show") hide_label=t("set-del-hide")>
                    <ul class="mt-4 list-disc ps-[18px] text-body-md leading-5 text-ink/80">
                        <li class="mb-[5px]">{t("set-del-lose")}</li>
                        <li>{t("set-del-when")}</li>
                    </ul>
                    <hr class="mx-[18px] mb-[17px] mt-[16px] border-ink/20" />
                    <label class="flex cursor-pointer items-start gap-[9px] text-body-md leading-6">
                        <input type="checkbox" prop:checked=move || agreed.get() on:change=move |ev| agreed.set(event_target_checked(&ev)) class="mt-0 h-6 w-6 shrink-0 cursor-pointer appearance-none rounded-hairline border border-ink/60 bg-page checked:border-ink checked:bg-ink" />
                        <span>{crate::i18n::t_args("set-del-check", &[("brand", brand.into())])}</span>
                    </label>
                    <button type="button" disabled=move || !agreed.get() on:click=move |_| delete.run(()) class="mb-[10px] mt-4 h-10 w-full rounded-control border border-ink/30 bg-page px-4 text-body-md font-medium text-[#c11119] disabled:cursor-not-allowed">{t("set-del-button")}</button>
                </CollapsibleCard>
                <p class="mt-9 text-label text-ink/70">{t("set-del-questions")}" "<a href=routes::HELP class="text-[#0080ff]">{t("set-del-help")}</a>"."</p>
            </div>
        </div>
    }
}
