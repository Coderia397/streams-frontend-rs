#!/usr/bin/env python3
"""Writes src/components/media/icons/app_icons.rs: the icons chosen to match Netflix's (owner's pick of the icon benchmark,
2026-10-24, note 117), each from the pack it matched best, as SVG bodies. Fetched from the Iconify API by name:
    python3 scripts/make_app_icons.py
Packs and licences: Remix Icon (Apache 2.0), Phosphor (MIT), Tabler Icons (MIT), Hugeicons Free (MIT), IBM Carbon (Apache 2.0).
Clips is drawn here (a card with a play triangle over a second card, Netflix's Clips tab): no pack has it."""
import json, pathlib, urllib.parse, urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "src/components/media/icons/app_icons.rs"

# Glyph: (pack prefix, Iconify name, what it is and why this one)
GLYPHS = {
    "Speaker": ("ri", "volume-up-line", "sound on: a box and cone with two waves"),
    "Muted": ("ri", "volume-mute-line", "sound off: the speaker with a small X"),
    "SpeakerLow": ("ri", "volume-down-line", "sound low: the speaker with one wave"),
    "Play": ("ri", "play-large-fill", "play: a large solid triangle with sharp corners"),
    "Back10": ("hugeicons", "go-backward-10-sec", "back 10 seconds: a thin ring with the 10 inside"),
    "Forward10": ("hugeicons", "go-forward-10-sec", "forward 10 seconds"),
    "ThumbUp": ("carbon", "thumbs-up", "I like this"),
    "ThumbUpFilled": ("carbon", "thumbs-up-filled", "I like this, set"),
    "ThumbDown": ("carbon", "thumbs-down", "Not for me"),
    "ThumbDownFilled": ("carbon", "thumbs-down-filled", "Not for me, set"),
    "Love": ("carbon", "thumbs-up-double", "Love this: IBM draws the pair itself"),
    "LoveFilled": ("carbon", "thumbs-up-double-filled", "Love this, set"),
    "Plus": ("ri", "add-line", "add (My List)"),
    "Check": ("ri", "check-line", "added (My List): Plus's partner in the same pack"),
    "Share": ("ri", "share-fill", "share: three linked solid dots"),
    "Search": ("ri", "search-line", "search"),
    "Home": ("ri", "home-6-line", "Home: a house with a short bar"),
    "HomeFilled": ("ri", "home-6-fill", "Home, the tab in use"),
    "Bell": ("ri", "notification-3-line", "notifications: a bell on a flat line"),
    "Info": ("ri", "information-line", "more information: an i in a ring"),
    "Back": ("ri", "arrow-left-line", "back"),
    "Cast": ("tabler", "cast", "cast"),
    "LockOpen": ("ri", "lock-unlock-line", "lock the controls (an open padlock)"),
    "Locked": ("ri", "lock-line", "the controls are locked"),
    "Report": ("ri", "flag-line", "report a problem"),
    "Clip": ("ph", "scissors-bold", "make a clip"),
    "Speed": ("ri", "speed-up-line", "playback speed: an arc and a needle"),
    "Episodes": ("ph", "cards-bold", "episodes: a landscape card over a second one"),
    "Subtitles": ("ph", "chat-centered-text-bold", "audio and subtitles: a box with lines and a centred tail"),
    "Next": ("ph", "skip-forward-bold", "next episode"),
    "Download": ("ri", "download-line", "download: an arrow onto a line"),
    "FullScreen": ("ri", "fullscreen-line", "full screen"),
    "ExitFullScreen": ("ri", "fullscreen-exit-line", "leave full screen"),
}
# Clips, drawn on a 24 grid at a 2 px line with round ends (the owner's cards + play)
CLIPS = ('<rect x="4.5" y="5.5" width="12" height="16" rx="2.2" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>'
         '<path d="M9 2.5H17C18.4 2.5 19.5 3.6 19.5 5V17" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>'
         '<path d="M9 10.6L13.4 13.5L9 16.4V10.6Z" fill="currentColor" stroke="currentColor" stroke-width="0.5" stroke-linejoin="round"/>')


def fetch():
    by_pack = {}
    for prefix, name, _ in GLYPHS.values():
        by_pack.setdefault(prefix, []).append(name)
    out = {}
    for prefix, names in by_pack.items():
        url = f"https://api.iconify.design/{prefix}.json?icons={urllib.parse.quote(','.join(names))}"
        data = json.load(urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (streams make_app_icons)"}), timeout=30))
        if data.get("not_found"):
            raise SystemExit(f"{prefix}: not found {data['not_found']}")
        for name, icon in data["icons"].items():
            out[(prefix, name)] = (icon.get("width", data.get("width", 24)), icon.get("height", data.get("height", 24)), icon["body"])
    return out


def main():
    icons = fetch()
    arms_box, arms_body, variants = [], [], []
    for glyph, (prefix, name, what) in GLYPHS.items():
        w, h, body = icons[(prefix, name)]
        variants.append(f"    /// {what} ({prefix}:{name})\n    {glyph},")
        arms_box.append(f"            Glyph::{glyph} => ({w}, {h}),")
        arms_body.append(f'            Glyph::{glyph} => r#"{body}"#,')
    variants.append("    /// Clips: a card with a play triangle over a second card (drawn; no pack has it)\n    Clips,")
    arms_box.append("            Glyph::Clips => (24, 24),")
    arms_body.append(f'            Glyph::Clips => r#"{CLIPS}"#,')
    rs = f'''//! The icons that stand for Netflix's own (owner's choice from the icon benchmark, 2026-10-24, note 117), each from the pack that
//! matched it best: Remix Icon (Apache 2.0), Phosphor (MIT), Tabler Icons (MIT), Hugeicons Free (MIT), IBM Carbon (Apache 2.0); Clips is
//! drawn. Generated by scripts/make_app_icons.py; do not edit by hand. An icon is 1em square in the text colour, so a text size class
//! sizes it as it sized the font icons it replaces.
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {{
{chr(10).join(variants)}
}}

impl Glyph {{
    /// The glyph's drawing area.
    pub fn view_box(self) -> (u32, u32) {{
        match self {{
{chr(10).join(arms_box)}
        }}
    }}

    /// The glyph's SVG body, in the text colour.
    pub fn body(self) -> &'static str {{
        match self {{
{chr(10).join(arms_body)}
        }}
    }}
}}

/// One icon, 1em square (a text size class sets its size), in the text colour.
#[component]
pub fn AppIcon(glyph: Glyph, #[prop(optional, into)] class: String, #[prop(optional, into)] style: String) -> impl IntoView {{
    let (w, h) = glyph.view_box();
    view! {{
        <svg class=format!("app-icon {{class}}") style=style viewBox=format!("0 0 {{w}} {{h}}") aria-hidden="true" focusable="false" inner_html=glyph.body()></svg>
    }}
}}
'''
    OUT.write_text(rs)
    print(f"wrote {OUT.relative_to(ROOT)}: {len(GLYPHS) + 1} glyphs")


if __name__ == "__main__":
    main()
