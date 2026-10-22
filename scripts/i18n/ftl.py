"""Small reader for the .ftl files used here: messages are 'key = value' blocks, and a value can continue on indented lines."""
import re

KEY = re.compile(r"^([a-z0-9][a-z0-9_-]*) =(.*)$")


def read(text):
    """Returns an ordered list of (key, value) where value is the message text with the common indent removed."""
    entries, key, first, rest = [], None, "", []
    for line in text.split("\n"):
        m = KEY.match(line)
        if m:
            if key is not None:
                entries.append((key, _join(first, rest)))
            key, first, rest = m.group(1), m.group(2).strip(), []
        elif key is not None and (line.startswith(" ") or line == ""):
            rest.append(line)
    if key is not None:
        entries.append((key, _join(first, rest)))
    return entries


def _join(first, rest):
    while rest and rest[-1] == "":
        rest.pop()
    if not rest:
        return first
    indents = [len(l) - len(l.lstrip()) for l in rest if l.strip()]
    cut = min(indents) if indents else 0
    body = "\n".join(l[cut:] if l.strip() else "" for l in rest)
    return (first + "\n" + body) if first else body


def write(entries):
    """Entries back to .ftl text. Single line values stay on the key line; others start on the next line, indented."""
    out = []
    for key, value in entries:
        if "\n" in value or value.startswith(("[", "*", ".")):
            out.append(f"{key} =\n" + "\n".join(("    " + l) if l.strip() else "" for l in value.split("\n")))
        else:
            out.append(f"{key} = {value}")
    return "\n".join(out) + "\n"


PLACEHOLDER = re.compile(r"\{\s*\$([A-Za-z_]+)\s*\}|\{\s*\$([A-Za-z_]+)\s*->")
MARKERS = re.compile(r"^(##\s|!!\s|>>|@@\s|\d+\.\s|-\s)")


def variables(value):
    return sorted({a or b for a, b in PLACEHOLDER.findall(value)})


def structure(value):
    """The ordered line markers (headings, steps, bullets, notes, folds, tools) of a body, to compare with the English one."""
    marks = []
    for line in value.split("\n"):
        m = MARKERS.match(line.strip())
        if m:
            t = m.group(1).strip()
            marks.append("#" if t[0].isdigit() else t)
    return marks


def tags(value):
    return (value.count("<b>"), value.count("</b>"), len(re.findall(r"<a=[^>]+>", value)), value.count("</a>"), len(re.findall(r'<a href=', value)))


def urls(value):
    return sorted(re.findall(r"<a=([^>]+)>", value) + re.findall(r'href="([^"]+)"', value))


def check(en_value, tr_value):
    """List of problems with a translation, empty when it keeps everything the app depends on."""
    problems = []
    if not tr_value.strip():
        return ["empty"]
    if variables(en_value) != variables(tr_value):
        problems.append(f"placeholders differ: {variables(en_value)} vs {variables(tr_value)}")
    if structure(en_value) != structure(tr_value):
        problems.append(f"line markers differ: {structure(en_value)} vs {structure(tr_value)}")
    if tags(en_value) != tags(tr_value):
        problems.append(f"tags differ: {tags(en_value)} vs {tags(tr_value)}")
    if urls(en_value) != urls(tr_value):
        problems.append(f"links differ: {urls(en_value)} vs {urls(tr_value)}")
    if tr_value.count("{") != tr_value.count("}"):
        problems.append("braces are not balanced")
    if ("->" in en_value) != ("->" in tr_value):
        problems.append("plural selection lost or added")
    if "->" in tr_value and "*[" not in tr_value:
        problems.append("plural selection has no default variant *[other]")
    return problems
