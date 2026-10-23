#!/usr/bin/env python3
"""One time codemod: replace hardcoded colors and radii in Tailwind class strings with the tokens in style/brand.css.
Run from the frontend root: python3 scripts/codemod_tokens.py [--dry]"""
import re, sys, pathlib

DRY = "--dry" in sys.argv
ROOT = pathlib.Path(__file__).resolve().parent.parent
FILES = sorted((ROOT / "src").rglob("*.rs"))

HEX = {
    "e50914": "brand", "f40612": "brand-hover", "b80710": "brand-active",
    "ff0055": "accent", "ff007f": "accent-alt", "46d369": "success",
    "141414": "page", "181818": "raised", "222": "placeholder", "222222": "placeholder",
    "1e1e1e": "card", "262626": "field", "2f2f2f": "panel", "2a2a2a": "menu", "3a3a3a": "lift",
    "333": "line", "333333": "line-strong" if False else "line", "4d4d4d": "line-strong",
    "e5e5e5": "ink-soft", "bdbec0": "ink-dim", "808080": "mute", "6d6d6e": "button-secondary",
}
UTIL = r"(bg|text|border|ring|from|to|via|divide|fill|stroke|accent|outline|decoration|caret|shadow)"
RE_HEX = re.compile(r"(?<![\w-])" + UTIL + r"-\[#([0-9a-fA-F]{3,6})\]")

RED = {"red-600": "brand", "red-500": "danger", "red-400": "danger-soft", "red-300": "danger-mid",
       "red-200": "danger-pale", "red-700": "danger-strong", "red-950": "danger-deep"}
RE_RED = re.compile(r"(?<![\w-])" + UTIL + r"-(red-(?:200|300|400|500|600|700|950))(?![\w-])")

RADIUS = {"": "control", "sm": "control", "md": "field", "lg": "card", "xl": "modal",
          "[4px]": "control", "[6px]": "field", "[8px]": "card", "[12px]": "modal", "[2px]": "hairline",
          "[var(--tile-radius)]": "tile"}
RE_RADIUS = re.compile(
    r"(?<![\w-])rounded((?:-(?:t|b|l|r|s|e|tl|tr|bl|br|ss|se|es|ee))?)"
    r"(?:-(sm|md|lg|xl|\[4px\]|\[6px\]|\[8px\]|\[12px\]|\[2px\]|\[var\(--tile-radius\)\]))?(?![\w\[-])"
)
RE_RGBA_ARB = re.compile(r"(?<=_)rgba\(229,9,20,([0-9.]+)\)")

stats = {"hex": 0, "red": 0, "radius": 0, "rgba": 0}


def sub_hex(m):
    h = m.group(2).lower()
    tok = HEX.get(h)
    if not tok:
        return m.group(0)
    stats["hex"] += 1
    return f"{m.group(1)}-{tok}"


def sub_red(m):
    stats["red"] += 1
    return f"{m.group(1)}-{RED[m.group(2)]}"


def sub_radius(m):
    d, size = m.group(1), m.group(2) or ""
    stats["radius"] += 1
    return f"rounded{d}-{RADIUS[size]}"


def sub_rgba(m):
    pct = int(round(float(m.group(1)) * 100))
    stats["rgba"] += 1
    return f"color-mix(in_srgb,var(--color-brand)_{pct}%,transparent)"


for f in FILES:
    src = f.read_text()
    out = RE_HEX.sub(sub_hex, src)
    out = RE_RED.sub(sub_red, out)
    out = RE_RADIUS.sub(sub_radius, out)
    out = RE_RGBA_ARB.sub(sub_rgba, out)
    if out != src:
        print(("would change " if DRY else "changed ") + str(f.relative_to(ROOT)))
        if not DRY:
            f.write_text(out)

print(stats)
