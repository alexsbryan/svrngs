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
//            group through when its link's does not answer (v1ui-reach (c));
//            and since v1ui-door (d), `maker`, who made the group, and
//            `since`, when this phone joined it, absent on one kept before.
//
// A lend's origin keeps a store of its own, `svrngs-lend` (below): what its
// worker dials with, never the person's key, which no lend's origin holds
// (u5-2; DESIGN §3, "Lent apps", call 6). Its pass and the pass's key are
// bytes the lend's own pages can read, trusted as the lend's pinned code
// is, until the pass lapses (DESIGN §8, risk 5): the worker cannot keep
// them alone, since it shares the origin and comes back through open.html
// (open.js:46). A key that cannot be copied lands with capability-signed.
//
// The key is bytes, never a WebCrypto signing key: iroh signs its handshake
// synchronously (iroh-1.0.2/src/tls/resolver.rs:92-95), and the issuer lets
// in only the key that dials it. So storage holds only ciphertext, and the
// page's own code can still unseal it — why a group trusts the origin it
// names with this page's code (DESIGN §8, risk 5). One key per person per
// group (DESIGN §6). Version 1 held one key unsealed (u4-3's dial, before a
// phone could join), never a person's: the upgrade drops it.
const DB = 'svrngs';
const VERSION = 2;

function open() {
  return openDb(DB, VERSION, (db) => {
    // A copy: objectStoreNames is live, and deleting while walking it skips names.
    // oxlint-disable-next-line unicorn/no-useless-spread
    for (const name of [...db.objectStoreNames]) db.deleteObjectStore(name);
    db.createObjectStore('seal');
    db.createObjectStore('groups', { keyPath: 'id' });
  });
}

// The database `name` at `version`, `upgrade` given it when it is new or older.
function openDb(name, version, upgrade) {
  return new Promise((done, fail) => {
    const req = indexedDB.open(name, version);
    req.onupgradeneeded = () => upgrade(req.result);
    req.onsuccess = () => done(req.result);
    req.onerror = () => fail(req.error);
  });
}

// One request in one transaction of `db`, answered once it commits.
async function run(store, mode, act, db = open) {
  db = await db();
  return new Promise((done, fail) => {
    const tx = db.transaction(store, mode);
    const req = act(tx.objectStore(store));
    tx.oncomplete = () => {
      db.close();
      done(req.result);
    };
    tx.onerror = () => fail(tx.error);
  });
}

// What a sealing key read back is, for the trace: Safari's worker on an
// iPhone failed to unseal with `OperationError` while its page and Firefox's
// worker did not (2026-10-05), and a key read back unusable and a key made
// anew over the page's fail the same way.
const described = (k) =>
  `${k?.constructor?.name} ${k?.algorithm?.name}/${k?.algorithm?.length} extractable=${k?.extractable} usages=${k?.usages}`;

// The sealing key, made when none is kept only if `make`: only `keep`
// makes one, and only the page keeps a group. Safari's worker on the
// operator's iPhone read none where its page had kept one (2026-10-05,
// u4-10; `STORE: the sealing key absent: making one` in its trace, in
// demo/runs/u4/2026-10-05-phones/), made its own and put it over the
// page's, and so left every key the page had sealed unopenable.
async function sealingKey(make) {
  const held = await run('seal', 'readonly', (s) => s.get('key'));
  if (held) {
    globalThis.__trace?.(`STORE: the sealing key read back: ${described(held)}`);
    return held;
  }
  if (!make) {
    globalThis.__trace?.('STORE: the sealing key absent, and only the page makes one: nothing is unsealed here');
    throw new Error('no sealing key is kept');
  }
  globalThis.__trace?.('STORE: the sealing key absent: making one');
  const made = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt']);
  await run('seal', 'readwrite', (s) => s.put(made, 'key'));
  return made;
}

const bound = (id, iv) => ({ name: 'AES-GCM', iv, additionalData: new TextEncoder().encode(id) });

// Keep a group joined: `key` is the person's 32 bytes, kept sealed alone;
// `maker`, who made the group, and `since`, when this phone joined it, in
// seconds, so two of one name are told apart (v1ui-door (d)).
export const keep = ({ id, name, person, token, key, maker, since }) => sealed(id, key).then((kept) => run('groups', 'readwrite', (s) => s.put({ id, name, person, token, ...kept, opened: Date.now(), maker, since })));

// `key` sealed, bound to `id`: its `iv` and `sealed`.
const sealed = async (id, key, iv = crypto.getRandomValues(new Uint8Array(12))) => ({ iv, sealed: new Uint8Array(await crypto.subtle.encrypt(bound(id, iv), await sealingKey(true), key)) });

// A visit (guests-link; GUESTS §3.9), kept never as a group, so no group
// lists it: beside the sealing key, under `visit <token>`, what the host's
// computer answered and the visit's key, sealed as a group's is, bound to
// the link's token. So the link opened again shows the visit again, or
// says it has ended.
export const keepVisit = (token, visit, key) => sealed(token, key).then((kept) => run('seal', 'readwrite', (s) => s.put({ ...visit, ...kept }, `visit ${token}`)));

