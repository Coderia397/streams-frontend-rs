// Subtitle files for the player (src/components/player/subtitles.rs): download, decode, read the cues and check they are real.
//
// The files come from SubDL (through the resolver's search) and from the stream's own provider. Measured on 2026-10-25: most are
// SRT in UTF-8, but some are in the language's older code page (Western, Arabic, Cyrillic ...), some in UTF-16, and some are not
// subtitles at all (a 101 byte note pointing at a download site). Each candidate is tried in order until one holds real cues.

export interface Cue {
  s: number; // start, seconds
  e: number; // end, seconds
  t: string; // text, lines joined by \n; <i>, <b> and <u> kept, every other tag removed
}

// fewer cues than this is not a subtitle file for a film or an episode
const MIN_CUES = 10;
const MAX_BYTES = 3_000_000;

// the older code page a subtitle file in this language is usually written in, when it is not UTF-8
const LEGACY: Record<string, string> = {
  AR: 'windows-1256', FA: 'windows-1256', UR: 'windows-1256',
  RU: 'windows-1251', UK: 'windows-1251', BG: 'windows-1251', SR: 'windows-1251', MK: 'windows-1251', BE: 'windows-1251',
  EL: 'windows-1253', TR: 'windows-1254', HE: 'windows-1255', VI: 'windows-1258', TH: 'windows-874',
  PL: 'windows-1250', CS: 'windows-1250', SK: 'windows-1250', HU: 'windows-1250', RO: 'windows-1250', HR: 'windows-1250', SL: 'windows-1250', BS: 'windows-1250', SQ: 'windows-1250',
  ET: 'windows-1257', LV: 'windows-1257', LT: 'windows-1257',
  ZH: 'gb18030', JA: 'shift_jis', KO: 'euc-kr',
};

function decode(buf: ArrayBuffer, lang: string): string {
  const b = new Uint8Array(buf);
  if (b.length >= 2 && b[0] === 0xff && b[1] === 0xfe) return new TextDecoder('utf-16le').decode(b);
  if (b.length >= 2 && b[0] === 0xfe && b[1] === 0xff) return new TextDecoder('utf-16be').decode(b);
  // UTF-16 without a mark: every other byte of plain text is zero
  if (b.length > 64) {
    let evenZero = 0, oddZero = 0;
    for (let i = 0; i < 64; i += 2) { if (b[i] === 0) evenZero++; if (b[i + 1] === 0) oddZero++; }
    if (oddZero > 24) return new TextDecoder('utf-16le').decode(b);
    if (evenZero > 24) return new TextDecoder('utf-16be').decode(b);
  }
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(b);
  } catch {
    return new TextDecoder(LEGACY[lang.toUpperCase()] ?? 'windows-1252').decode(b);
  }
}

function seconds(stamp: string): number {
  // 00:01:02,345 or 01:02.345 (WebVTT allows the hours to be left out)
  const m = /(?:(\d+):)?(\d{1,2}):(\d{2})[,.](\d{1,3})/.exec(stamp);
  if (!m) return NaN;
  return (Number(m[1] ?? 0) * 3600) + Number(m[2]) * 60 + Number(m[3]) + Number((m[4] ?? '0').padEnd(3, '0')) / 1000;
}

function clean(text: string): string {
  return text
    .replace(/\{\\[^}]*\}/g, '') // ASS style overrides some SRT files carry ({\an8})
    .replace(/<(?!\/?(i|b|u)>)[^>]*>/gi, '') // only italic, bold and underline survive
    .replace(/&nbsp;/g, ' ')
    .trim();
}

export function parseCues(text: string): Cue[] {
  const cues: Cue[] = [];
  const blocks = text.replace(/\r\n?/g, '\n').replace(/^﻿/, '').split(/\n{2,}/);
  for (const block of blocks) {
    const lines = block.split('\n');
    const at = lines.findIndex((l) => l.includes('-->'));
    if (at < 0) continue;
    const [from = '', to = ''] = (lines[at] ?? '').split('-->');
    const s = seconds(from), e = seconds(to);
    const t = clean(lines.slice(at + 1).join('\n'));
    if (Number.isFinite(s) && Number.isFinite(e) && e > s && t) cues.push({ s, e, t });
  }
  cues.sort((a, b) => a.s - b.s);
  return cues;
}

// One candidate list in, the first file that reads as real subtitles out: `{"url": ..., "cues": [...]}`, or "" when none does.
// `candidates` is a JSON array of {url, lang}.
export async function subtitleLoad(candidates: string): Promise<string> {
  let list: { url: string; lang: string }[] = [];
  try { list = JSON.parse(candidates); } catch { return ''; }
  for (const c of list) {
    try {
      const res = await fetch(c.url, { credentials: 'omit' });
      if (!res.ok) continue;
      const buf = await res.arrayBuffer();
      if (buf.byteLength > MAX_BYTES) continue;
      const cues = parseCues(decode(buf, c.lang || ''));
      if (cues.length >= MIN_CUES) return JSON.stringify({ url: c.url, cues });
    } catch {
      // a file that cannot be fetched or read: the next one
    }
  }
  return '';
}
