// Pictures for the website timeline (src/components/player/timeline_scrubber.rs), as Netflix shows over its bar (note 126). There is no
// picture sheet for our streams, so a second copy of the stream, muted and never shown, jumps to the moment under the pointer at the
// stream's lowest quality (hls.js; a plain file or Apple's own HLS as is), and that frame is drawn into the timeline's canvas. Frames are
// kept per STEP seconds, so going back over the bar costs nothing; only the newest moment asked for is fetched while another loads. The
// canvas shows only while it holds the picture of the moment under the pointer: otherwise the time pill shows alone, as on Netflix.
import { loadPlayer } from './hls';

const STEP = 10;
// frames kept (at 256 x 144 each is about 150 KB; 120 is 20 minutes of the bar in 10 s steps)
const KEEP = 120;

type Hlsish = { destroy(): void; startLoad?(at?: number): void };
type Preview = {
  video: HTMLVideoElement;
  hls: Hlsish | null;
  loaded: boolean;
  frames: Map<number, ImageBitmap>;
  wanted: number | null;
  busy: number | null;
  canvas: string;
  dead: boolean;
};

const previews = new Map<string, Preview>();
let counter = 0;

function appleHls(video: HTMLVideoElement): boolean {
  return navigator.vendor === 'Apple Computer, Inc.' && video.canPlayType('application/vnd.apple.mpegurl') !== '';
}

/** Opens a picture source for `url`; returns its handle ("" when there is none). */
export function previewOpen(url: string): string {
  const video = document.createElement('video');
  video.muted = true;
  video.playsInline = true;
  video.preload = 'auto';
  const handle = 'pv_' + (++counter);
  const p: Preview = { video, hls: null, loaded: false, frames: new Map(), wanted: null, busy: null, canvas: '', dead: false };
  previews.set(handle, p);
  if (/\.(mp4|m4v|webm|mkv|mov)(\?.*)?$/i.test(url) || appleHls(video)) {
    video.src = url;
    p.loaded = true;
    return handle;
  }
  loadPlayer()
    .then((m) => {
      if (p.dead) return;
      const hls = m.attachPreview(video, url);
      if (hls) p.hls = hls;
      else if (video.canPlayType('application/vnd.apple.mpegurl')) { video.src = url; p.loaded = true; }
      else p.dead = true;
      if (p.wanted !== null) fetchNext(p);
    })
    .catch(() => { p.dead = true; });
  return handle;
}

/** Shows the picture of `seconds` in the canvas `canvasId`: at once when it is kept, else when it has loaded (if the pointer is still
 *  there). The canvas is hidden meanwhile. */
export function previewShow(handle: string, seconds: number, canvasId: string): void {
  const p = previews.get(handle);
  if (!p || p.dead) return;
  const bucket = Math.max(0, Math.floor(seconds / STEP));
  p.canvas = canvasId;
  p.wanted = bucket;
  const kept = p.frames.get(bucket);
  if (kept) { draw(p, kept); return; }
  hide(p);
  fetchNext(p);
}

/** Closes the picture source and frees its frames. */
export function previewClose(handle: string): void {
  const p = previews.get(handle);
  if (!p) return;
  p.dead = true;
  p.hls?.destroy();
  p.video.removeAttribute('src');
  p.video.load();
  p.frames.forEach((f) => f.close());
  p.frames.clear();
  previews.delete(handle);
}

function canvasOf(p: Preview): HTMLCanvasElement | null {
  return p.canvas ? (document.getElementById(p.canvas) as HTMLCanvasElement | null) : null;
}

function hide(p: Preview): void {
  const c = canvasOf(p);
  if (c) c.style.visibility = 'hidden';
}

function draw(p: Preview, frame: ImageBitmap): void {
  const c = canvasOf(p);
  const ctx = c?.getContext('2d');
  if (!c || !ctx) return;
  ctx.drawImage(frame, 0, 0, c.width, c.height);
  c.style.visibility = 'visible';
}

/** Loads the newest moment asked for, one at a time. */
function fetchNext(p: Preview): void {
  if (p.dead || p.busy !== null || p.wanted === null || p.frames.has(p.wanted)) return;
  if (!p.loaded && !p.hls) return; // hls.js still arriving: fetched once it is attached
  const bucket = p.wanted;
  p.busy = bucket;
  const at = bucket * STEP;
  const video = p.video;
  let done = false;
  const finish = (): void => {
    if (done) return;
    done = true;
    video.removeEventListener('seeked', onSeeked);
    clearTimeout(timer);
    p.busy = null;
    fetchNext(p);
  };
  const onSeeked = (): void => {
    if (p.dead) return finish();
    createImageBitmap(video, { resizeWidth: 256, resizeHeight: 144, resizeQuality: 'medium' })
      .then((frame) => {
        if (p.dead) { frame.close(); return; }
        p.frames.set(bucket, frame);
        if (p.frames.size > KEEP) {
          const oldest = p.frames.keys().next().value;
          if (oldest !== undefined) { p.frames.get(oldest)?.close(); p.frames.delete(oldest); }
        }
        if (p.wanted === bucket) draw(p, frame);
      })
      .catch(() => {})
      .finally(finish);
  };
  // a moment that never loads (a refused piece, a slow host) does not hold the others back
  const timer = setTimeout(finish, 6000);
  video.addEventListener('seeked', onSeeked);
  if (p.hls && !p.loaded) {
    p.loaded = true;
    p.hls.startLoad?.(at);
  }
  video.currentTime = at;
}
