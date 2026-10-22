#!/usr/bin/env python3
"""Audit of the Help Centre (notes/108): structure, links both ways, tools, translations and the sitemap. Prints every problem found and a
summary line per check; exits 1 when a check fails.

  python3 scripts/help_audit.py [--lang xx] [--verbose]

Checks
  structure   every listed article has its four messages; related links resolve, 3 to 5, not to itself; one way related links listed
  links       every <a=...> in English bodies and policies: help articles exist, app paths are routes in src/lib.rs, mailto addresses are the
              brand's, https links listed for a manual check
  tools       every `@@ tool` line names a tool of src/pages/help/actions.rs
  app links   every /help/article/<slug> written anywhere in src or brand/ (the app linking into the Help Centre) resolves
  languages   per language: missing keys, values left in English, placeholders, line markers, tags and link targets that differ from English
  names       old product names (PStream, P-Stream) in any language file
  sitemap     help pages are listed on the Help Centre's host
"""
import collections, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts" / "i18n"))
import ftl  # noqa: E402

VERBOSE = "--verbose" in sys.argv
ONLY = sys.argv[sys.argv.index("--lang") + 1] if "--lang" in sys.argv else None
LANGS = ["ar", "de", "es", "fr", "hi", "it", "ja", "ko", "nl", "pl", "pt", "ru", "sv", "tr", "zh"]
NON_LATIN = {"ar", "hi", "ja", "ko", "ru", "zh"}
failures = collections.Counter()


def problem(check, text):
    failures[check] += 1
    print(f"  [{check}] {text}")


def load(lang):
    return dict(ftl.read((ROOT / "i18n" / f"{lang}.ftl").read_text()))


en = load("en")
data = (ROOT / "src/data/help.rs").read_text()
groups = re.findall(r'Group \{ id: "([a-z0-9-]+)", articles: &\[([^\]]*)\]', data)
listed = [s for _, arts in groups for s in re.findall(r'"([^"]+)"', arts)]
articles = dict((slug, re.findall(r'"([^"]+)"', rel)) for slug, rel in re.findall(r'Article \{ slug: "([a-z0-9-]+)", related: &\[([^\]]*)\]', data))
legal = re.findall(r'LegalDoc \{ id: "([a-z-]+)", path: "([^"]+)", sections: (\d+)', data)
tools = set(re.findall(r'"([a-z]+)"', re.search(r"pub const TOOLS: &\[&str\] = &\[([^\]]*)\]", (ROOT / "src/pages/help/actions.rs").read_text()).group(1)))
lib = (ROOT / "src/lib.rs").read_text()
routes = set(re.findall(r'path!\("([^"]+)"\)', lib))
brand = (ROOT / "src/brand.rs").read_text()
mailboxes = set(re.findall(r'"([a-z0-9._-]+@[a-z0-9.-]+)"', brand))


settings_mod = (ROOT / "src/pages/settings/mod.rs").read_text()
settings_paths = [m for m in re.findall(r'^\s*(\[[^\]]*\])(?: \| (\[[^\]]*\]))? (?:if [^=]*)?=> Screen::', settings_mod, re.M)]
settings_pats = []
for pair in settings_paths:
    for pat in pair:
        if pat:
            parts = re.findall(r'"([^"]+)"|(_|[a-z]+)', pat)
            settings_pats.append([a if a else None for a, b in parts])
static_files = {"/.well-known/" + m for m in re.findall(r'href="public/([^"]+)" data-target-path="\.well-known"', (ROOT / "index.html").read_text())}


def route_exists(path):
    path = path.split("?")[0].split("#")[0].rstrip("/") or "/"
    if path in static_files or (ROOT / "public" / path.lstrip("/")).is_file():
        return True
    if path.startswith("/settings/"):
        segs = path[len("/settings/"):].split("/")
        return any(len(p) == len(segs) and all(a is None or a == b for a, b in zip(p, segs)) for p in settings_pats)
    if path in routes:
        return True
    for r in routes:
        pat = "^" + re.sub(r":[a-z_]+", "[^/]+", r).replace("*any", ".*") + "$"
        if re.match(pat, path):
            return True
    return False


