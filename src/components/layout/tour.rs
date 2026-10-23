//! The first-visit guide: a short tour of the top bar for a profile that has not seen it. Steps are data (the elements they point at by `data-tour` name, and
//! translation keys for their text); adding a step or a new tour (another id or version) shows it once to every profile that has not seen that one. Close counts as seen.
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub struct Step {
    /// `data-tour` names of the elements the step points at; the card sits under the first one that is on the page, the highlight covers all that are
    pub targets: &'static [&'static str],
    pub title: &'static str,
    pub text: &'static str,
}

pub struct Tour {
    pub id: &'static str,
    /// Raise it to show the tour again to everyone who saw an older version
    pub version: u32,
    pub steps: &'static [Step],
}

pub const FIRST_VISIT: Tour = Tour {
    id: "first-visit",
    version: 1,
    steps: &[
        Step { targets: &["nav-tv", "nav-movies"], title: "tour-find-title", text: "tour-find-text" },
        Step { targets: &["nav-new"], title: "tour-new-title", text: "tour-new-text" },
        Step { targets: &["nav-list"], title: "tour-list-title", text: "tour-list-text" },
    ],
};

fn seen_key(tour: &Tour, profile: &str) -> String {
    format!("streams_tour_{}_v{}_{}", tour.id, tour.version, profile)
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn is_seen(key: &str) -> bool {
    storage().and_then(|s| s.get_item(key).ok().flatten()).is_some()
}

fn mark_seen(key: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(key, "1");
    }
}

/// The box (left, top, right, bottom) around the on-page elements named by `targets`, None when none is on the page.
fn target_box(targets: &[&str]) -> Option<(f64, f64, f64, f64)> {
    let doc = web_sys::window()?.document()?;
    let mut out: Option<(f64, f64, f64, f64)> = None;
    for t in targets {
        let Some(el) = doc.query_selector(&format!("[data-tour=\"{t}\"]")).ok().flatten() else { continue };
        let r = el.get_bounding_client_rect();
        if r.width() == 0.0 { continue; }
        out = Some(match out {
            None => (r.left(), r.top(), r.right(), r.bottom()),
            Some((l, t, rr, b)) => (l.min(r.left()), t.min(r.top()), rr.max(r.right()), b.max(r.bottom())),
        });
    }
    out
}

/// Whether this browser should show a tour: a desktop width (the top bar is only there from 640 px; the tour is for 1024 px and wider) on a browsing page.
fn can_show() -> bool {
    let Some(win) = web_sys::window() else { return false };
    let wide = win.inner_width().ok().and_then(|w| w.as_f64()).is_some_and(|w| w >= 1024.0);
    let path = win.location().pathname().unwrap_or_default();
    wide && (path.starts_with("/browse") || path.starts_with("/latest"))
}

