// The video bridge Rust binds to (src/components/hls_adapter.rs). Plain video files play in the element itself; HLS streams use hls.js,
// which is loaded the first time a stream plays (interop/hls-player.ts, its own file), not with every page.

export interface HlsInstance {
  destroy(): void;
  loadSource(url: string): void;
  attachMedia(video: HTMLVideoElement): void;
  audioTrack: number;
  currentLevel: number;
  media?: HTMLMediaElement | null;
  _native?: boolean;
  _video?: HTMLVideoElement;
}

const instances = new Map<string, HlsInstance>();
let counter = 0;

type Player = typeof import('./hls-player');
let player: Promise<Player> | null = null;

/** hls.js, fetched once. The address is made at run time so the bundler leaves the library out of this file. */
export function loadPlayer(): Promise<Player> {
  const src = new URL('/hls-player.js', location.origin).href;
  return (player ??= import(/* @vite-ignore */ src) as Promise<Player>);
}

/** Apple's browsers (Safari on a Mac, every browser on an iPhone or iPad) have HLS built in: the stream is read and decoded by the
 *  system, with no hls.js work on the main processor, and Picture in Picture, AirPlay and the lock screen's controls come with it.
 *  They play streams themselves; other browsers use hls.js. */
function appleHls(video: HTMLVideoElement): boolean {
  return navigator.vendor === 'Apple Computer, Inc.' && video.canPlayType('application/vnd.apple.mpegurl') !== '';
}

/** hls.js fetched ahead, while the player looks for the stream's address (both take seconds on a slow connection; one after the
 *  other they added up). Not on Apple's browsers, which do not use it. */
export function hlsWarm(): void {
  const probe = document.createElement('video');
  if (!appleHls(probe)) loadPlayer().catch(() => {});
}

/** The audio tracks of a stream the browser plays itself (Safari lists an HLS stream's audio renditions on the element). */
type NativeTrack = { label: string; language: string; enabled: boolean };
function nativeTracks(video?: HTMLVideoElement): NativeTrack[] {
  const list = (video as unknown as { audioTracks?: ArrayLike<NativeTrack> } | undefined)?.audioTracks;
  return list ? Array.from(list) : [];
}

/** The browser plays `url` in `video` by itself (a plain file, or HLS where the browser has it built in, as Safari). */
function native(video: HTMLVideoElement, url: string): HlsInstance {
  video.src = url;
  return {
    destroy: () => { video.src = ""; video.removeAttribute("src"); video.load(); },
    loadSource: (u: string) => { video.src = u; },
    attachMedia: () => {},
    audioTrack: 0,
    currentLevel: -1,
    _native: true,
    _video: video,
  };
}

export function hlsAttach(videoId: string, streamUrl: string): string {
  const video = document.getElementById(videoId) as HTMLVideoElement | null;
  if (!video) return "";
  const handle = "hls_" + (++counter);

  if (/\.(mp4|m4v|webm|mkv|mov)(\?.*)?$/i.test(streamUrl) || appleHls(video)) {
    instances.set(handle, native(video, streamUrl));
    return handle;
  }

  // Until hls.js has arrived the handle stands for the video element (seek and time read it); an audio or quality choice made
  // meanwhile is kept and applied when the stream starts.
  const waiting: HlsInstance = { destroy: () => {}, loadSource: () => {}, attachMedia: () => {}, audioTrack: 0, currentLevel: -1, media: video };
  instances.set(handle, waiting);
  const useNativeOrFail = () => {
    if (instances.get(handle) !== waiting) return;
    if (video.canPlayType('application/vnd.apple.mpegurl')) instances.set(handle, native(video, streamUrl));
    else video.dispatchEvent(new Event('error'));
  };
  loadPlayer()
    .then((p) => {
      if (instances.get(handle) !== waiting) return; // closed meanwhile
      const real = p.attach(video, streamUrl);
      if (!real) return useNativeOrFail();
      if (waiting.audioTrack) real.audioTrack = waiting.audioTrack;
      if (waiting.currentLevel !== -1) real.currentLevel = waiting.currentLevel;
      instances.set(handle, real);
    })
    .catch(useNativeOrFail);
  return handle;
}

export function hlsDestroy(handle: string): void {
  const inst = instances.get(handle);
  if (inst) {
    inst.destroy();
    instances.delete(handle);
  }
}

export function hlsSetAudioTrack(handle: string, trackIndex: number): void {
  const inst = instances.get(handle);
  if (!inst) return;
  if (inst._native) nativeTracks(inst._video).forEach((t, i) => { t.enabled = i === trackIndex; });
  else inst.audioTrack = trackIndex;
}

/** The audio tracks of the stream as JSON, `[{"i":0,"name":"English","lang":"en"}]`; empty when the stream has fewer than two. */
export function hlsAudioTracks(handle: string): string {
  const inst = instances.get(handle) as unknown as (HlsInstance & { audioTracks?: { name?: string; lang?: string }[] }) | undefined;
  if (!inst) return "[]";
  const tracks = inst._native
    ? nativeTracks(inst._video).map((t) => ({ name: t.label, lang: t.language }))
    : inst.audioTracks || [];
  if (tracks.length < 2) return "[]";
  return JSON.stringify(tracks.map((t, i) => ({ i, name: t.name || "", lang: t.lang || "" })));
}

/** The audio track playing now (the index into the list above). */
export function hlsGetAudioTrack(handle: string): number {
  const inst = instances.get(handle);
  if (!inst) return 0;
  if (inst._native) return Math.max(0, nativeTracks(inst._video).findIndex((t) => t.enabled));
  return inst.audioTrack;
}

export function hlsSetQuality(handle: string, level: number): void {
  const inst = instances.get(handle);
  if (inst && !inst._native) {
    inst.currentLevel = level;
  }
}

export function hlsSeek(handle: string, seconds: number): void {
  const inst = instances.get(handle);
  if (!inst) return;
  const video = inst._native ? inst._video : inst.media;
  if (video) video.currentTime = seconds;
}

export function hlsCurrentTime(handle: string): number {
  const inst = instances.get(handle);
  if (!inst) return 0;
  const video = inst._native ? inst._video : inst.media;
  return video ? video.currentTime : 0;
}

export function hlsDuration(handle: string): number {
  const inst = instances.get(handle);
  if (!inst) return 0;
  const video = inst._native ? inst._video : inst.media;
  return video ? video.duration || 0 : 0;
}