print("== structure")
for slug in listed:
    if slug.startswith("doc:"):
        if slug[4:] not in {d for d, _, _ in legal}:
            problem("structure", f"{slug} listed but no policy")
        continue
    if slug not in articles:
        problem("structure", f"{slug} listed but no Article entry")
    for part in ("title", "summary", "keywords", "body"):
        if f"help-art-{slug}-{part}" not in en:
            problem("structure", f"{slug}: no help-art-{slug}-{part} in en.ftl")
for slug, rel in articles.items():
    if slug not in listed:
        problem("structure", f"{slug} has an entry but is listed under no group")
    if not 3 <= len(rel) <= 5:
        problem("structure", f"{slug}: {len(rel)} related articles")
    for r in rel:
        if r not in articles:
            problem("structure", f"{slug}: related {r} does not exist")
        if r == slug:
            problem("structure", f"{slug}: related to itself")
orphans = [k for k in en if k.startswith("help-art-") and re.sub(r"^help-art-|-(title|summary|keywords|body)$", "", k) not in articles]
for k in orphans:
    problem("structure", f"orphan message {k}")
one_way = sorted((a, b) for a, rel in articles.items() for b in rel if a not in articles.get(b, []))
print(f"  related links one way only: {len(one_way)} of {sum(len(r) for r in articles.values())} (listed with --verbose)")
if VERBOSE:
    for a, b in one_way:
        print(f"    {a} -> {b}")

print("== links in English bodies and policies")
external = set()
link_keys = [k for k in en if k.startswith(("help-art-", "legal-", "help-ui-", "help-act-"))]
for k in link_keys:
    for target in re.findall(r"<a=([^>]+)>", en[k]) + re.findall(r'href="([^"]+)"', en[k]):
        if target.startswith("/help/article/"):
            if target.split("/")[-1].split("#")[0] not in articles:
                problem("links", f"{k}: {target} is no article")
        elif target.startswith("mailto:"):
            addr = target[7:].split("?")[0]
            if "{" not in addr and addr not in mailboxes:
                problem("links", f"{k}: {target} is not one of the brand's mailboxes {sorted(mailboxes)}")
        elif target.startswith("http"):
            external.add(target)
        elif target.startswith("/"):
            if not route_exists(target):
                problem("links", f"{k}: {target} is no route in src/lib.rs")
        elif "{" in target:
            pass
        else:
            problem("links", f"{k}: {target} is neither a path nor an address")
print(f"  external links to check by hand: {len(external)}")
for u in sorted(external):
    print(f"    {u}")

print("== tools")
for k in en:
    if k.endswith("-body"):
        for t in re.findall(r"^\s*@@\s*([a-z]+)", en[k], re.M):
            if t not in tools:
                problem("tools", f"{k}: @@ {t} is not a tool ({sorted(tools)})")

print("== app links into the Help Centre")
for f in list((ROOT / "src").rglob("*.rs")) + list((ROOT / "brand").rglob("*.html")) + list((ROOT / "i18n").glob("en.ftl")):
    for slug in re.findall(r"/help/article/([a-z0-9-]+)", f.read_text(errors="ignore")):
        if slug not in articles:
            problem("app links", f"{f.relative_to(ROOT)}: /help/article/{slug} does not exist")

