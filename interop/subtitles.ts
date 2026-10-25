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

// ── does the text match the language it was listed as ──────────────────────────────────────────────────────────────────
// Measured on 2026-10-25: a file listed as English showed Arabic script. The script of the letters must be the one the language is
// written in, and for languages written in Latin letters their common words must outnumber another language's.

type Script = 'latin' | 'arabic' | 'cyrillic' | 'greek' | 'hebrew' | 'han' | 'kana' | 'hangul' | 'thai' | 'devanagari' | 'other';

function scriptOf(ch: number): Script | null {
  if ((ch >= 0x41 && ch <= 0x5a) || (ch >= 0x61 && ch <= 0x7a) || (ch >= 0xc0 && ch <= 0x24f) || (ch >= 0x1e00 && ch <= 0x1eff)) return 'latin';
  if ((ch >= 0x600 && ch <= 0x6ff) || (ch >= 0x750 && ch <= 0x77f) || (ch >= 0xfb50 && ch <= 0xfdff) || (ch >= 0xfe70 && ch <= 0xfeff)) return 'arabic';
  if (ch >= 0x400 && ch <= 0x52f) return 'cyrillic';
  if (ch >= 0x370 && ch <= 0x3ff) return 'greek';
  if (ch >= 0x590 && ch <= 0x5ff) return 'hebrew';
  if ((ch >= 0x4e00 && ch <= 0x9fff) || (ch >= 0x3400 && ch <= 0x4dbf)) return 'han';
  if (ch >= 0x3040 && ch <= 0x30ff) return 'kana';
  if ((ch >= 0xac00 && ch <= 0xd7af) || (ch >= 0x1100 && ch <= 0x11ff)) return 'hangul';
  if (ch >= 0xe00 && ch <= 0xe7f) return 'thai';
  if (ch >= 0x900 && ch <= 0x97f) return 'devanagari';
  if (ch > 0x2ff && /\p{L}/u.test(String.fromCodePoint(ch))) return 'other';
  return null;
}

// the scripts a language's subtitles are written in (anything not listed: Latin)
const SCRIPTS: Record<string, Script[]> = {
  ar: ['arabic'], fa: ['arabic'], ur: ['arabic'], ps: ['arabic'], ku: ['arabic', 'latin'],
  ru: ['cyrillic'], uk: ['cyrillic'], bg: ['cyrillic'], mk: ['cyrillic'], be: ['cyrillic'], kk: ['cyrillic'], mn: ['cyrillic'], sr: ['cyrillic', 'latin'],
  el: ['greek'], he: ['hebrew'], yi: ['hebrew'], zh: ['han'], ja: ['han', 'kana'], ko: ['hangul', 'han'], th: ['thai'], hi: ['devanagari'], mr: ['devanagari'], ne: ['devanagari'],
  bn: ['other'], ta: ['other'], te: ['other'], ml: ['other'], kn: ['other'], gu: ['other'], pa: ['other'], si: ['other'], ka: ['other'], hy: ['other'], km: ['other'], my: ['other'], lo: ['other'], am: ['other'],
};

// common words of the languages written in Latin letters that subtitles most often come in
const WORDS: Record<string, string[]> = {
  en: ['the', 'you', 'and', 'to', 'is', 'that', 'it', 'what', 'this', 'of', 'are', 'have', 'don\'t', 'i\'m', 'we', 'with', 'for', 'not'],
  es: ['que', 'de', 'no', 'la', 'el', 'es', 'en', 'lo', 'los', 'por', 'qué', 'una', 'con', 'para', 'está', 'pero', 'esto'],
  fr: ['je', 'de', 'est', 'pas', 'le', 'la', 'vous', 'que', 'tu', 'les', 'et', 'un', 'il', 'ce', 'une', 'qui', 'nous', 'c\'est'],
  de: ['ich', 'die', 'der', 'und', 'nicht', 'du', 'das', 'ist', 'sie', 'es', 'wir', 'zu', 'ein', 'was', 'mit', 'mir', 'auf'],
  it: ['non', 'che', 'di', 'il', 'è', 'la', 'un', 'per', 'mi', 'ti', 'sono', 'cosa', 'questo', 'lo', 'ho', 'ci', 'come'],
  pt: ['que', 'não', 'de', 'o', 'é', 'você', 'a', 'eu', 'um', 'isso', 'para', 'se', 'com', 'uma', 'está', 'do', 'mas'],
  nl: ['ik', 'je', 'het', 'de', 'een', 'niet', 'dat', 'is', 'van', 'en', 'wat', 'we', 'zijn', 'hij', 'maar', 'heb'],
  pl: ['nie', 'to', 'się', 'jest', 'że', 'co', 'na', 'jak', 'tak', 'mi', 'ja', 'ale', 'mnie', 'tu', 'już', 'czy'],
  tr: ['bir', 'bu', 've', 'ne', 'ben', 'sen', 'mi', 'değil', 'için', 'çok', 'var', 'da', 'de', 'bana', 'o', 'şey'],
  sv: ['jag', 'det', 'du', 'är', 'inte', 'att', 'en', 'och', 'har', 'vi', 'på', 'som', 'med', 'han', 'vad'],
  id: ['yang', 'tidak', 'aku', 'kau', 'ini', 'itu', 'dan', 'apa', 'di', 'kita', 'saya', 'ada', 'dia', 'akan'],
  ro: ['nu', 'să', 'este', 'de', 'și', 'ce', 'în', 'am', 'pe', 'mai', 'sunt', 'eu', 'asta', 'cu'],
};

