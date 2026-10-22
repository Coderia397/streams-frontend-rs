#!/usr/bin/env python3
"""Replaces, adds or removes messages in one i18n/<lang>.ftl file, touching nothing else (the file is never re-written as a whole, so select
expressions and comments keep their layout).

  python3 scripts/i18n/apply.py <lang> <changes.json> [--after-key KEY]

changes.json: {"key": "value", "other-key": null}. A value with line breaks is written as an indented block (blank lines kept), followed by one
blank line, as the article bodies are. null removes the message. A key not in the file is inserted after the key that comes before it in en.ftl
(so every language keeps English's order); for en itself, after --after-key or after the previous new key. Prints what it did.
"""
import json, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
KEY = re.compile(r"^([a-z0-9][a-z0-9_-]*) =")


def blocks(lines):
    """[(key, start, end)] for every message: the key line and its continuation (indented lines, a `}` closing a select, blank lines inside)."""
    out, i = [], 0
    while i < len(lines):
        m = KEY.match(lines[i])
        if not m:
            i += 1
            continue
        j = i + 1
        while j < len(lines):
            line = lines[j]
            if line.startswith((" ", "\t")) or line.strip() == "}" and not line.startswith("}}"):
                j += 1
                continue
            if line == "":
                k = j
                while k < len(lines) and lines[k] == "":
                    k += 1
                if k < len(lines) and lines[k].startswith((" ", "\t")):
                    j = k
                    continue
            break
        out.append((m.group(1), i, j))
        i = j
    return out


def render(key, value):
    if "\n" not in value and not value.startswith(("[", "*", ".")):
        return [f"{key} = {value}"]
    body = [("    " + l) if l.strip() else "" for l in value.split("\n")]
    return [f"{key} ="] + body + [""]


def main():
    lang, changes = sys.argv[1], json.loads(pathlib.Path(sys.argv[2]).read_text())
    anchor = sys.argv[sys.argv.index("--after-key") + 1] if "--after-key" in sys.argv else None
    explicit_anchor = anchor is not None
    path = ROOT / "i18n" / f"{lang}.ftl"
    lines = path.read_text().split("\n")
    en_order = [k for k, _, _ in blocks((ROOT / "i18n" / "en.ftl").read_text().split("\n"))]
    done = {"replaced": 0, "added": 0, "removed": 0}
    for key, value in changes.items():
        found = {k: (s, e) for k, s, e in blocks(lines)}
        if key in found:
            s, e = found[key]
            # a block ends before the blank line that follows it; a rendered multi-line block brings its own, and runs of blank lines are folded below
            if value is None:
                lines[s:e] = []
                done["removed"] += 1
            else:
                lines[s:e] = render(key, value)
                done["replaced"] += 1
            continue
        if value is None:
            continue
        prev = None
        if lang == "en" and not explicit_anchor:
            # after the last message that shares the longest dash separated prefix with the new key (legal-privacy-2-b4 after legal-privacy-2-b3,
            # help-art-<new>-* at the end of the articles); the key just added wins when it shares as much
            def shared(a, b):
                n = 0
                for x, y in zip(a.split("-"), b.split("-")):
                    if x != y:
                        break
                    n += 1
                return n
            order = [k for k, _, _ in blocks(lines)]
            best = max((shared(key, k) for k in order), default=0)
            if best >= 2:
                prev = [k for k in order if shared(key, k) == best][-1]
                if anchor and anchor in found and shared(key, anchor) >= best:
                    prev = anchor
        elif lang == "en" or key not in en_order:
            prev = anchor
        else:
            idx = en_order.index(key)
            for k in reversed(en_order[:idx]):
                if k in found:
                    prev = k
                    break
        if prev and prev in found:
            s, at = found[prev]
            # after a multi-line block, past the blank line that closes it
            if at - s > 1 and at < len(lines) and lines[at] == "":
                at += 1
        else:
            at = len(lines)
            while at > 0 and lines[at - 1] == "":
                at -= 1
        lines[at:at] = render(key, value)
        done["added"] += 1
        anchor = key
    text = "\n".join(lines)
    text = re.sub(r"\n{3,}", "\n\n", text)
    path.write_text(text if text.endswith("\n") else text + "\n")
    print(lang, done)


if __name__ == "__main__":
    main()
