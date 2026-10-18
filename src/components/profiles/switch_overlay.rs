//! The moment between choosing a profile and its home page, drawn as Netflix does it (measured on frames of netflix.com taken 0.3 s apart): the logo at the
//! top left on the page colour, a square frame in the accent colour that turns once round the grey place of the picture, then the picture itself
//! with a ring in the accent colour turning round it. There is no name.
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::models::profile::Profile;

/// How long the overlay stays before the profile is opened, in milliseconds (the square and the first turns of the ring).
const SHOWN_MS: i32 = 1500;

#[component]
pub fn ProfileSwitchOverlay(
    profile: Profile,
    on_finish: Callback<()>,
) -> impl IntoView {
    let name = profile.name.clone();
    let avatar = profile.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
    let is_kids = profile.is_kids;

    Effect::new(move |_| {
        if let Some(win) = web_sys::window() {
            let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::wrap(Box::new(move || {
                on_finish.run(());
            }));
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), SHOWN_MS);
            cb.forget();
        }
    });

    // a Kids profile opens behind the Kids screen (bands of colour and the word); the others show the logo, the square and the ring
    if is_kids {
        return view! {
            <div class="fixed inset-0 z-(--z-takeover) overflow-hidden bg-page animate-in fade-in duration-(--motion-base)" role="status" aria-label=name>
                <crate::components::profiles::kids_splash::KidsSplash class="h-full w-full" />
            </div>
        }.into_any();
    }
    view! {
        <div class="fixed inset-0 z-(--z-takeover) overflow-hidden bg-page animate-in fade-in duration-(--motion-base)" role="status" aria-label=name.clone()>
            <img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="absolute left-[var(--app-x,56px)] top-[22px] h-[25px] w-auto" />
            <div class="absolute left-1/2 top-1/2 h-[188px] w-[188px] -translate-x-1/2 -translate-y-1/2">
                <span class="sw-square absolute left-1/2 top-1/2" aria-hidden="true"></span>
                <svg class="sw-arc absolute inset-0" width="188" height="188" viewBox="0 0 188 188" aria-hidden="true">
                    <circle cx="94" cy="94" r="90" fill="none" style="stroke: var(--color-brand)" stroke-width="7" stroke-linecap="round" stroke-dasharray="230 336" />
                </svg>
                <div class="absolute left-1/2 top-1/2 h-[67px] w-[67px] -translate-x-1/2 -translate-y-1/2 overflow-hidden bg-line">
                    <div class="sw-picture h-full w-full">
                        <img src=avatar alt="" class="h-full w-full object-cover" />
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}

/// The profile being switched to, for the screen shown at the app's root: any button that changes profile (the menu, the Kids and Exit Kids buttons) sets it, the screen
/// plays for a moment and then the profile is opened afresh, as the Who's watching screen does.
#[derive(Clone, Copy)]
pub struct SwitchState(pub RwSignal<Option<Profile>>);

pub fn provide_switch() {
    provide_context(SwitchState(RwSignal::new(None)));
}

/// Starts the switch to `profile` with the loading screen.
pub fn begin_switch(profile: Profile) {
    if let Some(state) = use_context::<SwitchState>() {
        state.0.set(Some(profile));
    }
}

#[component]
pub fn GlobalSwitchOverlay() -> impl IntoView {
    let state = use_context::<SwitchState>();
    view! {
        {move || state.and_then(|s| s.0.get()).map(|p| {
            let id = p.id.clone();
            view! {
                <ProfileSwitchOverlay profile=p on_finish=Callback::new(move |_| crate::store::activate_profile(&id)) />
            }
        })}
    }
}
