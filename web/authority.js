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
  const client = fetched.clientId && (await clients.get(fetched.clientId));
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
// The one source lend.js adds to a lent page's policy (trust_floor.txt
// `carried-signer`): the signer at the address `signerFor` derives.
export const SIGNER_FRAME = 'frame-src {signer}';
// A source list with the signer in it: `'none'` alone gives way to it,
// as any other source would; a list that holds it already is kept.
/**
 * @param {string[]} list
 * @param {string} signer
 */
// Of length 1, `list[0]` is its one source.
const withSigner = (list, signer) =>
  (list.length === 1 && /** @type {string} */ (list[0]).toLowerCase() === "'none'"
    ? []
    : list
  ).concat(signer);
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
  /** @type {(typeof Framed)[keyof typeof Framed][]} */
  const framed = [];
  const policy = header
    .split(',')
    .map((one) => {
      const directives = one.split(';');
      const words = directives.map((directive) => directive.trim().split(/\s+/));
      // `split` always answers one word at least, so `first` is.
      const at = (/** @type {string} */ name) =>
        words.findIndex(([first]) => /** @type {string} */ (first).toLowerCase() === name);
      const by =
        [Framed.Own, Framed.Child, Framed.Default].find((directive) => at(directive) >= 0) ??
        Framed.Open;
      framed.push(by);
      if (by === Framed.Open) return one;
      // Not `Open`, `by` is a directive the policy holds, so `at(by)` finds it.
      const [name, ...was] = /** @type {string[]} */ (words[at(by)]);
      if (by === Framed.Own) {
        if (!was.includes(signer))
          directives[at(by)] =
            // `^\s*` matches every string, if only its start.
            `${/** @type {RegExpExecArray} */ (/^\s*/.exec(/** @type {string} */ (directives[at(by)])))[0]}${name} ${withSigner(was, signer).join(' ')}`;
        return directives.join(';');
      }
      return `${one.replace(/[\s;]*$/, '')}; ${SIGNER_FRAME.replace('{signer}', withSigner(was, signer).join(' '))}`;
    })
    .join(',');
  return { policy, framed };
}

/**
 * A lend as a pass names it, in the hand-off (web/page/index.js `handing`).
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
 * fragment as it opens it (web/page/index.js `handing`), read there by
 * open.html (web/open.js). `proof` is the open's seal (`sealOpen`).
 * @typedef {object} Handoff
 * @property {string} label the lend opened
 * @property {Passed[]} lends it, and each it reads as you
 * @property {string} pass
 * @property {string | null} writes the pass that writes its kind's part of the record as you
 * @property {string} [group] the group's id; none for a guest's open
 * @property {string} holder the holder's public key
 * @property {string | null} seed the holder's seed, as hex, when it is new
 * @property {boolean} raw whether the holder goes as bytes
 * @property {string | null} relay
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
  'relay',
  'page',
  'unanswered',
];
/** @param {Handoff} handoff */
export const openMessage = (handoff) =>
  JSON.stringify(Object.fromEntries(OPEN_FIELDS.map((field) => [field, handoff[field]])));
/** @param {Uint8Array} bytes */
const hex = (bytes) => [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');
/** @param {string} text */
const bytes = (text) => Uint8Array.from(text.match(/../g) ?? [], (pair) => parseInt(pair, 16));
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
  return {
    proof:
      hex(iv) + hex(new Uint8Array(await crypto.subtle.encrypt(sealing(handoff, iv), key, seed))),
    w: hex(sealKey),
  };
}
// The seed the seal holds, opened with W; none where W or the open differ.
/**
 * @param {Handoff} handoff
 * @param {string | null} sealKey
 */
export async function openSeal(handoff, sealKey) {
  if (
    typeof handoff.proof !== 'string' ||
    !/^[0-9a-f]{24,}$/.test(handoff.proof) ||
    handoff.proof.length % 2 ||
    !/^[0-9a-f]{64}$/.test(sealKey ?? '')
  )
    return null;
  try {
    // Tested above, `sealKey` is 64 hex.
    const key = await crypto.subtle.importKey(
      'raw',
      bytes(/** @type {string} */ (sealKey)),
      'AES-GCM',
      false,
      ['decrypt'],
    );
    const all = bytes(handoff.proof);
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
/**
 * @param {CryptoKey} key
 * @param {unknown} holder
 */
export async function holderMatches(key, holder) {
  if (typeof holder !== 'string' || !/^[0-9a-f]{64}$/.test(holder)) return false;
  const publicKey = await crypto.subtle.importKey(
    'raw',
    // 64 hex, tested above, so `match` finds 32 pairs.
    Uint8Array.from(/** @type {RegExpMatchArray} */ (holder.match(/../g)), (pair) =>
      parseInt(pair, 16),
    ),
    'Ed25519',
    false,
    ['verify'],
  );
  const challenge = crypto.getRandomValues(new Uint8Array(32));
  return crypto.subtle.verify(
    'Ed25519',
    publicKey,
    await crypto.subtle.sign('Ed25519', key, challenge),
    challenge,
  );
}
