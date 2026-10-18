use leptos::prelude::*;


pub struct BottomNavItem {
    pub id: &'static str,
    pub icon: AnyView,
    pub label: &'static str,
    pub on_click: Callback<()>,
    /// False hides the tab (the Kids app has no Clips)
    pub shown: Signal<bool>,
}

#[component]
pub fn BottomNavMobile(
    items: Vec<BottomNavItem>,
    active_id: ReadSignal<&'static str>,
) -> impl IntoView {
    view! {
        <div class="fixed bottom-[calc(8px+env(safe-area-inset-bottom))] left-1/2 -translate-x-1/2 z-(--z-bottom-nav)
                    w-auto max-w-[calc(100%-2rem)]
                    bg-menu border border-ink/[0.08] rounded-full
                    px-2.5 py-[7px] shadow-popover">
            <div class="flex items-center">
                {items.into_iter().map(|item| {
                    let id = item.id;
                    let label = item.label;
                    let icon = item.icon;
                    let on_click = item.on_click;
                    let shown = item.shown;

                    let is_active = move || active_id.get() == id;

                    view! {
                        <button
                            on:click=move |_| on_click.run(())
                            class="relative flex flex-col items-center justify-center gap-0.5 rounded-full w-[76px] h-12 select-none transition-colors duration-(--motion-base) active:scale-95"
                            class=("hidden", move || !shown.get())
                            class=("bg-lift", move || is_active())
                            class=("text-ink", move || is_active())
                            class=("text-ink-silver-alt", move || !is_active())
                            class=("hover:text-ink/80", move || !is_active())
                        >
                            <span class="flex items-center justify-center h-6">
                                {icon}
                            </span>
                            <span 
                                class="text-caption whitespace-nowrap"
                                class=("font-medium", move || is_active())
                                class=("font-normal", move || !is_active())
                            >
                                {crate::i18n::t(label)}
                            </span>
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
