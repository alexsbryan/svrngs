import { hex } from './store.js?v=__VERSION__';

// Whether a request may cause a write signed as the person: one from a page
// of this origin, by the browser's Client, never a body's word; or a
// navigation one of them began, a form's post or a link (Events' "I came",
// a page with no script), by its referrer, which the browser sets from the
// page that began it and another site can drop but never make this origin's.
/**
 * What `maySign` reads of a service worker's `FetchEvent` and `Clients`,
 * which the page's libs (jsconfig.json `lib`) do not declare.
 * @typedef {{ request: Request, clientId: string }} Fetched
 * @typedef {{ get(id: string): Promise<{ url: string } | undefined> }} Clients
 */
/**
 * @param {Fetched} fetched
 * @param {Clients} clients
 * @param {string} origin
 */
export async function maySign(fetched, clients, origin) {
  const ours = (/** @type {string} */ url) => {
    try {
      return new URL(url).origin === origin;
    } catch {
      return false;
    }
  };
  if (fetched.request.mode === 'navigate') return ours(fetched.request.referrer);
  const client = fetched.clientId ? await clients.get(fetched.clientId) : undefined;
  const from = fetched.request.headers.get('origin');
  return !!client && ours(client.url) && (!from || from === origin);
}

// The signer is our platform's parent origin, never a URL supplied in app data.
/**
 * @param {string | URL} scope
 * @param {string} [site]
 */
export function signerFor(scope, site = '/') {
  const url = new URL(scope);
  const first = url.hostname.indexOf('.');
  if (first < 1) throw new Error('a lend must have an origin of its own');
  url.hostname = url.hostname.slice(first + 1);
  url.pathname = `${site}sign.html`;
  url.search = '';
  url.hash = '';
  return url.href;
}

// What a lent page's policy says of frames, policy by policy, which decides
// whether the signer is added to it (`framing`).
export const Framed = Object.freeze({
  Own: 'frame-src', // its own frame-src: the signer added to it
  Child: 'child-src', // frames fall back to its child-src: a frame-src of those, and the signer
  Default: 'default-src', // frames fall back to its default-src: the same
  Open: 'open', // nothing restricts frames: left as sent
});
/** @typedef {(typeof Framed)[keyof typeof Framed]} FramedBy */

// The one source lend.js adds to a lent page's policy (trust_floor.txt
// `carried-signer`): the signer at the address `signerFor` derives.
export const SIGNER_FRAME = 'frame-src {signer}';

// A source list with the signer in it: `'none'` alone gives way to it,
// as any other source would.
/**
 * @param {string[]} list
 * @param {string} signer
 */
function withSigner(list, signer) {
  // Of length 1, `list[0]` is its one source.
  const none = list.length === 1 && /** @type {string} */ (list[0]).toLowerCase() === "'none'";
  if (none) return [signer];
  return list.concat(signer);
}

// A lent app's own `Content-Security-Policy`, `header`, as the worker
// serves it: never replaced, never narrowed. The signer at `signer` is let
// in as a frame only where a policy already restricts frames — added to
// its `frame-src`, or, where frames fall back to its `child-src` or
// `default-src`, as a `frame-src` of that directive's sources and the
// signer, so frames are let in to all they were and workers keep their
// own. Each policy of a header that holds several (`, `) is judged alone,
// as a browser enforces each. No header, none: `{ policy: null }`.
/**
 * @param {string | null | undefined} header
 * @param {string} signer
 */
export function framing(header, signer) {
  if (header == null) return { policy: null, framed: [] };
  const each = header.split(',').map((one) => framedOne(one, signer));
  return {
    policy: each.map((one) => one.policy).join(','),
    framed: each.map((one) => one.by),
  };
}

// One policy of a header, `one`, with the signer let in as `framing` says,
// and which directive decided it.
/**
 * @param {string} one
 * @param {string} signer
 * @returns {{ policy: string, by: FramedBy }}
 */
function framedOne(one, signer) {
  const directives = one.split(';');
  const words = directives.map((directive) => directive.trim().split(/\s+/));
  // `split` always answers one word at least, so `first` is.
  const at = (/** @type {string} */ name) =>
    words.findIndex(([first]) => /** @type {string} */ (first).toLowerCase() === name);
  const by =
    [Framed.Own, Framed.Child, Framed.Default].find((directive) => at(directive) >= 0) ??
    Framed.Open;
  if (by === Framed.Open) return { policy: one, by };
  // Not `Open`, `by` is a directive the policy holds, so `at(by)` finds it.
  const index = at(by);
  const [name, ...was] = /** @type {string[]} */ (words[index]);
  const sources = withSigner(was, signer).join(' ');
  if (by === Framed.Child || by === Framed.Default) {
    const trimmed = one.replace(/[\s;]*$/, '');
    return { policy: `${trimmed}; ${SIGNER_FRAME.replace('{signer}', sources)}`, by };
  }
  // Its own frame-src: a list that holds the signer already is kept.
  if (!was.includes(signer)) {
    const directive = /** @type {string} */ (directives[index]);
    // `^\s*` matches every string, if only its start.
    const indent = /** @type {RegExpExecArray} */ (/^\s*/.exec(directive))[0];
    directives[index] = `${indent}${name} ${sources}`;
  }
  return { policy: directives.join(';'), by };
}

/**
 * A lend as a pass names it, in the hand-off (web/page/hand.js `handing`).
 * @typedef {object} Passed
 * @property {string} label
 * @property {string} key
 * @property {string | null} relay
 * @property {string} name
 * @property {string} kind
 * @property {string | null} group
 * @property {string} person
 * @property {string} [asks]
 * @property {number | null} version
 */

