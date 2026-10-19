use leptos::prelude::*;

/// "On this page" list. The entry for the section being read is highlighted as the page scrolls.
#[component]
pub fn Toc(items: Vec<(String, String)>, #[prop(into)] title: String) -> impl IntoView {
    let label_title = title.clone();
    let ids: Vec<String> = items.iter().map(|(id, _)| id.clone()).collect();
    let (active, set_active) = signal(ids.first().cloned().unwrap_or_default());
    crate::utils::listen::listen_window(leptos::ev::scroll, move |_| {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
        let mut current = ids.first().cloned().unwrap_or_default();
        for id in &ids {
            if doc.get_element_by_id(id).is_some_and(|el| el.get_bounding_client_rect().top() <= 140.0) {
                current = id.clone();
            }
        }
        set_active.set(current);
    });
    view! {
        <nav aria-label=label_title class="text-body-sm">
            <p class="mb-3 font-bold text-paper-ink">{title}</p>
            <ul class="border-s border-paper-line">
                {items
                    .into_iter()
                    .map(|(id, label)| {
                        let target = id.clone();
                        view! {
                            <li>
                                <a
                                    href=format!("#{id}")
                                    class="-ms-px block border-s-2 py-1.5 ps-4 hover:text-paper-ink"
                                    class=("border-brand", { let t = target.clone(); move || active.get() == t })
                                    class=("font-semibold", { let t = target.clone(); move || active.get() == t })
                                    class=("text-paper-ink", { let t = target.clone(); move || active.get() == t })
                                    class=("border-transparent", { let t = target.clone(); move || active.get() != t })
                                    class=("text-paper-mute", { let t = target.clone(); move || active.get() != t })
                                >
                                    {label}
                                </a>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </nav>
    }
}
