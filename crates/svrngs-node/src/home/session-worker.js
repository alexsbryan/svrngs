import { session, proof, Browser, browserAuthority } from '/svrngs-session.js';

// `self` as a service worker's: the worker's lib names it a
// `WorkerGlobalScope`, which every kind of worker shares
// (jsconfig.workers.json). Its own first, so `addEventListener` gives each
// listener its event's type, as web/sw.js says it.
/** @typedef {ServiceWorkerGlobalScope & typeof self} Scope */

/**
 * What an app's own worker hears, as this worker hands it on: a fetch,
 * with its request, its client and a way to answer; install and activate,
 * with `waitUntil` alone. Each `waitUntil` here waits for nothing.
 * @typedef {{ request: Request, clientId: string, respondWith: (given: Response | Promise<Response>) => void, waitUntil: () => void }} AppFetch
 * @typedef {{ waitUntil: () => void }} AppStage
 */

/** @type {((event: AppFetch) => void)[] | null} */
let app = null;
/**
 * `request` sent proved as this browser's session, `intent` saying whether
 * this origin's own page asked it; another origin's, or one before a
 * session is kept, sent as it is.
 * @param {Request} request
 */
async function authenticated(request, intent = false) {
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return fetch(request);
  const key = await session();
  if (!key) return fetch(request);
  const headers = new Headers(request.headers);
  headers.set('x-svrngs-intent', intent ? 'yes' : 'no');
  if (url.hostname === 'svrngs.localhost' && !['GET', 'HEAD'].includes(request.method)) {
    const digest = new Uint8Array(
      await crypto.subtle.digest('SHA-256', await request.clone().arrayBuffer()),
    );
    headers.set('x-svrngs-body', [...digest].map((b) => b.toString(16).padStart(2, '0')).join(''));
  }
  // A no-cors request's headers are guarded to the safelisted ones, which
  // drops the proof without a word: every stylesheet, image and classic
  // script would go unproved. Only this origin's own URLs reach here (the
  // first line); what same-origin refuses that no-cors took is a redirect
  // to another origin, which then fails instead of answering opaque.
  /** @type {RequestInit} */
  const mode = request.mode === 'no-cors' ? { mode: 'same-origin' } : {};
  const outgoing = new Request(request, { headers, cache: 'no-store', ...mode });
  const epoch = await (await fetch('/svrngs-session-epoch', { cache: 'no-store' })).text();
  outgoing.headers.set('x-svrngs-browser', await proof(key, outgoing, epoch));
  return fetch(outgoing);
}
/** @type {Scope} */ (self).addEventListener('install', () =>
  /** @type {Scope} */ (self).skipWaiting(),
);
/** @type {Scope} */ (self).addEventListener('activate', (e) =>
  e.waitUntil(/** @type {Scope} */ (self).clients.claim()),
);
// A page's message comes from its `Client` (session-client.js); one from
// any other source has no `url`, which the first test refuses.
/** @type {Scope} */ (self).addEventListener('message', (e) => {
  if (
    !(/** @type {Client | null} */ (e.source)?.url) ||
    new URL(/** @type {Client} */ (e.source).url).origin !== self.location.origin ||
    typeof e.data?.worker !== 'string'
  )
    return;
  e.waitUntil(
    (async () => {
      const url = new URL(e.data.worker, /** @type {Client} */ (e.source).url);
      if (url.origin !== self.location.origin) throw new Error('an app worker is its own origin’s');
      const res = await authenticated(new Request(url));
      if (!res.ok) throw new Error(`its worker answered ${res.status}`);
      /** @type {{ fetch: ((event: AppFetch) => void)[], install: ((event: AppStage) => void)[], activate: ((event: AppStage) => void)[] }} */
      const listeners = { fetch: [], install: [], activate: [] };
      // An app may listen for any type; these three are handed on, and a
      // listener's own type is the app's, never read here.
      const on = (/** @type {string} */ type, /** @type {(event: never) => void} */ f) =>
        /** @type {Partial<Record<string, ((event: never) => void)[]>>} */ (listeners)[type]?.push(
          f,
        );
      /**
       * @param {RequestInfo | URL} input
       * @param {RequestInit} [init]
       */
      const fetched = (input, init) =>
        authenticated(
          new Request(
            input instanceof Request
              ? input
              : new URL(input, /** @type {Scope} */ (self).registration.scope),
            init,
          ),
          true,
        );
      // The app's worker reads any name of the global, so it is read by name.
      const scope = new Proxy(
        /** @type {Record<string | symbol, unknown>} */ (/** @type {unknown} */ (self)),
        {
          get: (o, k) =>
            k === 'addEventListener'
              ? on
              : k === 'fetch'
                ? fetched
                : typeof o[k] === 'function'
                  ? o[k].bind(o)
                  : o[k],
        },
      );
      new Function('self', 'addEventListener', 'fetch', await res.text())(scope, on, fetched);
      app = listeners.fetch;
      for (const f of [...listeners.install, ...listeners.activate]) f({ waitUntil: () => {} });
      e.ports[0]?.postMessage({ ok: true });
    })().catch((err) => e.ports[0]?.postMessage({ error: String(err) })),
  );
});
/** @type {Scope} */ (self).addEventListener('fetch', (e) => {
  const url = new URL(e.request.url);
  if (
    url.origin !== self.location.origin ||
    [
      '/svrngs-session.js',
      '/svrngs-session-worker.js',
      '/svrngs-session-client.js',
      '/svrngs-session-epoch',
      '/svrngs-open',
    ].includes(url.pathname)
  )
    return;
  e.respondWith(
    (async () => {
      const authority = await browserAuthority(
        e,
        /** @type {Scope} */ (self).clients,
        self.location.origin,
      );
      if (!authority) {
        // A read capability is independent authority: forward it unsigned for
        // the node to judge. Never let a refusal bootstrap this frame's key.
        if (url.pathname.startsWith('/svrngs-as/')) {
          const response = await fetch(e.request);
          if (response.status !== 401) return response;
        }
        return new Response('Open this page as a tab, or from its own origin', { status: 403 });
      }
      const intent = authority === Browser.Own;
      /** @type {Response | Promise<Response> | undefined} */
      let given;
      for (const f of intent ? (app ?? []) : [])
        f({
          request: e.request,
          clientId: e.clientId,
          respondWith: (p) => {
            given ??= p;
          },
          waitUntil: () => {},
        });
      const response = await (given ?? authenticated(e.request, intent));
      if (
        url.hostname === 'svrngs.localhost' ||
        e.request.method !== 'GET' ||
        !response.headers.get('content-type')?.includes('text/html')
      )
        return response;
      const headers = new Headers(response.headers);
      headers.delete('content-length');
      headers.delete('content-encoding');
      const nonce = /'nonce-([^']+)'/.exec(headers.get('content-security-policy') ?? '')?.[1];
      const html = await response.text();
      const head = /<head[^>]*>/i.exec(html) ?? /<!doctype[^>]*>/i.exec(html);
      const at = head ? head.index + head[0].length : 0;
      const script = `<script${nonce ? ` nonce="${nonce}"` : ''} src="/svrngs-session-client.js"></script>`;
      return new Response(html.slice(0, at) + script + html.slice(at), {
        status: response.status,
        statusText: response.statusText,
        headers,
      });
    })(),
  );
});
