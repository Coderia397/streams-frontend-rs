// Compresses the app's wasm once, at brotli's highest level, next to it as <name>.br (functions/_middleware.js sends that copy to
// browsers that take brotli). Cloudflare compresses files on the fly at a light level: 1,896,868 bytes for the live wasm on
// 2026-10-24, against 1,416,492 for the same file at level 11 (note 119). Run after `trunk build --release`.
//   node scripts/precompress.mjs dist
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { brotliCompressSync, constants } from 'node:zlib';

const dir = process.argv[2] || 'dist';
for (const name of readdirSync(dir).filter((n) => n.endsWith('_bg.wasm'))) {
  const raw = readFileSync(join(dir, name));
  const br = brotliCompressSync(raw, {
    params: {
      [constants.BROTLI_PARAM_QUALITY]: constants.BROTLI_MAX_QUALITY,
      [constants.BROTLI_PARAM_LGWIN]: constants.BROTLI_MAX_WINDOW_BITS,
      [constants.BROTLI_PARAM_SIZE_HINT]: raw.length,
    },
  });
  writeFileSync(join(dir, `${name}.br`), br);
  console.log(`${name}: ${raw.length} bytes, ${br.length} as brotli 11`);
}
