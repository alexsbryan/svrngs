// SPDX-License-Identifier: AGPL-3.0-or-later
// The page's store (DESIGN §3, "Joining", call 1): what a phone keeps of
// each group it joined, in IndexedDB `svrngs`.
//
//   seal     `key`: the page's sealing key, AES-GCM 256, made once with
//            `extractable: false`, so no script can read its bytes out,
//            by the page when it first keeps a group, never by its worker;
//            and beside it each visit made here, `visit <token>` (below).
//   groups   by `id`: the group's name, the person's name in it, the link's
//            token it was joined by, when it was last opened, and the
//            person's key — 32 bytes — sealed: `iv` and `sealed`,
//            with the group's id as the seal's additional data, so a sealed
//            key opens only as the key of the group it was kept for; and
//            once opened, `computers`, whose each lending computer of the
//            last answer is, its key and relay, which the page reaches the
//            group through when its link's does not answer; `meets`, the
//            relays its keeper names, dialled beside the link's; and `maker`,
//            who made the group, and `since`, when this phone joined it,
//            absent on a group kept before the page kept them.
//
// A lend's origin keeps a store of its own, `svrngs-lend` (below): what its
// worker dials with, never the person's key, which no lend's origin holds
// (DESIGN §3, "Lent apps", call 6). Its pass and the pass's key are
// bytes the lend's own pages can read, trusted as the lend's pinned code
// is, until the pass lapses (DESIGN §8, risk 5): the worker cannot keep
// them alone, since it shares the origin and comes back through open.html
// (open.js `handedKept`). A key that cannot be copied would end that trust.
//
// The key is bytes, never a WebCrypto signing key: iroh signs its handshake
// synchronously (iroh-1.0.2/src/tls/resolver.rs:92-95), and the issuer lets
// in only the key that dials it. So storage holds only ciphertext, and the
// page's own code can still unseal it — why a group trusts the origin it
// names with this page's code (DESIGN §8, risk 5). One key per person per
// group (DESIGN §6). Version 1 of the database held one key unsealed, a
// test dial's from before a phone could join, never a person's: the upgrade
// drops it.
const DB = 'svrngs';
const VERSION = 2;

/**
 * A computer lending in a group, as the last read of it named it: its key,
 * the relay it is reached through, and whose it is.
 * @typedef {{ key: string, relay: string | null, person: string }} Computer
 */

/**
 * A yes or a stop said here to what a lend reads as the person, and when,
 * in seconds (web/page/hand.js `handing`, web/page/acts.js `stopping`).
 * @typedef {{ said: 'yes' | 'stop', at: number, lends?: string[] }} Said
 */

/**
 * A line of the person's chain this phone signed (web/page/acts.js `runs`):
 * its number and its id, the next line's `prev`, among what svrngs-core
 * wrote in it.
 * @typedef {{ seq: number, id: string }} Line
 */

/**
 * A group as `keep` is given it, the person's key in the clear.
 * @typedef {object} Joining
 * @property {string} id
 * @property {string} name
 * @property {string} person
 * @property {string} token the link's token it was joined by
 * @property {Uint8Array<ArrayBuffer>} key the person's 32 bytes
 * @property {string} [maker]
 * @property {number} [since]
 */

/**
 * A group as kept: `keep`'s, its key sealed; and once opened, what the
 * page learned of it, written back by `opened`.
 * @typedef {object} Group
 * @property {string} id
 * @property {string} name
 * @property {string} person
 * @property {string} token
 * @property {Uint8Array<ArrayBuffer>} iv
 * @property {Uint8Array<ArrayBuffer>} sealed
 * @property {number} opened when it was last opened, in ms
 * @property {string} [maker]
 * @property {number} [since]
 * @property {Computer[]} [computers]
 * @property {string[]} [meets] the relays the keeper says it meets at, as last read; none until a computer names them
 * @property {string[]} [addresses] each lend's label and alias, as last read
 * @property {Record<string, Said>} [reads] by a lend's label
 * @property {Line} [line] the last line signed here
 * @property {Line[]} [unsent] the lines signed and not yet carried
 */

/**
 * A visit as the host's computer answered it (web/page/visit.js
 * `visiting`): whose guest, until when, in seconds, and what it opens, each
 * lend's kind by its name.
 * @typedef {{ host: string, until: number, lent: Record<string, string>, version?: number }} Visit
 */

