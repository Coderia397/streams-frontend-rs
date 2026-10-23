//! Place holders while content loads (note 114): one look everywhere (`.skeleton` in style/components.css: the brand's surface and a
//! slow light that sweeps across, still for reduced motion), each shaped by the same classes and tokens as the thing it stands for (the
//! row title's line, the tile's width, shape and gap, the billboard's ratio and inset), so nothing moves when the content arrives. Every
//! place that waits draws one of these; a new kind of place holder is added here, not drawn by hand.
use leptos::prelude::*;

/// One block of a skeleton; `class` gives its size and corners.
#[component]
pub fn Bone(#[prop(into)] class: String) -> impl IntoView {
    view! { <div class=format!("skeleton {class}") aria-hidden="true"></div> }
}

/// A row title's line: the row title's own class keeps its height, a bar stands for the words.
#[component]
fn TitleLine(#[prop(default = "w-48")] width: &'static str) -> impl IntoView {
    view! {
        <h2 class="row-title px-(--app-x)" aria-hidden="true">
            <span class=format!("skeleton inline-block h-[0.7em] max-w-[60%] rounded-control align-middle {width}")></span>
        </h2>
    }
}

/// A row of tiles: the title line and the tiles in the row's own columns, shapes and gaps (5:7 posters on phones, 16:9 from 600 px).
#[component]
pub fn RowSkeleton(#[prop(default = 8)] tiles: usize) -> impl IntoView {
    view! {
        <div class="row-block" aria-hidden="true">
            <TitleLine />
            <div class="relative" style="container-type: inline-size;">
                <div class="flex gap-(--tile-gap) overflow-hidden py-[2px]">
                    <div class="row-edge"></div>
                    {(0..tiles).map(|_| view! { <div class="skeleton movie-card-container tile-width flex-none rounded-tile"></div> }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

/// Rows, spaced as the rows they stand for.
#[component]
pub fn RowsSkeleton(#[prop(default = 3)] rows: usize) -> impl IntoView {
    view! { <div class="rows-stack" role="status" aria-busy="true">{(0..rows).map(|_| view! { <RowSkeleton /> }).collect::<Vec<_>>()}</div> }
}

/// The tiles of a row already on the page (a row whose title is drawn and whose titles are on their way).
#[component]
pub fn TilesSkeleton(#[prop(default = 8)] tiles: usize) -> impl IntoView {
    (0..tiles).map(|_| view! { <div class="skeleton movie-card-container tile-width flex-none rounded-tile" aria-hidden="true"></div> }).collect::<Vec<_>>()
}

/// The ranked slots of a Top 10 row: the poster in the slot's right half, as the real card (topten-card in style/input.css).
#[component]
pub fn TopTenSlotsSkeleton(#[prop(default = 6)] slots: usize) -> impl IntoView {
    (0..slots)
        .map(|_| view! {
            <div class="relative flex flex-none items-end tile-width topten-card" aria-hidden="true">
                <div class="skeleton topten-poster ml-auto aspect-[7/10] w-1/2 rounded-r-[3.6cqw]"></div>
            </div>
        })
        .collect::<Vec<_>>()
}

/// A grid of tiles; `columns` is the grid's own class (columns and gaps).
#[component]
pub fn GridSkeleton(#[prop(into)] columns: String, #[prop(default = 10)] count: usize) -> impl IntoView {
    view! {
        <div class=format!("grid {columns}") role="status" aria-busy="true">
            {(0..count).map(|_| view! { <div class="skeleton aspect-video rounded-tile"></div> }).collect::<Vec<_>>()}
        </div>
    }
}

/// The website's billboard: the card in its margin and ratio, the logo, facts, synopsis and buttons where they stand.
#[component]
pub fn BillboardSkeleton() -> impl IntoView {
    view! {
        <div class="w-full billboard-margin" role="status" aria-busy="true">
            <div class="relative w-full aspect-[2.18/1] overflow-hidden rounded-billboard border border-ink/[0.04] bg-raised">
                <div class="absolute inset-x-0 bottom-0 flex flex-col items-start gap-3 p-9 lg:p-10">
                    <Bone class="h-16 w-[30%] max-w-[340px] rounded-card" />
                    <Bone class="h-4 w-[22%] max-w-[260px] rounded-control" />
                    <div class="flex w-[40%] max-w-[625px] flex-col gap-2">
                        <Bone class="h-3.5 w-full rounded-control" />
                        <Bone class="h-3.5 w-4/5 rounded-control" />
                    </div>
                    <div class="flex gap-3 pt-2">
                        <Bone class="h-12 w-28 rounded-button" />
                        <Bone class="h-12 w-32 rounded-button" />
                    </div>
                </div>
            </div>
        </div>
    }
}

/// The phone's hero card (billboard/hero/phone.rs): the card in its place, the logo, the facts and the two buttons.
#[component]
pub fn PhoneHeroSkeleton() -> impl IntoView {
    view! {
        <div class="relative w-full px-5 pb-5 pt-[calc(124px+env(safe-area-inset-top))]" role="status" aria-busy="true">
            <div class="relative mx-auto flex w-full max-w-[560px] flex-col justify-end overflow-hidden rounded-poster border border-ink/20 bg-raised p-4 aspect-[5/7]">
                <div class="flex w-full flex-col items-center gap-3">
                    <Bone class="h-10 w-1/2 rounded-field" />
                    <Bone class="h-3 w-1/3 rounded-control" />
                    <div class="mt-2 flex w-full gap-3">
                        <Bone class="h-[50px] flex-1 rounded-button" />
                        <Bone class="h-[50px] flex-1 rounded-button" />
                    </div>
                </div>
            </div>
        </div>
    }
}

/// The top of Home, Series and Films while it loads, in the form in use.
#[component]
pub fn HeroSkeleton() -> impl IntoView {
    view! { <crate::form::ByForm phone=|| view! { <PhoneHeroSkeleton /> } desktop=|| view! { <BillboardSkeleton /> } /> }
}

/// Rows of a phone list (search): the picture, two lines of title and the round button.
#[component]
pub fn ListSkeleton(#[prop(default = 6)] rows: usize) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3" role="status" aria-busy="true">
            {(0..rows).map(|_| view! {
                <div class="flex items-center gap-3">
                    <Bone class="aspect-video w-[33%] flex-none rounded-field" />
                    <div class="flex flex-1 flex-col gap-2"><Bone class="h-3.5 w-3/4 rounded-control" /><Bone class="h-3.5 w-1/2 rounded-control" /></div>
                    <Bone class="h-8 w-8 flex-none rounded-full" />
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Large cards one under the other (New & Popular on phones): the picture, then the title and two lines.
#[component]
pub fn CardsSkeleton(#[prop(default = 2)] count: usize) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6" role="status" aria-busy="true">
            {(0..count).map(|_| view! {
                <div class="flex flex-col gap-3">
                    <Bone class="aspect-video w-full rounded-modal" />
                    <Bone class="h-5 w-1/2 rounded-control" />
                    <Bone class="h-3.5 w-full rounded-control" />
                    <Bone class="h-3.5 w-4/5 rounded-control" />
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// The details window while the title loads: the trailer's area with the logo and the buttons where they stand (13 % up on computers),
/// then the facts, the synopsis and the column of names (media/details/info_modal.rs).
#[component]
pub fn DetailsSkeleton() -> impl IntoView {
    view! {
        <div role="status" aria-busy="true">
            <div class="relative aspect-video w-full bg-void">
                <div class="absolute bottom-6 left-6 flex w-[340px] max-w-[70%] flex-col gap-6 md:bottom-[13%] md:left-12">
                    <Bone class="h-16 w-3/4 rounded-card" />
                    <div class="flex items-center gap-2">
                        <Bone class="h-12 w-28 rounded-button" />
                        <Bone class="h-10 w-10 rounded-full" />
                        <Bone class="h-10 w-10 rounded-full" />
                    </div>
                </div>
            </div>
            <div class="grid grid-cols-1 gap-8 px-6 pb-8 pt-3 md:grid-cols-[2fr_1fr] md:px-12">
                <div class="flex flex-col gap-3">
                    <Bone class="h-4 w-1/2 rounded-control" />
                    <Bone class="h-4 w-full rounded-control" />
                    <Bone class="h-4 w-11/12 rounded-control" />
                    <Bone class="h-4 w-3/4 rounded-control" />
                </div>
                <div class="flex flex-col gap-3">
                    <Bone class="h-3.5 w-full rounded-control" />
                    <Bone class="h-3.5 w-4/5 rounded-control" />
                    <Bone class="h-3.5 w-2/3 rounded-control" />
                </div>
            </div>
        </div>
    }
}

/// Episode rows (the details window): the number, the picture and three lines, as an episode row.
#[component]
pub fn EpisodeRowsSkeleton(#[prop(default = 4)] rows: usize) -> impl IntoView {
    view! {
        <div class="space-y-2" role="status" aria-busy="true">
            {(0..rows).map(|_| view! {
                <div class="flex items-start gap-4 rounded-control border-b border-ink/5 p-4">
                    <Bone class="h-5 w-8 shrink-0 rounded-control" />
                    <Bone class="aspect-video w-28 shrink-0 rounded-control md:w-36" />
                    <div class="flex-1 space-y-2 py-1">
                        <Bone class="h-3 w-2/3 rounded-full" />
                        <Bone class="h-2.5 w-full rounded-full" />
                        <Bone class="h-2.5 w-4/5 rounded-full" />
                    </div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Notifications (the bell's panel): the picture and three lines, as a notification.
#[component]
pub fn NotificationsSkeleton(#[prop(default = 4)] rows: usize) -> impl IntoView {
    view! {
        <div class="flex flex-col" role="status" aria-busy="true">
            {(0..rows).map(|_| view! {
                <div class="mx-1 flex items-start gap-3 px-3 py-3">
                    <Bone class="h-[63px] w-[112px] shrink-0 rounded-field" />
                    <div class="flex flex-1 flex-col gap-2 pt-1">
                        <Bone class="h-3.5 w-1/2 rounded-control" />
                        <Bone class="h-3 w-3/4 rounded-control" />
                        <Bone class="h-3 w-1/3 rounded-control" />
                    </div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}
