//! The foot of the website's account pages: "Questions? Contact us." and the links in four columns, 14 px grey and underlined, as measured on netflix.com
//! (columns start at 202, 567, 932 and 1296 px of a 1851 px wide page, rows 34 px apart). Only from 1024 px wide; phones have none.
use crate::config::routes;
use crate::i18n::t;
use leptos::prelude::*;

#[component]
pub fn AccountFooter() -> impl IntoView {
    let consent = crate::consent::use_ui();
    let link = "text-body-sm text-chrome/70 underline hover:text-chrome";
    view! {
        <footer class="set-gutter hidden pb-16 pt-[64px] text-chrome lg:block">
            <p class="mb-[16px] text-body-md leading-[21px]">{t("set-questions")}" "<a href=routes::HELP_CONTACT class="underline">{t("footer-contact")}</a>"."</p>
            <nav class="grid grid-cols-4 gap-x-[20px] gap-y-[16px]" aria-label=t("footer-help")>
                <a href=routes::HELP class=link>{t("footer-help")}</a>
                <a href=crate::config::routes::TERMS class=link>{t("footer-terms")}</a>
                <a href=crate::config::routes::PRIVACY class=link>{t("footer-privacy")}</a>
                <button type="button" on:click=move |_| consent.open_center() class=format!("{link} text-start")>{t("footer-cookie-preferences")}</button>
                <a href=crate::config::routes::COOKIES class=link>{t("footer-cookie-policy")}</a>
                <a href=crate::config::routes::DMCA class=link>{t("footer-dmca")}</a>
                <a href=crate::config::routes::DISCLAIMER class=link>{t("footer-disclaimer")}</a>
                <a href=crate::config::routes::CREDITS class=link>{t("help-ui-legal-credits")}</a>
            </nav>
        </footer>
    }
}