/** @returns {Promise<IDBDatabase>} */
function open() {
  return openDb(DB, VERSION, (db) => {
    // A copy: objectStoreNames is live, and deleting while walking it skips names.
    for (const name of Array.from(db.objectStoreNames)) db.deleteObjectStore(name);
    db.createObjectStore('seal');
    db.createObjectStore('groups', { keyPath: 'id' });
  });
}

// The database `name` at `version`, `upgrade` given it when it is new or older.
/**
 * @param {string} name
 * @param {number} version
 * @param {(db: IDBDatabase) => void} upgrade
 * @returns {Promise<IDBDatabase>}
 */
function openDb(name, version, upgrade) {
  return new Promise((done, fail) => {
    const request = indexedDB.open(name, version);
    request.onupgradeneeded = () => upgrade(request.result);
    request.onsuccess = () => done(request.result);
    request.onerror = () => fail(request.error);
  });
}

// One request in one transaction of the database `opening` opens, the
// page's own unless a lend's origin names its own, answered once it commits.
/**
 * @template T
 * @param {string} store
 * @param {IDBTransactionMode} mode
 * @param {(store: IDBObjectStore) => IDBRequest<T>} act
 * @param {() => Promise<IDBDatabase>} [opening]
 * @returns {Promise<T>}
 */
async function run(store, mode, act, opening = open) {
  const db = await opening();
  return new Promise((done, fail) => {
    const transaction = db.transaction(store, mode);
    const request = act(transaction.objectStore(store));
    transaction.oncomplete = () => {
      db.close();
      done(request.result);
    };
    const refused = () => {
      db.close();
      fail(transaction.error ?? new Error('this browser’s storage refused'));
    };
    transaction.onabort = refused;
    transaction.onerror = refused;
  });
}

// One request on this origin's store `name`, its one object store kept by
// `keyPath`: the group's signer's (sign.js).
/**
 * @template T
 * @param {string} name
 * @param {string} keyPath
 * @param {IDBTransactionMode} mode
 * @param {(store: IDBObjectStore) => IDBRequest<T>} act
 * @returns {Promise<T>}
 */
export const stored = (name, keyPath, mode, act) =>
  run('kept', mode, act, () => openDb(name, 1, (db) => db.createObjectStore('kept', { keyPath })));

// What the `seal` store keeps under `name`, of whatever kind it was kept.
/** @param {string} name */
const inSeal = (name) => run('seal', 'readonly', (objects) => objects.get(name));

// What a sealing key read back is, for the trace: Safari's worker on an
// iPhone failed to unseal with `OperationError` while its page and Firefox's
// worker did not (2026-10-05), and a key read back unusable and a key made
// anew over the page's fail the same way.
/** @param {any} key what was read back, of no type the page can know */
const described = (key) => {
  const kind = `${key?.constructor?.name} ${key?.algorithm?.name}/${key?.algorithm?.length}`;
  return `${kind} extractable=${key?.extractable} usages=${key?.usages}`;
};

// The sealing key, made when none is kept only if `make`: only `keep`
// makes one, and only the page keeps a group. Safari's worker on the
// operator's iPhone read none where its page had kept one (2026-10-05;
// `STORE: the sealing key absent: making one` in its trace, in
// demo/runs/u4/2026-10-05-phones/), made its own and put it over the
// page's, and so left every key the page had sealed unopenable.
/**
 * @param {boolean} make
 * @returns {Promise<CryptoKey>}
 */
async function sealingKey(make) {
  const held = /** @type {CryptoKey | undefined} */ (await inSeal('key'));
  if (held) {
    globalThis.__trace?.(`STORE: the sealing key read back: ${described(held)}`);
    return held;
  }
  if (!make) {
    globalThis.__trace?.(
      'STORE: the sealing key absent, and only the page makes one: nothing is unsealed here',
    );
    throw new Error('no sealing key is kept');
  }
  globalThis.__trace?.('STORE: the sealing key absent: making one');
  const made = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, [
    'encrypt',
    'decrypt',
  ]);
  await run('seal', 'readwrite', (objects) => objects.put(made, 'key'));
  return made;
}

