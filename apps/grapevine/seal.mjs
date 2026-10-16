// The seal on every message (GRAPEVINE_DEMO §3, §13.5): tweetnacl's `box`,
// NaCl's crypto_box (X25519, XSalsa20-Poly1305), a named construction and
// never one assembled from parts (GRAPEVINE.md §4.4). Two functions and the
// key pairs; nothing else here touches the library.
//
//   keyPair()                 {public, secret}: one ask's own, base64
//   seal(body, to, secret)    {nonce, box}: `body` as JSON, sealed to `to`
//   open(sealed, other, secret)
//                             the body, or null when it does not open
//
// crypto_box's key is shared by the two ends, so the sender opens what they
// sealed with the recipient's public key, as the recipient opens it with
// the sender's. A key pair is made per ask, by the asker for `seal_to` and by
// each replier for their `from`, so the keys link nothing between asks.

import nacl from './vendor/tweetnacl/nacl-fast.js';

/**
 * The most an act may be, in bytes: svrngs-core's MAX_PAYLOAD_BYTES
 * (crates/svrngs-core/src/payload.rs), restated here because an app does not
 * link the crate. grapevine.test.mjs reads that line and fails if they part.
 */
export const ACT_MOST = 64 * 1024;
/**
 * The most a sealed body may be, in bytes of its JSON. Its box, the body and
 * a 16-byte tag in base64, is then at most 64,024 bytes, and a `msg` with its
 * keys, nonce and handle stays under ACT_MOST (grapevine.test.mjs).
 */
export const BODY_MOST = 48_000;

const b64 = (bytes) => Buffer.from(bytes).toString('base64');
/** `s` as the bytes its base64 says, or null when it is not `n` of them. */
function bytesOf(s, n) {
  if (typeof s !== 'string') return null;
  const bytes = Buffer.from(s, 'base64');
  return bytes.length === n && b64(bytes) === s ? new Uint8Array(bytes) : null;
}

/** A key pair for one ask. */
export function keyPair() {
  const { publicKey, secretKey } = nacl.box.keyPair();
  return { public: b64(publicKey), secret: b64(secretKey) };
}

/** `body` sealed to `to` by `secret`'s holder. A body past BODY_MOST is refused, never cut. */
export function seal(body, to, secret) {
  const text = new TextEncoder().encode(JSON.stringify(body));
  if (text.length > BODY_MOST) throw new Error(`a message is at most ${BODY_MOST} bytes sealed, and this one is ${text.length}`);
  const them = bytesOf(to, nacl.box.publicKeyLength);
  const mine = bytesOf(secret, nacl.box.secretKeyLength);
  if (!them || !mine) throw new Error('a message is sealed to a public key with a secret key, each base64 of 32 bytes');
  const nonce = nacl.randomBytes(nacl.box.nonceLength);
  return { nonce: b64(nonce), box: b64(nacl.box(text, nonce, them, mine)) };
}

/** What `sealed` says, opened with `other`'s public key and this end's `secret`; null when it does not open. */
export function open(sealed, other, secret) {
  const nonce = bytesOf(sealed?.nonce, nacl.box.nonceLength);
  const them = bytesOf(other, nacl.box.publicKeyLength);
  const mine = bytesOf(secret, nacl.box.secretKeyLength);
  const box = typeof sealed?.box === 'string' && sealed.box.length <= ACT_MOST ? Buffer.from(sealed.box, 'base64') : null;
  if (!nonce || !them || !mine || !box) return null;
  const text = nacl.box.open(new Uint8Array(box), nonce, them, mine);
  if (!text) return null;
  try {
    return JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(text));
  } catch {
    return null;
  }
}
