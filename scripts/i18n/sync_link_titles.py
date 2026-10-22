#!/usr/bin/env python3
"""Makes every help article link that reads like a title (four words or more) carry the exact title of the article it points to, in the same
language, so a reader sees the same name in the link and on the page it opens. Short phrases ("tell us") are left as written.

  python3 scripts/i18n/sync_link_titles.py <lang> [<lang> ...]
"""
import json, pathlib, re, subprocess, sys, tempfile
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "i18n"))
import ftl  # noqa: E402

for lang in sys.argv[1:]:
    d = dict(ftl.read((ROOT / "i18n" / f"{lang}.ftl").read_text()))
    changes = {}
    for k, v in d.items():
        if not (k.startswith("help-art-") and k.endswith("-body")):
            continue
        def fix(m):
            slug, text = m.group(1), m.group(2)
            title = d.get(f"help-art-{slug}-title")
            return f"<a=/help/article/{slug}>{title}</a>" if title and len(text.split()) >= 4 and text.strip() != title.strip() else m.group(0)
        nv = re.sub(r"<a=/help/article/([a-z0-9-]+)>([^<]*)</a>", fix, v)
        if nv != v:
            changes[k] = nv
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        json.dump(changes, f, ensure_ascii=False)
    subprocess.run([sys.executable, str(ROOT / "scripts/i18n/apply.py"), lang, f.name], check=True)