#[component]
pub fn TourHost() -> impl IntoView {
    let tour = &FIRST_VISIT;
    let profile = move || crate::store::use_profile_store().active_profile_id.get().unwrap_or_else(|| "guest".to_string());
    let (open, set_open) = signal(false);
    let (step, set_step) = signal(0usize);
    let (boxed, set_boxed) = signal(None::<(f64, f64, f64, f64)>);
    let card_ref = NodeRef::<leptos::html::Div>::new();

    // after the page has settled: show the tour to a profile that has not seen it
    Effect::new(move |_| {
        let key = seen_key(tour, &profile());
        if is_seen(&key) { return; }
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
            if can_show() && !is_seen(&key) { set_step.set(0); set_open.set(true); }
        });
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 1500);
        }
    });

    // a step whose elements are not on the page is skipped; the box is measured again when the step or the window changes
    Effect::new(move |_| {
        if !open.get() { return; }
        let mut i = step.get();
        while i < tour.steps.len() && target_box(tour.steps[i].targets).is_none() { i += 1; }
        if i >= tour.steps.len() { set_open.set(false); mark_seen(&seen_key(tour, &profile())); return; }
        if i != step.get_untracked() { set_step.set(i); return; }
        let b = target_box(tour.steps[i].targets);
        if b != boxed.get_untracked() { set_boxed.set(b); }
    });
    // the card takes the focus when it appears (a separate effect: setting the box here would draw the card again, and that would run this effect again)
    Effect::new(move |_| {
        if let Some(card) = card_ref.get() { let _ = card.focus(); }
    });
    crate::utils::listen::listen_window(leptos::ev::resize, move |_| {
        if open.get_untracked() { set_boxed.set(target_box(tour.steps[step.get_untracked().min(tour.steps.len() - 1)].targets)); }
    });

    let close = move || {
        set_open.set(false);
        mark_seen(&seen_key(tour, &profile()));
    };
    let next = move || {
        if step.get_untracked() + 1 >= tour.steps.len() { close(); } else { set_step.update(|s| *s += 1); }
    };
    let back = move || set_step.update(|s| *s = s.saturating_sub(1));

    view! {
        {move || {
            if !open.get() { return view! { <span /> }.into_any(); }
            let s = &tour.steps[step.get().min(tour.steps.len() - 1)];
            let (l, t, r, b) = boxed.get().unwrap_or((0.0, 0.0, 0.0, 0.0));
            let total = tour.steps.len() as i64;
            let n = step.get() as i64 + 1;
            let last = step.get() + 1 >= tour.steps.len();
            view! {
                <div class="fixed inset-0" style="z-index: 9000;" on:click=move |_| close()>
                    <div class="absolute inset-0 bg-chrome/60" />
                    <div
                        class="absolute rounded-full ring-2 ring-chrome-ink/80 pointer-events-none"
                        style=format!("left:{}px;top:{}px;width:{}px;height:{}px;", l - 6.0, t - 4.0, r - l + 12.0, b - t + 8.0)
                    />
                    <div
                        node_ref=card_ref
                        tabindex="-1"
                        role="dialog"
                        aria-modal="true"
                        aria-label=crate::i18n::t(s.title)
                        class="absolute w-[min(340px,calc(100vw-32px))] rounded-card bg-raised text-ink shadow-2xl p-5 outline-none"
                        style=format!("left:{}px;top:{}px;", l.max(16.0), b + 16.0)
                        on:click=|e| e.stop_propagation()
                        on:keydown=move |e: leptos::ev::KeyboardEvent| match e.key().as_str() {
                            "Escape" => close(),
                            "ArrowRight" => next(),
                            "ArrowLeft" => back(),
                            _ => {}
                        }
                    >
                        <button type="button" class="absolute top-3 right-3 text-ink/70 hover:text-ink" aria-label=crate::i18n::t("tour-close") on:click=move |_| close()>
                            <i class="ph-bold ph-x" aria-hidden="true"></i>
                        </button>
                        <h2 class="text-lg font-semibold pr-6">{crate::i18n::t(s.title)}</h2>
                        <p class="mt-2 text-sm text-ink/80">{crate::i18n::t(s.text)}</p>
                        <div class="mt-4 flex items-center justify-between">
                            <span class="text-xs text-ink/60">{crate::i18n::t_args("tour-step-of", &[("n", fluent_bundle::FluentValue::from(n)), ("total", fluent_bundle::FluentValue::from(total))])}</span>
                            <div class="flex gap-2">
                                <button type="button" class="rounded-button border border-ink/40 px-4 py-1.5 text-sm disabled:opacity-40" disabled=move || step.get() == 0 on:click=move |_| back()>{crate::i18n::t("tour-back")}</button>
                                <button type="button" class="rounded-button bg-ink text-void px-4 py-1.5 text-sm font-semibold" on:click=move |_| next()>{crate::i18n::t(if last { "tour-done" } else { "tour-next" })}</button>
                            </div>
                        </div>
                    </div>
                </div>
            }.into_any()
        }}
    }
}
