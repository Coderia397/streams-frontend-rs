//! Notifications on the phone form, as Netflix's Android app shows them behind the bell (recorded on a Pixel 8, 2026-10-23, note 116):
//! a page with a back arrow and the title, then one row a notice: a red dot at the edge while it is unread, a picture, the headline in
//! bold, the title in grey and when it came. A row opens its title. The website keeps the list under its bell, so this address sends a
//! computer Home (see lib.rs). The notices and what was read are the website bell's (layout/nav/website/notifications.rs).
use leptos::prelude::*;
use crate::components::layout::nav::website::notifications::{load_notifications, load_read_ids, save_read_ids, AppNotification};
use crate::components::layout::Layout;
use crate::store::use_ui_store;

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let ui_store = use_ui_store();
    let read = RwSignal::new(load_read_ids());
    let items = LocalResource::new(|| async { load_notifications().await });

    let open = move |n: AppNotification| {
        read.update(|set| {
            set.insert(n.id.clone());
            save_read_ids(set);
        });
        ui_store.modal_initial_time.set(0.0);
        ui_store.modal_current_time.set(0.0);
        ui_store.info_modal_movie_id.set(Some(n.movie_id));
        ui_store.info_modal_is_tv.set(n.is_tv);
        ui_store.info_modal_open.set(true);
    };

    view! {
        <Layout>
            <div class="min-h-screen pt-[calc(var(--nav-h-mobile)+env(safe-area-inset-top))] pb-28">
                <Suspense fallback=|| view! { <crate::components::skeleton::NotificationsSkeleton /> }>
                    {move || items.get().map(|list| if list.is_empty() {
                        view! { <p class="px-4 py-16 text-center text-body-md text-ink-body">{crate::i18n::t("notifications-dropdown-youre-all-caught-up-new")}</p> }.into_any()
                    } else {
                        list.into_iter().map(|n| {
                            let id = n.id.clone();
                            let unread = move || !read.with(|r| r.contains(&id));
                            let row = n.clone();
                            view! {
                                <button type="button" on:click=move |_| open(row.clone()) class="relative flex w-full items-start gap-3 py-2.5 pe-4 ps-5 text-left cursor-pointer active:bg-ink/5">
                                    <Show when=unread>
                                        <span class="absolute start-1.5 top-1/2 h-1.5 w-1.5 -translate-y-1/2 rounded-full bg-brand" aria-hidden="true"></span>
                                    </Show>
                                    <img src=n.backdrop.clone() alt="" loading="lazy" class="aspect-video w-[113px] shrink-0 rounded-[4px] object-cover bg-raised" />
                                    <span class="min-w-0 flex-1">
                                        <span class="block text-body-lg font-bold leading-snug text-ink">{n.headline.clone()}</span>
                                        <span class="mt-0.5 block truncate text-body text-ink-body">{n.body.clone()}</span>
                                        <span class="mt-0.5 block text-body-sm text-ink-body">{n.relative_time.clone()}</span>
                                    </span>
                                </button>
                            }
                        }).collect_view().into_any()
                    })}
                </Suspense>
            </div>
        </Layout>
    }
}
