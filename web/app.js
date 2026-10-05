// ported from commonwealth-ai sovereign/apps/ring-runtime/web/app.js@e1a43bcd2
// The page's logic. External (not inline) so a strict CSP can run here
// — see index.html's `script-src 'self' 'wasm-unsafe-eval'`: the wasm module
// needs `wasm-unsafe-eval`, and nothing else needs an allowance.
//
// The version query on the wasm import is stamped at build time (the content
// hash of the module), so a service worker or CDN cache cannot serve a stale
// runtime after a rebuild.
import init, { connect } from './wasm/svrngs_web.js?v=__VERSION__';

const status = document.getElementById('status');
const response = document.getElementById('response');
// The dial's layers go to the console at debug, never onto the page, whose
// words are DESIGN §1's; the lines are kept on `window.__lines` for a test
// to read. Upstream printed them in a "Layers" panel (index.html:42).
const lines = [];
window.__lines = lines;
window.__trace = (l) => {
  lines.push(l);
  console.debug(l);
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

// No key held: joining makes one (u4-5).
class NotJoined extends Error {}

function fail(msg) {
  status.textContent = msg;
  status.className = 'bad';
}

if (!token) {
  fail('Open this page from your group’s link.');
} else {
  try {
    await init();
    const key = await heldKey();
    if (!key) throw new NotJoined();
    status.textContent = 'Reaching your group…';
    const timeout = new Promise((r) => setTimeout(() => r('unanswered'), 120000));
    const dialled = await Promise.race([connect(key, token).catch(() => 'unanswered'), timeout]);
    if (dialled === 'unanswered') {
      fail('Your group’s computer did not answer. Try again later.');
    } else {
      // HTTP over the dial (web/src/http.rs): the answer is a Response.
      const res = await dialled.fetch('GET', path, [['accept', 'application/json']], null);
      const body = await res.text();
      status.textContent = res.ok ? 'You’re in' : 'Your group’s computer said no.';
      status.className = res.ok ? 'good' : 'bad';
      // The trace stays in the console: only the answer is the page's.
      if (res.ok) response.textContent = body;
      await dialled.close();
    }
  } catch (e) {
    console.error(e);
    fail(e instanceof NotJoined ? 'This phone has not joined your group yet.' : 'This browser could not open the page.');
  }
}

// Cache the app shell after first load, so a repeat visit needs no origin
// traffic at all (upstream's docs/RING_APP_LIBRARY.md: "no traffic after
// first load").
// Registered last and best-effort: the page works without it, and a failed
// registration must never break the dial.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('./sw.js').catch(() => {});
  });
}
