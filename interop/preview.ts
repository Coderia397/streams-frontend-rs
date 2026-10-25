// Timeline scrubber preview engine (src/components/player/timeline_scrubber.rs).
// Uses high-efficiency byte-range keyframe slicing via WebCodecs on HLS streams,
// with graceful fallback to video element seeking on plain media files or unsupported streams.

import { PreviewStrip } from './preview/strip';
import { isWebCodecsSupported } from './preview/decoder';
import { loadPlayer } from './hls';

const STEP = 10;
const KEEP = 120;

type Hlsish = { destroy(): void; startLoad?(at?: number): void };

type FallbackPreview = {
  video: HTMLVideoElement;
  hls: Hlsish | null;
  loaded: boolean;
  frames: Map<number, ImageBitmap>;
  wanted: number | null;
  busy: number | null;
  canvas: string;
  dead: boolean;
};

interface PreviewSession {
  url: string;
  strip: PreviewStrip | null;
  fallback: FallbackPreview | null;
  activeCanvas: string;
  debounceTimer: any;
  lastSeconds: number;
  dead: boolean;
}

const sessions = new Map<string, PreviewSession>();
let sessionCounter = 0;

function appleHls(video: HTMLVideoElement): boolean {
  return navigator.vendor === 'Apple Computer, Inc.' && video.canPlayType('application/vnd.apple.mpegurl') !== '';
}

function createFallbackPreview(url: string): FallbackPreview {
  const video = document.createElement('video');
  video.muted = true;
  video.playsInline = true;
  video.preload = 'auto';

  const fb: FallbackPreview = {
    video,
    hls: null,
    loaded: false,
    frames: new Map(),
    wanted: null,
    busy: null,
    canvas: '',
    dead: false,
  };

  if (/\.(mp4|m4v|webm|mkv|mov)(\?.*)?$/i.test(url) || appleHls(video)) {
    video.src = url;
    fb.loaded = true;
  } else {
    loadPlayer()
      .then((m) => {
        if (fb.dead) return;
        const hls = m.attachPreview(video, url);
        if (hls) {
          fb.hls = hls;
        } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
          video.src = url;
          fb.loaded = true;
        } else {
          fb.dead = true;
        }
        if (fb.wanted !== null) fetchNextFallback(fb);
      })
      .catch(() => {
        fb.dead = true;
      });
  }

  return fb;
}

/** Opens a high-efficiency preview source for `url`. `isHls` is supplied directly from Rust. */
export function previewOpen(url: string, isHls: boolean): string {
  const handle = 'pv_' + (++sessionCounter);
  const session: PreviewSession = {
    url,
    strip: null,
    fallback: null,
    activeCanvas: '',
    debounceTimer: null,
    lastSeconds: -1,
    dead: false,
  };

  if (isHls && isWebCodecsSupported()) {
    const strip = new PreviewStrip(url);
    session.strip = strip;
    strip
      .init()
      .then((ok) => {
        if (!ok && !session.dead) {
          // Fall back gracefully to hidden video if WebCodecs, fMP4, or DRM cannot be handled
          session.strip?.destroy();
          session.strip = null;
          session.fallback = createFallbackPreview(url);
          if (session.lastSeconds >= 0 && session.activeCanvas) {
            previewShow(handle, session.lastSeconds, session.activeCanvas);
          }
        }
      })
      .catch(() => {
        if (!session.dead) {
          session.strip?.destroy();
          session.strip = null;
          session.fallback = createFallbackPreview(url);
        }
      });
  } else {
    session.fallback = createFallbackPreview(url);
  }

  sessions.set(handle, session);
  return handle;
}

