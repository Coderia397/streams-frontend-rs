// Orchestrates the low-bandwidth timeline preview strip.
// Fetches a baseline coarse map (1 frame/min) in the background and resolves targeted keyframe slices on demand.

import { AES128CBC } from './aes';
import { extractKeyframeFromTs } from './ts-demux';
import { H264Decoder, isWebCodecsSupported } from './decoder';
import {
  parseLowestVariantUrl,
  parseMediaPlaylist,
  findSegmentIndex,
  getSampleIndices,
  MediaPlaylist,
  Segment,
} from './hls-parser';

const SLICE_SIZE = 49152; // 48 KB initial range slice
const EXTENDED_SLICE_SIZE = 98304; // 96 KB fallback slice if IDR slice exceeded initial boundary
const MAX_CACHE_FRAMES = 150; // In-memory frame cache limit

export class PreviewStrip {
  private masterUrl: string;
  private playlist: MediaPlaylist | null = null;
  private keyCache = new Map<string, Uint8Array>();
  private aes = new AES128CBC();
  private decoder = new H264Decoder();

  private frames = new Map<number, ImageBitmap>();
  private accessOrder: number[] = [];
  private activeFetches = new Map<number, Promise<ImageBitmap | null>>();
  private activeAborts = new Map<number, AbortController>();

  private destroyed = false;
  private initialized = false;
  private initPromise: Promise<boolean> | null = null;

  constructor(masterUrl: string) {
    this.masterUrl = masterUrl;
  }

  public async init(): Promise<boolean> {
    if (this.initPromise) return this.initPromise;
    this.initPromise = this.doInit();
    return this.initPromise;
  }

  private async doInit(): Promise<boolean> {
    if (!isWebCodecsSupported()) return false;

    try {
      // 1. Fetch and parse master playlist
      const masterRes = await fetch(this.masterUrl, { credentials: 'omit' });
      if (!masterRes.ok) return false;
      const masterText = await masterRes.text();
      const variantUrl = parseLowestVariantUrl(masterText, this.masterUrl);

      // 2. Fetch and parse lowest-resolution variant playlist
      const variantRes = await fetch(variantUrl, { credentials: 'omit' });
      if (!variantRes.ok) return false;
      const variantText = await variantRes.text();
      this.playlist = parseMediaPlaylist(variantText, variantUrl);

      if (this.destroyed || this.playlist.segments.length === 0) return false;

      // 3. Reject fMP4 streams or unsupported DRM schemes (forces fallback to hidden video)
      if (this.playlist.hasFmp4Map || this.playlist.hasUnsupportedEncryption) {
        return false;
      }

      // 4. Verify stream viability by resolving the first keyframe (try segment 0, then segment 1 if segment 0 fails)
      let testFrame = await this.fetchKeyframeForSegment(0, 'high');
      if (!testFrame && this.playlist.segments.length > 1 && !this.destroyed) {
        testFrame = await this.fetchKeyframeForSegment(1, 'high');
      }
      if (!testFrame || this.destroyed) {
        return false;
      }

      this.initialized = true;

      // 5. Start coarse baseline scan in background (1 frame every 60 seconds)
      this.startCoarseScan();
      return true;
    } catch {
      return false;
    }
  }

  /** Quietly pre-populates a coarse thumbnail map at low network priority. */
  private async startCoarseScan(): Promise<void> {
    if (!this.playlist || this.destroyed) return;
    const sampleIndices = getSampleIndices(this.playlist.segments, 60);

    for (const idx of sampleIndices) {
      if (this.destroyed) break;
      if (!this.frames.has(idx)) {
        await this.fetchKeyframeForSegment(idx, 'low');
        // Small 50ms pause between background requests to avoid competing with main video playback
        await new Promise((r) => setTimeout(r, 50));
      }
    }
  }

  /** Calculates IV for a segment. Uses explicit IV attribute or big-endian sequenceNumber per RFC 8216. */
  private getIvForSegment(seg: Segment): Uint8Array {
    if (seg.key?.ivHex) {
      const hex = seg.key.ivHex.replace(/^0x/i, '').padStart(32, '0');
      const iv = new Uint8Array(16);
      for (let i = 0; i < 16; i++) {
        iv[i] = parseInt(hex.substr(i * 2, 2), 16);
      }
      return iv;
    }

    const iv = new Uint8Array(16);
    const view = new DataView(iv.buffer);
    view.setUint32(12, seg.sequenceNumber, false);
    return iv;
  }

