//! colour: art. The Kids banner's drawings and the logo's hue based colours.
//! The parts of a Kids collection banner that give it its own look: the name set in the Kids lettering (round letters in bright colours with a white edge, each line tilted a little, a few
//! stars) and a scatter of confetti behind the panel. Everything is drawn from the collection's name and colour, so each banner differs and the same banner is the same every visit.
use leptos::prelude::*;

/// The name cut into lines of at most 13 letters (at most three), by words.
fn lines_of(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(last) if last.chars().count() + 1 + word.chars().count() <= 13 => {
                last.push(' ');
                last.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    // more than three lines: the extra words go on the last one
    while lines.len() > 3 {
        let tail = lines.pop().unwrap_or_default();
        if let Some(last) = lines.last_mut() {
            last.push(' ');
            last.push_str(&tail);
        }
    }
    lines
}

/// A number from the colour and a counter, the same each time: 0 up to but not including 1.
fn jitter(h: u32, n: u32) -> f32 {
    let x = h.wrapping_mul(2_654_435_761).wrapping_add(n.wrapping_mul(40_503)).wrapping_mul(2_246_822_519);
    ((x >> 8) % 10_000) as f32 / 10_000.0
}

/// The collection's name as Kids lettering.
#[component]
pub fn KidsCollectionLogo(#[prop(into)] text: String, #[prop(into)] hue: u32) -> impl IntoView {
    let font = "'Logo Kids', 'Fredoka', ui-rounded, 'Arial Rounded MT Bold', sans-serif";
    let lines = lines_of(&text);
    let line_h = 190.0f32;
    let height = lines.len() as f32 * line_h + 90.0;
    // each line in its own primary colour (red, blue, green, then orange, purple, pink), from a starting point that the collection's colour picks
    const INKS: [(&str, &str); 6] = [("#f0262e", "#b5121b"), ("#1a7cf2", "#0b4fb0"), ("#1fb85a", "#0f7a38"), ("#ff8a1f", "#c25a05"), ("#8a3df0", "#5a1fb5"), ("#f24fb0", "#b3237f")];
    let first = (hue / 60) as usize % INKS.len();
    let tilts = [-2.2f32, 1.6, -1.2];
    let mut shapes = String::new();
    for (i, line) in lines.iter().enumerate() {
        let chars = line.chars().count().max(1) as f32;
        let size = (170.0f32).min(840.0 / (chars * 0.58));
        let y = 40.0 + (i as f32 + 1.0) * line_h - 36.0;
        let (colour, deep) = INKS[(first + i * 2) % INKS.len()];
        let t = tilts[i % 3];
        let esc = line.replace('&', "&amp;").replace('<', "&lt;");
        shapes.push_str(&format!(
            r##"<g transform="rotate({t} 450 {y})"><text x="450" y="{y}" text-anchor="middle" font-family="{font}" font-weight="500" font-size="{size:.1}" fill="#fff" stroke="#fff" stroke-width="{e:.1}" stroke-linejoin="round" stroke-linecap="round" letter-spacing="{ls:.1}">{esc}</text><text x="450" y="{y2:.1}" text-anchor="middle" font-family="{font}" font-weight="500" font-size="{size:.1}" fill="{deep}" stroke="{deep}" stroke-width="{s:.1}" stroke-linejoin="round" letter-spacing="{ls:.1}">{esc}</text><text x="450" y="{y}" text-anchor="middle" font-family="{font}" font-weight="500" font-size="{size:.1}" fill="{colour}" stroke="{colour}" stroke-width="{s:.1}" stroke-linejoin="round" letter-spacing="{ls:.1}">{esc}</text></g>"##,
            e = size * 0.26,
            s = size * 0.03,
            ls = size * 0.03,
            y2 = y + size * 0.045,
        ));
    }
    // a few stars round the lettering
    let star = |cx: f32, cy: f32, r: f32, fill: &str| {
        format!(r##"<path transform="translate({cx:.0} {cy:.0}) rotate({rot:.0})" d="M0 {a:.1} C {b:.1} {b:.1} {b:.1} {b:.1} {a:.1} 0 C {b:.1} {nb:.1} {b:.1} {nb:.1} 0 {na:.1} C {nb:.1} {nb:.1} {nb:.1} {nb:.1} {na:.1} 0 C {nb:.1} {b:.1} {nb:.1} {b:.1} 0 {a:.1}Z" fill="{fill}" stroke="#fff" stroke-width="7" stroke-linejoin="round" paint-order="stroke"/>"##,
            rot = jitter(hue, cx as u32) * 30.0 - 15.0, a = r, na = -r, b = r * 0.22, nb = -r * 0.22)
    };
    let (fx, fy) = (-25.0f32, 70.0 + jitter(hue, 1) * 40.0);
    shapes.push_str(&star(fx, fy, 50.0, "#ffd23f"));
    // a face on it: two eyes and a smile
    shapes.push_str(&format!(r##"<circle cx="{a:.0}" cy="{b:.0}" r="4.5" fill="#6b4a00"/><circle cx="{c:.0}" cy="{b:.0}" r="4.5" fill="#6b4a00"/><path d="M{a:.0} {d:.0} q 10 11 20 0" fill="none" stroke="#6b4a00" stroke-width="4" stroke-linecap="round" transform="translate(-10 0)"/>"##, a = fx - 9.0, c = fx + 9.0, b = fy - 4.0, d = fy + 8.0));
    shapes.push_str(&star(925.0, height - 80.0 - jitter(hue, 2) * 30.0, 38.0, "#ff8fc7"));
    shapes.push_str(&star(935.0, 60.0 + jitter(hue, 3) * 30.0, 26.0, "#7fe3ff"));
    let markup = format!(r##"<svg viewBox="-90 0 1080 {height:.0}" class="w-full h-auto" style="filter: drop-shadow(0 6px 10px rgb(0 0 0 / 0.35));" role="img" aria-label="{label}" xmlns="http://www.w3.org/2000/svg">{shapes}</svg>"##, label = text.replace('"', "&quot;").replace('&', "&amp;"));
    view! { <span class="block w-[min(420px,86%)] md:w-[min(380px,100%)]" inner_html=markup></span> }
}

struct Palette {
    base: (&'static str, &'static str),
    shapes: [&'static str; 4],
}

/// Six colour schemes for a Kids collection's backdrop: a deep colour from top to bottom and four flat shapes in colours that sit well on it.
const PALETTES: [Palette; 6] = [
    Palette { base: ("#2f6dff", "#1a3fcf"), shapes: ["#ff8a1f", "#22b866", "#ffd23f", "#f0262e"] },
    Palette { base: ("#8a45f2", "#4d1fb0"), shapes: ["#f24fb0", "#ffd23f", "#2fd0e0", "#ff8a1f"] },
    Palette { base: ("#18b9b2", "#0a7389"), shapes: ["#ffd23f", "#f24fb0", "#ff8a1f", "#7a3df0"] },
    Palette { base: ("#ee4fb0", "#a31f82"), shapes: ["#ffd23f", "#2f8cff", "#22b866", "#8a45f2"] },
    Palette { base: ("#ff8d3f", "#d9492b"), shapes: ["#2f6dff", "#ffd23f", "#f24fb0", "#22b866"] },
    Palette { base: ("#34b96d", "#157a44"), shapes: ["#ffd23f", "#f24fb0", "#2f8cff", "#ff8a1f"] },
];

/// The designed backdrop of a Kids collection: a colour scheme (picked by the collection's colour), big flat shapes (rounded triangles, circles, blobs) placed from the same number, and,
/// when pictures are given, a few of the collection's titles set inside the shapes on the right (the page header uses them; the banner leaves them out, its titles are the row).
#[component]
pub fn KidsBackdrop(#[prop(into)] seed: u32, #[prop(optional, into)] images: Vec<String>) -> impl IntoView {
    let pal = &PALETTES[(seed / 60) as usize % PALETTES.len()];
    let mut body = String::new();
    // pictures: three frames on the right, set into a circle, a rounded triangle and a small circle
    let frames: [(&str, f32, f32, f32); 3] = [("tri", 1080.0, 260.0, 190.0), ("circle", 1330.0, 330.0, 150.0), ("circle", 1450.0, 95.0, 80.0)];
    let mut defs = String::new();
    let mut shapes = String::new();
    for (n, (kind, cx, cy, r)) in frames.iter().enumerate().take(images.len().min(3)) {
        let id = format!("kc{seed}-{n}");
        let clip = if *kind == "tri" {
            format!(r##"<clipPath id="{id}"><polygon points="{x1:.0},{y1:.0} {x2:.0},{y2:.0} {x3:.0},{y3:.0}" style="stroke-linejoin:round"/></clipPath>"##, x1 = cx - r, y1 = cy + r * 0.9, x2 = cx + r * 1.05, y2 = cy, x3 = cx - r, y3 = cy - r * 0.9)
        } else {
            format!(r##"<clipPath id="{id}"><circle cx="{cx:.0}" cy="{cy:.0}" r="{r:.0}"/></clipPath>"##)
        };
        defs.push_str(&clip);
        let col = pal.shapes[(n + 1) % 4];
        if *kind == "tri" {
            shapes.push_str(&format!(r##"<polygon points="{x1:.0},{y1:.0} {x2:.0},{y2:.0} {x3:.0},{y3:.0}" fill="{col}" stroke="{col}" stroke-width="70" stroke-linejoin="round"/>"##, x1 = cx - r, y1 = cy + r * 0.9, x2 = cx + r * 1.05, y2 = cy, x3 = cx - r, y3 = cy - r * 0.9));
        } else {
            shapes.push_str(&format!(r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{rr:.0}" fill="{col}"/>"##, rr = r + 22.0));
        }
        shapes.push_str(&format!(r##"<image href="{href}" x="{x:.0}" y="{y:.0}" width="{w:.0}" height="{w:.0}" preserveAspectRatio="xMidYMid slice" clip-path="url(#{id})"/>"##, href = images[n], x = cx - r * 1.4, y = cy - r * 1.4, w = r * 2.8));
    }
    // flat shapes in fixed zones, as the app's own banners are composed (a wedge at the top left, a blob at the top middle, a wedge at the bottom right, a circle at each lower corner),
    // each moved a little, resized and turned from the seed; the colours turn round the scheme, and a light colour never sits behind the text on the left
    let zones: [(&str, f32, f32, f32); 5] = [("tri", 150.0, 90.0, 190.0), ("ellipse", 720.0, 40.0, 170.0), ("tri", 1480.0, 430.0, 210.0), ("circle", 70.0, 470.0, 120.0), ("circle", 1320.0, 60.0, 130.0)];
    let turn = (seed % 4) as usize;
    let mut colours: Vec<&str> = (0..5).map(|n| pal.shapes[(n + turn) % 4]).collect();
    for left in [0usize, 3] {
        if colours[left] == "#ffd23f" { colours.swap(left, 2); }
    }
    for (n, (kind, zx, zy, zs)) in zones.iter().enumerate() {
        let n = n as u32;
        let col = colours[n as usize];
        let x = zx + (jitter(seed, n * 5) - 0.5) * 160.0;
        let y = zy + (jitter(seed, n * 5 + 1) - 0.5) * 80.0;
        let size = zs * (0.8 + jitter(seed, n * 5 + 2) * 0.5);
        let rot = (jitter(seed, n * 5 + 3) - 0.5) * 70.0;
        // with pictures on the right, the blob and the right circle stay out of the pictures' way
        if !images.is_empty() && n == 4 { continue; }
        match *kind {
            "tri" => body.push_str(&format!(r##"<polygon points="{x1:.0},{y1:.0} {x2:.0},{y2:.0} {x3:.0},{y3:.0}" fill="{col}" stroke="{col}" stroke-width="64" stroke-linejoin="round" transform="rotate({rot:.0} {x:.0} {y:.0})"/>"##, x1 = x - size, y1 = y + size * 0.8, x2 = x + size, y2 = y + size * 0.8, x3 = x, y3 = y - size)),
            "ellipse" => body.push_str(&format!(r##"<ellipse cx="{x:.0}" cy="{y:.0}" rx="{rx:.0}" ry="{size:.0}" fill="{col}" transform="rotate({rot:.0} {x:.0} {y:.0})"/>"##, rx = size * 1.3)),
            _ => body.push_str(&format!(r##"<circle cx="{x:.0}" cy="{y:.0}" r="{size:.0}" fill="{col}"/>"##)),
        }
    }
    let markup = format!(
        r##"<svg viewBox="0 0 1600 500" preserveAspectRatio="xMidYMid slice" class="h-full w-full" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="kb{seed}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{c0}"/><stop offset="1" stop-color="{c1}"/></linearGradient>{defs}</defs><rect width="1600" height="500" fill="url(#kb{seed})"/>{body}{shapes}</svg>"##,
        c0 = pal.base.0,
        c1 = pal.base.1,
    );
    view! { <span class="absolute inset-0 block" aria-hidden="true" inner_html=markup></span> }
}

/// A scatter of confetti (dots, bars and squiggles in bright colours) behind the banner's panel.
#[component]
pub fn KidsConfetti(#[prop(into)] hue: u32, /// Where the text sits across the banner, 0 to 100 percent: no confetti is dropped there
    #[prop(default = 50.0)] avoid: f32) -> impl IntoView {
    let palette = ["#ffd23f", "#ff8fc7", "#7fe3ff", "#9bf06a", "#ff9a5a", "#ffffff"];
    let mut s = String::new();
    let centre = avoid * 10.0;
    for n in 0..40u32 {
        let x = jitter(hue, n * 3) * 1000.0;
        let y = jitter(hue, n * 3 + 1) * 500.0;
        // the text's column stays clear, so the sentence can be read
        if (x - centre).abs() < 210.0 && y > 40.0 && y < 460.0 { continue; }
        let c = palette[(jitter(hue, n * 3 + 2) * palette.len() as f32) as usize % palette.len()];
        let kind = (jitter(hue, n + 100) * 3.0) as u32;
        let r = 6.0 + jitter(hue, n + 200) * 9.0;
        let rot = jitter(hue, n + 300) * 360.0;
        match kind {
            0 => s.push_str(&format!(r##"<circle cx="{x:.0}" cy="{y:.0}" r="{r:.1}" fill="{c}"/>"##)),
            1 => s.push_str(&format!(r##"<rect x="{x:.0}" y="{y:.0}" width="{w:.1}" height="{hh:.1}" rx="{rr:.1}" fill="{c}" transform="rotate({rot:.0} {x:.0} {y:.0})"/>"##, w = r * 2.6, hh = r * 0.8, rr = r * 0.4)),
            _ => s.push_str(&format!(r##"<path d="M{x:.0} {y:.0} q {q:.1} {m:.1} {w:.1} 0 t {w:.1} 0" fill="none" stroke="{c}" stroke-width="5" stroke-linecap="round" transform="rotate({rot:.0} {x:.0} {y:.0})"/>"##, q = r, m = -r * 1.4, w = r * 2.0)),
        }
    }
    let markup = format!(r##"<svg viewBox="0 0 1000 500" preserveAspectRatio="xMidYMid slice" class="h-full w-full" aria-hidden="true" xmlns="http://www.w3.org/2000/svg">{s}</svg>"##);
    view! { <span class="pointer-events-none absolute inset-0 -z-[5] block opacity-50" aria-hidden="true" inner_html=markup></span> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_wrap_by_words_into_at_most_three_lines() {
        assert_eq!(lines_of("Spooky Season"), vec!["Spooky Season"]);
        assert_eq!(lines_of("Shh, Secrets and Mysteries"), vec!["Shh, Secrets", "and Mysteries"]);
        assert!(lines_of("Ages 2 to 4: Play and Learn About Everything Here").len() <= 3);
    }

    #[test]
    fn jitter_is_stable_and_in_range() {
        assert_eq!(jitter(212, 5), jitter(212, 5));
        assert!((0..50).all(|n| (0.0..1.0).contains(&jitter(77, n))));
    }
}
