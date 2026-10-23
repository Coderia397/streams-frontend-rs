#!/usr/bin/env python3
"""Second codemod: z-index layers, type scale, shadows, grays and layout dimensions become tokens from style/brand.css."""
import re, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
FILES = sorted((ROOT / "src").rglob("*.rs"))

Z = {"9": "card-hover", "60": "grain", "79": "subnav", "80": "nav", "85": "nav-menu", "100": "dialog", "110": "tooltip",
     "200": "modal", "250": "modal-top", "300": "takeover", "9999": "screen", "10000": "info-modal", "10020": "bottom-nav",
     "10030": "drawer", "10040": "sheet-backdrop", "10050": "sheet", "10080": "share-backdrop", "10090": "share"}
TEXT = {"8": "pico", "9": "nano", "10": "micro", "11": "caption", "11.2": "caption-sm", "12": "fine", "13": "label",
        "14": "body-sm", "15": "body", "16": "body-md", "17": "body-lg", "18": "title-sm", "20": "title", "21": "title-lg",
        "22": "title-md", "24": "heading", "26": "heading-md", "28": "heading-lg", "38": "display-sm", "40": "display"}
SHADOW = {
 "shadow": {"0_8px_30px_rgba(0,0,0,0.55)": "popover", "0_3px_10px_rgba(0,0,0,0.75)": "chip", "0_8px_24px_rgba(0,0,0,0.5)": "card",
            "0_0_15px_color-mix(in_srgb,var(--color-brand)_60%,transparent)": "brand-glow", "0_0_50px_rgba(0,0,0,0.8)": "vignette",
            "0_0_15px_rgba(0,0,0,0.5)": "soft", "0_4px_12px_rgba(0,0,0,0.5)": "raise"},
 "drop-shadow": {"0_4px_14px_rgba(0,0,0,0.95)": "title", "0_2px_8px_rgba(0,0,0,1)": "text-hard", "0_2px_6px_rgba(0,0,0,0.6)": "text-soft",
                 "0_8px_30px_color-mix(in_srgb,var(--color-brand)_35%,transparent)": "brand-lg",
                 "0_2px_8px_color-mix(in_srgb,var(--color-brand)_40%,transparent)": "brand-sm",
                 "0_1px_3px_rgba(255,0,100,0.5)": "accent-pink", "0_1px_3px_rgba(155,93,229,0.5)": "accent-violet",
                 "0_2px_8px_rgba(0,0,0,0.8)": "text", "0_0_8px_rgba(0,0,0,0.8)": "halo", "0_2px_4px_rgba(0,0,0,0.4)": "text-faint"},
}
GRAY = {"d2d2d2": "ink-light", "bcbcbc": "ink-silver", "b3b3b3": "ink-body", "777": "ink-ash", "777777": "ink-ash",
        "404040": "lift-strong", "202020": "card-alt", "1f1f1f": "sheet", "1c1c1c": "sheet-alt", "c0c0c0": "ink-silver-alt",
        "d4d4d1": "ink-warm"}
UTIL = r"(bg|text|border|ring|from|to|via|divide|fill|stroke|accent|outline|decoration|caret)"
stats = {}
def bump(k): stats[k] = stats.get(k, 0) + 1

RE_Z = re.compile(r"(?<![\w-])z-\[(\d+)\](?![\w-])")
RE_TEXT = re.compile(r"(?<![\w-])text-\[(\d+(?:\.\d+)?)px\](?![\w-])")
RE_SH = re.compile(r"(?<![\w-])(drop-shadow|shadow)-\[([^\]\s]+)\]")
RE_GRAY = re.compile(r"(?<![\w-])" + UTIL + r"-\[#([0-9a-fA-F]{3,6})\]")

def z(m):
    n = Z.get(m.group(1));  bump("z") if n else None
    return f"z-(--z-{n})" if n else m.group(0)
def tx(m):
    n = TEXT.get(m.group(1)); bump("text") if n else None
    return f"text-{n}" if n else m.group(0)
def sh(m):
    n = SHADOW[m.group(1)].get(m.group(2)); bump("shadow") if n else None
    return f"{m.group(1)}-{n}" if n else m.group(0)
def gr(m):
    n = GRAY.get(m.group(2).lower()); bump("gray") if n else None
    return f"{m.group(1)}-{n}" if n else m.group(0)

LAYOUT = [("h-[80px]", "h-(--nav-h)"), ("top-[88px]", "top-(--nav-h)"),
          ("calc(52px+env(safe-area-inset-top))", "calc(var(--nav-h-mobile)+env(safe-area-inset-top))"),
          ("calc(76px+env(safe-area-inset-bottom))", "calc(var(--bottom-nav-h)+env(safe-area-inset-bottom))")]
for f in FILES:
    s = f.read_text(); o = s
    s = RE_Z.sub(z, s); s = RE_TEXT.sub(tx, s); s = RE_SH.sub(sh, s); s = RE_GRAY.sub(gr, s)
    for a, b in LAYOUT:
        if a in s: bump("layout"); s = s.replace(a, b)
    if s != o: f.write_text(s)
print(stats)
