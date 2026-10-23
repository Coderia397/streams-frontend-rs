//! Settings widgets: dialogs, toasts and messages that confirm or refuse.
use crate::i18n::t;
use leptos::prelude::*;

/// Dialog that asks before something that cannot be undone.
#[component]
pub fn ConfirmDialog(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(into)] body: String,
    #[prop(into)] confirm_label: String,
    on_confirm: Callback<()>,
    #[prop(optional)] danger: bool,
) -> impl IntoView {
    let confirm_look = if danger { "bg-danger text-ink hover:opacity-90" } else { "bg-ink text-void hover:bg-ink/90" };
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-(--z-dialog) flex items-end justify-center bg-void/70 p-4 sm:items-center" on:click=move |_| open.set(false)>
                <div
                    role="alertdialog"
                    aria-modal="true"
                    aria-label=title.clone()
                    class="w-full max-w-md rounded-card border border-ink/10 bg-page p-6 shadow-popover"
                    on:click=|ev| ev.stop_propagation()
                    on:keydown=move |ev| if ev.key() == "Escape" { open.set(false) }
                >
                    <h2 class="mb-2 text-heading font-bold text-ink">{title.clone()}</h2>
                    <p class="mb-6 text-body-md text-ink/70">{body.clone()}</p>
                    <div class="flex justify-end gap-3">
                        <button type="button" on:click=move |_| open.set(false) class="rounded-button px-5 py-2.5 text-body-md font-bold text-ink hover:bg-ink/10">{t("set-cancel")}</button>
                        <button type="button" on:click=move |_| { open.set(false); on_confirm.run(()); } class=format!("rounded-button px-5 py-2.5 text-body-md font-bold {confirm_look}")>{confirm_label.clone()}</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Message that appears for a moment after something was saved. Provided once by the settings page.
#[derive(Clone, Copy)]
pub struct Toast(pub RwSignal<Option<String>>);

/// How long a toast stays, in milliseconds.
pub const TOAST_MS: i32 = 2500;

pub fn toast(text: String) {
    if let Some(t) = use_context::<Toast>() {
        t.0.set(Some(text));
        if let Some(window) = web_sys::window() {
            let cb = wasm_bindgen::closure::Closure::once_into_js(move || t.0.set(None));
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(wasm_bindgen::JsCast::unchecked_ref(&cb), TOAST_MS);
        }
    }
}

#[component]
pub fn Toaster() -> impl IntoView {
    let Toast(message) = use_context::<Toast>().expect("Toast provided");
    view! {
        <div class="pointer-events-none fixed inset-x-0 bottom-6 z-(--z-dialog) flex justify-center px-4" role="status" aria-live="polite">
            {move || message.get().map(|m| view! { <p class="rounded-control bg-chrome px-5 py-3 text-body-md text-chrome-ink shadow-popover">{m}</p> })}
        </div>
    }
}
