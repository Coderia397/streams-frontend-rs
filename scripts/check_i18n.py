#!/usr/bin/env python3
"""Checks interface text keys. Fails when a key used in src is missing from i18n/en.ftl, or when en.ftl holds a key nothing uses.
Prints a coverage line per language file. --prune removes unused keys from en.ftl.
Genre names (genre-*) and policy text (legal-<id>-*) are looked up at runtime from data. Help center messages are looked up by id (help-art-<slug>-<part>, help-topic-<id>, help-group-<id>, help-ui-ql-<id>), so a key of
those families counts as used when its id appears as a string in src/data/help.rs."""
import re, sys, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
KEY = re.compile(r"^([a-z0-9_-]+) =", re.M)
def load(p):
    text = p.read_text()
    return {m.group(1): m.start() for m in KEY.finditer(text)}
en = load(ROOT / "i18n/en.ftl")
used = set()
help_data = (ROOT / "src/data/help.rs").read_text()
help_ids = set(re.findall(r'"([a-z0-9-]+)"', help_data))
for f in (ROOT / "src").rglob("*.rs"):
    if f.name != "i18n.rs":
        text = f.read_text()
        used |= set(re.findall(r'i18n::t(?:_args)?\("([^"]+)"', text))
        used |= set(re.findall(r'"([a-z0-9-]+)"', text)) & set(en)  # keys passed around as plain string constants
def family_id(key):
    for prefix in ("help-topic-", "help-group-", "help-ui-ql-"):
        if key.startswith(prefix):
            return key[len(prefix):]
    if key.startswith("help-art-"):
        return key[len("help-art-"):].rsplit("-", 1)[0]
tool_ids = set(re.findall(r'"([a-z]+)"', (ROOT / "src/pages/help/actions.rs").read_text().split("pub const TOOLS")[1].split("];")[0]))
legal_ids = {i for i in help_ids if f"legal-{i}-title" in en}
for k in en:
    fid = family_id(k)
    if fid and fid in help_ids:
        used.add(k)
    elif k.startswith("help-card-") and any(k.startswith(f"help-card-{t}-") for t in tool_ids):
        used.add(k)  # tool cards on the help home, by tool id
    elif k.startswith("mood-"):
        used.add(k)  # mood words, looked up at runtime (i18n::mood_label)
    elif k.startswith(("set-color-", "set-edge-", "set-event-")):
        used.add(k)  # subtitle colours and edge styles, and account event names, looked up at runtime on the settings screens
    elif k.startswith("email-"):
        used.add(k)  # sent by the mail Worker (streams-backend-rs/mail), not shown in the app
    elif k.startswith(("row-", "hook-", "topic-", "event-", "reason-", "collection-", "notif-")):
        used.add(k)  # row titles, hooks, topics, event names, content reasons and collections: the engine and the skip marks name them by key, looked up at runtime
    elif k.startswith("genre-"):
        used.add(k)  # looked up from the genre name at runtime (i18n::genre_label)
    elif k.startswith("legal-") and any(k.startswith(f"legal-{i}-") for i in legal_ids):
        used.add(k)  # policy text, looked up by document id and section number
missing = sorted(used - set(en)); unused = sorted(set(en) - used)
if "--prune" in sys.argv and unused:
    p = ROOT / "i18n/en.ftl"
    lines, out, skip = p.read_text().split("\n"), [], False
    for l in lines:
        m = KEY.match(l)
        if m:
            skip = m.group(1) in unused
        elif l and not l[0].isspace():
            skip = False
        if not skip:
            out.append(l)
    p.write_text("\n".join(out))
    unused = []
print(f"keys used {len(used)}, in en.ftl {len(en)}, missing {len(missing)}, unused {len(unused)}")
for k in missing: print("  missing:", k)
for k in unused[:10]: print("  unused:", k)
for f in sorted((ROOT / "i18n").glob("*.ftl")):
    if f.name != "en.ftl":
        d = load(f); print(f"  {f.stem}: {len([k for k in d if k in en])}/{len(en)} translated")
sys.exit(1 if missing or unused else 0)
