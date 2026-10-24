//! The share sheet, mounted once at the app's root and opened by `Share::open` (mod.rs). What each target does:
//! WhatsApp, X: their composer with the title and link, in a new tab (the app takes it when installed). Messages: the messages app with
//! the title and link. Snapchat: Snap's scan page with the link attached, the address Snap's own button opens: in a new tab on a
//! phone, which the app takes, and in a window with a Snapcode on a computer. Instagram: the system share sheet with the poster (Instagram is
//! one of its targets there, Stories among them); where a browser has no share sheet, the link is copied with a note to paste it into
//! Instagram. Copy link: the link copied, and a note saying whether it was.
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen_futures::JsFuture;
use super::{targets, use_share, ShareItem};

#[component]
pub fn ShareHost() -> impl IntoView {
    let share = use_share();
    let item = share.0;
    let close = move || item.set(None);
    // Back closes the sheet before it leaves the page
    crate::utils::back::close_on_back(Signal::derive(move || item.with(Option::is_some)), Callback::new(move |_| item.set(None)));
    // the note under the targets, and whether it reports a success
    let toast = RwSignal::new(None::<(String, bool)>);
    let toast_timer = StoredValue::new_local(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    let say = move |msg: String, ok: bool| {
        if let Some(h) = toast_timer.get_value() {
            h.clear();
        }
        toast.set(Some((msg, ok)));
        toast_timer.set_value(leptos::leptos_dom::helpers::set_timeout_with_handle(move || toast.set(None), Duration::from_millis(1800)).ok());
    };
    let copy = move |text: String, done: String| {
        let promise = super::copy_text(&text);
        leptos::task::spawn_local(async move {
            let copied = JsFuture::from(promise).await.ok().and_then(|v| v.as_bool()).unwrap_or(false);
            if copied { say(done, true) } else { say(crate::i18n::t("share-copy-failed"), false) }
        });
    };

    // the page under the sheet does not scroll; its own value is put back on close, so a sheet over the details window (which locks
    // the page itself) does not unlock it
    let saved_overflow = StoredValue::new(None::<String>);
    Effect::new(move |_| {
        let is_open = item.with(Option::is_some);
        let Some(body) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.body()) else { return };
        if is_open {
            if saved_overflow.get_value().is_none() {
                saved_overflow.set_value(Some(body.style().get_property_value("overflow").unwrap_or_default()));
            }
            let _ = body.style().set_property("overflow", "hidden");
        } else if let Some(prev) = saved_overflow.get_value() {
            let _ = body.style().set_property("overflow", &prev);
            saved_overflow.set_value(None);
        }
    });

    let open_tab = |url: String| {
        if let Some(w) = web_sys::window() {
            let _ = w.open_with_url_and_target(&url, "_blank");
        }
    };

    view! {
        {move || item.get().map(|ShareItem { title, url, image }| {
            let display_url = url.trim_start_matches("https://").trim_start_matches("http://").to_string();
            let (t, u) = (title.clone(), url.clone());
            let whatsapp = move |_| open_tab(targets::whatsapp(&t, &u));
            let (t, u) = (title.clone(), url.clone());
            let messages = move |_| {
                if let Some(w) = web_sys::window() {
                    let _ = w.location().set_href(&targets::messages(&t, &u));
                }
            };
            let (t, u, i) = (title.clone(), url.clone(), image.clone().unwrap_or_default());
            let instagram = move |_| {
                // the system sheet is asked for now, inside the tap; the result comes later
                let promise = super::share_system(&t, &u, &i);
                let u = u.clone();
                leptos::task::spawn_local(async move {
                    match JsFuture::from(promise).await.ok().and_then(|v| v.as_string()).as_deref() {
                        Some("shared") => close(),
                        Some("cancelled") => {}
                        _ => copy(u, crate::i18n::t("share-copied-instagram")),
                    }
                });
            };
            let u = url.clone();
            let snapchat = move |_| {
                let address = targets::snapchat(&u);
                let Some(w) = web_sys::window() else { return };
                if crate::form::current().is_phone() {
                    // a new tab, which the app takes: Snap's button sends the page itself, and coming back from the app then left
                    // the person on another page of ours (verified on the phone, 2026-10-24)
                    open_tab(address);
                } else {
                    let _ = w.open_with_url_and_target_and_features(&address, "_blank", "left=500,top=200,width=769,height=700");
                }
            };
            let (t, u) = (title.clone(), url.clone());
            let x = move |_| open_tab(targets::x(&t, &u));
            let u = url.clone();
            let copy_link = move |_| copy(u.clone(), crate::i18n::t("share-copied"));

            view! {
                <div class="fixed inset-0 z-(--z-share-backdrop) flex flex-col justify-end">
                    <div class="fixed inset-0 bg-void/60 transition-opacity duration-(--motion-base)" on:click=move |_| close() />
                    <div class="relative z-(--z-share) bg-sheet-alt rounded-t-sheet px-4 pt-2 pb-8 max-w-lg mx-auto w-full shadow-2xl transition-transform duration-(--motion-slow)">
                        <div class="w-10 h-1 rounded-full bg-ink/30 mx-auto mb-3" />

                        <div class="flex items-center gap-3 mb-5">
                            <div class="w-12 h-12 rounded-field overflow-hidden bg-ink/10 shrink-0">
                                {image.map(|src| view! { <img src=src alt="" class="w-full h-full object-cover" /> })}
                            </div>
                            <div class="min-w-0 flex-1">
                                <p class="text-ink font-bold text-body truncate">{title}</p>
                                <p class="text-ink/40 text-label truncate">{display_url}</p>
                            </div>
                            <button type="button" on:click=move |_| close() class="btn-circle btn-circle-sm" title=crate::i18n::t("action-buttons-close")>
                                <i class="ph-bold ph-x text-lg"></i>
                            </button>
                        </div>

                        <div class="grid grid-cols-4 gap-y-5 gap-x-2">
                            <Target on_click=whatsapp label="WhatsApp".to_string() disc="bg-whatsapp" icon="ph-fill ph-whatsapp-logo text-ink text-2xl" />
                            <Target on_click=messages label=crate::i18n::t("share-messages") disc="bg-messages" icon="ph-fill ph-chat-teardrop-dots text-ink text-2xl" />
                            <Target on_click=instagram label="Instagram".to_string() disc="bg-gradient-to-tr from-instagram-from via-instagram-via to-instagram-to" icon="ph ph-instagram-logo text-ink text-2xl" />
                            <Target on_click=snapchat label="Snapchat".to_string() disc="bg-snapchat" icon="ph-fill ph-snapchat-logo text-void text-2xl" />
                            <Target on_click=x label="X".to_string() disc="bg-void border border-ink/20" icon="ph-bold ph-x-logo text-ink text-xl" />
                            <Target on_click=copy_link label=crate::i18n::t("share-copy-link") disc="bg-lift hover:bg-line-strong transition-colors" icon="ph-bold ph-link text-ink text-xl" />
                        </div>

                        {move || toast.get().map(|(msg, ok)| view! {
                            <div class="mt-5 flex items-center justify-center gap-2 text-ink/90 text-label font-medium animate-fadeIn">
                                {ok.then(|| view! { <i class="ph-bold ph-check text-success"></i> })}
                                <span>{msg}</span>
                            </div>
                        })}
                    </div>
                </div>
            }
        })}
    }
}

/// One target: its round mark and its name under it.
#[component]
fn Target(on_click: impl FnMut(leptos::ev::MouseEvent) + 'static, label: String, disc: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <button type="button" on:click=on_click class="flex flex-col items-center gap-2 active:scale-95 transition-transform cursor-pointer">
            <div class=format!("w-14 h-14 rounded-full flex items-center justify-center shadow-md {disc}")>
                <i class=icon></i>
            </div>
            <span class="text-ink/85 text-caption text-center leading-tight">{label}</span>
        </button>
    }
}
