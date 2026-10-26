//! The settings hub (the Profile sheet of the Netflix app) and Manage Profiles.
use super::widgets::{ActionRow, LinkRow, Section};
use super::Screen;
use crate::components::profiles::add_edit_modal::{AddEditProfileModal, ProfileFormData};
use crate::i18n::t;
use crate::models::profile::Profile;
use crate::store::{use_profile_store, ProfileStore};
use leptos::prelude::*;
use crate::components::media::icons::app_icons::{AppIcon, Glyph};

/// Opens the "Sign out?" dialog that the settings page renders once.
#[derive(Clone, Copy)]
pub struct SignOutOpen(pub RwSignal<bool>);

/// Non kids profiles an account can hold. The Kids profile does not count toward it.
pub const MAX_PROFILES: usize = 5;

/// Keeps the profiles in this browser and sends the change to the account.
pub fn persist_profiles(store: ProfileStore) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(json) = serde_json::to_string(&store.profiles.get_untracked()) {
            let _ = storage.set_item(crate::keys::PROFILES, &json);
        }
    }
    if crate::services::account::has_account() {
        crate::services::sync::push_profiles(store);
    }
}

/// Signs out of the account and returns to the landing page with nothing left of this person's choices in the page.
pub fn sign_out_now() {
    leptos::task::spawn_local(async {
        // the session ends; the rule in lib.rs clears this person's data from the browser and loads the start page
        crate::services::account::sign_out_async().await;
    });
}

/// Makes `profile` the one in use. A profile with a PIN goes through the Who's watching screen so the PIN is asked.
pub fn switch_to(store: ProfileStore, profile: &Profile) {
    if profile.pin.is_some() {
        store.active_profile_id.set(None);
        return;
    }
    store.is_kids_mode.set(profile.is_kids);
    // the same switch as everywhere else: the phone and tablet flight, the Kids screen, or the desktop screen
    crate::components::profiles::switch_overlay::begin_switch(profile.clone());
}

fn avatar_of(p: &Profile) -> String {
    p.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string())
}

/// A profile's picture: its own, or the Kids picture for a Kids profile without one (not the empty silhouette).
fn picture(p: &Profile, size: f64, class: &'static str) -> AnyView {
    if p.is_kids && p.avatar_url.is_none() {
        view! { <span class=class><crate::components::profiles::kids_avatar::KidsAvatar size=size /></span> }.into_any()
    } else {
        view! { <img src=avatar_of(p) alt="" class=class /> }.into_any()
    }
}

#[component]
pub fn Hub() -> impl IntoView {
    let store = use_profile_store();
    let signout = use_context::<SignOutOpen>();
    let active = Memo::new(move |_| store.profiles.with(|l| store.active_profile_id.with(|id| l.iter().find(|p| Some(&p.id) == id.as_ref()).or(l.first()).cloned())));
    let others = Memo::new(move |_| {
        let id = active.get().map(|p| p.id);
        store.profiles.with(|l| l.iter().filter(|p| Some(&p.id) != id.as_ref()).cloned().collect::<Vec<_>>())
    });

    view! {
        <div class="set-sheet">
            {move || active.get().map(|p| view! {
                <div class="relative mb-6 flex flex-col items-center gap-3 rounded-modal bg-set-sheet-tile p-6 lg:bg-set-card">
                    <a href=Screen::Profile(p.id.clone()).path() class="flex flex-col items-center gap-3" aria-label=t("set-title-edit-profile")>
                        // radius: a profile picture, not the link's shape
                        {picture(&p, 96.0, "block h-24 w-24 overflow-hidden rounded-card object-cover")}
                        <span class="max-w-full truncate text-heading font-bold">{p.name.clone()}</span>
                    </a>
                    <a href=Screen::Profile(p.id.clone()).path() class="absolute end-4 top-4 btn-circle btn-circle-clear btn-circle-md" aria-label=t("set-title-edit-profile")>
                        <i class="ph-bold ph-pencil-simple text-2xl" aria-hidden="true"></i>
                    </a>
                </div>
            })}
            <ul class="mb-6 flex flex-wrap justify-center gap-5">
                {move || others.get().into_iter().map(|p| {
                    let target = p.clone();
                    view! {
                        <li>
                            <button type="button" on:click=move |_| switch_to(store, &target) class="flex w-20 flex-col items-center gap-2 text-label text-ink/80 hover:text-ink">
                                // radius: a profile picture, not the button's shape
                                {picture(&p, 64.0, "block h-16 w-16 overflow-hidden rounded-control object-cover")}
                                <span class="w-full truncate text-center">{p.name.clone()}</span>
                            </button>
                        </li>
                    }
                }).collect_view()}
            </ul>
            <div class="mb-10 flex justify-center">
                <a href=Screen::Profiles.path() class="rounded-button bg-set-sheet-tile px-6 py-3 text-body-md font-bold hover:bg-ink/20 lg:bg-ink/10">{t("set-manage-profiles")}</a>
            </div>
            <Section>
                <LinkRow icon="gear-six" title=t("set-nav-app") href=Screen::App.path() />
                <LinkRow icon="user" title=t("set-nav-account") href=Screen::Account.path() />
                <LinkRow icon="question" title=t("set-nav-help") href=crate::config::routes::HELP />
                <ActionRow icon="sign-out" title=t("set-sign-out") on_click=Callback::new(move |_| match signout { Some(s) => s.0.set(true), None => sign_out_now() }) />
            </Section>
        </div>
    }
}