/**
 * The hand-off: what the group's page hands a lend's origin in the
 * fragment as it opens it (web/page/hand.js `handing`), read there by
 * open.html (web/open.js). `proof` is the open's seal (`sealOpen`).
 * @typedef {object} Handoff
 * @property {string} label the lend opened
 * @property {Passed[]} lends it, and each it reads as you
 * @property {string} pass
 * @property {string | null} writes the pass that writes its kind's part of the record as you
 * @property {string} [group] the group's id; none for a guest's open
 * @property {string} holder the holder's public key
 * @property {string | null} [seed] the holder's seed, as hex, when it is new; gone
 *   once open.js hands the open on with no need of it
 * @property {boolean} raw whether the holder goes as bytes
 * @property {string[]} relays every relay the group's link carries that this page's
 *   policy takes, so its worker dials the public fallback and not a route the page refuses
 * @property {string} page the group's page, to come back to
 * @property {string[]} unanswered whose computers did not answer, by name
 * @property {string} [proof]
 */

// One projection authenticates routing/authority; mutable signing state and the
// private seed are excluded. The seed must separately prove the named holder.
/** @type {(keyof Handoff)[]} */
const OPEN_FIELDS = [
  'label',
  'lends',
  'pass',
  'writes',
  'group',
  'holder',
  'raw',
  'relays',
  'page',
  'unanswered',
];
/** @param {Handoff} handoff */
export const openMessage = (handoff) =>
  JSON.stringify(Object.fromEntries(OPEN_FIELDS.map((field) => [field, handoff[field]])));

// The bytes `text`, hex, spells: as many as it has whole pairs.
/** @param {string} text */
export const fromHex = (text) =>
  Uint8Array.from(text.match(/../g) ?? [], (pair) => parseInt(pair, 16));

// An Ed25519 key's 32-byte seed as PKCS #8 (RFC 8410): the one form
// WebCrypto imports a private Ed25519 key from in both engines. The signer
// has the seed as bytes, the open page as hex (`fromHex`).
/** @param {Uint8Array} seed */
export const pkcs8 = (seed) =>
  Uint8Array.from([...fromHex('302e020100300506032b657004220420'), ...seed]);

// The seal's parameters, with the open's message as its additional data:
// a seal opens only for the open it was made for.
/**
 * @param {Handoff} handoff
 * @param {Uint8Array<ArrayBuffer>} iv
 */
const sealing = (handoff, iv) => ({
  name: 'AES-GCM',
  iv,
  additionalData: new TextEncoder().encode(openMessage(handoff)),
});

// The open's seal (svrn-docs rings/current/HOLDER_APPROVAL.md §3.D): the
// approver's 32-byte seed under a fresh key W, bound to the open's message,
// so only the open the group's page issued opens it. The seal goes in the
// app's address; W only in the group origin's `__Host-` cookie, which the
// app can neither read nor plant.
/**
 * @param {Handoff} handoff
 * @param {Uint8Array<ArrayBuffer>} seed
 */
export async function sealOpen(handoff, seed) {
  const sealKey = crypto.getRandomValues(new Uint8Array(32));
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const key = await crypto.subtle.importKey('raw', sealKey, 'AES-GCM', false, ['encrypt']);
  const sealed = await crypto.subtle.encrypt(sealing(handoff, iv), key, seed);
  return { proof: hex(iv) + hex(new Uint8Array(sealed)), w: hex(sealKey) };
}

// The seed the seal holds, opened with W; none where W or the open differ.
/**
 * @param {Handoff} handoff
 * @param {string | null} sealKey
 */
export async function openSeal(handoff, sealKey) {
  const proof = handoff.proof;
  if (typeof proof !== 'string' || !/^[0-9a-f]{24,}$/.test(proof) || proof.length % 2) return null;
  if (typeof sealKey !== 'string' || !/^[0-9a-f]{64}$/.test(sealKey)) return null;
  try {
    const key = await crypto.subtle.importKey('raw', fromHex(sealKey), 'AES-GCM', false, [
      'decrypt',
    ]);
    const all = fromHex(proof);
    return new Uint8Array(
      await crypto.subtle.decrypt(sealing(handoff, all.slice(0, 12)), key, all.slice(12)),
    );
  } catch {
    return null;
  }
}

// The cookie that hands `label`'s open to the signer: `__Host-` so only
// its own origin, over TLS, at path `/`, can have set it.
/** @param {string} label */
export const handoffCookie = (label) => `__Host-svrngs-${label}`;

// The one value of the cookie `name` in `cookies` (document.cookie), read
// by its exact name; none where it is absent or set twice.
/**
 * @param {string} cookies
 * @param {string} name
 */
export function hostCookie(cookies, name) {
  const found = cookies
    .split(';')
    .map((cookie) => cookie.trim())
    .filter((cookie) => cookie.startsWith(`${name}=`));
  // One found, `found[0]` is it.
  return found.length === 1 ? /** @type {string} */ (found[0]).slice(name.length + 1) : null;
}

// Whether `key` is the private half of `holder`, a public key as hex: it
// signs a challenge the public half verifies.
/**
 * @param {CryptoKey} key
 * @param {unknown} holder
 */
export async function holderMatches(key, holder) {
  if (typeof holder !== 'string' || !/^[0-9a-f]{64}$/.test(holder)) return false;
  // 64 hex, tested above, so `fromHex` answers 32 bytes.
  const publicKey = await crypto.subtle.importKey('raw', fromHex(holder), 'Ed25519', false, [
    'verify',
  ]);
  const challenge = crypto.getRandomValues(new Uint8Array(32));
  return crypto.subtle.verify(
    'Ed25519',
    publicKey,
    await crypto.subtle.sign('Ed25519', key, challenge),
    challenge,
  );
}
