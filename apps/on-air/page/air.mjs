// What the station, the listener and the stranger's page share: the line
// and its signed bytes, the check of a line and of a piece, and the MP3 cut.
// Pure: no I/O; it sits in page/ since the page imports it as it is, and
// laws.test.mjs proves it.
import { ed25519 } from '@noble/curves/ed25519.js';
import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex, hexToBytes, utf8ToBytes, concatBytes } from '@noble/hashes/utils.js';

// The app's label, as the station asks `POST /v1/sign?app=on-air`.
export const APP = 'on-air';
// The domain a node signs an app's bytes under, ON_AIR §4.2's sketch:
// "svrngs-app:" || app || bytes. onair-sign makes it in svrngs-core; this
// is the one place the app spells it, to change with it.
export const appDomain = (app) => utf8ToBytes(`svrngs-app:${app}`);
export const signedBytes = (app, text) => concatBytes(appDomain(app), utf8ToBytes(text));

// The line's own text, its keys in one order, so what was signed is what is
// read: `{show, n, hash, at}`, and the last `{show, n, ended, at}`.
export const pieceLine = (show, n, hash, at) => JSON.stringify({ show, n, hash, at });
export const endedLine = (show, n, at) => JSON.stringify({ show, n, ended: true, at });

export const blake3hex = (bytes) => bytesToHex(blake3(bytes));

// A piece is played only if its bytes are the hash its line names.
export function checkPiece(bytes, hash) {
  if (!bytes || bytes.length === 0) return { ok: false, why: 'an empty piece' };
  const got = blake3hex(bytes);
  return got === hash ? { ok: true } : { ok: false, why: `its BLAKE3 is ${got.slice(0, 12)}…, not the line's ${String(hash).slice(0, 12)}…` };
}

// A published or pushed line: `{line, key, sig}`, the line as the text that
// was signed. It is the station's only if its signature checks under the
// station's key — the key the envelope claims is never trusted.
export function checkLine(envelope, stationKey, app = APP) {
  if (!envelope || typeof envelope.line !== 'string') return { ok: false, why: 'not a line' };
  if (!envelope.sig) return { ok: false, why: `unsigned: ${envelope.unsigned || 'it carries no signature'}` };
  let ok = false;
  try {
    ok = ed25519.verify(hexToBytes(envelope.sig), signedBytes(app, envelope.line), hexToBytes(stationKey));
  } catch (e) {
    return { ok: false, why: `its signature cannot be read: ${e.message}` };
  }
  if (!ok) {
    const claimed = envelope.key && envelope.key !== stationKey ? `; it names ${String(envelope.key).slice(0, 8)}…` : '';
    return { ok: false, why: `its signature does not check under the station's key ${stationKey.slice(0, 8)}…${claimed}` };
  }
  let line;
  try { line = JSON.parse(envelope.line); } catch { return { ok: false, why: 'signed, but not a line' }; }
  const piece = Number.isInteger(line.n) && typeof line.hash === 'string';
  if (typeof line.show !== 'string' || !(piece || (line.ended === true && Number.isInteger(line.n)))) {
    return { ok: false, why: 'signed, but not a line of a piece or its end' };
  }
  return { ok: true, line };
}

// ── The cut: MPEG audio Layer III frames, read from their headers ──────────
const KBPS = {
  1: [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320],
  2: [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],
};
const RATES = { 1: [44100, 48000, 32000], 2: [22050, 24000, 16000], 2.5: [11025, 12000, 8000] };

// The frames of an MP3, each `{at, len, seconds}`, after an ID3v2 tag and
// before an ID3v1 one; anything else that is not a Layer III frame is
// refused by where it stands, never skipped.
export function frames(bytes) {
  let at = 0;
  if (bytes[0] === 0x49 && bytes[1] === 0x44 && bytes[2] === 0x33) {
    const size = (bytes[6] << 21) | (bytes[7] << 14) | (bytes[8] << 7) | bytes[9];
    at = 10 + size + (bytes[5] & 0x10 ? 10 : 0);
  }
  const out = [];
  while (at < bytes.length) {
    if (bytes.length - at === 128 && bytes[at] === 0x54 && bytes[at + 1] === 0x41 && bytes[at + 2] === 0x47) break;
    if (bytes.length - at < 4 || bytes[at] !== 0xff || (bytes[at + 1] & 0xe0) !== 0xe0) {
      throw new Error(`no MP3 frame at byte ${at} of ${bytes.length}`);
    }
    const version = { 3: 1, 2: 2, 0: 2.5 }[(bytes[at + 1] >> 3) & 3];
    const layer = (bytes[at + 1] >> 1) & 3;
    const kbps = KBPS[version === 1 ? 1 : 2]?.[bytes[at + 2] >> 4];
    const rate = RATES[version]?.[(bytes[at + 2] >> 2) & 3];
    if (!version || layer !== 1 || !kbps || !rate) throw new Error(`byte ${at} is not a Layer III frame with a bitrate`);
    const len = Math.floor(((version === 1 ? 144 : 72) * kbps * 1000) / rate) + ((bytes[at + 2] >> 1) & 1);
    if (at + len > bytes.length) throw new Error(`the frame at byte ${at} runs past the end`);
    out.push({ at, len, seconds: (version === 1 ? 1152 : 576) / rate });
    at += len;
  }
  if (!out.length) throw new Error('no MP3 frames');
  return out;
}

// The MP3 cut on frame boundaries into pieces of at least `seconds` each,
// the last whatever is left.
export function cut(bytes, seconds = 4) {
  const pieces = [];
  let start = null, held = 0, end = 0;
  for (const f of frames(bytes)) {
    start ??= f.at;
    held += f.seconds;
    end = f.at + f.len;
    if (held >= seconds) {
      pieces.push({ bytes: bytes.subarray(start, f.at + f.len), seconds: held });
      start = null; held = 0;
    }
  }
  if (start !== null) pieces.push({ bytes: bytes.subarray(start, end), seconds: held });
  return pieces;
}