// The seal's parameters, with `id` as its additional data: what is sealed
// under one id does not open under another.
/**
 * @param {string} id
 * @param {Uint8Array<ArrayBuffer>} iv
 * @returns {AesGcmParams}
 */
const bound = (id, iv) => ({ name: 'AES-GCM', iv, additionalData: new TextEncoder().encode(id) });

// Keep a group joined: `key` is the person's 32 bytes, kept sealed alone;
// `maker`, who made the group, and `since`, when this phone joined it, in
// seconds, so two of one name are told apart.
/** @param {Joining} joining */
export async function keep({ id, name, person, token, key, maker, since }) {
  const kept = await sealed(id, key);
  return run('groups', 'readwrite', (objects) =>
    objects.put({ id, name, person, token, ...kept, opened: Date.now(), maker, since }),
  );
}

// `key` sealed, bound to `id`: its `iv` and `sealed`.
/**
 * @param {string} id
 * @param {Uint8Array<ArrayBuffer>} key
 * @returns {Promise<{ iv: Uint8Array<ArrayBuffer>, sealed: Uint8Array<ArrayBuffer> }>}
 */
async function sealed(id, key) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const seal = await sealingKey(true);
  return { iv, sealed: new Uint8Array(await crypto.subtle.encrypt(bound(id, iv), seal, key)) };
}

// A visit (svrn-docs rings/current/GUESTS.md §3.9), kept never as a group, so no group
// lists it: beside the sealing key, under `visit <token>`, what the host's
// computer answered and the visit's key, sealed as a group's is, bound to
// the link's token. So the link opened again shows the visit again, or
// says it has ended.
/**
 * @param {string} token
 * @param {Visit} visit
 * @param {Uint8Array<ArrayBuffer>} key
 */
export async function keepVisit(token, visit, key) {
  const kept = await sealed(token, key);
  return run('seal', 'readwrite', (objects) =>
    objects.put({ ...visit, ...kept }, `visit ${token}`),
  );
}

// The visit the link `token` made here, its key unsealed; else undefined.
/**
 * @param {string} token
 * @returns {Promise<(Visit & { key: Uint8Array<ArrayBuffer> }) | undefined>}
 */
export async function visitOf(token) {
  /** @typedef {Visit & Awaited<ReturnType<typeof sealed>>} Kept */
  const visit = /** @type {Kept | undefined} */ (await inSeal(`visit ${token}`));
  if (!visit) return undefined;
  return { ...visit, key: await unseal({ ...visit, id: token }) };
}

// The holder a lend's origin keeps, its public key by the lend's label
// (DESIGN §3), beside the sealing key, apart from
// any group's row, so no write of one drops it: `key` kept, `null`
// dropped, or with none read, the key kept answered (a write answers none).
/**
 * @param {string} label
 * @param {string | null} [key]
 * @returns {Promise<string | null | undefined>}
 */
export async function holder(label, key) {
  if (key === undefined) {
    return inSeal(`holder ${label}`);
  }
  await run('seal', 'readwrite', (objects) => objects.put(key, `holder ${label}`));
  return undefined;
}

// Whether the browser keeps this page's storage, asking it to where it
// does not yet: a Home Screen web app on the operator's iPhone (iOS 18.7,
// 2026-10-31) forgot its person within some ten hours, its `groups` store
// empty and nothing here deleting one, and there `navigator.storage`
// answered persisted, persist, persisted with `[false, true, true]`: iOS
// keeps it when asked, and the page had not asked. Answered as the page's
// trace says it, a browser that offers no asking among them.
// `navigator.storage` is absent where the page is not a secure context
// (the Storage standard's `[SecureContext]`), and `persist` in a worker
// (`[Exposed=Window]`).
/** @typedef {{ persisted(): Promise<boolean>, persist?: () => Promise<boolean> }} Keeping */
export async function persist() {
  const storage = /** @type {Keeping | undefined} */ (navigator.storage);
  if (!storage?.persist) return "this browser offers no way to ask it to keep the page's storage";
  if (await storage.persisted()) return "the browser keeps the page's storage already";
  const granted = await storage.persist();
  return `asked the browser to keep the page's storage: ${granted ? 'granted' : 'refused'}`;
}

// `label`'s origin holds this build's every piece, as its signer was told.
/** @param {string} label */
export const keepSaved = (label) =>
  run('seal', 'readwrite', (objects) => objects.put('__VERSION__', `saved ${label}`));

