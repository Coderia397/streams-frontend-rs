#!/usr/bin/env python3
"""Text sizes in the views come from the type scale in style/brand.css (text-label 13 px ... text-display-lg 56 px), so a size measured
on Netflix is set once and changes everywhere: this fails on a size typed into a class (text-[17px]). Add a token to the scale when a
measured size has none. Line heights (leading-[..]) are not checked: they are measured per place.
    python3 scripts/check_type.py
"""
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SIZE = re.compile(r'(?<![\w-])(?:[a-z]+:)*text-\[[0-9.]+(?:px|rem|em)\]')
problems = []
for f in sorted((ROOT / "src").rglob("*.rs")):
    for no, line in enumerate(f.read_text(encoding="utf-8").split("\n"), 1):
        if line.lstrip().startswith("//"):
            continue
        for m in SIZE.finditer(line):
            problems.append(f"{f.relative_to(ROOT)}:{no}  {m.group(0)}: use a size of the type scale (style/brand.css)")
print("\n".join(problems))
print(f"{len(problems)} text size(s) outside the type scale")
sys.exit(1 if problems else 0)