print("== languages")
help_keys = [k for k in en if k.startswith(("help-", "legal-"))]
for lang in [ONLY] if ONLY else LANGS:
    tr = load(lang)
    missing = [k for k in en if k not in tr]
    extra = [k for k in tr if k not in en]
    english, broken = [], []
    for k, v in en.items():
        if k not in tr:
            continue
        t = tr[k]
        letters = re.sub(r"\{[^}]*\}|<[^>]+>|https?://\S+|\S+@\S+", "", v)
        if t == v and len(re.findall(r"[A-Za-z]{3,}", letters)) >= 3:
            english.append(k)
        elif lang in NON_LATIN and len(re.findall(r"\b[A-Za-z]{4,}\b", re.sub(r"\{[^}]*\}|<[^>]+>|https?://\S+|\S+@\S+", "", t))) >= 6:
            english.append(k)
        for p in ftl.check(v, t):
            broken.append((k, p))
    help_missing = [k for k in missing if k in help_keys]
    print(f"  {lang}: missing {len(missing)} (help {len(help_missing)}), extra {len(extra)}, left in English {len(english)}, structure problems {len(broken)}")
    for k in missing[:40]:
        problem(f"{lang} missing", k)
    for k in english[:40]:
        problem(f"{lang} english", f"{k}: {tr[k][:90]!r}")
    for k, p in broken[:40]:
        problem(f"{lang} structure", f"{k}: {p[:160]}")

print("== link texts match the titles they point to")
for lang in ["en"] + ([ONLY] if ONLY else LANGS):
    d = load(lang)
    bad = 0
    for k, v in d.items():
        if not k.startswith("help-art-") or not k.endswith("-body"):
            continue
        for slug, text in re.findall(r"<a=/help/article/([a-z0-9-]+)>([^<]*)</a>", v):
            title = d.get(f"help-art-{slug}-title", en.get(f"help-art-{slug}-title", ""))
            # a link may be a short phrase (\"tell us\"); only a text that looks like a title (four words or more) must be the title
            if len(text.split()) >= 4 and text.strip() != title.strip():
                bad += 1
                problem(f"{lang} link text", f"{k}: '{text}' links {slug} titled '{title}'")
    print(f"  {lang}: {bad} link texts differ from the title")

print("== quoted interface text and scripts")
SCRIPTS = {"cjk": r"[\u4e00-\u9fff]", "kana": r"[\u3040-\u30ff]", "hangul": r"[\uac00-\ud7af]", "cyrillic": r"[\u0400-\u04ff]", "arabic": r"[\u0600-\u06ff]", "devanagari": r"[\u0900-\u097f]"}
OWN = {"ja": {"cjk", "kana"}, "zh": {"cjk"}, "ko": {"hangul", "cjk"}, "ru": {"cyrillic"}, "ar": {"arabic"}, "hi": {"devanagari"}}
for lang in ["en"] + ([ONLY] if ONLY else LANGS):
    d = load(lang)
    body = d.get("help-art-playback-unavailable-body", "")
    for key in ("player-unavailable", "player-error-no-source"):
        if d.get(key) and d[key] not in body:
            problem(f"{lang} quote", f"help-art-playback-unavailable-body does not quote {key} as the player shows it")
    for k, v in d.items():
        for name, rx in SCRIPTS.items():
            if name not in OWN.get(lang, set()) and re.search(rx, v):
                problem(f"{lang} script", f"{k}: {name} characters in a {lang} message")

print("== old names")
for f in sorted((ROOT / "i18n").glob("*.ftl")):
    for n, line in enumerate(f.read_text().split("\n"), 1):
        if re.search(r"p-?stream", line, re.I):
            problem("names", f"{f.name}:{n}: {line.strip()[:100]}")

print("== sitemap")
# public/sitemap.xml lists every page on the main host; functions/_middleware.js gives each host its own sitemap (the help pages on help.streams.watch)
sm = (ROOT / "public/sitemap.xml").read_text()
mw = (ROOT / "functions/_middleware.js").read_text()
if "/sitemap.xml" not in mw or "HELP" not in mw:
    problem("sitemap", "functions/_middleware.js no longer splits the sitemap per host")
listed_slugs = set(re.findall(r"/help/article/([a-z0-9-]+)</loc>", sm))
for slug in articles:
    if slug not in listed_slugs:
        problem("sitemap", f"/help/article/{slug} is not in public/sitemap.xml (run scripts/make_sitemap.py)")
for slug in listed_slugs - set(articles):
    problem("sitemap", f"/help/article/{slug} is in the sitemap but is no article")

print("\nsummary:", dict(failures) or "no problems")
sys.exit(1 if failures else 0)
