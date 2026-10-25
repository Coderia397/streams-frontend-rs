// Pure TypeScript unpadded AES-128-CBC decryptor for partial HLS media segment slices.
// Avoids WebCrypto's PKCS#7 padding validation failure on truncated byte ranges.

export class AES128CBC {
  private rcon = [0x0, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];
  private subMix0 = new Uint32Array(256);
  private subMix1 = new Uint32Array(256);
  private subMix2 = new Uint32Array(256);
  private subMix3 = new Uint32Array(256);
  private invSubMix0 = new Uint32Array(256);
  private invSubMix1 = new Uint32Array(256);
  private invSubMix2 = new Uint32Array(256);
  private invSubMix3 = new Uint32Array(256);
  private sBox = new Uint32Array(256);
  private invSBox = new Uint32Array(256);

  private keySize = 0;
  private keySchedule = new Uint32Array(0);
  private invKeySchedule = new Uint32Array(0);

  constructor() {
    this.initTables();
  }

  private initTables(): void {
    const sBox = this.sBox;
    const invSBox = this.invSBox;
    const subMix0 = this.subMix0;
    const subMix1 = this.subMix1;
    const subMix2 = this.subMix2;
    const subMix3 = this.subMix3;
    const invSubMix0 = this.invSubMix0;
    const invSubMix1 = this.invSubMix1;
    const invSubMix2 = this.invSubMix2;
    const invSubMix3 = this.invSubMix3;

    const d = new Uint32Array(256);
    for (let i = 0; i < 256; i++) {
      d[i] = i < 128 ? i << 1 : (i << 1) ^ 0x11b;
    }

    let x = 0;
    let xi = 0;
    for (let i = 0; i < 256; i++) {
      let sx = xi ^ (xi << 1) ^ (xi << 2) ^ (xi << 3) ^ (xi << 4);
      sx = (sx >>> 8) ^ (sx & 0xff) ^ 0x63;
      sBox[x] = sx;
      invSBox[sx] = x;

      const x2 = d[x]!;
      const x4 = d[x2]!;
      const x8 = d[x4]!;

      const t1 = (d[sx]! * 0x101) ^ (sx * 0x1010100);
      subMix0[x] = (t1 << 24) | (t1 >>> 8);
      subMix1[x] = (t1 << 16) | (t1 >>> 16);
      subMix2[x] = (t1 << 8) | (t1 >>> 24);
      subMix3[x] = t1;

      const t2 = (x8 * 0x1010101) ^ (x4 * 0x10001) ^ (x2 * 0x101) ^ (x * 0x1010100);
      invSubMix0[sx] = (t2 << 24) | (t2 >>> 8);
      invSubMix1[sx] = (t2 << 16) | (t2 >>> 16);
      invSubMix2[sx] = (t2 << 8) | (t2 >>> 24);
      invSubMix3[sx] = t2;

      if (!x) {
        x = xi = 1;
      } else {
        x = x2 ^ d[d[d[x8 ^ x2]!]!]!;
        xi ^= d[d[xi]!]!;
      }
    }
  }

  private toUint32Array(buf: Uint8Array): Uint32Array {
    const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
    const words = new Uint32Array(buf.byteLength >>> 2);
    for (let i = 0; i < words.length; i++) {
      words[i] = view.getUint32(i * 4);
    }
    return words;
  }

  public setKey(keyBytes: Uint8Array): void {
    const key = this.toUint32Array(keyBytes);
    this.keySize = key.length;
    const ksRows = (this.keySize + 6 + 1) * 4;
    this.keySchedule = new Uint32Array(ksRows);
    this.invKeySchedule = new Uint32Array(ksRows);

    const sbox = this.sBox;
    const rcon = this.rcon;
    const invSubMix0 = this.invSubMix0;
    const invSubMix1 = this.invSubMix1;
    const invSubMix2 = this.invSubMix2;
    const invSubMix3 = this.invSubMix3;

    let prev = 0;
    for (let ksRow = 0; ksRow < ksRows; ksRow++) {
      if (ksRow < this.keySize) {
        prev = this.keySchedule[ksRow] = key[ksRow]!;
        continue;
      }
      let t = prev;
      if (ksRow % this.keySize === 0) {
        t = (t << 8) | (t >>> 24);
        t =
          (sbox[t >>> 24]! << 24) |
          (sbox[(t >>> 16) & 0xff]! << 16) |
          (sbox[(t >>> 8) & 0xff]! << 8) |
          sbox[t & 0xff]!;
        t ^= rcon[(ksRow / this.keySize) | 0]! << 24;
      }
      this.keySchedule[ksRow] = prev = (this.keySchedule[ksRow - this.keySize]! ^ t) >>> 0;
    }

    for (let invKsRow = 0; invKsRow < ksRows; invKsRow++) {
      const ksRow = ksRows - invKsRow;
      const t = invKsRow & 3 ? this.keySchedule[ksRow]! : this.keySchedule[ksRow - 4]!;
      if (invKsRow < 4 || ksRow <= 4) {
        this.invKeySchedule[invKsRow] = t;
      } else {
        this.invKeySchedule[invKsRow] =
          invSubMix0[sbox[t >>> 24]!]! ^
          invSubMix1[sbox[(t >>> 16) & 0xff]!]! ^
          invSubMix2[sbox[(t >>> 8) & 0xff]!]! ^
          invSubMix3[sbox[t & 0xff]!]!;
      }
      this.invKeySchedule[invKsRow] = this.invKeySchedule[invKsRow]! >>> 0;
    }
  }