// Said while the browser has not been let reach the home network
// (KEEP_GOING_AT_HOME 21): Chromium's permission, which others refuse to name.
const LOCAL =
  'This browser asks before it reaches devices on your home network. Allow it, so this phone can find your group when the internet is out.';

// What keeps this phone from going on at home with the apps `lent`: none when
// it is ready (KEEP_GOING_AT_HOME 29). The group itself is `opened`'s, which
// refuses first if it cannot be kept. Each is what a check found, never that
// something was once opened: an app's mark is its signer's, set when that
// origin held all of the signer's own build, so a new build asks it again.
/**
 * @template {{ label: string }} L
 * @param {L[]} lent
 * @param {(lend: L) => string} named how a person calls one
 * @returns {Promise<string[]>}
 */
export async function unready(lent, named) {
  const apps = await Promise.all(lent.map((lend) => inSeal(`saved ${lend.label}`)));
  const keeps = await navigator.storage.persisted();
  const local = await navigator.permissions
    .query({ name: /** @type {PermissionName} */ ('local-network-access') })
    .catch(() => null);
  return [
    ...lent
      .filter((_, at) => apps[at] !== '__VERSION__')
      .map((lend) => `${named(lend)} is not saved here: open it once while online.`),
    ...(keeps ? [] : ['The browser has not agreed to keep what this page saves.']),
    ...(local && local.state !== 'granted' ? [LOCAL] : []),
  ];
}

// Every group kept, the last opened first. Their keys stay sealed.
export async function groups() {
  const kept = await run(
    'groups',
    'readonly',
    (objects) => /** @type {IDBRequest<Group[]>} */ (objects.getAll()),
  );
  return kept.sort((left, right) => right.opened - left.opened);
}

// The group was opened now, as `group` says it: it is the one the page
// opens without a link. Answers the group as kept.
/**
 * @param {Group} group
 * @returns {Promise<Group>}
 */
export async function opened(group) {
  const now = { ...group, opened: Date.now() };
  await run('groups', 'readwrite', (objects) => objects.put(now));
  return now;
}

// The person's key in `group`, unsealed: 32 bytes, held only in memory.
/** @param {{ id: string, iv: Uint8Array<ArrayBuffer>, sealed: Uint8Array<ArrayBuffer> }} group */
export async function unseal(group) {
  const seal = await sealingKey(false);
  try {
    return new Uint8Array(
      await crypto.subtle.decrypt(bound(group.id, group.iv), seal, group.sealed),
    );
  } catch (/** @type {any} */ error) {
    globalThis.__trace?.(unsealing(group, error));
    throw error;
  }
}

// Why unsealing `group`'s key failed, and the sizes and kinds of what it
// was given, for the trace.
/**
 * @param {{ iv?: any, sealed?: any }} group as the store held it, of whatever kinds
 * @param {any} error
 */
const unsealing = (group, error) => {
  const sealedWas = `sealed ${group.sealed?.byteLength} bytes (${group.sealed?.constructor?.name})`;
  const ivWas = `iv ${group.iv?.byteLength} bytes (${group.iv?.constructor?.name})`;
  return `STORE: unsealing failed: ${error?.name}: ${error?.message}; ${sealedWas}, ${ivWas}`;
};

// What a lend's worker tells its pages when a message to it was lost.
export const LOST = 'a message was lost on its way to the worker';

// How long, in ms, a worker has to say it holds what it was handed.
const HANDING = 5_000;

// Hand `message` to `worker` in memory, never storage (Safari's worker on
// an iPhone read nothing its page had kept, 2026-10-05); answers whether it
// said it holds it within HANDING, or no at once when it says a message
// was lost on its way (lend.js, `messageerror`).
/**
 * @param {ServiceWorker} worker
 * @param {unknown} message
 * @returns {Promise<boolean>}
 */
export function handOver(worker, message) {
  const { port1, port2 } = new MessageChannel();
  const held = new Promise((done) => {
    port1.onmessage = () => done(true);
    navigator.serviceWorker.addEventListener('message', (received) => {
      if (received.data !== LOST) return;
      console.debug(`STORE: the worker said ${LOST}`);
      done(false);
    });
    setTimeout(() => done(false), HANDING);
  });
  worker.postMessage(message, [port2]);
  return held;
}