  /** Fetches key bytes from cache or network. */
  private async getKeyBytes(keyUrl: string): Promise<Uint8Array | null> {
    if (this.keyCache.has(keyUrl)) {
      return this.keyCache.get(keyUrl)!;
    }
    try {
      const res = await fetch(keyUrl, { credentials: 'omit' });
      if (!res.ok) return null;
      const ab = await res.arrayBuffer();
      if (ab.byteLength !== 16) return null;
      const bytes = new Uint8Array(ab);
      this.keyCache.set(keyUrl, bytes);
      return bytes;
    } catch {
      return null;
    }
  }

  /** Decrypts an AES-128 segment slice using its active key and sequence-derived IV. */
  private async decryptSegmentSlice(slice: Uint8Array, seg: Segment): Promise<Uint8Array | null> {
    if (!seg.key) return slice;
    if (seg.key.method !== 'AES-128') return null;

    const keyBytes = await this.getKeyBytes(seg.key.keyUrl);
    if (!keyBytes) return null;

    this.aes.setKey(keyBytes);
    const iv = this.getIvForSegment(seg);
    return this.aes.decrypt(slice, iv);
  }

  /** Fetches a single segment's keyframe via byte-range slice. */
  private async fetchKeyframeForSegment(
    segIndex: number,
    priority: 'high' | 'low' = 'high'
  ): Promise<ImageBitmap | null> {
    if (this.frames.has(segIndex)) return this.frames.get(segIndex)!;
    if (this.activeFetches.has(segIndex)) return this.activeFetches.get(segIndex)!;
    if (!this.playlist || segIndex < 0 || segIndex >= this.playlist.segments.length) return null;

    const seg = this.playlist.segments[segIndex];
    if (!seg) return null;
    const abortCtrl = new AbortController();
    this.activeAborts.set(segIndex, abortCtrl);

    const promise = (async (): Promise<ImageBitmap | null> => {
      try {
        // Request opening byte range
        const slice = await this.fetchRange(seg.url, 0, SLICE_SIZE - 1, abortCtrl.signal, priority);
        if (!slice || this.destroyed) return null;

        // Decrypt if segment is encrypted
        let tsBytes: Uint8Array | null = slice;
        if (seg.key) {
          tsBytes = await this.decryptSegmentSlice(slice, seg);
          if (!tsBytes || this.destroyed) return null;
        }

        // Demux MPEG-TS
        let keyframe = extractKeyframeFromTs(tsBytes);

        // Edge case: if keyframe was cut off, extend in steps up to 256 KB (covers 720p/1080p keyframes)
        if (!keyframe && slice.length === SLICE_SIZE) {
          let currentCombined = slice;
          const extensionSteps = [98304, 131072, 262144]; // 96 KB, 128 KB, then 256 KB
          for (const targetSize of extensionSteps) {
            if (keyframe || currentCombined.length >= targetSize || this.destroyed) break;
            const ext = await this.fetchRange(seg.url, currentCombined.length, targetSize - 1, abortCtrl.signal, priority);
            if (!ext || this.destroyed) break;

            const combined = new Uint8Array(currentCombined.length + ext.length);
            combined.set(currentCombined, 0);
            combined.set(ext, currentCombined.length);
            currentCombined = combined;

            let decryptedCombined: Uint8Array | null = currentCombined;
            if (seg.key) {
              decryptedCombined = await this.decryptSegmentSlice(currentCombined, seg);
            }
            if (decryptedCombined && !this.destroyed) {
              keyframe = extractKeyframeFromTs(decryptedCombined);
            }
          }
        }

        if (!keyframe || this.destroyed) return null;

        // Decode via persistent hardware decoder instance
        const bitmap = await this.decoder.decode(keyframe);
        if (this.destroyed) {
          bitmap.close();
          return null;
        }

        // Cache result
        this.cacheFrame(segIndex, bitmap);
        return bitmap;
      } catch {
        return null;
      } finally {
        this.activeFetches.delete(segIndex);
        this.activeAborts.delete(segIndex);
      }
    })();

    this.activeFetches.set(segIndex, promise);
    return promise;
  }

