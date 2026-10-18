//! The logo of a collection, made to look hand made: lettering on a sticker or a paper cut out, as the logos of the app's collections are. The title is set in a hand lettered face
//! and given a finish (a die cut sticker border, a paper cut out, marker on a highlighter swipe, a rough ink stamp, a double outline), a little tilt, and a second colour for the last word.
//! Ten lettering treatments x five finishes make 50 designs; a collection always has the same one (it follows its id) and its colours come from its hue. The gallery of all 50 is at `/dev/logos`.

use leptos::prelude::*;

pub const TREATMENTS: usize = 10;
pub const FINISHES: usize = 5;
pub const DESIGNS: usize = TREATMENTS * FINISHES;

/// The designs fitting a theme: the sticker finish (#0, #5 and kin) suits drama, romance and warm themes; the rougher stamp and label strips suit dark and tense ones; the clean outlines
/// suit light, family and seasonal ones. Chosen from the words of the collection id.
fn pool_for(id: &str) -> &'static [usize] {
    let has = |words: &[&str]| words.iter().any(|w| id.contains(w));
    if has(&["horror", "halloween", "eerie", "haunt", "true_crime", "neo_noir", "friday_13", "slow_burn", "mind_bending", "whodunit"]) {
        &[17, 22, 3, 13, 47, 37]
    } else if has(&["romance", "love", "date", "valentine", "cozy", "feel_good", "tear", "drama", "mothers", "binge", "family", "friends"]) {
        &[0, 5, 15, 20, 30, 35]
    } else if has(&["christmas", "hanukkah", "thanksgiving", "easter", "summer", "spring", "autumn", "winter", "new_year", "diwali", "july4", "pride", "st_patrick", "lunar"]) {
        &[4, 19, 24, 34, 39, 44]
    } else {
        &[0, 5, 10, 15, 20, 25, 30, 35, 40, 45]
    }
}

thread_local! {
    /// The designs already used on the page that is open (cleared when the address changes), so two banners of one page do not look alike.
    static USED: std::cell::RefCell<(String, Vec<(String, usize)>)> = std::cell::RefCell::new((String::new(), Vec::new()));
}

/// The design number of a collection: from the pool that fits its theme, by a hash of its id (so it is the same on every visit), skipping the designs the page already shows.
pub fn design_of(id: &str) -> usize {
    let pool = pool_for(id);
    let start = id.bytes().fold(7usize, |h, b| h.wrapping_mul(131).wrapping_add(b as usize));
    let path = web_sys::window().and_then(|w| w.location().pathname().ok()).unwrap_or_default();
    USED.with(|u| {
        let mut u = u.borrow_mut();
        if u.0 != path { *u = (path, Vec::new()); }
        // the same collection keeps its design while the page is open
        if let Some((_, d)) = u.1.iter().find(|(i, _)| i == id) { return *d; }
        let pick = (0..pool.len()).map(|k| pool[(start + k) % pool.len()]).find(|d| !u.1.iter().any(|(_, x)| x == d)).unwrap_or(pool[start % pool.len()]);
        u.1.push((id.to_string(), pick));
        pick
    })
}

/// A lettering treatment: the face, the case, the tilt, the slant and the size of the last word compared with the rest.
struct Treatment {
    family: &'static str,
    uppercase: bool,
    rotate: f32,
    skew: f32,
    tracking: &'static str,
    /// font size factor (hand lettered faces differ a lot in how big they look)
    size: f32,
    /// the words go on two lines, the last one larger
    stack: bool,
}

const TREATMENTS_TABLE: [Treatment; TREATMENTS] = [
    // 0 marker capitals
    Treatment { family: "'Logo Marker', cursive", uppercase: true, rotate: -4.0, skew: -6.0, tracking: "0.01em", size: 0.92, stack: true },
    // 1 brush
    Treatment { family: "'Logo Brush', cursive", uppercase: false, rotate: -5.0, skew: 0.0, tracking: "0", size: 1.2, stack: true },
    // 2 scrawl
    Treatment { family: "'Logo Scrawl', cursive", uppercase: true, rotate: -3.0, skew: 0.0, tracking: "0.02em", size: 0.62, stack: true },
    // 3 hand cut block
    Treatment { family: "'Logo Cutout', var(--font-brand)", uppercase: true, rotate: -3.0, skew: -4.0, tracking: "0.01em", size: 1.08, stack: true },
    // 4 graffiti
    Treatment { family: "'Logo Graffiti', cursive", uppercase: false, rotate: -4.0, skew: 0.0, tracking: "0", size: 1.15, stack: true },
    // 5 editorial italic serif
    Treatment { family: "'Logo Editorial', ui-serif, Georgia, serif", uppercase: false, rotate: -2.0, skew: 0.0, tracking: "-0.01em", size: 1.15, stack: false },
    // 6 casual hand
    Treatment { family: "'Logo Casual', cursive", uppercase: false, rotate: -3.0, skew: 0.0, tracking: "0.01em", size: 1.15, stack: true },
    // 7 handwriting
    Treatment { family: "'Logo Hand', cursive", uppercase: false, rotate: -4.0, skew: 0.0, tracking: "0", size: 1.0, stack: true },
    // 8 black italic serif
    Treatment { family: "'Logo Playfair', ui-serif, Georgia, serif", uppercase: false, rotate: -2.0, skew: 0.0, tracking: "-0.02em", size: 0.98, stack: false },
    // 9 cut paper headline
    Treatment { family: "'Logo Abril', ui-serif, Georgia, serif", uppercase: true, rotate: -2.0, skew: 0.0, tracking: "0.01em", size: 0.9, stack: true },
];

