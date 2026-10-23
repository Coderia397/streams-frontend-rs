use leptos::prelude::*;
use std::time::Duration;

#[component]
pub fn TooltipWrapper(
    label: String,
    children: Children,
    #[prop(default = String::new())] class: String,
) -> impl IntoView {
    let show = RwSignal::new(false);
    let timer_handle = RwSignal::new_local(None::<leptos::leptos_dom::helpers::TimeoutHandle>);

    // a tooltip is for a pointer that rests (a mouse or a pen); a tap fires pointer and mouse events too, and its tooltip stuck
    let handle_mouse_enter = move |e: leptos::ev::PointerEvent| {
        if !crate::form::input::hovers(&e) || timer_handle.get_untracked().is_some() {
            return;
        }
        let handle = leptos::leptos_dom::helpers::set_timeout_with_handle(
            move || {
                timer_handle.set(None);
                show.set(true);
            },
            Duration::from_millis(300),
        );
        if let Ok(h) = handle {
            timer_handle.set(Some(h));
        }
    };

    let handle_mouse_leave = move |_: leptos::ev::PointerEvent| {
        if let Some(h) = timer_handle.get_untracked() {
            h.clear();
            timer_handle.set(None);
        }
        show.set(false);
    };

    let label_stored = StoredValue::new(label);

    view! {
        <div
            class=format!("relative flex items-center justify-center {}", class)
            on:pointerenter=handle_mouse_enter
            on:pointerleave=handle_mouse_leave
        >
            <div
                class="absolute bottom-full left-1/2 mb-3 flex flex-col items-center z-(--z-tooltip) pointer-events-none transition-all duration-(--motion-quick)"
                style=move || if show.get() {
                    "transform: translateX(-50%) translateY(0px) scale(1); opacity: 1; visibility: visible;"
                } else {
                    "transform: translateX(-50%) translateY(4px) scale(0.9); opacity: 0; visibility: hidden;"
                }
            >
                <div class="bg-ink-soft text-page text-body font-extrabold px-5 py-3 rounded-micro shadow-card whitespace-nowrap leading-none select-none">
                    {label_stored.get_value()}
                </div>
                <div class="w-0 h-0 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-t-[6px] border-t-ink-soft -mt-[1px]" />
            </div>
            {children()}
        </div>
    }
}
