// Minimal, high-performance MPEG-TS demuxer for H.264 IDR keyframe extraction.
// Isolates SPS, PPS, and all complete IDR slices for the first access unit.

export interface KeyframeData {
  codec: string;
  avcc: Uint8Array;
  chunkData: Uint8Array;
}

export function extractKeyframeFromTs(d: Uint8Array): KeyframeData | null {
  const len = d.length;
  if (len < 188) return null;

  // Collect 188-byte packets starting with sync byte 0x47
  const pkts: Uint8Array[] = [];
  for (let i = 0; i <= len - 188; i += 188) {
    if (d[i] === 0x47) {
      pkts.push(d.subarray(i, i + 188));
    }
  }
  if (pkts.length === 0) return null;

  const pid = (p: Uint8Array) => (((p[1] ?? 0) & 0x1f) << 8) | (p[2] ?? 0);
  const payload = (p: Uint8Array): Uint8Array | null => {
    const p3 = p[3] ?? 0;
    const afc = (p3 >> 4) & 3;
    if (afc === 0 || afc === 2) return null;
    const off = 4 + (afc === 3 ? 1 + (p[4] ?? 0) : 0);
    return off < 188 ? p.subarray(off) : null;
  };

  // 1. Locate PMT PID from PAT (PID 0)
  let pmtPid: number | null = null;
  for (const p of pkts) {
    if (pid(p) === 0 && ((p[1] ?? 0) & 0x40)) {
      const pl = payload(p);
      if (pl && pl.length > 11) {
        const sec = pl.subarray(1 + (pl[0] ?? 0));
        pmtPid = (((sec[10] ?? 0) & 0x1f) << 8) | (sec[11] ?? 0);
        break;
      }
    }
  }

  // 2. Locate video PID (H.264 / AVC stream_type = 0x1b) from PMT
  let videoPid: number | null = null;
  if (pmtPid !== null) {
    for (const p of pkts) {
      if (pid(p) === pmtPid && ((p[1] ?? 0) & 0x40)) {
        const pl = payload(p);
        if (pl) {
          const sec = pl.subarray(1 + (pl[0] ?? 0));
          const secLen = (((sec[1] ?? 0) & 0x0f) << 8) | (sec[2] ?? 0);
          const pil = (((sec[10] ?? 0) & 0x0f) << 8) | (sec[11] ?? 0);
          let i = 12 + pil;
          while (i < 3 + secLen - 4) {
            const streamType = sec[i] ?? 0;
            const elemPid = (((sec[i + 1] ?? 0) & 0x1f) << 8) | (sec[i + 2] ?? 0);
            const esLen = (((sec[i + 3] ?? 0) & 0x0f) << 8) | (sec[i + 4] ?? 0);
            if (streamType === 0x1b) {
              videoPid = elemPid;
              break;
            }
            i += 5 + esLen;
          }
        }
        break;
      }
    }
  }

  if (videoPid === null) {
    videoPid = 256;
  }

  // 3. Assemble PES payload for video PID
  const esChunks: Uint8Array[] = [];
  let totalEsLen = 0;

  for (const p of pkts) {
    if (pid(p) !== videoPid) continue;
    let pl = payload(p);
    if (!pl) continue;
    if ((p[1] ?? 0) & 0x40) {
      // Payload unit start indicator: strip PES packet header
      if (pl.length > 8) {
        const hdrLen = 9 + (pl[8] ?? 0);
        if (hdrLen < pl.length) {
          pl = pl.subarray(hdrLen);
        }
      }
    }
    esChunks.push(pl);
    totalEsLen += pl.length;
  }

  if (esChunks.length === 0) return null;

  const es = new Uint8Array(totalEsLen);
  let writeOffset = 0;
  for (const chunk of esChunks) {
    es.set(chunk, writeOffset);
    writeOffset += chunk.length;
  }

  // 4. Parse Annex-B NAL units
  // Crucial: A NAL unit is only complete if delimited by a subsequent start code
  const nals: { type: number; data: Uint8Array; complete: boolean }[] = [];
  let cursor = 0;
  while (cursor < es.length - 3) {
    let startPrefix = -1;
    if (es[cursor] === 0 && es[cursor + 1] === 0) {
      if (es[cursor + 2] === 1) startPrefix = cursor + 3;
      else if (cursor + 3 < es.length && es[cursor + 2] === 0 && es[cursor + 3] === 1) startPrefix = cursor + 4;
    }

    if (startPrefix !== -1) {
      let nextPrefix = -1;
      for (let j = startPrefix; j < es.length - 2; j++) {
        if (es[j] === 0 && es[j + 1] === 0 && (es[j + 2] === 1 || (j + 3 < es.length && es[j + 2] === 0 && es[j + 3] === 1))) {
          nextPrefix = j;
          break;
        }
      }
      const complete = nextPrefix !== -1;
      const end = complete ? nextPrefix : es.length;
      const nalData = es.subarray(startPrefix, end);
      if (nalData.length > 0) {
        nals.push({
          type: (nalData[0] ?? 0) & 0x1f,
          data: nalData,
          complete,
        });
      }
      cursor = end;
    } else {
      cursor++;
    }
  }

  // 5. Extract SPS (7) and PPS (8)
  const sps = nals.find((n) => n.type === 7 && n.complete)?.data;
  const pps = nals.find((n) => n.type === 8 && n.complete)?.data;

  if (!sps || !pps || sps.length < 4) return null;

  // 6. Collect ALL IDR slices (type 5) for the first access unit up to the next AU boundary (e.g. AUD type 9 or non-IDR slice type 1)
  const idrSlices: Uint8Array[] = [];
  let seenIdr = false;
  for (const nal of nals) {
    if (nal.type === 5) {
      // Must be bounded by a subsequent start code to not be truncated
      if (!nal.complete) {
        // Cut-off slice! Return null so the caller retries with the extended slice
        return null;
      }
      idrSlices.push(nal.data);
      seenIdr = true;
    } else if (seenIdr && (nal.type === 9 || nal.type === 1 || nal.type === 7)) {
      // Reached next access unit boundary
      break;
    }
  }

  if (idrSlices.length === 0) return null;

  const sps1 = sps[1] ?? 0x64;
  const sps2 = sps[2] ?? 0x00;
  const sps3 = sps[3] ?? 0x1f;

  const p1 = sps1.toString(16).padStart(2, '0');
  const p2 = sps2.toString(16).padStart(2, '0');
  const p3 = sps3.toString(16).padStart(2, '0');
  const codec = `avc1.${p1}${p2}${p3}`;

  // Build standard avcC record
  const avcc = new Uint8Array(11 + sps.length + pps.length);
  avcc[0] = 1; // configurationVersion
  avcc[1] = sps1; // AVCProfileIndication
  avcc[2] = sps2; // profile_compatibility
  avcc[3] = sps3; // AVCLevelIndication
  avcc[4] = 0xff; // lengthSizeMinusOne (3 -> 4 bytes)
  avcc[5] = 0xe1; // numOfSequenceParameterSets (1)
  avcc[6] = (sps.length >> 8) & 0xff;
  avcc[7] = sps.length & 0xff;
  avcc.set(sps, 8);

  const ppsOffset = 8 + sps.length;
  avcc[ppsOffset] = 1; // numOfPictureParameterSets (1)
  avcc[ppsOffset + 1] = (pps.length >> 8) & 0xff;
  avcc[ppsOffset + 2] = pps.length & 0xff;
  avcc.set(pps, ppsOffset + 3);

  // Package all IDR slices with 4-byte big-endian length prefix
  let totalChunkLen = 0;
  for (const slice of idrSlices) {
    totalChunkLen += 4 + slice.length;
  }

  const chunkData = new Uint8Array(totalChunkLen);
  const view = new DataView(chunkData.buffer);
  let chunkOffset = 0;
  for (const slice of idrSlices) {
    view.setUint32(chunkOffset, slice.length, false);
    chunkData.set(slice, chunkOffset + 4);
    chunkOffset += 4 + slice.length;
  }

  return {
    codec,
    avcc,
    chunkData,
  };
}