  /** Performs an HTTP byte-range fetch with stream cancellation fallback if range is ignored. */
  private async fetchRange(
    url: string,
    start: number,
    end: number,
    signal: AbortSignal,
    priority: 'high' | 'low'
  ): Promise<Uint8Array | null> {
    try {
      const res = await fetch(url, {
        headers: { Range: `bytes=${start}-${end}` },
        signal,
        credentials: 'omit',
        priority: priority as any,
      });

      if (!res.ok) return null;

      // Expected HTTP 206 Partial Content
      if (res.status === 206) {
        const ab = await res.arrayBuffer();
        return new Uint8Array(ab);
      }

      // Edge case: Server ignored Range header and sent HTTP 200 (full segment)
      // Read only requested amount from stream and cancel immediately to avoid full download
      if (res.body) {
        const reader = res.body.getReader();
        const needed = end - start + 1;
        const chunks: Uint8Array[] = [];
        let received = 0;

        while (received < needed) {
          const { done, value } = await reader.read();
          if (done || !value) break;
          chunks.push(value);
          received += value.length;
        }
        try { await reader.cancel(); } catch {}

        const out = new Uint8Array(Math.min(received, needed));
        let pos = 0;
        for (const c of chunks) {
          const take = Math.min(c.length, out.length - pos);
          out.set(c.subarray(0, take), pos);
          pos += take;
          if (pos >= out.length) break;
        }
        return out;
      }

      return null;
    } catch {
      return null;
    }
  }

  private cacheFrame(segIndex: number, bitmap: ImageBitmap): void {
    if (this.frames.has(segIndex)) {
      this.frames.get(segIndex)?.close();
    }
    this.frames.set(segIndex, bitmap);
    this.accessOrder.push(segIndex);

    // Evict oldest if exceeding capacity
    while (this.frames.size > MAX_CACHE_FRAMES && this.accessOrder.length > 0) {
      const oldest = this.accessOrder.shift()!;
      if (this.frames.has(oldest)) {
        this.frames.get(oldest)?.close();
        this.frames.delete(oldest);
      }
    }
  }

  /** The kept picture of the piece covering `seconds`, or null when that piece has none yet. */
  public getExactCachedFrame(seconds: number): ImageBitmap | null {
    if (!this.playlist) return null;
    return this.frames.get(findSegmentIndex(this.playlist.segments, seconds)) ?? null;
  }

  /** Returns the nearest cached frame within maxDistanceSeconds (default 75s). */
  public getNearestCachedFrame(seconds: number, maxDistanceSeconds = 75): ImageBitmap | null {
    if (!this.playlist || this.frames.size === 0) return null;
    const targetIdx = findSegmentIndex(this.playlist.segments, seconds);
    if (this.frames.has(targetIdx)) return this.frames.get(targetIdx)!;

    let closestIdx: number | null = null;
    let minDiffSeconds = Infinity;

    for (const idx of this.frames.keys()) {
      const seg = this.playlist.segments[idx];
      if (!seg) continue;
      const segMid = seg.startTime + seg.duration / 2;
      const diff = Math.abs(segMid - seconds);
      if (diff < minDiffSeconds) {
        minDiffSeconds = diff;
        closestIdx = idx;
      }
    }

    if (closestIdx !== null && minDiffSeconds <= maxDistanceSeconds) {
      return this.frames.get(closestIdx) || null;
    }

    return null;
  }

  /** Returns the segment index covering the requested timestamp. */
  public getSegmentIndex(seconds: number): number {
    if (!this.playlist) return 0;
    return findSegmentIndex(this.playlist.segments, seconds);
  }

  /** Resolves the exact keyframe for the hovered seconds. */
  public async getExactFrame(seconds: number): Promise<ImageBitmap | null> {
    if (!this.initialized) {
      const ok = await this.init();
      if (!ok) return null;
    }
    if (!this.playlist) return null;

    const targetIdx = findSegmentIndex(this.playlist.segments, seconds);
    return this.fetchKeyframeForSegment(targetIdx, 'high');
  }

  public destroy(): void {
    this.destroyed = true;
    for (const ctrl of this.activeAborts.values()) {
      try { ctrl.abort(); } catch {}
    }
    this.activeAborts.clear();
    this.activeFetches.clear();

    for (const bmp of this.frames.values()) {
      try { bmp.close(); } catch {}
    }
    this.frames.clear();
    this.accessOrder = [];
    this.decoder.close();
  }
}
