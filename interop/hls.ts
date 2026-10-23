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
function loadPlayer(): Promise<Player> {
  const src = new URL('/hls-player.js', location.origin).href;
  return (player ??= import(/* @vite-ignore */ src) as Promise<Player>);
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

  if (/\.(mp4|m4v|webm|mkv|mov)(\?.*)?$/i.test(streamUrl)) {
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
  if (inst && !inst._native) {
    inst.audioTrack = trackIndex;
  }
}

/** The audio tracks of the stream as JSON, `[{"i":0,"name":"English","lang":"en"}]`; empty when the stream has fewer than two. */
export function hlsAudioTracks(handle: string): string {
  const inst = instances.get(handle) as unknown as { _native?: boolean; audioTracks?: { name?: string; lang?: string }[] } | undefined;
  if (!inst || inst._native || !inst.audioTracks || inst.audioTracks.length < 2) return "[]";
  return JSON.stringify(inst.audioTracks.map((t, i) => ({ i, name: t.name || "", lang: t.lang || "" })));
}

/** The audio track playing now (the index into the list above). */
export function hlsGetAudioTrack(handle: string): number {
  const inst = instances.get(handle);
  return inst && !inst._native ? inst.audioTrack : 0;
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
