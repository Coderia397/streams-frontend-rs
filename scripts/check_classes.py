"""Class names in the Rust views that no stylesheet rule backs: a misspelt token, a utility this Tailwind version does not make (bg-size-[..],
a shadow with /alpha), or a name from a plugin that is not loaded (animate-in, fade-in, used on six screens before 2026-10-23 with no effect).
Reads the built stylesheet in dist/ (run after a build). Icons (ph-*) come from the icon font's own stylesheet; HOOKS are names the code looks
up or that only label a block."""
import collections, glob, re, sys

HOOKS = {"spot-card", "rating-bubble", "collection-row", "collection-hero"}
root = sys.argv[1] if len(sys.argv) > 1 else "."
sheets = glob.glob(f"{root}/dist/output-*.css")
if not sheets:
    print("no built stylesheet in dist/: build first"); sys.exit(0)
css = open(sheets[0], encoding="utf-8").read()

def unescape(name):
    name = re.sub(r"\\([0-9a-fA-F]{1,6}) ?", lambda m: chr(int(m.group(1), 16)), name)
    return re.sub(r"\\(.)", r"\1", name)

rules = {unescape(m.group(1)) for m in re.finditer(r"\.((?:\\[0-9a-fA-F]{1,6} ?|\\.|[A-Za-z0-9_-])+)", css)}
uses = collections.defaultdict(list)
for f in glob.glob(f"{root}/src/**/*.rs", recursive=True):
    text = open(f, encoding="utf-8").read()
    for m in re.finditer(r'class=(?:\(\s*|\{\s*)?"([^"]*)"', text):
        line = text.count("\n", 0, m.start()) + 1
        for token in m.group(1).split():
            uses[token].append(f"{f[len(root) + 1:]}:{line}")
skip = re.compile(r"^(group|peer)(/.*)?$|[{}$]|^ph(-|$)")
missing = {t: at for t, at in uses.items() if t not in rules and t not in HOOKS and not skip.search(t)}
for token, at in sorted(missing.items()):
    print(f"{token}  {', '.join(at[:3])}")
print(f"{len(missing)} class name(s) without a rule")
sys.exit(1 if missing else 0)