/// True while the account can take another profile (the Kids profile does not count toward the limit).
pub fn can_add_profile(store: ProfileStore) -> bool {
    store.profiles.with(|l| l.iter().filter(|p| !p.is_kids).count() < MAX_PROFILES) || store.profiles.with(|l| !l.iter().any(|p| p.is_kids))
}

/// Adds the profile the modal describes, keeps it and sends it to the account.
pub fn add_profile(store: ProfileStore, data: ProfileFormData) {
    let name = data.name.trim().to_string();
    if name.is_empty() {
        return;
    }
    let id = format!("p{}", js_sys::Date::now() as u64);
    crate::components::layout::tour::mark_new(&id);
    store.profiles.update(|list| {
        let order = list.iter().map(|p| p.sort_order).max().unwrap_or(0) + 1;
        list.push(Profile { id, name, avatar_url: data.avatar_url, is_kids: data.is_kids, is_default: Some(false), pin: None, sort_order: order });
    });
    persist_profiles(store);
}

/// "Add Profile" under the profile list of the Profiles page: a wide grey button, the Add a profile dialog, a "Profile added"
/// toast, and the limit line. The button is gone at the limit, as on the Netflix website.
#[component]
pub fn AddProfile() -> impl IntoView {
    let store = use_profile_store();
    let adding = RwSignal::new(false);
    let on_save = Callback::new(move |data: ProfileFormData| {
        add_profile(store, data);
        adding.set(false);
        super::widgets::toast(t("set-profile-added"));
    });
    view! {
        <div class="px-5 pb-5 pt-2">
            <Show when=move || can_add_profile(store)>
                <button type="button" on:click=move |_| adding.set(true) class="h-12 w-full rounded-button bg-glass-hover text-title-sm font-medium hover:bg-glass">{t("set-add-profile-button")}</button>
                <p class="mt-4 text-center text-body-sm text-chrome/70">{crate::i18n::t_args("set-add-profile-limit", &[("max", (MAX_PROFILES as i64).into())])}</p>
            </Show>
            <Show when=move || adding.get()>
                <AddEditProfileModal mode="add" initial=None on_save=on_save on_delete=None on_cancel=Callback::new(move |_| adding.set(false)) />
            </Show>
        </div>
    }
}

