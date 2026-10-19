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

/// Phones and tablets (narrower than a desktop window): the profile switch is the app's, a picture that flies to its place in the bar.
pub fn is_compact() -> bool {
    web_sys::window().and_then(|w| w.inner_width().ok()).and_then(|v| v.as_f64()).is_some_and(|w| w < 1024.0)
}

/// How long the picture stays in the middle before the profile is opened and the picture leaves, in milliseconds.
const HOLD_MS: i32 = 1100;

fn after(ms: i32, f: impl Fn() + 'static) {
    if let Some(win) = web_sys::window() {
        let cb = wasm_bindgen::closure::Closure::<dyn Fn()>::wrap(Box::new(f));
        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), ms);
        cb.forget();
    }
}

fn put(el: &web_sys::Element, css: &str) {
    let _ = el.set_attribute("style", css);
}

/// The switch as the Netflix app draws it (frames of the app): a black screen, the picture of the profile in the middle with a red arc turning under it while the profile opens, then the picture
/// shrinks and moves down to its place in the bar (the bar's profile tab on a phone, the header's picture on a tablet) while the black fades out, so that the page is just there. The place is
/// measured, not assumed, so it holds at every width. It is drawn with plain elements that this function adds to the page and takes away again, not with components: it has to outlive the
/// Who's watching screen and the page that replaces it, and no component or signal of theirs may be involved when they are gone.
pub fn run_compact_switch(profile: &Profile, on_activate: impl Fn() + 'static) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { on_activate(); return };
    let Some(body) = doc.body() else { on_activate(); return };
    let avatar = profile.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
    let (Ok(root), Ok(veil), Ok(arc), Ok(tile), Ok(img)) = (doc.create_element("div"), doc.create_element("div"), doc.create_element("div"), doc.create_element("div"), doc.create_element("img")) else { on_activate(); return };
    let _ = root.set_attribute("role", "status");
    let _ = root.set_attribute("aria-label", &profile.name);
    put(&root, "position:fixed;inset:0;z-index:10100;");
    put(&veil, "position:absolute;inset:0;background:#000;opacity:0;transition:opacity .15s ease-out;");
    put(&arc, "position:absolute;left:50%;top:50%;width:min(46vw,280px);height:min(46vw,280px);transform:translate(-50%,calc(-50% + min(50vw,300px)));opacity:0;transition:opacity .2s;");
    arc.set_inner_html(r##"<svg class="sw-phone-arc" style="width:100%;height:100%" viewBox="0 0 100 100" aria-hidden="true"><defs><linearGradient id="sw-phone-g" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#e50914" stop-opacity="0"/><stop offset="1" stop-color="#e50914"/></linearGradient></defs><circle cx="50" cy="50" r="44" fill="none" stroke="url(#sw-phone-g)" stroke-width="4.5" stroke-linecap="round" stroke-dasharray="190 276"/></svg>"##);
    put(&tile, "position:absolute;left:50%;top:44%;width:min(42vw,260px);height:min(42vw,260px);overflow:hidden;border-radius:10%;opacity:0;transform:translate(-50%,-50%);transition:opacity .12s ease-out;will-change:transform;");
    let _ = img.set_attribute("src", &avatar);
    let _ = img.set_attribute("alt", "");
    put(&img, "width:100%;height:100%;object-fit:cover;display:block;");
    let _ = tile.append_child(&img);
    let _ = root.append_child(&veil);
    let _ = root.append_child(&arc);
    let _ = root.append_child(&tile);
    let _ = body.append_child(&root);
    // the next frame: the black comes in and the picture and the arc appear
    {
        let (veil, arc, tile) = (veil.clone(), arc.clone(), tile.clone());
        after(16, move || {
            let _ = veil.set_attribute("style", "position:absolute;inset:0;background:#000;opacity:1;transition:opacity .15s ease-out;");
            let _ = arc.set_attribute("style", "position:absolute;left:50%;top:50%;width:min(46vw,280px);height:min(46vw,280px);transform:translate(-50%,calc(-50% + min(50vw,300px)));opacity:1;transition:opacity .2s;");
            let _ = tile.set_attribute("style", "position:absolute;left:50%;top:44%;width:min(42vw,260px);height:min(42vw,260px);overflow:hidden;border-radius:10%;opacity:1;transform:translate(-50%,-50%);transition:opacity .12s ease-out;will-change:transform;");
        });
    }
    // the picture is held in the middle for a moment; opening the profile loads the home page afresh (which ends this page), so the picture is handed on: the next page puts the same screen up at
    // once (index.html) and `resume_compact_switch` there flies the picture to its place in the bar
    let handed = profile.avatar_url.clone().unwrap_or_else(|| crate::brand::DEFAULT_AVATAR.to_string());
    after(HOLD_MS, move || {
        if let Some(storage) = web_sys::window().and_then(|w| w.session_storage().ok().flatten()) {
            let _ = storage.set_item("streams_sw_pending", &serde_json::json!({ "a": handed, "t": js_sys::Date::now() as u64 }).to_string());
        }
        on_activate();
    });
}

/// At the start of the app: when the page before handed a profile switch on (see `run_compact_switch`), the screen it left is already up (index.html); this flies its picture to the bar and takes it away.
pub fn resume_compact_switch() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Some(root) = doc.get_element_by_id("sw-boot") else { return };
    let kids = root.children();
    let (Some(veil), Some(arc), Some(tile)) = (kids.item(0), kids.item(1), kids.item(2)) else { root.remove(); return };
    // whatever happens, the screen does not stay
    { let root = root.clone(); after(9000, move || root.remove()); }
    fly(root, veil, arc, tile, 0);
}

