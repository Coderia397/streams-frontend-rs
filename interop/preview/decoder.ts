// WebCodecs hardware video decoding wrapper.
// Keeps a persistent VideoDecoder instance across frames to avoid repeated 250ms+ startup overheads.

import { KeyframeData } from './ts-demux';

declare const VideoDecoder: any;
declare const EncodedVideoChunk: any;

export function isWebCodecsSupported(): boolean {
  return typeof VideoDecoder !== 'undefined' && typeof EncodedVideoChunk !== 'undefined';
}

function arraysEqual(a: Uint8Array | null, b: Uint8Array | null): boolean {
  if (a === b) return true;
  if (!a || !b || a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

export class H264Decoder {
  private decoder: any = null;
  private configuredCodec: string | null = null;
  private configuredAvcc: Uint8Array | null = null;
  private queue: Promise<any> = Promise.resolve();
  private closed = false;

  private latestFrame: any = null;
  private latestError: any = null;

  public async decode(data: KeyframeData): Promise<ImageBitmap> {
    if (this.closed) throw new Error('Decoder is closed');

    return new Promise<ImageBitmap>((resolve, reject) => {
      this.queue = this.queue.then(async () => {
        if (this.closed) {
          reject(new Error('Decoder is closed'));
          return;
        }
        try {
          const bmp = await this.doDecode(data);
          resolve(bmp);
        } catch (err) {
          reject(err);
        }
      });
    });
  }

  private async doDecode(data: KeyframeData): Promise<ImageBitmap> {
    this.latestFrame = null;
    this.latestError = null;

    if (!this.decoder || this.decoder.state === 'closed') {
      this.initDecoder();
    }

    const needConfig =
      this.configuredCodec !== data.codec ||
      !arraysEqual(this.configuredAvcc, data.avcc);

    if (needConfig) {
      try {
        this.decoder.configure({
          codec: data.codec,
          description: data.avcc,
        });
        this.configuredCodec = data.codec;
        this.configuredAvcc = data.avcc;
      } catch (err) {
        this.initDecoder();
        throw err;
      }
    }

    const chunk = new EncodedVideoChunk({
      type: 'key',
      timestamp: 0,
      data: data.chunkData,
    });

    try {
      this.decoder.decode(chunk);

      await Promise.race([
        this.decoder.flush(),
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error('WebCodecs decode timed out')), 2000)
        ),
      ]);
    } catch (err) {
      this.initDecoder();
      throw err;
    }

    if (this.latestError) {
      const err = this.latestError;
      this.initDecoder();
      throw err;
    }

    if (!this.latestFrame) {
      this.initDecoder();
      throw new Error('No frame output from WebCodecs');
    }

    const frame = this.latestFrame;
    this.latestFrame = null;

    try {
      const bmp = await createImageBitmap(frame, {
        resizeWidth: 256,
        resizeHeight: 144,
        resizeQuality: 'medium',
      });
      return bmp;
    } finally {
      try {
        frame.close();
      } catch {}
    }
  }

  private initDecoder(): void {
    if (this.decoder) {
      try {
        this.decoder.close();
      } catch {}
      this.decoder = null;
    }
    this.configuredCodec = null;
    this.configuredAvcc = null;

    this.decoder = new VideoDecoder({
      output: (videoFrame: any) => {
        if (this.latestFrame) {
          try {
            this.latestFrame.close();
          } catch {}
        }
        this.latestFrame = videoFrame;
      },
      error: (err: any) => {
        this.latestError = err;
      },
    });
  }

  public close(): void {
    this.closed = true;
    if (this.latestFrame) {
      try {
        this.latestFrame.close();
      } catch {}
      this.latestFrame = null;
    }
    if (this.decoder) {
      try {
        this.decoder.close();
      } catch {}
      this.decoder = null;
    }
  }
}
