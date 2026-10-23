use leptos::prelude::*;

#[component]
pub fn ReadsPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-page text-ink pt-24 pb-12 px-4 md:px-12">
            <h1 class="text-2xl md:text-3xl font-bold mb-6">{crate::i18n::t("reads-title")}</h1>
            <div class="px-4 py-8 bg-raised/50 rounded-card border border-ink/10">
                <div class="text-ink/40 text-sm">{crate::i18n::t("reads-empty")}</div>
            </div>
        </div>
    }
}
