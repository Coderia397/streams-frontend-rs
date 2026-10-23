use leptos::prelude::*;
use leptos::html::Div;
use std::collections::HashSet;
use crate::store::use_ui_store;

#[derive(Clone, Debug, PartialEq)]
pub struct AppNotification {
    pub id: String,
    pub movie_id: u32,
    pub is_tv: bool,
    pub headline: String,
    pub body: String,
    pub backdrop: String,
    pub relative_time: String,
}

fn load_read_ids() -> HashSet<String> {
    let mut set = HashSet::new();
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(raw)) = storage.get_item(crate::keys::NOTIFICATIONS_READ) {
            if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&raw) {
                for id in parsed {
                    set.insert(id);
                }
            }
        }
    }
    set
}

fn save_read_ids(ids: &HashSet<String>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let vec: Vec<&String> = ids.iter().collect();
        if let Ok(json) = serde_json::to_string(&vec) {
            let _ = storage.set_item(crate::keys::NOTIFICATIONS_READ, &json);
        }
    }
}

/// How many of the current notices this browser has not opened yet (for the bell of the mobile header).
pub async fn unread_count() -> usize {
    let read = load_read_ids();
    crate::services::notifications::fetch().await.into_iter()
        .filter(|n| n.item.backdrop_url("w300").or_else(|| n.item.poster_url("w300")).is_some())
        .filter(|n| !read.contains(&format!("notif-{}-{}", if n.item.is_tv() { "tv" } else { "movie" }, n.item.id)))
        .count()
}

#[component]
pub fn NotificationsDropdown() -> impl IntoView {
    let ui_store = use_ui_store();
    let (open, set_open) = signal(false);
    let (read_ids, set_read_ids) = signal(load_read_ids());
    let (is_loading, set_is_loading) = signal(true);
    let (items, set_items) = signal(Vec::<AppNotification>::new());

    let root_ref = NodeRef::<Div>::new();

    // Real arrivals: new releases, titles due soon and what is trending (see services/notifications.rs)
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            let list: Vec<AppNotification> = crate::services::notifications::fetch()
                .await
                .into_iter()
                .filter_map(|n| {
                    let backdrop = n.item.backdrop_url("w300").or_else(|| n.item.poster_url("w300"))?;
                    Some(AppNotification {
                        id: format!("notif-{}-{}", if n.item.is_tv() { "tv" } else { "movie" }, n.item.id),
                        movie_id: n.item.id,
                        is_tv: n.item.is_tv(),
                        headline: n.headline(),
                        body: n.item.display_title().to_string(),
                        backdrop,
                        relative_time: n.time_label(),
                    })
                })
                .collect();
            set_items.set(list);
            set_is_loading.set(false);
        });
    });

    // a press anywhere else closes the list
    crate::utils::listen::on_press_outside(
        move |target| crate::utils::listen::contains(root_ref, target),
        move || if open.get_untracked() { set_open.set(false) },
    );

    let unread_count = move || {
        let current_reads = read_ids.get();
        items.get().iter().filter(|n| !current_reads.contains(&n.id)).count()
    };

    let toggle_open = move |_| {
        set_open.update(|o| *o = !*o);
    };

    let on_select = move |notif: AppNotification| {
        let notif_id = notif.id.clone();
        set_read_ids.update(|set| {
            set.insert(notif_id);
            save_read_ids(set);
        });
        set_open.set(false);
        ui_store.hero_paused_by_modal.set(false);
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(notif.movie_id));
        ui_store.info_modal_is_tv.set(notif.is_tv);
        ui_store.info_modal_open.set(true);
    };

    let has_unread = move || unread_count() > 0;

    view! {
        <div node_ref=root_ref class="relative">
            // Bell trigger button
            <button
                on:click=toggle_open
                class="relative btn-circle btn-circle-clear btn-circle-md"
                // open: the grey disc of the other round buttons, as Netflix's bell
                class=("!bg-glass", move || open.get())
                title=crate::i18n::t("notifications-dropdown-notifications")
                aria-label=crate::i18n::t("notifications-dropdown-notifications")
            >
                <i class="ph ph-bell text-heading"></i>
                <Show when=has_unread>
                    <span class="absolute top-[2px] right-[2px] min-w-[16px] h-[16px] px-1 rounded-full bg-brand text-ink text-micro font-medium flex items-center justify-center leading-none">
                        {move || {
                            let c = unread_count();
                            if c > 99 { "99+".to_string() } else { c.to_string() }
                        }}
                    </span>
                </Show>
            </button>

            // Dropdown panel
            <Show when=move || open.get()>
                // Netflix's redesigned menus (measured 2026-10-23): 6 px under the button, #232323, 12 px corners, a soft shadow
                <div class="absolute right-0 top-[calc(100%+6px)] w-[360px] sm:w-[420px] max-h-[560px] overflow-y-auto bg-popover rounded-menu shadow-menu z-50 scrollbar-hide">
                    // Header
                    <div class="px-4 py-3">
                        <p class="text-ink font-bold text-body">{crate::i18n::t("notif-title")}</p>
                    </div>
                    <Show
                        when=move || !is_loading.get()
                        fallback=move || view! { <crate::components::skeleton::NotificationsSkeleton /> }
                    >
                        <Show
                            when=move || !items.get().is_empty()
                            fallback=move || view! {
                                <p class="text-ink/50 text-center text-body-md py-10 px-6">
                                    {crate::i18n::t("notifications-dropdown-youre-all-caught-up-new")}
                                </p>
                            }
                        >
                            <div class="flex flex-col">
                                {move || items.get().into_iter().map(|n| {
                                    let notif = n.clone();
                                    let on_click_notif = notif.clone();

                                    view! {
                                        <button
                                            on:click=move |_| on_select(on_click_notif.clone())
                                            class="w-full flex items-start gap-3 px-3 py-3 text-left hover:bg-ink/[0.08] rounded-menu-item mx-1 transition-colors cursor-pointer group"
                                        >
                                            // Backdrop thumbnail (16:9)
                                            <div class="w-[112px] h-[63px] rounded-field overflow-hidden bg-zinc-800 shrink-0">
                                                <img
                                                    src=n.backdrop.clone()
                                                    alt=n.body.clone()
                                                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-(--motion-slow)"
                                                    loading="lazy"
                                                />
                                            </div>

                                            // Text: headline big+bold, body smaller/muted, time tiny gray
                                            <div class="flex-1 min-w-0 pt-0.5">
                                                <p class="text-ink text-body font-bold leading-snug mb-0.5">{n.headline}</p>
                                                <p class="text-ink/60 text-label leading-snug line-clamp-2">{n.body}</p>
                                                <p class="text-ink/40 text-fine mt-1">{n.relative_time}</p>
                                            </div>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>
                    </Show>
                </div>
            </Show>
        </div>
    }
}
