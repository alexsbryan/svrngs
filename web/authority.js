// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// Whether a request may cause a write signed as the person: one from a page
// of this origin, by the browser's Client, never a body's word; or a
// navigation one of them began, a form's post or a link (Events' "I came",
// a page with no script), by its referrer, which the browser sets from the
// page that began it and another site can drop but never make this origin's.
export async function maySign(fetched, clients, origin) {
  const ours = (url) => {
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
const withSigner = (list, signer) =>
  (list.length === 1 && list[0].toLowerCase() === "'none'" ? [] : list).concat(signer);
// A lent app's own `Content-Security-Policy`, `header`, as the worker
// serves it: never replaced, never narrowed. The signer at `signer` is let
// in as a frame only where a policy already restricts frames — added to
// its `frame-src`, or, where frames fall back to its `child-src` or
// `default-src`, as a `frame-src` of that directive's sources and the
// signer, so frames are let in to all they were and workers keep their
// own. Each policy of a header that holds several (`, `) is judged alone,
// as a browser enforces each. No header, none: `{ policy: null }`.
export function framing(header, signer) {
  if (header == null) return { policy: null, framed: [] };
  const framed = [];
  const policy = header
    .split(',')
    .map((one) => {
      const directives = one.split(';');
      const words = directives.map((d) => d.trim().split(/\s+/));
      const at = (name) => words.findIndex(([n]) => n.toLowerCase() === name);
      const by = [Framed.Own, Framed.Child, Framed.Default].find((d) => at(d) >= 0) ?? Framed.Open;
      framed.push(by);
      if (by === Framed.Open) return one;
      const [name, ...was] = words[at(by)];
      if (by === Framed.Own) {
        if (!was.includes(signer))
          directives[at(by)] =
            `${/^\s*/.exec(directives[at(by)])[0]}${name} ${withSigner(was, signer).join(' ')}`;
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
export const openMessage = (h) =>
  JSON.stringify(Object.fromEntries(OPEN_FIELDS.map((k) => [k, h[k]])));
const hex = (bytes) => [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
const bytes = (text) => Uint8Array.from(text.match(/../g) ?? [], (s) => parseInt(s, 16));
const sealing = (h, iv) => ({
  name: 'AES-GCM',
  iv,
  additionalData: new TextEncoder().encode(openMessage(h)),
});
// The open's seal (svrn-docs rings/current/HOLDER_APPROVAL.md §3.D): the
// approver's 32-byte seed under a fresh key W, bound to the open's message,
// so only the open the group's page issued opens it. The seal goes in the
// app's address; W only in the group origin's `__Host-` cookie, which the
// app can neither read nor plant.
export async function sealOpen(h, seed) {
  const w = crypto.getRandomValues(new Uint8Array(32)),
    iv = crypto.getRandomValues(new Uint8Array(12));
  const key = await crypto.subtle.importKey('raw', w, 'AES-GCM', false, ['encrypt']);
  return {
    proof: hex(iv) + hex(new Uint8Array(await crypto.subtle.encrypt(sealing(h, iv), key, seed))),
    w: hex(w),
  };
}
// The seed the seal holds, opened with W; none where W or the open differ.
export async function openSeal(h, w) {
  if (
    typeof h.proof !== 'string' ||
    !/^[0-9a-f]{24,}$/.test(h.proof) ||
    h.proof.length % 2 ||
    !/^[0-9a-f]{64}$/.test(w ?? '')
  )
    return null;
  try {
    const key = await crypto.subtle.importKey('raw', bytes(w), 'AES-GCM', false, ['decrypt']);
    const all = bytes(h.proof);
    return new Uint8Array(
      await crypto.subtle.decrypt(sealing(h, all.slice(0, 12)), key, all.slice(12)),
    );
  } catch {
    return null;
  }
}
// The cookie that hands `label`'s open to the signer: `__Host-` so only
// its own origin, over TLS, at path `/`, can have set it.
export const handoffCookie = (label) => `__Host-svrngs-${label}`;
// The one value of the cookie `name` in `cookies` (document.cookie), read
// by its exact name; none where it is absent or set twice.
export function hostCookie(cookies, name) {
  const found = cookies
    .split(';')
    .map((c) => c.trim())
    .filter((c) => c.startsWith(`${name}=`));
  return found.length === 1 ? found[0].slice(name.length + 1) : null;
}
export async function holderMatches(key, holder) {
  if (typeof holder !== 'string' || !/^[0-9a-f]{64}$/.test(holder)) return false;
  const publicKey = await crypto.subtle.importKey(
    'raw',
    Uint8Array.from(holder.match(/../g), (s) => parseInt(s, 16)),
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