/// Sends the picture to the visible `[data-profile-avatar]`; waits up to about 1.5 s for the page to put it there (the profile has just been opened), and fades out without a flight when it never shows.
fn fly(root: web_sys::Element, veil: web_sys::Element, arc: web_sys::Element, tile: web_sys::Element, tries: u32) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { root.remove(); return };
    let target = doc.query_selector_all("[data-profile-avatar]").ok().and_then(|list| {
        (0..list.length()).filter_map(|i| list.item(i)).filter_map(|n| n.dyn_into::<web_sys::Element>().ok()).map(|e| e.get_bounding_client_rect()).find(|r| r.width() > 4.0 && r.height() > 4.0)
    });
    let Some(t) = target else {
        if tries < 80 {
            after(50, move || fly(root.clone(), veil.clone(), arc.clone(), tile.clone(), tries + 1));
        } else {
            put(&veil, "position:absolute;inset:0;background:#000;opacity:0;transition:opacity .42s ease-out;");
            put(&tile, "position:absolute;left:50%;top:44%;width:min(42vw,260px);height:min(42vw,260px);overflow:hidden;border-radius:10%;opacity:0;transform:translate(-50%,-50%);transition:opacity .3s;");
            after(600, move || root.remove());
        }
        return;
    };
    let from = tile.get_bounding_client_rect();
    let (fx, fy) = (from.left() + from.width() / 2.0, from.top() + from.height() / 2.0);
    let (tx, ty) = (t.left() + t.width() / 2.0, t.top() + t.height() / 2.0);
    let scale = t.width() / from.width().max(1.0);
    put(&arc, "position:absolute;left:50%;top:50%;width:min(46vw,280px);height:min(46vw,280px);transform:translate(-50%,calc(-50% + min(50vw,300px)));opacity:0;transition:opacity .2s;");
    put(&veil, "position:absolute;inset:0;background:#000;opacity:0;transition:opacity .42s ease-out .1s;");
    put(&tile, &format!("position:absolute;left:50%;top:44%;width:min(42vw,260px);height:min(42vw,260px);overflow:hidden;border-radius:10%;opacity:1;will-change:transform;transform:translate(-50%,-50%) translate({:.1}px,{:.1}px) scale({:.4});transition:transform .52s cubic-bezier(.32,.08,.24,1);", tx - fx, ty - fy, scale));
    after(620, move || root.remove());
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
    // a phone or a tablet: the picture flies to the bar (plain elements, no component involved)
    if is_compact() && !profile.is_kids {
        let id = profile.id.clone();
        run_compact_switch(&profile, move || crate::store::activate_profile(&id));
        return;
    }
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
