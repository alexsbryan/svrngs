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

// Bytes as hex: a proof's nonce, and the digest of the body it covers.
/** @param {Uint8Array} bytes */
export const hex = (bytes) => [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');

/**
 * The proof a request carries: when, a nonce, and the key's HMAC over its
 * host, method, path, body digest, intent and the session's `epoch`.
 * @param {CryptoKey} key
 * @param {Request} request
 * @param {string} epoch
 */
export async function proof(key, request, epoch) {
  const url = new URL(request.url);
  const nonce = hex(crypto.getRandomValues(new Uint8Array(16)));
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

// ── an app's own worker, in a realm of its own ────────────────────────────
//
// An app that registers a service worker is given a dedicated worker its
// page starts at the app's script (session-client.js on a computer, shim.js
// on a phone). That script and all it loads come through svrngs's worker
// like any request, so the realm runs under the policy the app's own pages
// get, and no app code runs in svrngs's worker. svrngs's worker hands it,
// over a port, the subresources the page that started it asks of the lend
// (`SUBRESOURCES`), takes an answer it gives, and decides everything else
// itself. Every document and worker comes from the lend, under the policy
// svrngs serves it with: an answer's headers are its own, and a document's
// or a worker's would set its policy. A realm lasts while its page is open.
// Both workers import this: session-worker.js, and web/lend.js
// (scripts/build-web.sh puts this file beside it).

/** @typedef {{ script: string, isolated: boolean, port: MessagePort, ready: boolean }} Realm */
// What a realm may answer: the Fetch standard's subresource destinations,
// never a navigation, a frame, an embed or object, or a worker's script.
const SUBRESOURCES = new Set(
  ',audio,audioworklet,font,image,json,manifest,paintworklet,script,style,track,video,xslt'.split(
    ',',
  ),
);
/** Each open page's realm, by the page's client id. @type {Map<string, Realm>} */
const realms = new Map();

/**
 * A page's realm, from its message: `realm` the script's URL, `isolated`
 * whether the page is cross-origin isolated, its ports the realm's and one
 * to tell the page by, `kept`, then `ready` once the realm's prelude runs.
 * Taken from a window of this origin alone, never a worker; whether it was
 * one.
 * @param {{ source: unknown, data: unknown, ports: readonly MessagePort[] }} message
 */
export function realmKept({ source, data, ports: [port, told] }) {
  const page = /** @type {{ id: string, type: string, url: string } | null} */ (source);
  const asked = /** @type {{ realm?: unknown, isolated?: unknown } | null} */ (data);
  const script = asked?.realm;
  const here = (/** @type {unknown} */ url) =>
    typeof url === 'string' && URL.canParse(url) && new URL(url).origin === self.location.origin;
  if (page?.type !== 'window' || !here(page.url) || !here(script) || !port || !told) return false;
  const isolated = asked?.isolated === true;
  const realm = { script: /** @type {string} */ (script), isolated, port, ready: false };
  port.onmessage = () => {
    realm.ready = true;
    told.postMessage('ready');
  };
  realms.set(page.id, realm);
  told.postMessage('kept');
  return true;
}

/**
 * Each page of this origin's asked for its realm again, as svrngs's worker
 * starts: a worker the browser stopped keeps none of `realms`, which is its
 * memory, so each page hands its realm anew (`registered`).
 * @param {{ matchAll: (options: { type: 'window' }) => Promise<readonly { postMessage: (message: string) => void }[]> }} clients
 */
export async function realmsAsked(clients) {
  for (const page of await clients.matchAll({ type: 'window' })) page.postMessage('svrngs-realms');
}

/**
 * The realm whose own script `fetched` asks, for the page that started it.
 * @param {{ request: Request, clientId: string }} fetched
 */
export const realmScript = ({ request, clientId }) => {
  const realm = realms.get(clientId);
  return request.destination === 'worker' && realm?.script === request.url ? realm : undefined;
};

/**
 * `response`, `realm`'s script as the lend answered it, its headers kept,
 * the policy the app's pages get among them, with svrngs's prelude first.
 * A service worker's script names no embedder policy, which a dedicated
 * worker an isolated page starts must, or Chromium and WebKit refuse to
 * start it: the page's own is given it there, `require-corp`, the one both
 * take (measured 2026-11-17, Chromium 147, WebKit 26.4).
 * @param {Response} response
 * @param {Realm} realm
 */
export async function preluded(response, realm) {
  const headers = new Headers(response.headers);
  headers.delete('content-length');
  headers.delete('content-encoding');
  const embedder = 'cross-origin-embedder-policy';
  if (realm.isolated && !headers.has(embedder)) headers.set(embedder, 'require-corp');
  const { status, statusText } = response;
  const source = `(${prelude.toString()})();\n${await response.text()}`;
  return new Response(source, { status, statusText, headers });
}

/**
 * The answer a realm gives `fetched`, or none: the realm of the page that
 * asked, once ready, and only for a subresource.
 * @param {{ request: Request, clientId: string }} fetched
 * @returns {Promise<Response | undefined>}
 */
export async function realmAnswer(fetched) {
  const realm = realms.get(fetched.clientId);
  if (!realm?.ready || !SUBRESOURCES.has(fetched.request.destination)) return;
  const { url, method, mode, destination, headers } = fetched.request;
  const body = ['GET', 'HEAD'].includes(method)
    ? null
    : await fetched.request.clone().arrayBuffer();
  const { port1, port2 } = new MessageChannel();
  realm.port.postMessage({ url, method, mode, destination, headers: [...headers], body }, [port2]);
  /** @type {(ResponseInit & { body: ArrayBuffer | null }) | null} */
  const answer = await new Promise((done) => (port1.onmessage = ({ data }) => done(data)));
  return answer ? new Response(answer.body, answer) : undefined;
}

/**
 * `navigator.serviceWorker.register` in an app's page, as the page's shim
 * answers it (session-client.js, shim.js, which import this): a dedicated
 * worker at the app's script, its realm handed to svrngs's worker,
 * `svrngs`, and kept there (`realmKept`) before the worker starts, so its
 * script comes with the prelude; settled once the prelude runs. Its
 * registration is a stand-in: svrngs's worker is the one the browser holds.
 * @param {ServiceWorker} svrngs
 * @param {string | URL} script
 * @param {RegistrationOptions} [options]
 */
export async function registered(svrngs, script, options) {
  const url = new URL(script, location.href);
  /** @type {Worker | undefined} */
  let app;
  // The realm handed to svrngs's worker and kept there, then to the app's
  // worker, started the first time; again each time svrngs's worker starts
  // afresh and asks (`realmsAsked`).
  const hand = async () => {
    const realm = new MessageChannel();
    const told = new MessageChannel();
    const heard = () => new Promise((done) => (told.port1.onmessage = done));
    let next = heard();
    svrngs.postMessage({ realm: url.href, isolated: crossOriginIsolated }, [
      realm.port1,
      told.port2,
    ]);
    // An older svrngs's worker, which takes no realm, says nothing.
    const unheard = new Error("svrngs's worker did not answer: open this page again");
    await Promise.race([next, new Promise((_, no) => setTimeout(no, 10000, unheard))]);
    next = heard();
    const started = (app ??= new Worker(url, { type: options?.type ?? 'classic' }));
    started.postMessage('svrngs-realm', [realm.port2]);
    await new Promise((ready, failed) => {
      void next.then(ready);
      started.onerror = (error) =>
        failed(new Error(`this app's worker did not start: ${error.message}`));
    });
  };
  await hand();
  navigator.serviceWorker.addEventListener('message', ({ data }) => {
    if (data === 'svrngs-realms') void hand();
  });
  // What an app's page reads of its registration; it has no more.
  return /** @type {ServiceWorkerRegistration} */ (
    /** @type {unknown} */ ({
      scope: new URL(options?.scope ?? './', url).href,
      active: svrngs,
      update: async () => {},
      unregister: async () => false,
    })
  );
}

// svrngs's prelude, the first line of an app's worker in its realm
// (`preluded`), so its own source and nothing it closes over: what an app's
// service worker reads of its global stands in, and each fetch svrngs's
// worker hands over arrives as a `fetch` event, for the app's listeners to
// answer by `respondWith`, or not.
function prelude() {
  Object.assign(self, {
    registration: { scope: new URL('./', location.href).href },
    clients: { claim: async () => {}, matchAll: async () => [] },
    skipWaiting: async () => {},
  });
  let started = false;
  addEventListener('message', (message) => {
    const port = message.data === 'svrngs-realm' ? message.ports[0] : undefined;
    if (!port) return;
    message.stopImmediatePropagation();
    // Once, though each start of svrngs's worker hands a port anew.
    for (const type of started ? [] : ['install', 'activate'])
      dispatchEvent(Object.assign(new CustomEvent(type), { waitUntil() {} }));
    started = true;
    port.onmessage = ({ data, ports: [reply] }) => {
      const { url, method, headers, body, mode, destination } = data;
      const request = new Request(url, { method, headers, body });
      // As the browser asked it: a navigation's mode no `Request` takes.
      Object.defineProperties(request, {
        mode: { value: mode },
        destination: { value: destination },
      });
      /** @type {Promise<Response> | undefined} */
      let given;
      const respondWith = (/** @type {Response | PromiseLike<Response>} */ answer) =>
        (given ??= Promise.resolve(answer));
      dispatchEvent(
        Object.assign(new CustomEvent('fetch'), { request, respondWith, waitUntil() {} }),
      );
      void Promise.resolve(given)
        .then(async (response) =>
          response && response.status > 0
            ? {
                body: response.body && (await response.arrayBuffer()),
                status: response.status,
                statusText: response.statusText,
                headers: [...response.headers],
              }
            : null,
        )
        .catch(() => null)
        .then((answer) => reply?.postMessage(answer));
    };
    port.postMessage('ready');
  });
}

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
