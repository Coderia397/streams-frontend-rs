#!/bin/sh
# Type checks and bundles interop/index.ts into interop/dist/streams-interop.js (ES module, bound from Rust). Fails the build on any error.
# Uses the project local Node in .tools/node when present, otherwise the Node on PATH. Runs in the Trunk pre_build hook.
set -eu
cd "$(dirname "$0")/.."
[ -d .tools/node/bin ] && PATH="$PWD/.tools/node/bin:$PATH"
command -v node >/dev/null 2>&1 || { echo "build_interop: node not found (install it or run scripts/setup_tools.sh)" >&2; exit 1; }
[ -d node_modules ] || npm ci
node node_modules/typescript/bin/tsc -p interop/tsconfig.json
node_modules/.bin/esbuild interop/index.ts --bundle --format=esm --target=es2022 --minify --outfile=interop/dist/streams-interop.js --log-level=warning
# hls.js on its own, loaded by interop/hls.ts the first time a stream plays (index.html copies it to the site's root)
node_modules/.bin/esbuild interop/hls-player.ts --bundle --format=esm --target=es2022 --minify --outfile=interop/dist/hls-player.js --log-level=warning