#[component]
pub fn ManageProfiles() -> impl IntoView {
    let store = use_profile_store();
    let adding = RwSignal::new(false);
    let can_add = move || can_add_profile(store);
    let on_save = Callback::new(move |data: ProfileFormData| {
        add_profile(store, data);
        adding.set(false);
    });

    view! {
        <div>
            <p class="mb-6 text-body-md text-ink/60">{t("set-profiles-hint")}</p>
            <ul class="grid grid-cols-2 gap-6 sm:grid-cols-3">
                {move || store.profiles.get().into_iter().map(|p| view! {
                    <li>
                        <a href=Screen::Profile(p.id.clone()).path() class="group flex flex-col items-center gap-3 text-center">
                            <span class="relative block h-28 w-28 overflow-hidden rounded-card">
                                {picture(&p, 112.0, "block h-full w-full object-cover opacity-70")}
                                <span class="absolute inset-0 flex items-center justify-center bg-void/40">
                                    <i class="ph-bold ph-pencil-simple text-3xl text-ink" aria-hidden="true"></i>
                                </span>
                            </span>
                            <span class="w-full truncate text-body-md font-semibold">{p.name.clone()}</span>
                            {p.is_kids.then(|| view! { <span class="text-label text-ink/50">{t("set-kids-profile")}</span> })}
                        </a>
                    </li>
                }).collect_view()}
                <Show when=can_add>
                    <li>
                        <button type="button" on:click=move |_| adding.set(true) class="flex w-full flex-col items-center gap-3">
                            <span class="flex h-28 w-28 items-center justify-center rounded-card bg-ink/10 hover:bg-ink/20">
                                <AppIcon glyph=Glyph::Plus class="text-4xl text-ink/80" />
                            </span>
                            <span class="text-body-md font-semibold">{t("set-add-profile")}</span>
                        </button>
                    </li>
                </Show>
            </ul>
            <ImportProfile />
            <Show when=move || adding.get()>
                <AddEditProfileModal mode="add" initial=None on_save=on_save on_delete=None on_cancel=Callback::new(move |_| adding.set(false)) />
            </Show>
        </div>
    }
}

/// "Import a profile": picks a file saved by Transfer a profile and adds it to this account.
#[component]
fn ImportProfile() -> impl IntoView {
    use super::prefs::use_prefs;
    use super::widgets::{toast, ActionRow, Message, Section, Tone};
    let store = use_profile_store();
    let prefs = use_prefs();
    let input = NodeRef::<leptos::html::Input>::new();
    let note = RwSignal::new(None::<(Tone, String)>);
    let on_pick = move |_| {
        let Some(el) = input.get() else { return };
        let Some(file) = el.files().and_then(|f| f.get(0)) else { return };
        leptos::task::spawn_local(async move {
            let text = wasm_bindgen_futures::JsFuture::from(file.text()).await.ok().and_then(|v| v.as_string()).unwrap_or_default();
            match super::transfer::import_profile(&text, store, prefs) {
                Ok(name) => {
                    note.set(None);
                    toast(crate::i18n::t_args("set-import-done", &[("name", name.into())]));
                }
                Err(e) => note.set(Some((Tone::Error, t(match e {
                    super::transfer::ImportError::NotAProfile => "set-import-invalid",
                    super::transfer::ImportError::TooNew => "set-import-newer",
                    super::transfer::ImportError::Full => "set-import-full",
                })))),
            }
        });
        el.set_value("");
    };
    view! {
        <div class="mt-8">
            <Message text=note />
            <Section>
                <ActionRow icon="file-arrow-up" title=t("set-import-title") hint=t("set-import-hint") on_click=Callback::new(move |_| { if let Some(el) = input.get() { el.click(); } }) />
            </Section>
            <input node_ref=input type="file" accept=".json,application/json" class="hidden" on:change=on_pick />
        </div>
    }
}

/// The Profile sheet of the phone app: a sheet rising from the bottom to the top of the screen (as the app, note 116), with a handle, the title "Profile", a close button and the same content as the settings hub.
#[component]
pub fn ProfileSheet(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-(--z-sheet-backdrop) lg:hidden" on:click=move |_| open.set(false)></div>
            <div role="dialog" aria-label=t("set-profile-sheet") class="set-tiles theme-paper-wide fixed inset-x-0 bottom-0 top-[env(safe-area-inset-top)] z-(--z-sheet) flex flex-col overflow-hidden rounded-t-sheet bg-set-sheet text-ink shadow-popover lg:hidden">
                <div class="relative flex h-14 shrink-0 items-center justify-center">
                    <span class="absolute left-1/2 top-2 h-1 w-9 -translate-x-1/2 rounded-full bg-ink/30" aria-hidden="true"></span>
                    <h2 class="text-heading font-bold text-ink/80">{t("set-profile-sheet")}</h2>
                    <button type="button" on:click=move |_| open.set(false) class="absolute end-3 top-1/2 -translate-y-1/2 btn-circle btn-circle-md" aria-label=t("action-buttons-close")>
                        <i class="ph-bold ph-x text-body-lg" aria-hidden="true"></i>
                    </button>
                </div>
                <div class="min-h-0 flex-1 overflow-y-auto px-4 pb-10">
                    <Hub />
                </div>
            </div>
        </Show>
    }
}
