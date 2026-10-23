use streams_frontend_rs::App;

fn main() {
    console_error_panic_hook::set_once();
    streams_frontend_rs::legacy_keys::migrate();
    mark("streams:start");
    wasm_bindgen_futures::spawn_local(async {
        streams_frontend_rs::i18n::load().await;
        mark("streams:text");
        leptos::mount::mount_to_body(App);
        mark("streams:mounted");
        // The splash only appears if starting takes longer than 1.2 s and the last visit is long ago (see index.html). It fades out.
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(streams_frontend_rs::keys::BOOT_SEEN, &(js_sys::Date::now() as u64).to_string());
        }
        if let Some(boot) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("boot")) {
            let _ = boot.class_list().add_1("done");
            let gone = wasm_bindgen::closure::Closure::once_into_js(move || boot.remove());
            if let Some(w) = web_sys::window() {
                let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(wasm_bindgen::JsCast::unchecked_ref(&gone), 220);
            }
        }
    });
}


/// A point in the start sequence on the page's performance timeline (DevTools, `performance.getEntriesByName`), to measure how long starting takes.
fn mark(name: &str) {
    if let Some(p) = web_sys::window().and_then(|w| w.performance()) {
        let _ = p.mark(name);
    }
}
