//! Settings widgets: cards that open and close.
use leptos::prelude::*;

/// A bordered card with a title and a plus or minus button; the content shows while it is open (the website's "Cancel instead" and "Delete" cards).
#[component]
pub fn CollapsibleCard(#[prop(into)] title: String, #[prop(into)] show_label: String, #[prop(into)] hide_label: String, #[prop(default = true)] open: bool, children: ChildrenFn) -> impl IntoView {
    let open = RwSignal::new(open);
    let (show_label, hide_label) = (StoredValue::new(show_label), StoredValue::new(hide_label));
    view! {
        <section class="rounded-control border border-ink/20 px-[25px] py-[15px] shadow-sm">
            <div class="flex items-center justify-between">
                <p class="text-body-md font-medium leading-5">{title}</p>
                <button type="button" on:click=move |_| open.update(|o| *o = !*o) aria-expanded=move || open.get().to_string() aria-label=move || if open.get() { hide_label.get_value() } else { show_label.get_value() } class="btn-circle btn-circle-clear btn-circle-sm">
                    <i class=move || if open.get() { "ph ph-minus text-body-md" } else { "ph ph-plus text-body-md" } aria-hidden="true"></i>
                </button>
            </div>
            <Show when=move || open.get()>{children()}</Show>
        </section>
    }
}
