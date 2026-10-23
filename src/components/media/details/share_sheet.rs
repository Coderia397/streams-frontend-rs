use leptos::prelude::*;
use leptos::portal::Portal;
use std::time::Duration;

#[component]
pub fn ShareSheet(
    open: RwSignal<bool>,
    title: String,
    url: String,
    #[prop(optional)] thumbnail_url: Option<String>,
) -> impl IntoView {
    let toast = RwSignal::new(None::<String>);
    let toast_timer = RwSignal::new_local(None::<leptos::leptos_dom::helpers::TimeoutHandle>);

    let set_toast_msg = move |msg: String| {
        if let Some(h) = toast_timer.get_untracked() {
            h.clear();
        }
        toast.set(Some(msg));
        let handle = leptos::leptos_dom::helpers::set_timeout_with_handle(
            move || {
                toast.set(None);
            },
            Duration::from_millis(1800),
        );
        if let Ok(h) = handle {
            toast_timer.set(Some(h));
        }
    };

    let title_stored = StoredValue::new(title);
    let url_stored = StoredValue::new(url);
    let thumb_stored = StoredValue::new(thumbnail_url);

    let on_close = move |_| {
        open.set(false);
    };

    let copy_to_clipboard = move |text: &str| {
        if let Some(w) = web_sys::window() {
            let nav = w.navigator();
            let clip = nav.clipboard();
            let _ = clip.write_text(text);
        }
    };

    // Body scroll lock while open. The page's own value is kept and put back on close, so a sheet opened over the details
    // window (which locks the page itself) does not unlock it.
    let saved_overflow = StoredValue::new(None::<String>);
    Effect::new(move |_| {
        let is_open = open.get();
        let Some(body) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.body()) else { return };
        if is_open {
            saved_overflow.set_value(Some(body.style().get_property_value("overflow").unwrap_or_default()));
            let _ = body.style().set_property("overflow", "hidden");
        } else if let Some(prev) = saved_overflow.get_value() {
            let _ = body.style().set_property("overflow", &prev);
            saved_overflow.set_value(None);
        }
    });

    view! {
        <Portal>
            {move || if open.get() {
                let t_val = title_stored.get_value();
                let u_val = url_stored.get_value();
                let thumb_val = thumb_stored.get_value();
                let display_url = u_val.trim_start_matches("https://").trim_start_matches("http://").to_string();

                let handle_whatsapp = {
                    let t = t_val.clone();
                    let u = u_val.clone();
                    move |_| {
                        let text = format!("{} {}", t, u);
                        let encoded = js_sys::encode_uri_component(&text);
                        if let Some(w) = web_sys::window() {
                            let _ = w.open_with_url_and_target(&format!("https://wa.me/?text={}", String::from(encoded)), "_blank");
                        }
                    }
                };

                let handle_messages = {
                    let t = t_val.clone();
                    let u = u_val.clone();
                    move |_| {
                        let encoded = js_sys::encode_uri_component(&format!("{} {}", t, u));
                        if let Some(w) = web_sys::window() {
                            let _ = w.location().set_href(&format!("sms:?&body={}", String::from(encoded)));
                        }
                    }
                };

                let handle_instagram = {
                    let u = u_val.clone();
                    move |_| {
                        copy_to_clipboard(&u);
                        set_toast_msg(crate::i18n::t("share-copied-instagram"));
                    }
                };

                let handle_snapchat = {
                    let u = u_val.clone();
                    move |_| {
                        copy_to_clipboard(&u);
                        set_toast_msg(crate::i18n::t("share-copied-snapchat"));
                    }
                };

                let handle_x = {
                    let t = t_val.clone();
                    let u = u_val.clone();
                    move |_| {
                        let t_enc = js_sys::encode_uri_component(&t);
                        let u_enc = js_sys::encode_uri_component(&u);
                        if let Some(w) = web_sys::window() {
                            let _ = w.open_with_url_and_target(&format!("https://twitter.com/intent/tweet?text={}&url={}", String::from(t_enc), String::from(u_enc)), "_blank");
                        }
                    }
                };

                let handle_copy_link = {
                    let u = u_val.clone();
                    move |_| {
                        copy_to_clipboard(&u);
                        set_toast_msg(crate::i18n::t("share-copied"));
                    }
                };

                view! {
                    <div class="fixed inset-0 z-(--z-share-backdrop) flex flex-col justify-end">
                        // Backdrop
                        <div
                            class="fixed inset-0 bg-void/60 transition-opacity duration-(--motion-base)"
                            on:click=on_close
                        />

                        // Sheet content
                        <div class="relative z-(--z-share) bg-sheet-alt rounded-t-sheet px-4 pt-2 pb-8 max-w-lg mx-auto w-full shadow-2xl transition-transform duration-(--motion-slow)">
                            <div class="w-10 h-1 rounded-full bg-ink/30 mx-auto mb-3" />

                            <div class="flex items-center gap-3 mb-5">
                                <div class="w-12 h-12 rounded-field overflow-hidden bg-ink/10 shrink-0">
                                    {if let Some(src) = thumb_val {
                                        view! { <img src=src alt="" class="w-full h-full object-cover" /> }.into_any()
                                    } else {
                                        view! { <div class="w-full h-full bg-ink/10" /> }.into_any()
                                    }}
                                </div>
                                <div class="min-w-0 flex-1">
                                    <p class="text-ink font-bold text-body truncate">{t_val}</p>
                                    <p class="text-ink/40 text-label truncate">{display_url}</p>
                                </div>
                                <button
                                    type="button"
                                    on:click=on_close
                                    class="btn-circle btn-circle-sm"
                                    title=crate::i18n::t("action-buttons-close")
                                >
                                    <i class="ph-bold ph-x text-lg"></i>
                                </button>
                            </div>

                            // App grid
                            <div class="grid grid-cols-4 gap-y-5 gap-x-2">
                                // WhatsApp
                                <button
                                    type="button"
                                    on:click=handle_whatsapp
                                    class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer"
                                >
                                    <div class="w-14 h-14 rounded-full flex items-center justify-center shadow-md bg-whatsapp">
                                        <i class="ph-fill ph-whatsapp-logo text-ink text-2xl"></i>
                                    </div>
                                    <span class="text-ink/85 text-caption text-center leading-tight">"WhatsApp"</span>
                                </button>

                                // Messages
                                <button
                                    type="button"
                                    on:click=handle_messages
                                    class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer"
                                >
                                    <div class="w-14 h-14 rounded-full flex items-center justify-center shadow-md bg-messages">
                                        <i class="ph-fill ph-chat-teardrop-dots text-ink text-2xl"></i>
                                    </div>
                                    <span class="text-ink/85 text-caption text-center leading-tight">{crate::i18n::t("share-messages")}</span>
                                </button>

                                // Instagram Stories
                                <button
                                    type="button"
                                    on:click=handle_instagram
                                    class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer"
                                >
                                    <div class="w-14 h-14 rounded-full flex items-center justify-center shadow-md bg-gradient-to-tr from-instagram-from via-instagram-via to-instagram-to">
                                        <i class="ph ph-instagram-logo text-ink text-2xl"></i>
                                    </div>
                                    <span class="text-ink/85 text-caption text-center leading-tight">"Instagram"</span>
                                </button>

                                // Snapchat
                                <button
                                    type="button"
                                    on:click=handle_snapchat
                                    class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer"
                                >
                                    <div class="w-14 h-14 rounded-full flex items-center justify-center shadow-md bg-snapchat">
                                        <i class="ph-fill ph-snapchat-logo text-void text-2xl"></i>
                                    </div>
                                    <span class="text-ink/85 text-caption text-center leading-tight">"Snapchat"</span>
                                </button>

                                // X (Twitter)
                                <button
                                    type="button"
                                    on:click=handle_x
                                    class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer"
                                >
                                    <div class="w-14 h-14 rounded-full flex items-center justify-center shadow-md bg-void border border-ink/20">
                                        <i class="ph-bold ph-x-logo text-ink text-xl"></i>
                                    </div>
                                    <span class="text-ink/85 text-caption text-center leading-tight">"X"</span>
                                </button>

                                // Copy Link
                                <button
                                    type="button"
                                    on:click=handle_copy_link
                                    class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer"
                                >
                                    <div class="w-14 h-14 rounded-full bg-lift flex items-center justify-center shadow-md hover:bg-line-strong transition-colors">
                                        <i class="ph-bold ph-link text-ink text-xl"></i>
                                    </div>
                                    <span class="text-ink/85 text-caption text-center leading-tight">{crate::i18n::t("share-copy-link")}</span>
                                </button>
                            </div>

                            // Toast notification
                            {move || toast.get().map(|msg| view! {
                                <div class="mt-5 flex items-center justify-center gap-2 text-ink/90 text-label font-medium animate-fadeIn">
                                    <i class="ph-bold ph-check text-success"></i>
                                    <span>{msg}</span>
                                </div>
                            })}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <span /> }.into_any()
            }}
        </Portal>
    }
}