/** Shows the picture corresponding to `seconds` on canvas `canvasId`. */
export function previewShow(handle: string, seconds: number, canvasId: string): void {
  const session = sessions.get(handle);
  if (!session || session.dead) return;

  session.activeCanvas = canvasId;
  session.lastSeconds = seconds;

  if (session.strip) {
    // 1. Immediately draw the nearest cached baseline frame within 75s (or hide canvas if none nearby)
    const nearest = session.strip.getNearestCachedFrame(seconds, 75);
    if (nearest) {
      drawCanvas(canvasId, nearest, 0.75); // Dimmed slightly to indicate seeking
    } else {
      hideCanvas(canvasId);
    }

    // 2. Debounce exact keyframe fetch by 75ms to avoid spamming network on rapid pointer sweeps
    if (session.debounceTimer) clearTimeout(session.debounceTimer);
    session.debounceTimer = setTimeout(() => {
      if (session.dead || session.lastSeconds !== seconds) return;

      session.strip?.getExactFrame(seconds).then((exactFrame) => {
        if (!exactFrame || session.dead || !session.strip) return;
        // Verify user is still hovering within the same segment (evaluated after playlist is loaded)
        if (session.strip.getSegmentIndex(session.lastSeconds) === session.strip.getSegmentIndex(seconds)) {
          drawCanvas(canvasId, exactFrame, 1.0); // Full crisp opacity
        }
      });
    }, 75);
    return;
  }

  if (session.fallback) {
    const fb = session.fallback;
    if (fb.dead) return;
    const bucket = Math.max(0, Math.floor(seconds / STEP));
    fb.canvas = canvasId;
    fb.wanted = bucket;
    const kept = fb.frames.get(bucket);
    if (kept) {
      drawCanvas(canvasId, kept, 1.0);
      return;
    }
    // Hide canvas while seeking to new bucket to prevent showing previous moment's frame
    hideCanvas(canvasId);
    fetchNextFallback(fb);
  }
}

/** Closes the preview source and frees allocated bitmap resources. */
export function previewClose(handle: string): void {
  const session = sessions.get(handle);
  if (!session) return;

  session.dead = true;

  if (session.debounceTimer) {
    clearTimeout(session.debounceTimer);
    session.debounceTimer = null;
  }

  if (session.strip) {
    session.strip.destroy();
    session.strip = null;
  }

  if (session.fallback) {
    const fb = session.fallback;
    fb.dead = true;
    try {
      fb.hls?.destroy();
    } catch {}
    fb.video.removeAttribute('src');
    fb.video.load();
    fb.frames.forEach((f) => {
      try {
        f.close();
      } catch {}
    });
    fb.frames.clear();
    session.fallback = null;
  }

  sessions.delete(handle);
}

function drawCanvas(canvasId: string, frame: ImageBitmap, opacity = 1.0): void {
  const c = document.getElementById(canvasId) as HTMLCanvasElement | null;
  const ctx = c?.getContext('2d');
  if (!c || !ctx) return;

  ctx.clearRect(0, 0, c.width, c.height);
  ctx.drawImage(frame, 0, 0, c.width, c.height);
  c.style.opacity = opacity < 1.0 ? String(opacity) : '1.0';
  c.style.visibility = 'visible';
}

function hideCanvas(canvasId: string): void {
  const c = document.getElementById(canvasId) as HTMLCanvasElement | null;
  if (c) {
    c.style.visibility = 'hidden';
  }
}

function fetchNextFallback(fb: FallbackPreview): void {
  if (fb.dead || fb.busy !== null || fb.wanted === null || fb.frames.has(fb.wanted)) return;
  if (!fb.loaded && !fb.hls) return;

  const bucket = fb.wanted;
  fb.busy = bucket;
  const at = bucket * STEP;
  const video = fb.video;
  let done = false;

  const finish = (): void => {
    if (done) return;
    done = true;
    video.removeEventListener('seeked', onSeeked);
    video.removeEventListener('loadeddata', onSeeked);
    clearTimeout(timer);
    fb.busy = null;
    fetchNextFallback(fb);
  };

  const onSeeked = (): void => {
    if (fb.dead) return finish();
    if (video.readyState < HTMLMediaElement.HAVE_CURRENT_DATA) {
      video.addEventListener('loadeddata', onSeeked, { once: true });
      return;
    }
    createImageBitmap(video, { resizeWidth: 256, resizeHeight: 144, resizeQuality: 'medium' })
      .then((frame) => {
        if (fb.dead) {
          try {
            frame.close();
          } catch {}
          return;
        }
        fb.frames.set(bucket, frame);
        if (fb.frames.size > KEEP) {
          const oldest = fb.frames.keys().next().value;
          if (oldest !== undefined) {
            try {
              fb.frames.get(oldest)?.close();
            } catch {}
            fb.frames.delete(oldest);
          }
        }
        if (fb.wanted === bucket && fb.canvas) {
          drawCanvas(fb.canvas, frame, 1.0);
        }
      })
      .catch(() => {})
      .finally(finish);
  };

  const timer = setTimeout(finish, 4000);
  video.addEventListener('seeked', onSeeked);
  if (fb.hls && !fb.loaded) {
    fb.loaded = true;
    fb.hls.startLoad?.(at);
  }
  video.currentTime = at;
}
