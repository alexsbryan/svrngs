// The conformance canary's checks (serve.mjs), run in its page by whoever
// opened it: each answers `{ ok, saw }`, what it found beside whether it
// passed. Every address is relative to the page, so a check finds the
// canary wherever it opens, but the root's, which asks for the root.
(() => {
  const SIZE = 64 * 1024 * 1024;
  const byte = (i) => (i * 7 + (i >> 10)) & 255;
  const made = (from, to) => {
    const out = new Uint8Array(to - from + 1);
    for (let i = from; i <= to; i++) out[i - from] = byte(i);
    return out;
  };
  const sha = async (b) => [...new Uint8Array(await crypto.subtle.digest('SHA-256', b))].map((x) => x.toString(16).padStart(2, '0')).join('');
  const within = (ms, p) => Promise.race([p, new Promise((_, no) => setTimeout(() => no(new Error(`nothing within ${ms} ms`)), ms))]);
  const judged = (f) => async (...a) => {
    try {
      return await within(60_000, f(...a));
    } catch (e) {
      return { ok: false, saw: String(e) };
    }
  };
  window.conformance = {
    // A page asking for `/`-absolute paths: `/abs.js` ran.
    root: judged(async () => ({ ok: window.__abs === 'ran', saw: String(window.__abs) })),
    // SharedArrayBuffer, which COOP and COEP give a page.
    isolated: judged(async () => {
      const sab = typeof SharedArrayBuffer === 'function' && new SharedArrayBuffer(8).byteLength === 8;
      return { ok: self.crossOriginIsolated === true && sab, saw: `crossOriginIsolated ${self.crossOriginIsolated}, SharedArrayBuffer ${sab}` };
    }),
    // A Host check, and an absolute redirect built from the Host.
    host: judged(async () => {
      const res = await fetch('redirect');
      const text = await res.text();
      const want = new URL('landed', location.href).href;
      return { ok: res.ok && text === 'landed' && res.url === want, saw: `${res.status} ${res.url}: ${text}` };
    }),
    // A range of the 64 MiB file, sha-exact.
    range: judged(async () => {
      const [from, to] = [20_000_000, 20_000_000 + 4 * 1024 * 1024 - 1];
      const res = await fetch('big.bin', { headers: { range: `bytes=${from}-${to}` } });
      const got = new Uint8Array(await res.arrayBuffer());
      const ok = res.status === 206 && res.headers.get('content-range') === `bytes ${from}-${to}/${SIZE}` && (await sha(got)) === (await sha(made(from, to)));
      return { ok, saw: `${res.status} ${res.headers.get('content-range')} ${got.length} bytes` };
    }),
    // The whole 64 MiB uploaded, which the canary reads as it streams.
    upload: judged(async () => {
      const body = made(0, SIZE - 1);
      const want = `${SIZE} ${await sha(body)}`;
      const res = await fetch('upload', { method: 'POST', body: new Blob([body]) });
      const text = await res.text();
      return { ok: res.ok && text === want, saw: `${res.status} ${text}` };
    }),
    // A WebSocket echo.
    websocket: judged(
      () =>
        new Promise((done) => {
          const url = new URL('ws', location.href);
          url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
          const ws = new WebSocket(url);
          const said = `ping ${Math.random()}`;
          ws.onopen = () => ws.send(said);
          ws.onmessage = (e) => {
            done({ ok: e.data === said, saw: `echoed ${e.data}` });
            ws.close();
          };
          ws.onerror = () => done({ ok: false, saw: 'the socket failed' });
        }),
    ),
    // Its own service worker, answering `sw-probe`.
    worker: judged(async () => {
      await navigator.serviceWorker.register('app-sw.js', { scope: './' });
      await navigator.serviceWorker.ready;
      if (!navigator.serviceWorker.controller) {
        await new Promise((r) => navigator.serviceWorker.addEventListener('controllerchange', r, { once: true }));
      }
      const text = await (await fetch('sw-probe')).text();
      return { ok: text === "from the app's worker", saw: text };
    }),
    // A cookie sign-in: the cookie set, then sent.
    signin: judged(async () => {
      const { who } = await (await fetch('signin', { method: 'POST' })).json();
      const me = await (await fetch('me')).json();
      return { ok: me.cookie.includes(`canary-${who}=`) && me.cookie.includes(`canary-${who}-hidden=1`), saw: me.cookie, who };
    }),
    // After a reload: the page reads its cookie, and not the HttpOnly one.
    cookie: judged(async (who) => {
      const c = document.cookie;
      return { ok: c.includes(`canary-${who}=`) && !c.includes('hidden'), saw: c };
    }),
    // Local storage: what was here before this page wrote, then written.
    stored: judged(async (who) => {
      const before = localStorage.getItem('canary');
      localStorage.setItem('canary', who);
      return { ok: true, saw: before };
    }),
    // After a reload: what this page wrote reads back.
    storage: judged(async (who) => ({ ok: localStorage.getItem('canary') === who, saw: localStorage.getItem('canary') })),
  };
})();
