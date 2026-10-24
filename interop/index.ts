// The only entry point Rust binds to (#[wasm_bindgen(module = "/interop/dist/streams-interop.js")]). Every export here has a matching
// extern declaration in src/components/hls_adapter.rs, src/components/share/mod.rs or src/utils/ambient.rs; tsc checks the TypeScript side of the signatures.
import * as ambient from './ambient';

export {
  hlsAttach,
  hlsDestroy,
  hlsSetAudioTrack,
  hlsAudioTracks,
  hlsGetAudioTrack,
  hlsSetQuality,
  hlsSeek,
  hlsCurrentTime,
  hlsDuration,
} from './hls';
export { applyAmbientColor, setThemeColor } from './theme';
export { resumeSeal, resumeOpen, resumeForget, resumeFingerprint } from './resume';
export { sharePrepare, shareSystem, copyText } from './share';

export function getCachedAmbientColor(imageUrl: string): string {
  const res = ambient.getCachedAmbientColor(imageUrl);
  return res ? JSON.stringify(res) : '';
}

export function getLastAmbientColor(): string {
  const res = ambient.getLastAmbientColor();
  return res ? JSON.stringify(res) : '';
}

export function extractAmbientColor(imageUrl: string, callback: (res: string) => void): void {
  ambient.extractAmbientColor(imageUrl).then((res) => {
    callback(res ? JSON.stringify(res) : '');
  });
}
