import { session, proof, hex, Browser, browserAuthority } from '/svrngs-session.js';

// `self` as a service worker's: the worker's lib names it a
// `WorkerGlobalScope`, which every kind of worker shares
// (jsconfig.workers.json). Its own first, so `addEventListener` gives each
// listener its event's type, as web/sw.js says it.
/** @typedef {ServiceWorkerGlobalScope & typeof self} Scope */
const worker = /** @type {Scope} */ (self);

/**
 * `request`, this origin's, sent proved as this browser's session,
 * `intent` saying whether this origin's own page asked it; one before a
 * session is kept, sent as it is.
 * @param {Request} request
 * @param {boolean} intent
 */
async function authenticated(request, intent) {
  const url = new URL(request.url);
  const key = await session();
  if (!key) return fetch(request);
  const headers = new Headers(request.headers);
  headers.set('x-svrngs-intent', intent ? 'yes' : 'no');
  if (url.hostname === 'svrngs.localhost' && !['GET', 'HEAD'].includes(request.method)) {
    const digest = new Uint8Array(
      await crypto.subtle.digest('SHA-256', await request.clone().arrayBuffer()),
    );
    headers.set('x-svrngs-body', hex(digest));
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
worker.addEventListener('install', () => worker.skipWaiting());
worker.addEventListener('activate', (activation) => activation.waitUntil(worker.clients.claim()));
worker.addEventListener('fetch', (fetchEvent) => {
  const url = new URL(fetchEvent.request.url);
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
  fetchEvent.respondWith(served(fetchEvent, url));
});

// The answer to a fetch of this origin's, `fetchEvent` at `url`: refused
// unless the browser's authority is read from it, else the node's to it
// proved, a page with the session's client in it.
/**
 * @param {FetchEvent} fetchEvent
 * @param {URL} url
 */
async function served(fetchEvent, url) {
  const authority = await browserAuthority(fetchEvent, worker.clients, self.location.origin);
  if (!authority) return unproved(fetchEvent.request, url);
  const response = await authenticated(fetchEvent.request, authority === Browser.Own);
  if (
    url.hostname === 'svrngs.localhost' ||
    fetchEvent.request.method !== 'GET' ||
    !response.headers.get('content-type')?.includes('text/html')
  )
    return response;
  return withClient(response);
}

// A request no authority of the browser's is read from.
/**
 * @param {Request} request
 * @param {URL} url
 */
async function unproved(request, url) {
  // A read capability is independent authority: forward it unsigned for
  // the node to judge. Never let a refusal bootstrap this frame's key.
  if (url.pathname.startsWith('/svrngs-as/')) {
    const response = await fetch(request);
    if (response.status !== 401) return response;
  }
  return new Response('Open this page as a tab, or from its own origin', { status: 403 });
}

// A page, `response`, with the session's client script in its head, under
// the page's own script nonce when its policy names one.
/** @param {Response} response */
async function withClient(response) {
  const headers = new Headers(response.headers);
  headers.delete('content-length');
  headers.delete('content-encoding');
  const nonce = /'nonce-([^']+)'/.exec(headers.get('content-security-policy') ?? '')?.[1];
  const html = await response.text();
  const head = /<head[^>]*>/i.exec(html) ?? /<!doctype[^>]*>/i.exec(html);
  const at = head ? head.index + head[0].length : 0;
  const nonced = nonce ? ` nonce="${nonce}"` : '';
  const script = `<script${nonced} src="/svrngs-session-client.js"></script>`;
  return new Response(html.slice(0, at) + script + html.slice(at), {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}
