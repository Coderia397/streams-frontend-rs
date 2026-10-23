#!/usr/bin/env python3
"""Fourth codemod: brand, white and glass buttons become the component classes in style/components.css."""
import re, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
RE_CLASS = re.compile(r'class="([^"{}]+)"')
st = {"brand": 0, "ink": 0, "glass": 0}
def drop(toks, pats):
    return [t for t in toks if not any(re.fullmatch(p, t) for p in pats)]
def conv(m):
    toks = m.group(1).split()
    s = set(toks)
    if "bg-brand" in s and ({"hover:bg-brand-hover", "hover:bg-brand-active"} & s):
        st["brand"] += 1
        toks = drop(toks, [r"bg-brand", r"hover:bg-brand-(hover|active)", r"text-ink", r"font-bold", r"transition-colors", r"cursor-pointer"])
        return 'class="' + " ".join(["btn-brand"] + toks) + '"'
    if "bg-ink" in s and "text-void" in s and any(re.fullmatch(r"hover:bg-(neutral-200|gray-100|ink/90|ink/80)", t) for t in toks):
        st["ink"] += 1
        toks = drop(toks, [r"bg-ink", r"text-void", r"hover:bg-(neutral-200|gray-100|ink/90|ink/80)", r"font-bold", r"transition-colors", r"cursor-pointer"])
        return 'class="' + " ".join(["btn-ink"] + toks) + '"'
    if any(t.startswith("bg-button-secondary/") for t in toks) and any(t.startswith("hover:bg-button-secondary") for t in toks) and "text-ink" in s:
        st["glass"] += 1
        toks = drop(toks, [r"bg-button-secondary/\d+", r"hover:bg-button-secondary(/\d+)?", r"text-ink", r"font-bold", r"cursor-pointer"])
        return 'class="' + " ".join(["btn-glass"] + toks) + '"'
    return m.group(0)
for f in sorted((ROOT / "src").rglob("*.rs")):
    s = f.read_text(); o = s
    s = RE_CLASS.sub(conv, s)
    if s != o: f.write_text(s)
print(st)
