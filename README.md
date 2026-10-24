# Streams

**[streams.watch](https://streams.watch)**: free films and series, picked for you. Rows made for your taste, trailers, profiles
with a Kids profile, and a [Help Centre](https://help.streams.watch/help).

This repository is the web app: a Rust and WebAssembly front end built with [Leptos](https://leptos.dev), shaped after the apps
people already know on phones, tablets and computers.

## Build

- Development: `trunk serve` (port 8080)
- Release: `trunk build --release`
- Styles: Tailwind CSS v4 through the standalone `./tailwindcss` binary, run by Trunk's pre-build hook
- Browser glue: `interop/` (TypeScript, bundled by esbuild)

The site runs on Cloudflare Pages; `scripts/cloudflare_build.sh` is its build, and `functions/_middleware.js` gives each public page
its head and a plain first view while the app loads.