// ── a lend's origin ──────────────────────────────────────────────────
//
// What a lend's origin keeps, by name, in IndexedDB `svrngs-lend`: what
// the page handed it (`pass <label>`: the pass, its holder's key, the
// lends it opens, the relay and the page to ask for another), the
// cookies its worker keeps for each lend (`cookies <label>`), an app's
// own worker (`worker <label>`), and the worker's trace (`trace <label>`,
// lend.js). Never the person's key.
const lendDb = () => openDb('svrngs-lend', 1, (db) => db.createObjectStore('kept'));

/** @param {string} name */
export const lendKept = (name) => run('kept', 'readonly', (objects) => objects.get(name), lendDb);
/**
 * @param {string} name
 * @param {unknown} value
 */
export const lendKeep = (name, value) =>
  run('kept', 'readwrite', (objects) => objects.put(value, name), lendDb);

// Bytes as hex, as the page hands a holder's seed and the worker signs a line.
/** @param {Uint8Array} bytes */
export const hex = (bytes) => [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');

// A lend's address on a phone, `<label>.<the page's host>`: built by the
// page for a lend and by a lend's worker for another, read back by its
// first name — here alone in the page's files (ralph/REVIEW_FINDINGS.md #49).
/**
 * @param {string} protocol
 * @param {string} label
 * @param {string} pageHost
 */
export const lendOrigin = (protocol, label, pageHost) => `${protocol}//${label}.${pageHost}`;

/** @param {string} hostname */
export function labelOf(hostname) {
  // `split` answers one part at least.
  return /** @type {string} */ (hostname.split('.')[0]);
}

// ── the runtime's module ────────────────────────────────────────────────
//
// The wasm, fetched once a version (ralph/REVIEW_FINDINGS.md #51). A cold
// join once took it three times: the page fetched it as wasm-bindgen's glue
// names it, without the version; the worker's install cached it with `?v=`;
// and the worker's `init()` fetched it again, since a worker's own fetches
// never pass through its cache. Now it has one URL, with the version, and
// one entry in the version's cache, which whoever asks first fills — the
// page, on a cold join — and the worker reads. __VERSION__ is stamped at
// build time, as in sw.js.
export const CACHES = 'svrngs-web-';
export const CACHE = `${CACHES}__VERSION__`;
// What a lend's origin saves, so a phone reopens with the public site gone
// (KEEP_GOING_AT_HOME 10), and one piece's address, this build's.
export const SAVED =
  'open.html open.js lend.js shim.js store.js authority.js away.html wasm/svrngs_web.js wasm/svrngs_web_bg.wasm'.split(
    ' ',
  );
/** @param {string} file */
export const savedAt = (file) => new URL(`./${file}?v=__VERSION__`, import.meta.url).href;
// Whether all of them are here, at this origin.
export const whole = async () => {
  const cache = await caches.open(CACHE);
  return (await Promise.all(SAVED.map((file) => cache.match(savedAt(file))))).every(Boolean);
};
const RUNTIME = new URL('./wasm/svrngs_web_bg.wasm?v=__VERSION__', import.meta.url);

// The module's `Response`, for wasm-bindgen's `init({ module_or_path })`:
// from the version's cache, else fetched and kept there. A page a worker
// serves asks the worker, which answers from its cache: an older worker,
// before it is replaced, answers with its own version's module, which must
// not be kept under this version's URL. Where the browser gives no cache
// (an origin that is not https), fetched each time.
export async function runtime() {
  // Both are absent where the origin is not secure: a service worker and the
  // cache are `[SecureContext]` (Service Workers, "navigator.serviceWorker"
  // and "caches").
  const workers = /** @type {ServiceWorkerContainer | undefined} */ (self.navigator.serviceWorker);
  if (workers?.controller) return fetch(RUNTIME);
  const cacheStorage = /** @type {CacheStorage | undefined} */ (self.caches);
  const cache = await cacheStorage?.open(CACHE).catch(() => null);
  const hit = await cache?.match(RUNTIME);
  if (hit) return hit;
  const response = await fetch(RUNTIME);
  if (cache && response.ok) {
    cache
      .put(RUNTIME, response.clone())
      .catch((error) => console.debug(`STORE: the runtime's module was not kept: ${error}`));
  }
  return response;
}
