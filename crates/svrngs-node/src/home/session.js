// Browser authority stays in this origin's IndexedDB, whose port is part of its identity.
export async function session(value) {
  const db = await new Promise((yes, no) => {
    const r = indexedDB.open('svrngs-browser', 1);
    r.onupgradeneeded = () => r.result.createObjectStore('keys');
    r.onsuccess = () => yes(r.result);
    r.onerror = () => no(r.error);
  });
  return new Promise((yes, no) => {
    const tx = db.transaction('keys', value ? 'readwrite' : 'readonly');
    const store = tx.objectStore('keys');
    const r = value ? store.put(value, 'key') : store.get('key');
    tx.oncomplete = () => {
      db.close();
      yes(value ?? r.result);
    };
    tx.onerror = () => {
      db.close();
      no(tx.error);
    };
  });
}

export async function proof(key, request, epoch) {
  const url = new URL(request.url);
  const nonce = [...crypto.getRandomValues(new Uint8Array(16))]
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  const stamp = `${Math.floor(Date.now() / 1000)}.${nonce}`;
  const message = `${url.host}\n${request.method}\n${url.pathname}${url.search}\n${stamp}\n${request.headers.get('x-svrngs-body') ?? ''}\n${request.headers.get('x-svrngs-intent') ?? ''}\n${epoch}`;
  const tag = new Uint8Array(
    await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(message)),
  );
  return `${stamp}.${btoa(String.fromCharCode(...tag))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')}`;
}

export const Browser = Object.freeze({ Own: 'own', Tab: 'tab' });
export async function browserAuthority(fetched, clients, origin) {
  const request = fetched.request;
  const from = request.headers.get('origin');
  if (from && from !== origin) return null;
  const client = fetched.clientId && (await clients.get(fetched.clientId));
  if ((client && new URL(client.url).origin === origin) || from === origin) return Browser.Own;
  // A navigation WebKit shows no `Origin` and no client of: its referrer,
  // which the browser sets to the page that started it and no page sets to
  // another origin's, says whether that page is this origin's own. Withheld,
  // it proves nothing.
  if (
    request.mode === 'navigate' &&
    !from &&
    URL.canParse(request.referrer ?? '') &&
    new URL(request.referrer).origin === origin
  )
    return Browser.Own;
  // A whole tab's read, typed, bookmarked or followed, as a cookie the
  // browser sends on a top-level GET once was: never a frame, never a
  // write. A worker is shown no `Sec-Fetch-*` header in Chromium or WebKit,
  // so whether a person or a page started it cannot be read here.
  const tab =
    ['GET', 'HEAD'].includes(request.method) &&
    request.mode === 'navigate' &&
    request.destination === 'document';
  return tab ? Browser.Tab : null;
}

if (typeof document !== 'undefined' && document.getElementById('svrngs-session-state')) {
  const said = document.getElementById('svrngs-session-state');
  (async () => {
    let at = location.pathname + location.search;
    if (location.pathname === '/svrngs-open' && new URLSearchParams(location.search).has('code')) {
      const answer = await fetch(location.href, { method: 'POST', cache: 'no-store' });
      if (!answer.ok)
        throw new Error('This open was spent or expired. Open svrngs from your computer again.');
      const enrolled = await answer.json();
      const bytes = Uint8Array.from(atob(enrolled.key.replace(/-/g, '+').replace(/_/g, '/')), (c) =>
        c.charCodeAt(0),
      );
      await session(
        await crypto.subtle.importKey('raw', bytes, { name: 'HMAC', hash: 'SHA-256' }, false, [
          'sign',
        ]),
      );
      at = enrolled.at;
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
  })().catch((e) => {
    said.textContent = e.message;
  });
}
