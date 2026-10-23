use leptos::prelude::*;

#[component]
pub fn HeroSkeleton() -> impl IntoView {
    view! {
        <div class="w-full relative overflow-hidden bg-transparent select-none">
            // 1. MOBILE SKELETON (< 640px)
            <div class="block sm:hidden relative w-full px-5 pt-[calc(124px+env(safe-area-inset-top))] pb-6 flex flex-col items-center justify-center animate-pulse">
                <div class="absolute inset-x-0 top-0 h-[50vh] bg-gradient-to-b from-ink/[0.04] to-transparent pointer-events-none -z-10" />
                <div class="absolute top-[15%] w-[80%] aspect-square rounded-full bg-ink/[0.02] filter blur-[40px] pointer-events-none -z-10" />

                <div class="w-full max-w-[560px] aspect-[5/7] bg-page rounded-poster border border-ink/20 relative flex flex-col justify-end p-4">
                    <div class="absolute inset-0 bg-gradient-to-t from-void/90 via-void/40 to-transparent pointer-events-none" />

                    <div class="relative z-10 w-full flex flex-col items-center gap-3">
                        <div class="h-10 w-1/2 bg-ink/10 rounded-field" />
                        <div class="h-3 w-1/3 bg-ink/[0.06] rounded-control" />
                        <div class="flex w-full gap-3 mt-2">
                            <div class="h-[50px] sm:h-[56px] flex-1 bg-ink/10 rounded-control" />
                            <div class="h-[50px] sm:h-[56px] flex-1 bg-ink/[0.05] rounded-control" />
                        </div>
                    </div>
                </div>
            </div>

            // 2. TABLET SKELETON (640px - 767px)
            <div class="hidden sm:flex md:hidden relative w-full h-[60vh] min-h-[480px] bg-page flex-col justify-end pl-12 pb-12 pr-6 animate-pulse">
                <div class="absolute inset-0 bg-gradient-to-tr from-void/90 via-void/40 to-transparent pointer-events-none" />
                <div class="absolute inset-0 bg-ink/[0.02] pointer-events-none" />

                <div class="relative z-10 w-full max-w-[480px] flex flex-col items-start gap-4">
                    <div class="h-16 w-3/4 bg-ink/10 rounded-card shadow-lg" />
                    <div class="h-4 w-1/2 bg-ink/[0.08] rounded-control" />
                    <div class="flex gap-3 w-full max-w-[340px]">
                        <div class="h-[50px] sm:h-[56px] w-[140px] bg-ink/10 rounded-control" />
                        <div class="h-[50px] sm:h-[56px] w-[140px] bg-ink/[0.05] rounded-control" />
                    </div>
                </div>
            </div>

            // 3. DESKTOP SKELETON (>= 768px)
            <div class="hidden md:block w-full billboard-margin">
                <div class="relative w-full aspect-[2.18/1] rounded-billboard overflow-hidden bg-raised border border-ink/[0.04] p-9 flex flex-col justify-end animate-pulse">
                    <div class="relative z-20 w-full max-w-xl flex flex-col items-start gap-3">
                        <div class="h-16 w-3/5 bg-ink/10 rounded-card shadow-2xl" />
                        <div class="flex gap-2">
                            <div class="h-4 w-16 bg-ink/10 rounded-control" />
                            <div class="h-4 w-14 bg-ink/10 rounded-control" />
                            <div class="h-4 w-16 bg-ink/10 rounded-control" />
                            <div class="h-5 w-5 bg-ink/20 rounded-full" />
                        </div>
                        <div class="space-y-2 w-full max-w-md">
                            <div class="h-3.5 w-full bg-ink/[0.07] rounded-control" />
                            <div class="h-3.5 w-4/5 bg-ink/[0.07] rounded-control" />
                        </div>
                        <div class="flex gap-3 pt-2">
                            <div class="h-10 w-28 bg-ink/15 rounded-full" />
                            <div class="h-10 w-32 bg-ink/10 rounded-full" />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
