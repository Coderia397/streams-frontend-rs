use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    let consent = crate::consent::use_ui();
    view! {
        <footer class="w-full bg-page text-mute py-12 px-6 md:px-20 lg:px-32 xl:px-44 2xl:px-56 text-sm mt-auto border-t border-ink/5">
            <div class="max-w-[1000px] mx-auto">
                // Links Grid
                <nav class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-y-4 gap-x-8 mb-8 text-xs md:text-sm">
                    <a href="/browse" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("nav-home")}</a>
                    <a href="/browse/genre/83" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-tv-shows")}</a>
                    <a href="/browse/genre/34399" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-movies")}</a>
                    <a href="/latest" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("nav-new-popular")}</a>
                    <a href="/browse/my-list" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("nav-my-list")}</a>
                    <a href="/settings" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-settings")}</a>
                    <a href=crate::config::routes::PRIVACY class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-privacy")}</a>
                    <a href=crate::config::routes::TERMS class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-terms")}</a>
                    <a href=crate::config::routes::DMCA class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-dmca")}</a>
                    <a href=crate::config::routes::DISCLAIMER class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-disclaimer")}</a>
                    <a href=crate::config::routes::COOKIES class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-cookie-policy")}</a>
                    <a href="/contact" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-contact")}</a>
                    <a href="/help" class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-help")}</a>
                    <a href=crate::config::routes::CREDITS class="hover:underline hover:text-ink transition-colors">{crate::i18n::t("help-ui-legal-credits")}</a>
                    <button on:click=move |_| consent.open_center() class="text-start hover:underline hover:text-ink transition-colors">{crate::i18n::t("footer-cookie-preferences")}</button>
                </nav>

                // Copyright
                <div class="text-xs text-ink/30 mt-8">
                    {format!("\u{a9} 2026 {}", crate::brand::COMPANY)}
                </div>
            </div>
        </footer>
    }
}
