#!/usr/bin/env python3
"""Finds user visible text that is typed into the code instead of coming from the translation files (i18n/en.ftl through t(...)).
Looks inside view! macros for (1) text nodes written as a quoted string and (2) the accessibility and label attributes aria-label, title,
placeholder, alt, aria-description, aria-placeholder, aria-roledescription and label written as a quoted string.

  python3 scripts/check_literals.py              check against scripts/literals_baseline.json (exit 1 if any file has more than its baseline)
  python3 scripts/check_literals.py --update     write the baseline (do this after fixing some, never to hide new ones)
  python3 scripts/check_literals.py --list       print every finding

New code must not add findings; the baseline only goes down. Standard library only."""
import json, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASE = ROOT / "scripts" / "literals_baseline.json"
ATTR = re.compile(r'\b(aria-label|title|placeholder|alt|aria-description|aria-placeholder|aria-roledescription|label)\s*=\s*"([^"{}]*)"')
TEXT_LINE = re.compile(r'^\s*"([^"{}]*)"\s*$')
TEXT_INLINE = re.compile(r'>\s*"([^"{}]*)"\s*<')
LETTERS = re.compile(r"[A-Za-zÀ-ɏЀ-ӿ؀-ۿ぀-ヿ一-鿿]{2,}")
SKIP_VALUES = {"HD", "4K", "UHD", "NR", "U", "PG", "12", "15", "18", "TV", "PIN", "OK", "VPN", "DNS", "HLS", "Streams", "TOP 10"}


def real_text(s: str) -> bool:
    """Words a person reads. CSS classes, translation keys and identifiers are not: they have no capital letter to start a word in ASCII text."""
    s = s.strip()
    if not LETTERS.search(s) or s in SKIP_VALUES or s.startswith(("ph-", "http", "/", "#", ".")):
        return False
    if s.isascii() and not re.match(r"[A-Z]", s):
        return False
    return True


def findings(path: pathlib.Path):
    out, depth, in_view = [], 0, False
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if "view!" in line and "{" in line:
            in_view, depth = True, 0
        if in_view:
            depth += line.count("{") - line.count("}")
            for rx, kind in ((ATTR, "attribute"),):
                for m in rx.finditer(line):
                    if real_text(m.group(2)):
                        out.append((n, kind + " " + m.group(1), m.group(2)))
            for m in TEXT_INLINE.finditer(line):
                if real_text(m.group(1)):
                    out.append((n, "text", m.group(1)))
            m = TEXT_LINE.match(line)
            if m and real_text(m.group(1)):
                out.append((n, "text", m.group(1)))
            if depth <= 0 and "view!" not in line:
                in_view = False
    return out


def main():
    result = {}
    for p in sorted((ROOT / "src").rglob("*.rs")):
        f = findings(p)
        if f:
            result[str(p.relative_to(ROOT))] = f
    total = sum(len(v) for v in result.values())
    if "--list" in sys.argv:
        for file, items in result.items():
            for n, kind, text in items:
                print(f"{file}:{n}: {kind}: {text!r}")
    counts = {k: len(v) for k, v in result.items()}
    if "--update" in sys.argv:
        BASE.write_text(json.dumps(counts, indent=1, sort_keys=True) + "\n")
        print(f"baseline written: {total} findings in {len(counts)} files")
        return 0
    base = json.loads(BASE.read_text()) if BASE.exists() else {}
    worse = {k: (base.get(k, 0), c) for k, c in counts.items() if c > base.get(k, 0)}
    print(f"{total} hard coded texts in {len(counts)} files (baseline {sum(base.values())})")
    for k, (b, c) in sorted(worse.items()):
        print(f"  MORE: {k} {b} -> {c}")
    top = sorted(counts.items(), key=lambda kv: -kv[1])[:8]
    print("largest:", ", ".join(f"{k.split('/')[-1]} {c}" for k, c in top))
    return 1 if worse else 0


sys.exit(main())
