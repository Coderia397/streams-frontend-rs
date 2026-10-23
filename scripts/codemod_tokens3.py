#!/usr/bin/env python3
"""Third codemod: base white and black become the ink and void tokens; transition durations become motion tokens."""
import re, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
RE_BASE = re.compile(r"(?<![\w-])((?:bg|text|border(?:-[tblrxyse])?|divide|ring|from|to|via|fill|stroke|placeholder|outline|accent|caret|decoration|shadow))-(white|black)(?![\w-])")
RE_DUR = re.compile(r"(?<![\w-])duration-(75|100|150|200|300|500|700|1000)(?![\w-])")
MOTION = {"75": "instant", "100": "quick", "150": "fast", "200": "base", "300": "slow", "500": "slower", "700": "slowest", "1000": "glacial"}
st = {"base": 0, "dur": 0}
def base(m): st["base"] += 1; return f"{m.group(1)}-{'ink' if m.group(2) == 'white' else 'void'}"
def dur(m): st["dur"] += 1; return f"duration-(--motion-{MOTION[m.group(1)]})"
for f in sorted((ROOT / "src").rglob("*.rs")):
    s = f.read_text(); o = s
    s = RE_BASE.sub(base, s); s = RE_DUR.sub(dur, s)
    if s != o: f.write_text(s)
print(st)
