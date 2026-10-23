"""Colours in the components that the brand does not control: a hex or rgb() colour written in a class, a style or an SVG attribute, or one
of Tailwind's own palette colours (bg-white, text-orange-400 ...). Every colour lives in style/brand.css as a token (bg-brand, text-ink-body,
--color-whatsapp ...), so a change of brand, or a change Netflix makes, is made once there (owner, 2026-10-23).

Black and white written in a style (shades, fades and masks: rgba(0,0,0,a), #000, #fff) are allowed there; a class uses bg-void and
text-ink. Exceptions: illustrations drawn in colour (a file whose first lines say `colour: art`) and a line with a `colour:` comment on it or up to
three lines above it, saying why (a third party's mark is a token too, so this is rare).
    python3 scripts/check_colours.py
"""
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
HEX = re.compile(r'(?<![\w&])#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3})(?![\w-])')
RGB = re.compile(r'\b(?:rgba?|hsla?)\(\s*\d')
PALETTE = re.compile(r'\b(?:bg|text|border(?:-[trblxyse])?|from|via|to|fill|stroke|ring|outline|shadow|decoration|accent|caret|divide|placeholder)-(?:(?:red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|slate|gray|zinc|neutral|stone)-\d{2,3}|white|black)\b')
# black and white in a style: shades, fades and masks
SHADE = re.compile(r'rgba?\(\s*0\s*,\s*0\s*,\s*0\s*(?:,\s*[\d.{}:]+\s*)?\)|rgba?\(\s*255\s*,\s*255\s*,\s*255\s*(?:,\s*[\d.{}:]+\s*)?\)|rgb\(0 0 0(?: / [\d.]+)?\)|(?<![\w&])#(?:000000|ffffff|000|fff)(?![\w-])')
STRING = re.compile(r'"(?:[^"\\]|\\.)*"|r#*"[^"]*"#*')

problems = []
for f in sorted((ROOT / "src").rglob("*.rs")):
    text = f.read_text(encoding="utf-8")
    head = "\n".join(text.split("\n")[:5])
    if "colour: art" in head:
        continue
    lines = text.split("\n")
    in_tests = False
    for i, line in enumerate(lines):
        if line.strip().startswith("#[cfg(test)]"):
            in_tests = True
        if in_tests or line.lstrip().startswith("//"):
            continue
        if any("colour:" in lines[j] for j in range(max(0, i - 3), i + 1)):
            continue
        for s in STRING.findall(line):
            plain = SHADE.sub("", s)
            hit = HEX.search(plain) or RGB.search(plain) or PALETTE.search(s)
            if hit:
                problems.append(f"{f.relative_to(ROOT)}:{i + 1}  {hit.group(0)}")
                break

for p in problems:
    print(p)
print(f"{len(problems)} colour(s) outside the brand tokens")
sys.exit(1 if problems else 0)
