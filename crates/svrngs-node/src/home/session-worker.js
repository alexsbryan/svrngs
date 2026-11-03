// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
import { session, proof, Browser, browserAuthority } from '/svrngs-session.js';
let app = null;
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
  const mode = request.mode === 'no-cors' ? { mode: 'same-origin' } : {};
  const outgoing = new Request(request, { headers, cache: 'no-store', ...mode });
  const epoch = await (await fetch('/svrngs-session-epoch', { cache: 'no-store' })).text();
  outgoing.headers.set('x-svrngs-browser', await proof(key, outgoing, epoch));
  return fetch(outgoing);
}
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));
self.addEventListener('message', (e) => {
  if (
    !e.source?.url ||
    new URL(e.source.url).origin !== self.location.origin ||
    typeof e.data?.worker !== 'string'
  )
    return;
  e.waitUntil(
    (async () => {
      const url = new URL(e.data.worker, e.source.url);
      if (url.origin !== self.location.origin) throw new Error('an app worker is its own origin’s');
      const res = await authenticated(new Request(url));
      if (!res.ok) throw new Error(`its worker answered ${res.status}`);
      const listeners = { fetch: [], install: [], activate: [] };
      const on = (type, f) => listeners[type]?.push(f);
      const fetched = (input, init) =>
        authenticated(
          new Request(
            input instanceof Request ? input : new URL(input, self.registration.scope),
            init,
          ),
          true,
        );
      const scope = new Proxy(self, {
        get: (o, k) =>
          k === 'addEventListener'
            ? on
            : k === 'fetch'
              ? fetched
              : typeof o[k] === 'function'
                ? o[k].bind(o)
                : o[k],
      });
      new Function('self', 'addEventListener', 'fetch', await res.text())(scope, on, fetched);
      app = listeners.fetch;
      for (const f of [...listeners.install, ...listeners.activate]) f({ waitUntil: () => {} });
      e.ports[0]?.postMessage({ ok: true });
    })().catch((err) => e.ports[0]?.postMessage({ error: String(err) })),
  );
});
self.addEventListener('fetch', (e) => {
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
      const authority = await browserAuthority(e, self.clients, self.location.origin);
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
