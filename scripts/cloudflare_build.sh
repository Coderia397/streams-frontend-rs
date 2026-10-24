#!/usr/bin/env bash
# Build the site on a Linux builder (Cloudflare Pages). Installs a minimal Rust toolchain with the wasm target, Trunk and the
# Tailwind standalone binary, then runs a release build into dist/. Versions match the ones used on the dev machine.
#   Pages settings: build command `bash scripts/cloudflare_build.sh`, output directory `dist`.
#   Optional environment variable: STREAMS_ENGINE_URL (base URL of the recommendation engine, no trailing slash).
set -euo pipefail

TRUNK_VERSION="${TRUNK_VERSION:-0.21.14}"
TAILWIND_VERSION="${TAILWIND_VERSION:-v4.0.9}"

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable -t wasm32-unknown-unknown
fi
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown

mkdir -p .tools
if [ ! -x .tools/trunk ]; then
  curl -sSfL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xz -C .tools
fi
curl -sSfL -o tailwindcss "https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_VERSION}/tailwindcss-linux-x64"
chmod +x tailwindcss

# The release profile in Cargo.toml (full LTO, one codegen unit) needs about 20 minutes here, which is the builder's limit.
# Full LTO with 16 codegen units: with Leptos type erasure (.cargo/config.toml) it built in 4 min 24 s on the owner's Mac against
# 4 min 40 s for thin LTO without it, and the wasm is 34% smaller than the thin LTO build (note 119, 2026-10-24). Set
# CARGO_PROFILE_RELEASE_LTO=thin in the Pages settings to go back if a build ever runs long.
export CARGO_PROFILE_RELEASE_LTO="${CARGO_PROFILE_RELEASE_LTO:-fat}"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-16}"

# The interop bundle needs Node 22 or newer and the pinned npm packages. The Pages image may ship an older Node, so fetch a
# pinned Node into .tools when the one on PATH is missing or too old (checksum verified).
NODE_VERSION="${NODE_VERSION:-22.23.3}"
if ! command -v node >/dev/null 2>&1 || [ "$(node -p 'process.versions.node.split(".")[0]')" -lt 22 ]; then
  NODE_FILE="node-v${NODE_VERSION}-linux-x64.tar.gz"
  mkdir -p .tools/node
  curl -sSfL -o ".tools/${NODE_FILE}" "https://nodejs.org/dist/v${NODE_VERSION}/${NODE_FILE}"
  curl -sSfL -o .tools/SHASUMS256.txt "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt"
  (cd .tools && grep " ${NODE_FILE}\$" SHASUMS256.txt | sha256sum -c -)
  tar -xzf ".tools/${NODE_FILE}" -C .tools/node --strip-components=1
  export PATH="$PWD/.tools/node/bin:$PATH"
fi
node --version
npm ci

rustc --version
./.tools/trunk --version
./.tools/trunk build --release
# the wasm compressed once at brotli's highest level, served by functions/_middleware.js (Cloudflare's own compression is lighter)
node scripts/precompress.mjs dist
ls -la dist | head -20
