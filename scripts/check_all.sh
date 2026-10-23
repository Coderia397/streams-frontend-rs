#!/bin/sh
# One command for everything that can be checked without a person: the code compiles for the browser, clippy finds no errors, no text is typed into the pages instead of the translation
# files, every message has its English text, buttons and fields take the shared corner radius, and (with the dev server running on :8080) the pages load and the rows page round.
#   sh scripts/check_all.sh            all checks
#   SMOKE=0 sh scripts/check_all.sh    without the browser test
set -e
cd "$(dirname "$0")/.."
echo "== compile (wasm)";  cargo check --target wasm32-unknown-unknown --profile phone
echo "== clippy errors";   cargo clippy --target wasm32-unknown-unknown --profile phone -- -A warnings
echo "== hard coded texts"; python3 scripts/check_literals.py
echo "== translations";    python3 scripts/check_i18n.py | tail -20
echo "== corner radius";   python3 scripts/check_radius.py
echo "== class names";     python3 scripts/check_classes.py
echo "== colours";         python3 scripts/check_colours.py
if [ "${SMOKE:-1}" != "0" ]; then echo "== browser smoke test"; uv run --quiet --with websockets python3 scripts/smoke_test.py; fi
echo "all checks done"
