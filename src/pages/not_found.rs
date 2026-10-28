//! Netflix's page for an address the site does not have: "Lost your way?" over a still from Lost in Space, at `/NotFound?prev=<address>`
//! (netflix.com read live on 2026-10-27, note 138). Sizes follow the width as Netflix's do (measured at 1736 px: heading 4.5 vw, text
//! 1.65 vw, button 1.2 vw, error code 1.8 vw, credit 1.05 vw), with a floor for phones.
use crate::i18n::{t, t_args};
use leptos::prelude::*;

/// The still behind the page: Lost in Space (TMDB tv 75758), an icy ridge like the one Netflix shows, without words on it.
const STILL: &str = "/hqSTgj4AmxSfGssj4hoYwlzq6mv.jpg";
/// Netflix's own code for this page, kept as the page shows it.
const ERROR_CODE: &str = "NSES-404";

#[component]
pub fn NotFoundPage() -> impl IntoView {
    // the tab says the brand alone, as Netflix's says "Netflix"
    crate::pages::help::set_site_title("");
    let still = |size: &str| format!("{}/{size}{STILL}", crate::config::TMDB_IMAGE_URL);
    // the 1280 px copy stands for screens up to 1920 px, so a computer does not wait for the 3840 px original behind the words
    let srcset = format!("{} 780w, {} 1920w, {} 3840w", still("w780"), still("w1280"), still("original"));
    view! {
        <div class="relative min-h-screen overflow-hidden bg-panel font-sans text-ink">
            <img src=still("w1280") srcset=srcset sizes="100vw" alt="" class="absolute inset-0 h-full w-full object-cover" />
            <header class="fixed inset-x-0 top-0 z-(--z-nav) flex h-[68px] items-center bg-void/97 px-6 lg:px-10">
                <a href=crate::config::routes::HOME aria-label=crate::brand::NAME>
                    <img src=crate::brand::LOGO_WORDMARK_PRIMARY alt=crate::brand::NAME class="h-[25px] w-auto" />
                </a>
            </header>
            <main class="relative flex min-h-screen flex-col items-center justify-center px-4 pt-[68px] text-center">
                <h1 class="mb-[2vw] text-[clamp(36px,4.5vw,96px)] leading-[1.2] font-medium [text-shadow:0_1px_2px_rgb(0_0_0/0.57)]">{t("notfound-title")}</h1>
                <p class="mb-[1em] max-w-[max(50vw,288px)] text-[clamp(18px,1.65vw,36px)] leading-[1.2] font-light [text-shadow:0_1px_2px_rgb(0_0_0/0.57)]">{t("notfound-body")}</p>
                <a href=crate::config::routes::HOME class="rounded-button bg-ink px-[clamp(20px,1.8vw,40px)] py-[clamp(8px,0.6vw,14px)] text-[clamp(16px,1.2vw,26px)] leading-[1.5] font-medium text-void">
                    {t_args("notfound-home", &[("brand", crate::brand::NAME.into())])}
                </a>
                <div class="mt-[1.5vw]">
                    <span class="inline-block border-s-2 border-brand px-[1vw] text-[clamp(18px,1.8vw,40px)] leading-[2.2] font-thin tracking-[0.1em]">
                        {t("notfound-code")}" "<strong class="font-medium">{ERROR_CODE}</strong>
                    </span>
                </div>
            </main>
            <span class="fixed bottom-[2vw] end-[4vw] text-[clamp(11px,1.05vw,22px)] font-thin tracking-[0.1em] text-ink/57">
                {t("notfound-from")}" "<strong class="font-medium text-ink/77">{t("notfound-still")}</strong>
            </span>
        </div>
    }
}