/// `n` text shadows laid round a circle of radius `r`: a thick rounded-control border, as the cut edge of a sticker.
fn ring(n: usize, r: f32, color: &str) -> String {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            format!("{:.2}px {:.2}px 0 {}", r * a.cos(), r * a.sin(), color)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The finish, from the hue `h`. Returns the CSS for the letters and the CSS of the highlighter swipe behind them (empty unless the finish has one).
fn finish(f: usize, h: u32) -> (String, String) {
    let accent = h;
    match f {
        // die cut sticker (the filter `sticker_filter` makes the cream shape and the coloured edge): dark letters
        0 => (format!("color: hsl({h} 40% 7%); filter: url(#logo-sticker-{h});"), String::new()),
        // paper cut out: cream letters, a dark hairline, a hard offset in the colour of the collection
        1 => (
            format!("color: hsl(45 60% 94%); text-shadow: {}, 4px 5px 0 hsl({h} 60% 42%), 7px 9px 12px hsl({h} 50% 4% / 0.5);", ring(14, 2.2, &format!("hsl({h} 40% 12%)"))),
            String::new(),
        ),
        // marker on a highlighter swipe
        2 => (
            "color: hsl(0 0% 6%); text-shadow: 0 0 1px hsl(0 0% 6%);".to_string(),
            format!("linear-gradient(transparent 52%, hsl({h} 90% 66% / 0.92) 52%, hsl({h} 90% 66% / 0.92) 90%, transparent 90%)"),
        ),
        // ink stamp: the colour of the collection, roughened edges, a little ghost
        3 => (
            format!("color: hsl({h} 40% 66%); text-shadow: 2px 2px 0 hsl({h} 40% 26% / 0.55), -1px -1px 0 hsl(45 60% 96% / 0.35); filter: url(#logo-rough);"),
            String::new(),
        ),
        // double outline: cream letters, a coloured border, a cream border outside it
        _ => (
            format!(
                "color: hsl(45 70% 96%); text-shadow: {}, {}, 3px 6px 12px hsl({accent} 40% 6% / 0.5);",
                ring(18, 3.6, &format!("hsl({accent} 75% 52%)")),
                ring(22, 7.0, "hsl(45 70% 96%)")
            ),
            String::new(),
        ),
    }
}

/// The CSS of a design (0 to 49) for a word mark of the hue `h`.
pub fn design_style(design: usize, h: u32) -> String {
    let t = &TREATMENTS_TABLE[(design % DESIGNS) / FINISHES];
    let (letters, swipe) = finish((design % DESIGNS) % FINISHES, h);
    let swipe_css = if swipe.is_empty() { String::new() } else { format!(" background-image: {swipe}; padding: 0 0.12em;") };
    format!(
        "font-family: {}; font-weight: 400; letter-spacing: {}; line-height: 1.02; text-transform: {}; transform: rotate({}deg) skewX({}deg); transform-origin: center; text-wrap: balance; {} {}",
        t.family, t.tracking, if t.uppercase { "uppercase" } else { "none" }, t.rotate, t.skew, letters, swipe_css
    )
}

/// The font size factor of a design (hand lettered faces differ in how large they look).
pub fn design_size(design: usize) -> f32 {
    TREATMENTS_TABLE[(design % DESIGNS) / FINISHES].size
}

/// The title as a word mark. `design` is the number of the look (from `design_of` of the collection's id); `size` is the base font size in px.
#[component]
pub fn CollectionLogo(
    #[prop(into)] text: String,
    #[prop(into)] hue: u32,
    #[prop(default = 0)] design: usize,
    #[prop(default = 44.0)] size: f32,
) -> impl IntoView {
    let t = &TREATMENTS_TABLE[(design % DESIGNS) / FINISHES];
    let finish_index = (design % DESIGNS) % FINISHES;
    let style = format!("{} font-size: {:.1}px;", design_style(design, hue), size * design_size(design));
    // the last word in a second colour (not on the finishes that already use two)
    let accent = format!("hsl({hue} 55% 72%)");
    let words = text.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    // the word in the second colour: the second of three or more (as "Ya" in "Love Ya Forever"), otherwise the last
    let accent_at = if words.len() >= 3 { 1 } else { words.len().saturating_sub(1) };
    let accent_style = if finish_index == 0 {
        // a blue with a brush stroke under it
        format!("color: hsl(212 45% 36%); background: url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 12' preserveAspectRatio='none'><path d='M2 9 C 30 2, 70 2, 98 7' stroke='%23346a96' stroke-width='5' fill='none' stroke-linecap='round'/></svg>\") no-repeat bottom / 100% 0.16em; padding-bottom: 0.14em; display: inline-block; font-size: 1.12em;")
    } else if finish_index == 3 {
        format!("color: {accent};")
    } else {
        String::new()
    };
    let word_view = |i: usize, w: &String| {
        let st = if words.len() > 1 && i == accent_at && finish_index != 4 { accent_style.clone() } else { String::new() };
        view! { <span style=st>{w.clone()}{" "}</span> }
    };
    let body = if t.stack && words.len() > 1 {
        // two lines: the last word alone on the second, larger
        let first: Vec<_> = words[..words.len() - 1].iter().enumerate().map(|(i, w)| word_view(i, w)).collect();
        let last = word_view(words.len() - 1, &words[words.len() - 1]);
        view! { <span class="block">{first}</span><span class="block" style="font-size: 1.22em;">{last}</span> }.into_any()
    } else {
        let all: Vec<_> = words.iter().enumerate().map(|(i, w)| word_view(i, w)).collect();
        view! { <span>{all}</span> }.into_any()
    };
    // the sticker needs a filter of its own for this hue and size (its radii are in px)
    let sticker = (finish_index == 0).then(|| sticker_filter(hue, size * design_size(design)));
    view! {
        {sticker}
        <span class="inline-block max-w-full text-center" style=style>{body}</span>
    }
}

/// The SVG filter that turns lettering into a sticker, as in the app's logos: the letters and the gaps between them are grown into one rounded-control cream shape (grown, smoothed, merged), a
/// second, larger shape in the colour of the collection goes behind it as the printed edge, a faint paper grain and a little roughness on the edge, and a soft shadow under all of it.
fn sticker_filter(h: u32, font_px: f32) -> impl IntoView {
    let r1 = (font_px * 0.24).max(4.0);
    let r2 = r1 + (font_px * 0.15).max(3.0);
    let edge = format!("hsl({h} 55% 66%)");
    // built as text: the view macro does not make the SVG filter elements as SVG, so the filter would stay empty
    let markup = format!(
        r##"<svg width="0" height="0" style="position:absolute" aria-hidden="true"><filter id="logo-sticker-{h}" x="-30%" y="-45%" width="160%" height="190%" color-interpolation-filters="sRGB">
<feMorphology in="SourceAlpha" operator="dilate" radius="{r1:.1}" result="grow1"/>
<feGaussianBlur in="grow1" stdDeviation="{s1:.1}" result="soft1"/>
<feComponentTransfer in="soft1" result="shape1"><feFuncA type="linear" slope="9" intercept="-3.4"/></feComponentTransfer>
<feMorphology in="SourceAlpha" operator="dilate" radius="{r2:.1}" result="grow2"/>
<feGaussianBlur in="grow2" stdDeviation="{s2:.1}" result="soft2"/>
<feComponentTransfer in="soft2" result="shape2"><feFuncA type="linear" slope="9" intercept="-3.4"/></feComponentTransfer>
<feTurbulence type="fractalNoise" baseFrequency="0.045" numOctaves="2" seed="4" result="rough"/>
<feDisplacementMap in="shape1" in2="rough" scale="2.4" result="cut1"/>
<feDisplacementMap in="shape2" in2="rough" scale="3" result="cut2"/>
<feFlood flood-color="#f1e6cd" result="cream"/>
<feComposite in="cream" in2="cut1" operator="in" result="paper"/>
<feFlood flood-color="{edge}" result="ink"/>
<feComposite in="ink" in2="cut2" operator="in" result="border"/>
<feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="9" result="grain"/>
<feColorMatrix in="grain" type="matrix" values="0 0 0 0 0.62  0 0 0 0 0.45  0 0 0 0 0.30  0 0 0 -1.9 1.0" result="speck"/>
<feComposite in="speck" in2="cut1" operator="in" result="grained"/>
<feMerge result="sticker"><feMergeNode in="border"/><feMergeNode in="paper"/><feMergeNode in="grained"/><feMergeNode in="SourceGraphic"/></feMerge>
<feDropShadow in="sticker" dx="0" dy="4" stdDeviation="6" flood-color="#000" flood-opacity="0.5"/>
</filter></svg>"##,
        s1 = r1 * 0.45,
        s2 = r2 * 0.4,
    );
    view! { <span inner_html=markup></span> }
}

/// An invisible filter that roughens the edges of letters a little, like ink on paper (used by the stamp finish).
#[component]
pub fn LogoFilters() -> impl IntoView {
    let markup = r##"<svg width="0" height="0" style="position:absolute" aria-hidden="true"><filter id="logo-rough" x="-5%" y="-5%" width="110%" height="110%"><feTurbulence type="fractalNoise" baseFrequency="0.04" numOctaves="3" seed="7" result="n"/><feDisplacementMap in="SourceGraphic" in2="n" scale="3.5"/></filter></svg>"##;
    view! { <span inner_html=markup></span> }
}

const GALLERY_TITLE: &str = concat!("Collection", " logos"); // developer page: not translated
const SAMPLE_TEXT: &str = concat!("Love Ya", " Forever");

/// The gallery of the 50 designs, at `/dev/logos`: each with its number, so one can be chosen for a collection (`design_of` picks by hash until then).
#[component]
pub fn LogoGalleryPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-page px-8 py-10 text-ink">
            <LogoFilters />
            <h1 class="mb-8 text-3xl font-bold">{GALLERY_TITLE}</h1>
            <h2 class="mb-4 text-xl font-bold">"Kids collections"</h2>
            <div class="mb-12 grid grid-cols-1 gap-6 xl:grid-cols-2">
                {[("Spooky Season", "Friendly ghosts and silly monsters, fun and not frightening", 28u32), ("Besties and Buddies", "Friends who stick together, in good times and silly ones", 200), ("Shh, Secrets and Mysteries", "Hidden doors, clues and gentle mysteries", 265), ("Christmas Magic", "Snow, presents and festive songs", 140), ("All the Animals", "Lions, pups, penguins and plenty more", 95), ("For Little Ones", "Gentle shows and songs for the youngest viewers", 330)].into_iter().map(|(name, sentence, hue)| view! {
                    <div class="relative isolate flex min-h-[330px] items-center overflow-hidden rounded-card">
                        <super::kids_banner::KidsBackdrop seed=hue />
                        <div class="relative flex w-[40%] flex-none flex-col items-center px-5 text-center">
                            <super::kids_banner::KidsCollectionLogo text=name hue=hue />
                            <p class="mt-3 rounded-full bg-white px-4 py-1 text-caption font-bold uppercase tracking-[0.2em] text-[#e30b15] shadow-md">"Streams Kids"</p>
                            <p class="mt-4 max-w-[26ch] text-body leading-snug text-ink" style="font-family: 'Logo Kids','Fredoka',ui-rounded,sans-serif; font-weight: 500;">{sentence}</p>
                            <span class="mt-4 inline-flex h-10 items-center rounded-full bg-white px-6 text-body-md font-bold text-[#e30b15] shadow-lg">"Explore all"</span>
                        </div>
                        <div class="relative flex flex-1 gap-3 overflow-hidden pe-3">
                            {(0..4).map(|i| view! { <div class="aspect-video w-[30%] flex-none rounded-tile bg-black/35" style=format!("opacity: {}", 1.0 - i as f32 * 0.12)></div> }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
            <h2 class="mb-4 text-xl font-bold">"Adult designs"</h2>
            <div class="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
                {(0..DESIGNS).map(|d| {
                    let hue = (d * 47) as u32 % 360;
                    view! {
                        <div class="relative overflow-hidden rounded-card border border-ink/10 p-6" style=format!("background: radial-gradient(circle at 30% 20%, hsl({hue} 38% 20%), hsl({hue} 45% 9%));")>
                            <span class="absolute right-3 top-2 text-xs text-ink/50">{format!("#{d}")}</span>
                            <div class="flex min-h-[170px] items-center justify-center">
                                <CollectionLogo text=SAMPLE_TEXT hue=hue design=d size=40.0 />
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
