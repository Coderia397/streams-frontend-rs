#!/usr/bin/env python3
"""Checks that buttons, links drawn as buttons, text fields and dropdowns take their corners from the shared radius (note 111).
The radius lives in style/brand.css: --radius-button for every button with text (class rounded-button, or a btn-* class, which carries it),
--radius-input for text fields and dropdowns (set by a rule in style/components.css, so they need no class), --radius-menu and
--radius-menu-item for menus. Icon buttons are circles (rounded-full).

Fails when a <button> or <a> carries another radius class (rounded-control, rounded-card ...), when a btn-* button also carries a radius
class, or when an <input> or <select> carries one. An element that is not a button in looks (an avatar tile, a card) is allowed when a
comment with `radius:` and the reason sits on its line or up to three lines above it.
    python3 scripts/check_radius.py
"""
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
RADIUS = re.compile(r'(?<![\w-])(?:[a-z]+:)?rounded(?:-(?:[a-z0-9]+(?:-[a-z0-9]+)*|\[[^\]]+\]))?(?![\w-])')
BUTTON_OK = {"rounded-button", "rounded-full", "rounded-menu-item", "rounded-none"}
SIDES = re.compile(r'rounded-(?:t|b|l|r|s|e|tl|tr|bl|br|ss|se|es|ee)(?:-|$)')
TAG = re.compile(r'<([a-zA-Z][a-zA-Z0-9_:]*)\b')

problems = []
for f in sorted((ROOT / "src").rglob("*.rs")):
    text = f.read_text(encoding="utf-8")
    lines = text.split("\n")
    for m in RADIUS.finditer(text):
        cls = m.group(0).split(":")[-1]
        if SIDES.match(cls):
            continue
        line_no = text.count("\n", 0, m.start()) + 1
        line = lines[line_no - 1]
        if line.lstrip().startswith("//"):
            continue
        tags = TAG.findall(text[max(0, m.start() - 1500):m.start()])
        tag = tags[-1] if tags else ""
        allowed = any("radius:" in lines[i] for i in range(max(0, line_no - 4), line_no))
        if allowed:
            continue
        where = f"{f.relative_to(ROOT)}:{line_no}"
        if tag in ("button", "a"):
            if re.search(r'\bbtn-(?:brand|ink|glass)\b', line):
                problems.append(f"{where}  btn-* already carries the button radius; remove {cls}")
            elif cls not in BUTTON_OK:
                problems.append(f"{where}  <{tag}> with {cls}: use rounded-button (or rounded-full for an icon button)")
        elif tag in ("input", "select"):
            if 'type="checkbox"' in line or 'type="radio"' in line or "accent-" in line:
                continue
            problems.append(f"{where}  <{tag}> with {cls}: fields take --radius-input from components.css; remove the class")

# A button or link drawn with a full border and no corners at all (a square box among pills: the language menus copied Netflix's old page
# until 2026-10-23). A btn-* class carries its own corners; a `radius:` comment allows a real exception.
SQUARE = re.compile(r'<(button|a)\b')
for f in sorted((ROOT / "src").rglob("*.rs")):
    text = f.read_text(encoding="utf-8")
    lines = text.split("\n")
    for m in SQUARE.finditer(text):
        rest = text[m.end():m.end() + 900]
        stop = rest.find("<")
        head = rest[:stop if stop >= 0 else len(rest)]
        at = head.find("class=")
        if at < 0:
            continue
        # every string of the class attribute (a plain string, or the branches of an if), up to the end of its line
        strings = re.findall(r'"([^"]*)"', head[at:].split("\n")[0])
        def square(s):
            bare = [t.split(":")[-1] for t in s.split()]
            bordered = any(t == "border" or re.fullmatch(r"border-[0-9]+|border-\[[0-9.]+px\]", t) for t in bare)
            return bordered and not any(t.startswith("rounded") or t.startswith("btn-") for t in bare)
        if not any(square(x) for x in strings):
            continue
        line_no = text.count("\n", 0, m.start()) + 1
        if any("radius:" in lines[i] for i in range(max(0, line_no - 4), min(len(lines), line_no + 3))):
            continue
        problems.append(f"{f.relative_to(ROOT)}:{line_no}  <{m.group(1)}> with a border and square corners: use a btn-* preset or rounded-button")

for p in problems:
    print(p)
print(f"{len(problems)} problem(s)")
sys.exit(1 if problems else 0)
