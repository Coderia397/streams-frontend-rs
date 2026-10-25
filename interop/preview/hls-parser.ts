// Parser for master and variant HLS M3U8 playlists.
// Identifies the lowest resolution video variant, segment URLs, durations, sequence numbers, and active encryption keys.

export interface SegmentKeyInfo {
  method: string;
  keyUrl: string;
  ivHex: string | null;
}

export interface Segment {
  index: number;
  url: string;
  duration: number;
  startTime: number;
  sequenceNumber: number;
  key: SegmentKeyInfo | null;
}

export interface MediaPlaylist {
  mediaSequence: number;
  segments: Segment[];
  totalDuration: number;
  hasFmp4Map: boolean;
  hasUnsupportedEncryption: boolean;
}

/** Parses a master M3U8 playlist and returns the URL of the lowest bandwidth video variant. */
export function parseLowestVariantUrl(masterText: string, masterUrl: string): string {
  const lines = masterText.split(/\r?\n/);
  let lowestBw = Infinity;
  let lowestUrl: string | null = null;

  for (let i = 0; i < lines.length; i++) {
    const line = (lines[i] ?? '').trim();
    if (line.startsWith('#EXT-X-STREAM-INF:')) {
      const bwMatch = line.match(/BANDWIDTH=(\d+)/i);
      const bw = bwMatch && bwMatch[1] ? parseInt(bwMatch[1], 10) : 0;
      let nextLine = '';
      for (let j = i + 1; j < lines.length; j++) {
        const candidate = (lines[j] ?? '').trim();
        if (candidate && !candidate.startsWith('#')) {
          nextLine = candidate;
          break;
        }
      }
      if (nextLine && bw < lowestBw) {
        lowestBw = bw;
        lowestUrl = resolveUrl(nextLine, masterUrl);
      }
    }
  }

  return lowestUrl || masterUrl;
}

/** Parses a media playlist into its constituent segments, timing, sequence numbers, and encryption keys. */
export function parseMediaPlaylist(mediaText: string, mediaUrl: string): MediaPlaylist {
  const lines = mediaText.split(/\r?\n/);
  const segments: Segment[] = [];
  let mediaSequence = 0;
  let currentStart = 0;
  let activeKey: SegmentKeyInfo | null = null;
  let hasFmp4Map = false;
  let hasUnsupportedEncryption = false;
  let pendingDuration = 0;

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line) continue;

    if (line.startsWith('#EXT-X-MEDIA-SEQUENCE:')) {
      const seqMatch = line.match(/#EXT-X-MEDIA-SEQUENCE:\s*(\d+)/i);
      if (seqMatch && seqMatch[1]) {
        mediaSequence = parseInt(seqMatch[1], 10);
      }
    } else if (line.startsWith('#EXT-X-MAP:')) {
      hasFmp4Map = true;
    } else if (line.startsWith('#EXT-X-KEY:')) {
      const methodMatch = line.match(/METHOD=([^,\s]+)/i);
      const method = (methodMatch && methodMatch[1] ? methodMatch[1] : 'NONE').toUpperCase();

      if (method === 'NONE') {
        activeKey = null;
      } else if (method === 'AES-128') {
        const uriMatch = line.match(/URI="([^"]+)"/i);
        const ivMatch = line.match(/IV=0x([0-9A-Fa-f]+)/i);
        if (uriMatch && uriMatch[1]) {
          activeKey = {
            method: 'AES-128',
            keyUrl: resolveUrl(uriMatch[1], mediaUrl),
            ivHex: ivMatch && ivMatch[1] ? ivMatch[1] : null,
          };
        }
      } else {
        hasUnsupportedEncryption = true;
        activeKey = {
          method,
          keyUrl: '',
          ivHex: null,
        };
      }
    } else if (line.startsWith('#EXTINF:')) {
      const durMatch = line.match(/#EXTINF:\s*([\d.]+)/i);
      pendingDuration = durMatch && durMatch[1] ? parseFloat(durMatch[1]) : 0;
    } else if (!line.startsWith('#')) {
      const segIndex = segments.length;
      const segUrl = resolveUrl(line, mediaUrl);
      if (/\.(mp4|m4s)($|\?)/i.test(segUrl)) {
        hasFmp4Map = true;
      }
      segments.push({
        index: segIndex,
        url: segUrl,
        duration: pendingDuration,
        startTime: currentStart,
        sequenceNumber: mediaSequence + segIndex,
        key: activeKey ? { ...activeKey } : null,
      });
      currentStart += pendingDuration;
      pendingDuration = 0;
    }
  }

  return {
    mediaSequence,
    segments,
    totalDuration: currentStart,
    hasFmp4Map,
    hasUnsupportedEncryption,
  };
}

/** Resolves relative URLs against a base URL. */
export function resolveUrl(relativeOrAbsolute: string, baseUrl: string): string {
  try {
    return new URL(relativeOrAbsolute, baseUrl).href;
  } catch {
    return relativeOrAbsolute;
  }
}

/** Finds the segment index covering the requested timestamp. */
export function findSegmentIndex(segments: Segment[], seconds: number): number {
  if (segments.length === 0) return 0;
  if (seconds <= 0) return 0;

  let low = 0;
  let high = segments.length - 1;

  while (low <= high) {
    const mid = (low + high) >>> 1;
    const seg = segments[mid];
    if (!seg) break;
    if (seconds < seg.startTime) {
      high = mid - 1;
    } else if (seconds >= seg.startTime + seg.duration) {
      low = mid + 1;
    } else {
      return mid;
    }
  }

  return Math.min(Math.max(0, high), segments.length - 1);
}

/** Returns sample segment indices spaced roughly by intervalSeconds (for baseline coarse maps). */
export function getSampleIndices(segments: Segment[], intervalSeconds: number): number[] {
  if (segments.length === 0) return [];
  const indices: number[] = [0];
  let nextTargetTime = intervalSeconds;

  for (let i = 1; i < segments.length; i++) {
    const seg = segments[i];
    if (seg && seg.startTime >= nextTargetTime) {
      indices.push(i);
      nextTargetTime = seg.startTime + intervalSeconds;
    }
  }

  return indices;
}