  public decrypt(ciphertext: Uint8Array, ivBytes: Uint8Array): Uint8Array<ArrayBuffer> {
    const blockAlignedLen = ciphertext.byteLength & ~0x0f;
    if (blockAlignedLen === 0) return new Uint8Array(0);

    const nRounds = this.keySize + 6;
    const invKeySchedule = this.invKeySchedule;
    const invSBOX = this.invSBox;
    const invSubMix0 = this.invSubMix0;
    const invSubMix1 = this.invSubMix1;
    const invSubMix2 = this.invSubMix2;
    const invSubMix3 = this.invSubMix3;

    const initVector = this.toUint32Array(ivBytes);
    let iv0 = initVector[0]!;
    let iv1 = initVector[1]!;
    let iv2 = initVector[2]!;
    let iv3 = initVector[3]!;

    const inputView = new DataView(ciphertext.buffer, ciphertext.byteOffset, blockAlignedLen);
    const output = new Uint8Array(blockAlignedLen);
    const outputView = new DataView(output.buffer);

    for (let offset = 0; offset < blockAlignedLen; offset += 16) {
      const in0 = inputView.getUint32(offset);
      const in1 = inputView.getUint32(offset + 4);
      const in2 = inputView.getUint32(offset + 8);
      const in3 = inputView.getUint32(offset + 12);

      let s0 = in0 ^ invKeySchedule[0]!;
      let s1 = in3 ^ invKeySchedule[1]!;
      let s2 = in2 ^ invKeySchedule[2]!;
      let s3 = in1 ^ invKeySchedule[3]!;

      let ksRow = 4;
      for (let i = 1; i < nRounds; i++) {
        const t0 =
          invSubMix0[s0 >>> 24]! ^
          invSubMix1[(s1 >> 16) & 0xff]! ^
          invSubMix2[(s2 >> 8) & 0xff]! ^
          invSubMix3[s3 & 0xff]! ^
          invKeySchedule[ksRow]!;
        const t1 =
          invSubMix0[s1 >>> 24]! ^
          invSubMix1[(s2 >> 16) & 0xff]! ^
          invSubMix2[(s3 >> 8) & 0xff]! ^
          invSubMix3[s0 & 0xff]! ^
          invKeySchedule[ksRow + 1]!;
        const t2 =
          invSubMix0[s2 >>> 24]! ^
          invSubMix1[(s3 >> 16) & 0xff]! ^
          invSubMix2[(s0 >> 8) & 0xff]! ^
          invSubMix3[s1 & 0xff]! ^
          invKeySchedule[ksRow + 2]!;
        const t3 =
          invSubMix0[s3 >>> 24]! ^
          invSubMix1[(s0 >> 16) & 0xff]! ^
          invSubMix2[(s1 >> 8) & 0xff]! ^
          invSubMix3[s2 & 0xff]! ^
          invKeySchedule[ksRow + 3]!;

        s0 = t0;
        s1 = t1;
        s2 = t2;
        s3 = t3;
        ksRow += 4;
      }

      const out0 =
        (invSBOX[s0 >>> 24]! << 24) ^
        (invSBOX[(s1 >> 16) & 0xff]! << 16) ^
        (invSBOX[(s2 >> 8) & 0xff]! << 8) ^
        invSBOX[s3 & 0xff]! ^
        invKeySchedule[ksRow]!;
      const out1 =
        (invSBOX[s1 >>> 24]! << 24) ^
        (invSBOX[(s2 >> 16) & 0xff]! << 16) ^
        (invSBOX[(s3 >> 8) & 0xff]! << 8) ^
        invSBOX[s0 & 0xff]! ^
        invKeySchedule[ksRow + 1]!;
      const out2 =
        (invSBOX[s2 >>> 24]! << 24) ^
        (invSBOX[(s3 >> 16) & 0xff]! << 16) ^
        (invSBOX[(s0 >> 8) & 0xff]! << 8) ^
        invSBOX[s1 & 0xff]! ^
        invKeySchedule[ksRow + 2]!;
      const out3 =
        (invSBOX[s3 >>> 24]! << 24) ^
        (invSBOX[(s0 >> 16) & 0xff]! << 16) ^
        (invSBOX[(s1 >> 8) & 0xff]! << 8) ^
        invSBOX[s2 & 0xff]! ^
        invKeySchedule[ksRow + 3]!;

      outputView.setUint32(offset, out0 ^ iv0);
      outputView.setUint32(offset + 4, out3 ^ iv1);
      outputView.setUint32(offset + 8, out2 ^ iv2);
      outputView.setUint32(offset + 12, out1 ^ iv3);

      iv0 = in0;
      iv1 = in1;
      iv2 = in2;
      iv3 = in3;
    }

    return output;
  }
}