// The visit the link `token` made here, its key unsealed; else undefined.
export const visitOf = (token) => run('seal', 'readonly', (s) => s.get(`visit ${token}`)).then(async (v) => v && { ...v, key: await unseal({ ...v, id: token }) });

// The holder a lend's origin keeps, its public key by the lend's label
// (capability-signed (H); DESIGN §3), beside the sealing key, apart from
// any group's row, so no write of one drops it: `key` kept, `null`
// dropped, or with none read.
export const holder = (label, key) =>
  key === undefined ? run('seal', 'readonly', (s) => s.get(`holder ${label}`)) : run('seal', 'readwrite', (s) => s.put(key, `holder ${label}`));

// Every group kept, the last opened first. Their keys stay sealed.
export const groups = async () => (await run('groups', 'readonly', (s) => s.getAll())).sort((a, b) => b.opened - a.opened);

// The group was opened now, as `group` says it: it is the one the page
// opens without a link. Answers the group as kept.
export const opened = (group, now = { ...group, opened: Date.now() }) => run('groups', 'readwrite', (s) => s.put(now)).then(() => now);

// The person's key in `group`, unsealed: 32 bytes, held only in memory.
export async function unseal(group) {
  const seal = await sealingKey(false);
  try {
    return new Uint8Array(await crypto.subtle.decrypt(bound(group.id, group.iv), seal, group.sealed));
  } catch (e) {
    globalThis.__trace?.(
      `STORE: unsealing failed: ${e?.name}: ${e?.message}; sealed ${group.sealed?.byteLength} bytes (${group.sealed?.constructor?.name}), iv ${group.iv?.byteLength} bytes (${group.iv?.constructor?.name})`,
    );
    throw e;
  }
}

// What a lend's worker tells its pages when a message to it was lost.
export const LOST = 'a message was lost on its way to the worker';

// Hand `message` to `worker` in memory, never storage (u4-10: Safari's
// worker on an iPhone read nothing its page had kept); answers whether it
// said it holds it within `wait` ms, or no at once when it says a message
// was lost on its way (lend.js, `messageerror`).
export function handOver(worker, message, wait = 5_000) {
  const { port1, port2 } = new MessageChannel();
  const held = new Promise((done) => {
    port1.onmessage = () => done(true);
    navigator.serviceWorker.addEventListener('message', (e) => e.data === LOST && (console.debug(`STORE: the worker said ${LOST}`), done(false)));
    setTimeout(() => done(false), wait);
  });
  worker.postMessage(message, [port2]);
  return held;
}

// ── a lend's origin (u5-2) ─────────────────────────────────────────────
//
// What a lend's origin keeps, by name, in IndexedDB `svrngs-lend`: what
// the page handed it (`pass <label>`: the pass, its holder's key, the
// lends it opens, the relay and the page to ask for another), the
// cookies its worker keeps for each lend (`cookies <label>`), an app's
// own worker (`worker <label>`), and the worker's trace (`trace <label>`,
// lend.js). Never the person's key.
const lendDb = () => openDb('svrngs-lend', 1, (db) => db.createObjectStore('kept'));

export const lendKept = (name) => run('kept', 'readonly', (s) => s.get(name), lendDb);
export const lendKeep = (name, value) => run('kept', 'readwrite', (s) => s.put(value, name), lendDb);

// ── the runtime's module ────────────────────────────────────────────────
//
// The wasm, fetched once a version (u4-9; ralph/REVIEW_FINDINGS.md #51). A
// cold join took it three times in u4-8's dry run: the page fetched it as
// wasm-bindgen's glue names it, without the version; the worker's install
// cached it with `?v=`; and the worker's `init()` fetched it again, since
// a worker's own fetches never pass through its cache. Now it has one URL,
// with the version, and one entry in the version's cache, which whoever
// asks first fills — the page, on a cold join — and the worker reads.
// __VERSION__ is stamped at build time, as in sw.js.
// A lend's address on a phone, `<label>.<the page's host>` (#49): built by
// the page for a lend and by a lend's worker for another, read back by its
// first name — here alone in the page's files.
export const lendOrigin = (protocol, label, pageHost) => `${protocol}//${label}.${pageHost}`;
export const labelOf = (hostname) => hostname.split('.')[0];

export const CACHE = 'svrngs-web-__VERSION__';
const RUNTIME = new URL('./wasm/svrngs_web_bg.wasm?v=__VERSION__', import.meta.url);

// The module's `Response`, for wasm-bindgen's `init({ module_or_path })`:
// from the version's cache, else fetched and kept there. A page a worker
// serves asks the worker, which answers from its cache: an older worker,
// before it is replaced, answers with its own version's module, which must
// not be kept under this version's URL. Where the browser gives no cache
// (an origin that is not https), fetched each time.
export async function runtime() {
  if (self.navigator?.serviceWorker?.controller) return fetch(RUNTIME);
  const cache = await self.caches?.open(CACHE).catch(() => null);
  const hit = await cache?.match(RUNTIME);
  if (hit) return hit;
  const res = await fetch(RUNTIME);
  if (cache && res.ok) {
    cache.put(RUNTIME, res.clone()).catch((e) => console.debug(`STORE: the runtime's module was not kept: ${e}`));
  }
  return res;
}
