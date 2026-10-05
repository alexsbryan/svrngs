// ported from commonwealth-ai sovereign/apps/ring-runtime/web/app.js@e1a43bcd2
// The guest page's logic. External (not inline) so a strict CSP can run here
// — see index.html's `script-src 'self' 'wasm-unsafe-eval'`: the wasm module
// needs `wasm-unsafe-eval`, and nothing else needs an allowance.
//
// The version query on the wasm import is stamped at build time (the content
// hash of the module), so a service worker or CDN cache cannot serve a stale
// runtime after a rebuild.
import init, { dial } from './wasm/svrngs_web.js?v=__VERSION__';

const status = document.getElementById('status');
const response = document.getElementById('response');
const traceEl = document.getElementById('trace');
const labelEl = document.getElementById('label');

const lines = [];
window.__trace = (l) => {
  lines.push(l);
  traceEl.textContent = lines.join('\n');
};

// The link's fragment is its token, `<origin>#<token>`, which the dial
// parses with svrngs-core (DESIGN §3, call 5). The fragment is never sent to
// this page's origin, which is why the link rides there and not in the
// query. The page asks the issuer what the group's people lend.
const token = location.hash.slice(1);
const path = '/lends';

// The person's key, as this page's store holds it: 32 bytes under `key` in
// IndexedDB `svrngs`, object store `person`, or null when it holds none.
// Making the key and sealing it is joining's (u4-5; DESIGN §3, call 1).
function heldKey() {
  return new Promise((resolve, reject) => {
    const open = indexedDB.open('svrngs', 1);
    open.onupgradeneeded = () => open.result.createObjectStore('person');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const get = open.result.transaction('person').objectStore('person').get('key');
      get.onerror = () => reject(get.error);
      get.onsuccess = () => resolve(get.result ? new Uint8Array(get.result) : null);
    };
  });
}

function fail(msg) {
  status.textContent = msg;
  status.className = 'bad';
}

if (!token) {
  fail('Open this page from a guest link — its fragment carries token= and iroh=.');
} else {
  try {
    await init();
    const key = await heldKey();
    status.textContent = 'Dialing the wall over the relay…';
    const pending = dial(key || new Uint8Array(), token, path);
    const timeout = new Promise((r) => setTimeout(() => r('__TIMEOUT__'), 120000));
    const res = await Promise.race([pending, timeout]);
    if (res === '__TIMEOUT__') {
      fail('Timed out — the last layer below is where it hung.');
    } else {
      const body = (res.split('RESPONSE-BODY: ')[1] || '').trim();
      const st = (res.match(/RESPONSE-STATUS: (.*)/) || [])[1] || '';
      status.textContent = st.startsWith('HTTP/1.1 2') ? 'Connected' : 'Refused';
      status.className = st.startsWith('HTTP/1.1 2') ? 'good' : 'bad';
      response.textContent = body || res;
    }
  } catch (e) {
    fail('Could not run the runtime: ' + (e && e.stack ? e.stack : String(e)));
  }
}

// Cache the app shell after first load, so a repeat visit needs no origin
// traffic at all (docs/RING_APP_LIBRARY.md: "no traffic after first load").
// Registered last and best-effort: the page works without it, and a failed
// registration must never break the dial.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('./sw.js').catch(() => {});
  });
}
