// SPDX-License-Identifier: AGPL-3.0-or-later
// The page's store (DESIGN §3, "Joining", call 1): what a phone keeps of
// each group it joined, in IndexedDB `svrngs`.
//
//   seal     `key`: the page's sealing key, AES-GCM 256, made once with
//            `extractable: false`, so no script can read its bytes out,
//            by the page when it first keeps a group, never by its worker.
//   groups   by `id`: the group's name, the person's name in it, the link's
//            token it was joined by, when it was last opened, the keys
//            of the people its list last named as lending, and the
//            person's key — 32 bytes — sealed: `iv` and `sealed`, with the
//            group's id as the seal's additional data, so a sealed key
//            opens only as the key of the group it was kept for.
//
// The key is bytes, never a WebCrypto signing key: iroh signs its handshake
// synchronously (iroh-1.0.2/src/tls/resolver.rs:92-95), and the issuer lets
// in only the key that dials it. So storage holds only ciphertext, and the
// page's own code can still unseal it — why a group trusts the origin it
// names with this page's code (DESIGN §11, risk 5). One key per person per
// group (DESIGN §6). Version 1 held one key unsealed (u4-3's dial, before a
// phone could join), never a person's: the upgrade drops it.
const DB = 'svrngs';
const VERSION = 2;

function open() {
  return new Promise((done, fail) => {
    const req = indexedDB.open(DB, VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      for (const name of [...db.objectStoreNames]) db.deleteObjectStore(name);
      db.createObjectStore('seal');
      db.createObjectStore('groups', { keyPath: 'id' });
    };
    req.onsuccess = () => done(req.result);
    req.onerror = () => fail(req.error);
  });
}

// One request in one transaction, answered once the transaction commits.
async function run(store, mode, act) {
  const db = await open();
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

// Keep a group joined: `key` is the person's 32 bytes, kept sealed alone.
export async function keep({ id, name, person, token, key }) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const sealed = new Uint8Array(await crypto.subtle.encrypt(bound(id, iv), await sealingKey(true), key));
  await run('groups', 'readwrite', (s) => s.put({ id, name, person, token, iv, sealed, opened: Date.now() }));
}

// Every group kept, the last opened first. Their keys stay sealed.
export async function groups() {
  const all = await run('groups', 'readonly', (s) => s.getAll());
  return all.sort((a, b) => b.opened - a.opened);
}

// The group was opened now: it is the one the page opens without a link.
// Answers the group as kept.
export async function opened(group) {
  const now = { ...group, opened: Date.now() };
  await run('groups', 'readwrite', (s) => s.put(now));
  return now;
}

// The keys of the people `group`'s list last named as lending: the
// runtime dials each of them with this group's key (sw.js).
export async function listed(group, lenders) {
  await run('groups', 'readwrite', (s) => s.put({ ...group, lenders }));
}

// The group whose key reaches `lender`: the one whose list last named
// them, else the one opened last.
export async function groupFor(lender) {
  const kept = await groups();
  return kept.find((g) => g.lenders?.includes(lender)) ?? kept[0];
}

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