export function matchesLanguage(cues: Cue[], tag: string): boolean {
  const lang = (tag || '').toLowerCase().split('-')[0] ?? '';
  if (!lang) return true;
  const sample = cues.slice(0, 400).map((c) => c.t.replace(/<[^>]+>/g, '')).join(' ');
  const counts: Partial<Record<Script, number>> = {};
  let letters = 0;
  for (const ch of sample) {
    const sc = scriptOf(ch.codePointAt(0) ?? 0);
    if (sc) { counts[sc] = (counts[sc] ?? 0) + 1; letters++; }
  }
  if (letters < 50) return true;
  const expected = SCRIPTS[lang] ?? ['latin'];
  const share = expected.reduce((n, sc) => n + (counts[sc] ?? 0), 0) / letters;
  if (share < 0.6) return false;
  // languages that share a script, told apart by the letters only one of them uses
  const own = (re: RegExp) => (sample.match(re) ?? []).length / letters;
  const persian = own(/[پچژگ]/g), ukrainian = own(/[іїєґІЇЄҐ]/g), russianOnly = own(/[ыэъЫЭЪ]/g);
  if (lang === 'ar' && persian > 0.01) return false;
  if (lang === 'fa' && persian < 0.002) return false;
  if (lang === 'uk' && (ukrainian < 0.005 || russianOnly > 0.005)) return false;
  if (lang === 'ru' && ukrainian > 0.005) return false;
  // Latin letters: the listed language's common words against the others'
  if (expected[0] === 'latin' && WORDS[lang]) {
    const words = sample.toLowerCase().split(/[^\p{L}']+/u);
    const score = (l: string) => { const set = new Set(WORDS[l]); let n = 0; for (const w of words) if (set.has(w)) n++; return n; };
    const own = score(lang);
    let best = 0;
    for (const l of Object.keys(WORDS)) if (l !== lang) best = Math.max(best, score(l));
    if (best > 40 && own * 2 < best) return false;
  }
  return true;
}

// lines that advertise a site or a team rather than say anything in the film
const ADS = /(opensubtitles|subscene|subdl\.|yts\.|yify|addic7ed|podnapisi|subtitlecat|advertise your product|subtitles? (by|ripped|synced|downloaded)|sync(ed)? (and|&) correct(ed|ions)|www\.|https?:\/\/|\.(com|net|org|tv|me|io)\b)/i;

// One candidate list in, the first file that reads as real subtitles in the listed language out: `{"url": ..., "index": n, "cues": [...]}`,
// or "" when none does. `candidates` is a JSON array of {url, lang}; `duration` (seconds, 0 when not known yet) rules out a file made
// for another cut: one that runs well past the end, or stops long before it.
export async function subtitleLoad(candidates: string, duration: number): Promise<string> {
  let list: { url: string; lang: string }[] = [];
  try { list = JSON.parse(candidates); } catch { return ''; }
  for (let i = 0; i < list.length; i++) {
    const c = list[i];
    if (!c) continue;
    try {
      const res = await fetch(c.url, { credentials: 'omit' });
      if (!res.ok) continue;
      const buf = await res.arrayBuffer();
      if (buf.byteLength > MAX_BYTES) continue;
      const cues = parseCues(decode(buf, c.lang || '')).filter((cue) => !ADS.test(cue.t));
      if (cues.length < MIN_CUES) continue;
      if (!matchesLanguage(cues, c.lang)) continue;
      const end = cues[cues.length - 1]?.e ?? 0;
      if (duration > 600 && (end > duration + 180 || end < duration * 0.5)) continue;
      return JSON.stringify({ url: c.url, index: i, cues });
    } catch {
      // a file that cannot be fetched or read: the next one
    }
  }
  return '';
}
