use leptos::prelude::*;
use leptos::html::{Div, Input};
use leptos_router::hooks::{use_navigate, use_query_map};

#[component]
pub fn SearchBar() -> impl IntoView {
    let query_map = use_query_map();
    let query_val = move || query_map.read().get("q").unwrap_or_default();
    
    let (is_active, set_is_active) = signal(false);
    let (input_val, set_input_val) = signal(String::new());
    
    let input_ref = NodeRef::<Input>::new();
    let container_ref = NodeRef::<Div>::new();

    let location = leptos_router::hooks::use_location();
    let is_search_page = move || location.pathname.get().starts_with("/search");

    // Sync input val with URL on mount or URL change
    Effect::new(move |_| {
        let q = query_val();
        set_input_val.set(q.clone());
        if !q.is_empty() || is_search_page() {
            set_is_active.set(true);
        }
    });

    // Auto-focus input when opened
    Effect::new(move |_| {
        if is_active.get() || is_search_page() {
            if let Some(input_el) = input_ref.get() {
                let _ = input_el.focus();
            }
        }
    });

    // a press anywhere else folds an empty search away (not on the search page)
    crate::utils::listen::on_press_outside(
        move |target| crate::utils::listen::contains(container_ref, target),
        move || {
            if !is_search_page() && input_val.get_untracked().trim().is_empty() {
                set_is_active.set(false);
            }
        },
    );

    let navigate = use_navigate();
    let timer_id = StoredValue::new(None::<leptos::leptos_dom::helpers::TimeoutHandle>);
    on_cleanup(move || if let Some(h) = timer_id.get_value() { h.clear() });
    
    // 300ms live debounced search query execution (Task 090)
    let navigate_debounced = navigate.clone();
    let on_input = move |e: leptos::ev::Event| {
        let val = event_target_value(&e);
        set_input_val.set(val.clone());
        
        // the framework's timer: the previous one is cleared, the closure freed after it runs (one leaked per key press before), and none
        // is left when the bar goes
        if let Some(h) = timer_id.get_value() {
            h.clear();
        }
        let nav = navigate_debounced.clone();
        let query = val.clone();
        let handle = leptos::leptos_dom::helpers::set_timeout_with_handle(
            move || {
                let target_url = if query.trim().is_empty() {
                    "/search".to_string()
                } else {
                    let encoded = js_sys::encode_uri_component(&query).as_string().unwrap_or(query.clone());
                    format!("/search?q={}", encoded)
                };
                nav(&target_url, Default::default());
            },
            std::time::Duration::from_millis(300),
        );
        timer_id.set_value(handle.ok());
    };

    let navigate_toggle = navigate.clone();
    let toggle_search = move |_| {
        let active = is_active.get();
        if !active {
            set_is_active.set(true);
            if let Some(input_el) = input_ref.get() {
                let _ = input_el.focus();
            }
            if input_val.get().is_empty() {
                navigate_toggle("/search", Default::default());
            }
        }
    };

    // Instant clear (X) button (Task 090)
    let stored_navigate = StoredValue::new(navigate.clone());
    let clear_search = move |_| {
        timer_id.update_value(|id| if let Some(h) = id.take() { h.clear() });
        set_input_val.set("".to_string());
        stored_navigate.with_value(|n| n("/search", Default::default()));
        if let Some(input_el) = input_ref.get() {
            let _ = input_el.focus();
        }
    };

    let on_keydown = move |e: leptos::ev::KeyboardEvent| {
        if e.key() == "Escape" {
            timer_id.update_value(|id| if let Some(h) = id.take() { h.clear() });
            set_input_val.set("".to_string());
            stored_navigate.with_value(|n| n("/browse", Default::default()));
            set_is_active.set(false);
        }
    };

    let effective_active = move || is_active.get() || is_search_page() || !input_val.get().is_empty();

    view! {
        <div node_ref=container_ref class="relative flex items-center">
            <div 
                // Netflix: closed = 40x40 icon button, open = 400x56 field, radius 28, #161616, 1px grey 70% border
                class="relative flex items-center transition-all duration-(--motion-slow) ease-out overflow-hidden rounded-input"
                class=("bg-raised", effective_active)
                class=("border", effective_active)
                class=("border-mute/70", effective_active)
                class=("w-[220px]", effective_active)
                class=("min-[960px]:w-[400px]", effective_active)
                class=("h-14", effective_active)
                class=("px-4", effective_active)
                class=("bg-transparent", move || !effective_active())
                class=("border-transparent", move || !effective_active())
                class=("w-10", move || !effective_active())
                class=("h-10", move || !effective_active())
                class=("justify-center", move || !effective_active())
            >
                <button
                    on:click=toggle_search
                    class="focus:outline-none flex items-center justify-center z-10 shrink-0 w-6 h-6"
                    aria-label=crate::i18n::t("search-bar-search")
                >
                    <i class="ph ph-magnifying-glass text-heading cursor-pointer select-none transition-colors duration-(--motion-slow)"
                       class=("text-ink", effective_active)
                       class=("hover:text-gray-300", move || !effective_active())>
                    </i>
                </button>

                <input
                    node_ref=input_ref
                    type="text"
                    placeholder=crate::i18n::t("search-bar-titles-people-genres")
                    class="bg-transparent border-none outline-none text-ink text-body-md ml-3 transition-all duration-(--motion-slow) font-sans"
                    class=("w-full", effective_active)
                    class=("opacity-100", effective_active)
                    class=("w-0", move || !effective_active())
                    class=("opacity-0", move || !effective_active())
                    class=("pointer-events-none", move || !effective_active())
                    prop:value=move || input_val.get()
                    on:input=on_input
                    on:keydown=on_keydown
                    aria-label=crate::i18n::t("search-bar-search-input")
                />

                <Show when=move || effective_active()>
                    <button
                        on:mousedown=clear_search
                        class="text-ink/70 hover:text-ink cursor-pointer mx-1 flex-shrink-0 transition-all duration-(--motion-base) hover:scale-125 active:scale-95"
                        aria-label=crate::i18n::t("search-bar-clear-search")
                    >
                        <i class="ph-bold ph-x text-title"></i>
                    </button>
                </Show>
            </div>
        </div>
    }
}
