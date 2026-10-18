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
    // each line in its own bright colour, a step round the colour wheel from the collection's colour
    let steps = [0u32, 38, 318];
    let tilts = [-2.2f32, 1.6, -1.2];
    let mut shapes = String::new();
    for (i, line) in lines.iter().enumerate() {
        let chars = line.chars().count().max(1) as f32;
        let size = (170.0f32).min(840.0 / (chars * 0.58));
        let y = 40.0 + (i as f32 + 1.0) * line_h - 36.0;
        let colour = format!("hsl({} 92% 57%)", (hue + steps[i % 3]) % 360);
        let deep = format!("hsl({} 85% 40%)", (hue + steps[i % 3]) % 360);
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
    shapes.push_str(&star(-25.0, 70.0 + jitter(hue, 1) * 40.0, 46.0, "#ffd23f"));
    shapes.push_str(&star(925.0, height - 80.0 - jitter(hue, 2) * 30.0, 38.0, "#ff8fc7"));
    shapes.push_str(&star(935.0, 60.0 + jitter(hue, 3) * 30.0, 26.0, "#7fe3ff"));
    let markup = format!(r##"<svg viewBox="-90 0 1080 {height:.0}" class="w-full h-auto" style="filter: drop-shadow(0 6px 10px rgb(0 0 0 / 0.35));" role="img" aria-label="{label}" xmlns="http://www.w3.org/2000/svg">{shapes}</svg>"##, label = text.replace('"', "&quot;").replace('&', "&amp;"));
    view! { <span class="block w-[min(420px,86%)] md:w-[min(380px,100%)]" inner_html=markup></span> }
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
