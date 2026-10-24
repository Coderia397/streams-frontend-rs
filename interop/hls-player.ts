// Loaded only when a stream plays (interop/hls.ts imports it at the first stream): hls.js and its recovery rules. Built as its own
// file (interop/dist/hls-player.js, copied to the site's root), so the library (about 590 KB) is no longer part of every page's start.
import Hls from 'hls.js';
import type { HlsInstance } from './hls';

/** hls.js playing `streamUrl` in `video`, or null when this browser cannot run it (the caller then uses the browser's own HLS). */
export function attach(video: HTMLVideoElement, streamUrl: string): HlsInstance | null {
  if (!Hls.isSupported()) return null;
    const hls = new Hls({
      enableWorker: true,
      capLevelToPlayerSize: true,
      // a quality the device drops frames at (a slow phone or TV decoding a high bitrate) is not chosen again: measured by the
      // player while it plays, so each device finds its own ceiling
      capLevelOnFPSDrop: true,
      // the first piece of video is fetched while the stream's index is still being read: playback starts sooner
      startFragPrefetch: true,
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
  return hls as unknown as HlsInstance;
}
