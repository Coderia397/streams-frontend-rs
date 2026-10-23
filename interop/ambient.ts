export interface AmbientRGB {
  r: number;
  g: number;
  b: number;
}

// version 3: the artwork's second colour (its first when there is no distinct second), a little more saturated (the app tints the top of the page with it)
// All colours live in one stored map, keyed by the picture's file name (the same picture at another size has the same colour), the newest
// KEEP of them; the colour of the last billboard shown has a key of its own, read at start for the first tint.
const STORE_KEY = 'streams-ambient3';
const LAST_KEY = 'streams-ambient3:last';
const KEEP = 400;
// before the map, every picture had a key of its own ('streams-ambient2:<url>', 'streams-ambient3:<url>'), never removed
const OLD_PREFIXES = ['streams-ambient2:', 'streams-ambient3:'];

let cache: Map<string, AmbientRGB> | null = null;

function fileKey(imageUrl: string): string {
  return imageUrl.includes('image.tmdb.org/t/p/') ? imageUrl.slice(imageUrl.lastIndexOf('/') + 1) : imageUrl;
}

function load(): Map<string, AmbientRGB> {
  if (cache) return cache;
  cache = new Map();
  try {
    const raw = localStorage.getItem(STORE_KEY);
    if (raw) {
      for (const [k, v] of JSON.parse(raw) as [string, AmbientRGB][]) cache.set(k, v);
    } else {
      // first start with the map: keep the version 3 colours, drop the version 2 ones, remove the old keys
      const old: string[] = [];
      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i);
        if (k && k !== LAST_KEY && OLD_PREFIXES.some((p) => k.startsWith(p))) old.push(k);
      }
      for (const k of old) {
        if (k.startsWith('streams-ambient3:')) {
          try { cache.set(fileKey(k.slice('streams-ambient3:'.length)), JSON.parse(localStorage.getItem(k) ?? '')); } catch {}
        }
        localStorage.removeItem(k);
      }
      save();
    }
  } catch {}
  return cache;
}

function save(): void {
  if (!cache) return;
  while (cache.size > KEEP) cache.delete(cache.keys().next().value as string);
  try { localStorage.setItem(STORE_KEY, JSON.stringify([...cache])); } catch {}
}

export function getCachedAmbientColor(imageUrl: string): AmbientRGB | null {
  if (!imageUrl) return null;
  return load().get(fileKey(imageUrl)) ?? null;
}

export function getLastAmbientColor(): AmbientRGB | null {
  try {
    const raw = localStorage.getItem(LAST_KEY);
    if (raw) return JSON.parse(raw);
  } catch {}
  return null;
}

export function extractAmbientColor(imageUrl: string): Promise<AmbientRGB | null> {
  if (!imageUrl) return Promise.resolve(null);
  const cached = getCachedAmbientColor(imageUrl);
  if (cached) return Promise.resolve(cached);

  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';

    // Optimization: TMDB w92 thumbnail is only 2-4KB, downloads in ~15ms,
    // and produces the exact same color palette as a 5MB original backdrop.
    let fastUrl = imageUrl;
    if (fastUrl.includes('image.tmdb.org/t/p/')) {
      fastUrl = fastUrl.replace(/\/t\/p\/[^/]+/, '/t/p/w92');
    }
    img.src = fastUrl.includes('?') ? `${fastUrl}&cors=true` : `${fastUrl}?cors=true`;

    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d', { willReadFrequently: true });
        if (!ctx) return resolve(null);

        canvas.width = 32;
        canvas.height = 32;
        ctx.drawImage(img, 0, 0, 32, 32);
        const data = ctx.getImageData(0, 0, 32, 32).data;

        // Group the pixels into colours (3 bits a channel, 512 groups), leaving out see through, near black and near white pixels
        const groups = new Map<number, { n: number; r: number; g: number; b: number }>();
        let total = 0;
        for (let i = 0; i < data.length; i += 4) {
          const pr = data[i] ?? 0, pg = data[i + 1] ?? 0, pb = data[i + 2] ?? 0, pa = data[i + 3] ?? 0;
          if (pa <= 180) continue;
          const brightness = (pr * 299 + pg * 587 + pb * 114) / 1000;
          if (brightness <= 20 || brightness >= 235) continue;
          const key = ((pr >> 5) << 6) | ((pg >> 5) << 3) | (pb >> 5);
          const grp = groups.get(key) ?? { n: 0, r: 0, g: 0, b: 0 };
          grp.n++; grp.r += pr; grp.g += pg; grp.b += pb;
          groups.set(key, grp);
          total++;
        }
        if (total === 0) return resolve(null);

        const colours = [...groups.values()]
          .map((c) => ({ n: c.n, r: c.r / c.n, g: c.g / c.n, b: c.b / c.n }))
          .sort((a, b) => b.n - a.n);
        const sat = (c: { r: number; g: number; b: number }) => {
          const max = Math.max(c.r, c.g, c.b), min = Math.min(c.r, c.g, c.b);
          return max === 0 ? 0 : (max - min) / max;
        };
        const dist = (a: { r: number; g: number; b: number }, b: { r: number; g: number; b: number }) =>
          Math.hypot(a.r - b.r, a.g - b.g, a.b - b.b);
        const first = colours[0]!;
        // the second colour: the most common one that covers at least 6 % of the picture and is clearly different from the first and not grey;
        // failing that, a weaker difference; failing that, the first colour
        const second =
          colours.find((c) => c !== first && c.n >= total * 0.06 && dist(c, first) > 60 && sat(c) >= 0.2) ??
          colours.find((c) => c !== first && c.n >= total * 0.04 && dist(c, first) > 40) ??
          first;

        let ar = Math.round(second.r);
        let ag = Math.round(second.g);
        let ab = Math.round(second.b);

        // a little more saturated than before: pulled 12 % toward grey (version 2 pulled 25 %), so it keeps the artwork's own tone
        const mean = (ar + ag + ab) / 3;
        ar = Math.round(ar * 0.88 + mean * 0.12);
        ag = Math.round(ag * 0.88 + mean * 0.12);
        ab = Math.round(ab * 0.88 + mean * 0.12);

        // Luminance between 34 and 112: a bright poster gives a clearly lit top of the page, a dark one a deep tint
        const lum = (ar * 299 + ag * 587 + ab * 114) / 1000;
        const target = lum > 112 ? 112 : lum < 34 ? 34 : lum;
        const scale = target / Math.max(lum, 1);
        ar = Math.min(255, Math.round(ar * scale));
        ag = Math.min(255, Math.round(ag * scale));
        ab = Math.min(255, Math.round(ab * scale));

        const rgb = { r: ar, g: ag, b: ab };
        const map = load();
        map.delete(fileKey(imageUrl));
        map.set(fileKey(imageUrl), rgb);
        save();
        try { localStorage.setItem(LAST_KEY, JSON.stringify(rgb)); } catch {}
        resolve(rgb);
      } catch {
        resolve(null);
      }
    };
    img.onerror = () => resolve(null);
  });
}
