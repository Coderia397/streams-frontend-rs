use leptos::prelude::*;
use crate::models::profile::Profile;
use crate::components::profiles::choose_icon::ChooseIconModal;

#[derive(Clone, Debug)]
pub struct ProfileFormData {
    pub name: String,
    pub avatar_url: Option<String>,
    pub is_kids: bool,
}

#[component]
pub fn AddEditProfileModal(
    #[prop(into)] mode: String, // "add" or "edit"
    initial: Option<Profile>,
    on_save: Callback<ProfileFormData>,
    on_delete: Option<Callback<()>>,
    on_cancel: Callback<()>,
    /// False for the main profile: it can never be a Kids profile.
    #[prop(default = true)] allow_kids: bool,
) -> impl IntoView {
    let initial_name = initial.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    // a new profile starts with one of the six plain faces, picked at random
    let initial_avatar = initial.as_ref().and_then(|p| p.avatar_url.clone()).unwrap_or_else(crate::data::avatars::random_smile);
    let initial_kids = initial.as_ref().map(|p| p.is_kids).unwrap_or(false);
    // the lock is shown here, never changed here: a PIN is set, changed or removed in Settings, where the account checks the current PIN
    // or a recent sign in (Manage Profiles let anyone untick it, 2026-10-25)
    let has_pin = initial.as_ref().is_some_and(|p| p.pin.is_some());
    let lock_href = initial.as_ref().map(|p| format!("/settings/profile/{}/lock", p.id)).unwrap_or_default();

    let (name, set_name) = signal(initial_name);
    let (avatar_url, set_avatar_url) = signal(initial_avatar);
    let (is_kids, set_kids) = signal(initial_kids);

    let (picking_avatar, set_picking_avatar) = signal(false);
    let (confirm_delete, set_confirm_delete) = signal(false);

    // Save is always pressable; an empty name shows "Please enter a name" under the field, as Netflix does
    let (name_missing, set_name_missing) = signal(false);
    let can_save = move || !name.get().trim().is_empty();

    let is_edit = mode == "edit";
    let modal_title = if is_edit { crate::i18n::t("profile-edit-title") } else { crate::i18n::t("set-add-a-profile") };

    let handle_save = move |_| {
        if name.get().trim().is_empty() {
            set_name_missing.set(true);
            return;
        }
        if !can_save() { return; }
        on_save.run(ProfileFormData {
            name: name.get().trim().to_string(),
            avatar_url: Some(avatar_url.get()),
            is_kids: is_kids.get(),
        });
    };

    view! {
        <div class="fixed inset-0 z-(--z-modal) flex items-center justify-center bg-void/80 backdrop-blur-sm px-4 animate-in fade-in duration-(--motion-base)">
            <div
                class="relative w-full max-w-[560px] max-h-[90vh] overflow-y-auto bg-page rounded-field shadow-2xl border border-ink/10 p-6 sm:p-10 text-ink"
            >
                // Close button
                <button
                    on:click=move |_| on_cancel.run(())
                    class="absolute top-5 right-5 btn-circle btn-circle-clear btn-circle-sm text-ink/50 hover:text-ink"
                    aria-label=crate::i18n::t("action-buttons-close")
                >
                    <i class="ph-bold ph-x text-xl"></i>
                </button>

                <h2 class="text-2xl sm:text-3xl font-bold text-ink text-center mb-1">
                    {modal_title}
                </h2>
                <p class="text-ink/40 text-sm text-center mb-8">
                    {if is_edit { crate::i18n::t("profile-edit-hint") } else { crate::i18n::t_args("profile-add-hint", &[("brand", fluent_bundle::FluentValue::from(crate::brand::NAME))]) }}
                </p>

                // Avatar + Name Input (Task 100)
                <div class="flex flex-col sm:flex-row items-center sm:items-start gap-6 mb-8">
                    // Avatar tile with pencil overlay
                    <button
                        type="button"
                        on:click=move |_| set_picking_avatar.set(true)
                        // radius: the avatar tile
                        class="relative w-24 h-24 sm:w-28 sm:h-28 rounded-field overflow-hidden bg-ink/10 shrink-0 shadow-md group hover:ring-4 hover:ring-ink/20 transition-all active:scale-95 cursor-pointer"
                    >
                        <img
                            src=avatar_url
                            alt=crate::i18n::t("navbar-mobile-avatar")
                            class="w-full h-full object-cover block"
                        />
                        <div class="absolute inset-0 bg-void/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                            <i class="ph-bold ph-pencil-simple text-ink text-2xl"></i>
                        </div>
                    </button>

                    // Name input
                    <div class="flex-1 w-full space-y-2">
                        <input
                            type="text"
                            prop:value=move || name.get()
                            on:input=move |e| { set_name.set(event_target_value(&e)); set_name_missing.set(false); }
                            placeholder=crate::i18n::t("add-edit-modal-name")
                            maxlength="30"
                            class=move || format!("w-full bg-transparent border px-4 py-3 text-ink text-base placeholder:text-ink/30 outline-none transition-colors {}", if name_missing.get() { "border-danger-soft" } else { "border-ink/20 focus:border-ink" })
                        />
                        <Show when=move || name_missing.get()>
                            <p class="flex items-center gap-1.5 text-label text-danger-soft"><i class="ph ph-warning" aria-hidden="true"></i>{crate::i18n::t("set-profile-name-required")}</p>
                        </Show>
                    </div>
                </div>

                // Kids Mode Toggle (the main profile has none: it is never a Kids profile)
                <div class=if allow_kids { "border-t border-ink/10 py-5" } else { "hidden" }>
                    <label class="flex items-center justify-between gap-4 cursor-pointer">
                        <div>
                            <span class="text-ink font-semibold text-body">{crate::i18n::t("profile-kids")}</span>
                            <p class="text-ink/40 text-label mt-0.5">
                                {crate::i18n::t("add-edit-modal-only-see-childfriendly-series-and")}
                            </p>
                        </div>
                        <input
                            type="checkbox"
                            role="switch"
                            prop:checked=move || is_kids.get()
                            on:change=move |e| {
                                let on = event_target_checked(&e);
                                set_kids.set(on);
                                // a picture that is not for Kids is replaced by a plain face
                                if on && !crate::data::avatars::kids_ok(&avatar_url.get_untracked()) { set_avatar_url.set(crate::data::avatars::random_smile()); }
                            }
                            class="peer sr-only"
                        />
                        <span class="relative h-[26px] w-[46px] shrink-0 rounded-full bg-ink/40 transition-colors peer-checked:bg-brand peer-focus-visible:outline-2 after:absolute after:start-[3px] after:top-[3px] after:h-5 after:w-5 after:rounded-full after:bg-chrome-ink after:transition-transform peer-checked:after:translate-x-5 rtl:peer-checked:after:-translate-x-5" aria-hidden="true"></span>
                    </label>
                </div>

                // Profile Lock: shown, and changed in Settings (not part of Add a profile)
                <Show when=move || is_edit>
                    <a href=lock_href.clone() class="flex items-center justify-between gap-4 border-t border-ink/10 py-5">
                        <div>
                            <span class="text-ink font-semibold text-body">{crate::i18n::t("profile-lock")}</span>
                            <p class="text-ink/40 text-label mt-0.5">{crate::i18n::t("add-edit-modal-require-a-4digit-pin-to")}</p>
                        </div>
                        <span class="flex items-center gap-2 text-ink/60 text-body">
                            {if has_pin { crate::i18n::t("set-lock-on") } else { crate::i18n::t("set-lock-off") }}
                            <i class="ph-bold ph-caret-right rtl:-scale-x-100" aria-hidden="true"></i>
                        </span>
                    </a>
                </Show>

                // Delete profile section (Task 099)
                {move || {
                    if is_edit && on_delete.is_some() {
                        let on_del = on_delete.unwrap();
                        view! {
                            <div class="border-t border-ink/10 py-5">
                                {if confirm_delete.get() {
                                    view! {
                                        <div class="bg-danger-deep/40 border border-danger/30 rounded-card p-4 space-y-3">
                                            <p class="text-sm text-danger-pale">
                                                {crate::i18n::t("add-edit-modal-delete-this-profile-all-viewing")}
                                            </p>
                                            <div class="flex items-center gap-3">
                                                <button
                                                    type="button"
                                                    on:click=move |_| on_del.run(())
                                                    class="btn-brand px-4 py-1.5 text-xs active:scale-95"
                                                >
                                                    {crate::i18n::t("add-edit-modal-yes-delete")}
                                                </button>
                                                <button
                                                    type="button"
                                                    on:click=move |_| set_confirm_delete.set(false)
                                                    class="px-4 py-1.5 border border-ink/30 text-ink/70 hover:text-ink rounded-button text-xs transition-colors cursor-pointer"
                                                >
                                                    {crate::i18n::t("continue-watching-options-sheet-cancel")}
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            type="button"
                                            on:click=move |_| set_confirm_delete.set(true)
                                            class="border border-ink/20 hover:border-danger text-ink/60 hover:text-danger-soft px-4 py-2 rounded-button text-sm font-semibold transition-colors cursor-pointer"
                                        >
                                            {crate::i18n::t("add-edit-modal-delete-profile")}
                                        </button>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div /> }.into_any()
                    }
                }}

                // Actions: Save as a wide button, Cancel under it (as in the Netflix dialog)
                <div class="flex flex-col items-stretch gap-2 pt-4">
                    <button
                        type="button"
                        on:click=handle_save
                        class="w-full rounded-button bg-ink py-4 text-body-md font-bold text-void transition-colors hover:bg-ink/90 cursor-pointer active:scale-[0.99]"
                    >
                        {crate::i18n::t("add-edit-modal-save")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| on_cancel.run(())
                        class="w-full py-3 text-body-md font-bold text-ink hover:underline cursor-pointer"
                    >
                        {crate::i18n::t("continue-watching-options-sheet-cancel")}
                    </button>
                </div>
            </div>

            // Avatar picker gallery overlay (Task 101)
            <Show when=move || picking_avatar.get()>
                <ChooseIconModal
                    current_avatar_url=Some(avatar_url.get())
                    kids=is_kids.get_untracked()
                    name=name.get_untracked()
                    on_select=Callback::new(move |url| {
                        set_avatar_url.set(url);
                        set_picking_avatar.set(false);
                    })
                    on_cancel=Callback::new(move |_| {
                        set_picking_avatar.set(false);
                    })
                />
            </Show>
        </div>
    }
}
