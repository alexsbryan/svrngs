// The laws air.mjs keeps: a line is the station's only under its key and the
// app's domain, a piece only if it is its hash, and the cut keeps every
// frame, whole, in order. Run by scripts/check.sh with every app's laws.
import './vendored.mjs';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const air = await import('./page/air.mjs');
const { ed25519 } = await import('@noble/curves/ed25519.js');
const { bytesToHex, hexToBytes, utf8ToBytes } = await import('@noble/hashes/utils.js');

// One MPEG-1 Layer III frame at 128 kbit/s and 44.1 kHz, no padding: 417
// bytes, 1152 samples.
const frame = (fill) => { const f = new Uint8Array(417).fill(fill); f.set([0xff, 0xfb, 0x90, 0x64]); return f; };
const mp3 = (n) => { const b = new Uint8Array(417 * n); for (let i = 0; i < n; i++) b.set(frame(i % 251), i * 417); return b; };

test('the cut keeps every frame, whole and in order, in pieces of at least 4 s', () => {
  const whole = mp3(400);
  const pieces = air.cut(whole, 4);
  assert.deepEqual(pieces.map((p) => p.bytes.length / 417), [154, 154, 92]);
  assert.ok(pieces.slice(0, -1).every((p) => p.seconds >= 4 && p.seconds < 4 + 1152 / 44100));
  for (const p of pieces) assert.deepEqual([...p.bytes.subarray(0, 2)], [0xff, 0xfb]);
  assert.deepEqual(Buffer.concat(pieces.map((p) => p.bytes)), Buffer.from(whole));
});

test('the cut refuses what is not a frame, saying where', () => {
  const bad = mp3(10);
  bad[417 * 3] = 0;
  assert.throws(() => air.cut(bad), /no MP3 frame at byte 1251/);
});

test('a piece is played only as its hash', () => {
  const piece = mp3(3);
  const hash = air.blake3hex(piece);
  assert.equal(air.checkPiece(piece, hash).ok, true);
  const flipped = piece.slice(); flipped[500] ^= 1;
  assert.match(air.checkPiece(flipped, hash).why, /BLAKE3 is/);
  assert.equal(air.checkPiece(new Uint8Array(0), hash).why, 'an empty piece');
});

test("a line is the station's only under its key and the app's domain", () => {
  const station = ed25519.utils.randomSecretKey(), other = ed25519.utils.randomSecretKey();
  const key = bytesToHex(ed25519.getPublicKey(station));
  const line = air.pieceLine('evening', 3, 'ab'.repeat(32), 1);
  const sign = (sk, bytes) => bytesToHex(ed25519.sign(bytes, sk));
  const ok = air.checkLine({ line, key, sig: sign(station, air.signedBytes(air.APP, line)) }, key);
  assert.deepEqual(ok, { ok: true, line: JSON.parse(line) });
  // Another computer's signature, though the envelope claims the station's key.
  assert.match(air.checkLine({ line, key, sig: sign(other, air.signedBytes(air.APP, line)) }, key).why, /does not check/);
  // The station's key over the bare text, as another domain would sign it.
  assert.match(air.checkLine({ line, key, sig: sign(station, utf8ToBytes(line)) }, key).why, /does not check/);
  // Another app's signature.
  assert.match(air.checkLine({ line, key, sig: sign(station, air.signedBytes('other', line)) }, key).why, /does not check/);
  // A line changed after it was signed.
  const sig = sign(station, air.signedBytes(air.APP, line));
  assert.match(air.checkLine({ line: line.replace('3', '4'), key, sig }, key).why, /does not check/);
  assert.match(air.checkLine({ line, key, sig: null, unsigned: 'POST /v1/sign answered 404' }, key).why, /^unsigned: POST/);
  const ended = air.endedLine('evening', 9, 2);
  assert.equal(air.checkLine({ line: ended, key, sig: sign(station, air.signedBytes(air.APP, ended)) }, key).line.ended, true);
});

// svrngs-core's frozen vector (crates/svrngs-core/src/app_tests.rs,
// `a_frozen_vector_for_a_stranger`): what a node signs for an app is what
// this page checks, byte for byte.
test("svrngs-core's signature for an app checks here", () => {
  const key = '8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c';
  const sig = '6296480dc80a73a3f3872dac92fb777835e837829b20c7f818ab7ea922abdf401452e404d04f7a39cf613161bdb80a89ba55bfc43700b0a678f63b5c4110440f';
  assert.equal(ed25519.verify(hexToBytes(sig), air.signedBytes('notes', 'line 812'), hexToBytes(key)), true);
  assert.equal(ed25519.verify(hexToBytes(sig), air.signedBytes('notes', 'line 813'), hexToBytes(key)), false);
});
