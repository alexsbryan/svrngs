// Browser authority stays in this origin's IndexedDB, whose port is part of its identity.
/**
 * This origin's session key: kept, given `value`, or read, `undefined`
 * before one is kept.
 * @param {CryptoKey} [value]
 * @returns {Promise<CryptoKey | undefined>}
 */
export async function session(value) {
  /** @type {IDBDatabase} */
  const db = await new Promise((yes, no) => {
    const request = indexedDB.open('svrngs-browser', 1);
    request.onupgradeneeded = () => request.result.createObjectStore('keys');
    request.onsuccess = () => yes(request.result);
    request.onerror = () => no(request.error);
  });
  return new Promise((yes, no) => {
    const tx = db.transaction('keys', value ? 'readwrite' : 'readonly');
    const store = tx.objectStore('keys');
    const request = value ? store.put(value, 'key') : store.get('key');
    tx.oncomplete = () => {
      db.close();
      yes(value ?? request.result);
    };
    tx.onerror = () => {
      db.close();
      no(tx.error);
    };
  });
}

/**
 * The proof a request carries: when, a nonce, and the key's HMAC over its
 * host, method, path, body digest, intent and the session's `epoch`.
 * @param {CryptoKey} key
 * @param {Request} request
 * @param {string} epoch
 */
export async function proof(key, request, epoch) {
  const url = new URL(request.url);
  const nonce = [...crypto.getRandomValues(new Uint8Array(16))]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
  const stamp = `${Math.floor(Date.now() / 1000)}.${nonce}`;
  // A line each, in the order the node signs them over to check this
  // proof (session.rs `Sessions::proved`, its `format!` of `message`).
  const signed = [
    url.host,
    request.method,
    `${url.pathname}${url.search}`,
    stamp,
    request.headers.get('x-svrngs-body') ?? '',
    request.headers.get('x-svrngs-intent') ?? '',
    epoch,
  ];
  const message = signed.join('\n');
  const tag = new Uint8Array(
    await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(message)),
  );
  return `${stamp}.${btoa(String.fromCharCode(...tag))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')}`;
}

export const Browser = Object.freeze({ Own: 'own', Tab: 'tab' });
/**
 * Whose a request is, by the fetch the worker is handed: this origin's
 * own page, a whole tab's read, or neither (`null`). The fetch and the
 * clients are a worker's `FetchEvent` and `Clients`, read for the fields
 * here, as this module is the page's too, whose lib declares neither.
 * @param {{ request: Request, clientId: string }} fetched
 * @param {{ get: (id: string) => Promise<{ url: string } | undefined> }} clients
 * @param {string} origin
 */
export async function browserAuthority(fetched, clients, origin) {
  const request = fetched.request;
  const from = request.headers.get('origin');
  if (from && from !== origin) return null;
  const client = fetched.clientId ? await clients.get(fetched.clientId) : undefined;
  if ((client && new URL(client.url).origin === origin) || from === origin) return Browser.Own;
  if (referredHere(request, from, origin)) return Browser.Own;
  return wholeTab(request) ? Browser.Tab : null;
}

// A navigation WebKit shows no `Origin` and no client of: its referrer,
// which the browser sets to the page that started it and no page sets to
// another origin's, says whether that page is this origin's own. Withheld,
// it proves nothing.
/**
 * @param {Request} request
 * @param {string | null} from its `Origin`
 * @param {string} origin
 */
const referredHere = (request, from, origin) =>
  request.mode === 'navigate' &&
  !from &&
  URL.canParse(request.referrer) &&
  new URL(request.referrer).origin === origin;

// A whole tab's read, typed, bookmarked or followed, as a cookie the
// browser sends on a top-level GET once was: never a frame, never a
// write. A worker is shown no `Sec-Fetch-*` header in Chromium or WebKit,
// so whether a person or a page started it cannot be read here.
/** @param {Request} request */
const wholeTab = (request) =>
  ['GET', 'HEAD'].includes(request.method) &&
  request.mode === 'navigate' &&
  request.destination === 'document';

if (typeof document !== 'undefined' && document.getElementById('svrngs-session-state')) {
  // Found by the test above.
  const said = /** @type {HTMLElement} */ (document.getElementById('svrngs-session-state'));
  (async () => {
    // The fragment too: the server never sees it, and a page that routes
    // by it, as Jellyfin's `#/details?id=` does, opens where it was sent
    // (crates/svrngs/tests/scenes/movie_night.rs, Movie Night's copy).
    let at = location.pathname + location.search + location.hash;
    if (location.pathname === '/svrngs-open' && new URLSearchParams(location.search).has('code')) {
      const answer = await fetch(location.href, { method: 'POST', cache: 'no-store' });
      if (!answer.ok)
        throw new Error('This open was spent or expired. Open svrngs from your computer again.');
      const enrolled = await answer.json();
      const bytes = Uint8Array.from(
        atob(enrolled.key.replace(/-/g, '+').replace(/_/g, '/')),
        (char) => char.charCodeAt(0),
      );
      await session(
        await crypto.subtle.importKey('raw', bytes, { name: 'HMAC', hash: 'SHA-256' }, false, [
          'sign',
        ]),
      );
      at = enrolled.at + location.hash;
      history.replaceState(null, '', '/svrngs-open');
    }
    if (!(await session())) return;
    await navigator.serviceWorker.register('/svrngs-session-worker.js', {
      type: 'module',
      scope: '/',
    });
    await navigator.serviceWorker.ready;
    if (!navigator.serviceWorker.controller)
      await new Promise((yes) =>
        navigator.serviceWorker.addEventListener('controllerchange', yes, { once: true }),
      );
    location.replace(at);
  })().catch((error) => {
    said.textContent = error.message;
  });
}
