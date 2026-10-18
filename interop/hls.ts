import Hls from 'hls.js';

interface HlsInstance {
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

export function hlsAttach(videoId: string, streamUrl: string): string {
  const video = document.getElementById(videoId) as HTMLVideoElement | null;
  if (!video) return "";
  const handle = "hls_" + (++counter);

  const isExplicitDirect = /\.(mp4|m4v|webm|mkv|mov)(\?.*)?$/i.test(streamUrl);
  if (isExplicitDirect) {
    video.src = streamUrl;
    instances.set(handle, {
      destroy: () => { video.src = ""; video.removeAttribute("src"); video.load(); },
      loadSource: (url: string) => { video.src = url; },
      attachMedia: () => {},
      audioTrack: 0,
      currentLevel: -1,
      _native: true,
      _video: video,
    });
    return handle;
  }

  if (typeof Hls !== 'undefined' && Hls.isSupported()) {
    const hls = new Hls({
      enableWorker: true,
      capLevelToPlayerSize: true,
      maxBufferLength: 30,
      maxMaxBufferLength: 60,
      backBufferLength: 30,
    });

    let mediaRecoveryAttempts = 0;
    let networkRecoveryAttempts = 0;

    hls.on(Hls.Events.ERROR, (_event, data) => {
      if (data.fatal) {
        switch (data.type) {
          case Hls.ErrorTypes.NETWORK_ERROR:
            if (networkRecoveryAttempts < 3) {
              networkRecoveryAttempts++;
              console.warn('[HLS] Network error encountered, recovery attempt', networkRecoveryAttempts);
              hls.startLoad();
            } else {
              console.error('[HLS] Fatal network error, falling back to native if supported...');
              if (video && video.canPlayType('application/vnd.apple.mpegurl')) {
                hls.destroy();
                video.src = streamUrl;
                video.play().catch(() => {});
              } else {
                video.dispatchEvent(new Event('error'));
              }
            }
            break;
          case Hls.ErrorTypes.MEDIA_ERROR:
            if (mediaRecoveryAttempts < 2) {
              mediaRecoveryAttempts++;
              console.warn('[HLS] Media error encountered (stall/video freeze), recovering...');
              hls.recoverMediaError();
            } else if (mediaRecoveryAttempts === 2) {
              mediaRecoveryAttempts++;
              console.warn('[HLS] Media error recovery #2, swapping audio codec...');
              hls.swapAudioCodec();
              hls.recoverMediaError();
            } else {
              console.error('[HLS] Fatal media error unrecoverable, falling back to native...');
              if (video && video.canPlayType('application/vnd.apple.mpegurl')) {
                hls.destroy();
                video.src = streamUrl;
                video.play().catch(() => {});
              } else {
                video.dispatchEvent(new Event('error'));
              }
            }
            break;
          default:
            console.error('[HLS] Fatal unrecoverable error:', data);
            if (video && video.canPlayType('application/vnd.apple.mpegurl')) {
              hls.destroy();
              video.src = streamUrl;
              video.play().catch(() => {});
            } else {
              video.dispatchEvent(new Event('error'));
            }
            break;
        }
      }
    });

    hls.loadSource(streamUrl);
    hls.attachMedia(video);
    instances.set(handle, hls);
  } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
    video.src = streamUrl;
    instances.set(handle, {
      destroy: () => { video.src = ""; video.removeAttribute("src"); video.load(); },
      loadSource: (url: string) => { video.src = url; },
      attachMedia: () => {},
      audioTrack: 0,
      currentLevel: -1,
      _native: true,
      _video: video,
    });
  }
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
