// Minimal repro: does a frame at the group origin, under a same-site (or
// cross-site) top-level app page, see the group origin's first-party storage?
// One HTTPS server (all *.test / *.sh / *.co.uk names, reached via proxy.js),
// one plain-HTTP server (the *.localhost layout the svrngs tests use).
const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');

const WRITE = `<!doctype html><meta charset=utf-8><title>group</title><script>
(async () => {
  const tag = 'written-' + location.host;
  localStorage.setItem('v', tag);
  document.cookie = 'v=' + tag + '; Secure; SameSite=None; Path=/';
  if (location.protocol === 'http:') document.cookie = 'v=' + tag + '; Path=/';
  const key = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, false, ['sign', 'verify']);
  const db = await new Promise((ok, no) => {
    const r = indexedDB.open('d', 1);
    r.onupgradeneeded = () => r.result.createObjectStore('kv');
    r.onsuccess = () => ok(r.result); r.onerror = () => no(r.error);
  });
  await new Promise((ok, no) => {
    const tx = db.transaction('kv', 'readwrite');
    tx.objectStore('kv').put(tag, 'v');
    tx.objectStore('kv').put(key.privateKey, 'k');
    tx.oncomplete = ok; tx.onerror = () => no(tx.error);
  });
  window.__done = { tag, extractable: key.privateKey.extractable };
})().catch((e) => { window.__done = { error: String(e) }; });
</script>`;

const FRAME = `<!doctype html><meta charset=utf-8><title>frame</title><button id=b style="width:200px;height:100px">go</button><script>
const read = async () => {
  const out = { origin: location.origin };
  if (location.search.includes('saa')) {
    try { await document.requestStorageAccess(); out.saa = 'granted'; } catch (e) { out.saa = 'THREW ' + e.name + ': ' + e.message; }
  }
  try { out.ls = localStorage.getItem('v'); } catch (e) { out.ls = 'THREW ' + e.name; }
  out.cookie = document.cookie || null;
  try { out.hasStorageAccess = document.hasStorageAccess ? await document.hasStorageAccess() : 'n/a'; } catch (e) { out.hasStorageAccess = 'THREW ' + e.name; }
  try {
    const db = await new Promise((ok, no) => {
      const r = indexedDB.open('d', 1);
      r.onupgradeneeded = () => { out.idbFresh = true; r.result.createObjectStore('kv'); };
      r.onsuccess = () => ok(r.result); r.onerror = () => no(r.error);
    });
    const get = (k) => new Promise((ok, no) => {
      const q = db.transaction('kv').objectStore('kv').get(k);
      q.onsuccess = () => ok(q.result); q.onerror = () => no(q.error);
    });
    out.idb = (await get('v')) ?? null;
    const k = await get('k');
    if (k instanceof CryptoKey) {
      const sig = await crypto.subtle.sign({ name: 'ECDSA', hash: 'SHA-256' }, k, new TextEncoder().encode('x'));
      out.key = 'signed ' + sig.byteLength + 'B, extractable=' + k.extractable;
    } else out.key = k === undefined ? null : 'not a CryptoKey: ' + typeof k;
  } catch (e) { out.idb = 'THREW ' + e.name + ': ' + e.message; }
  parent.postMessage(out, '*');
};
if (location.search.includes('saa')) document.getElementById('b').onclick = read; else read();
</script>`;

const APP = (src) => `<!doctype html><meta charset=utf-8><title>app</title><script>
addEventListener('message', (e) => { window.__result = { from: e.origin, ...e.data }; });
</script><iframe src="${src}"></iframe>`;

function handler(scheme) {
  return (req, res) => {
    const host = req.headers.host;
    const u = new URL(req.url, scheme + '://' + host);
    if (process.env.LOG) console.log(scheme, host, u.pathname + u.search);
    res.setHeader('content-type', 'text/html; charset=utf-8');
    if (u.pathname === '/frame.html') return res.end(FRAME);
    if (u.searchParams.has('g')) return res.end(APP(scheme + '://' + u.searchParams.get('g') + '/frame.html' + (u.searchParams.has('saa') ? '?saa' : '')));
    if (u.pathname === '/') return res.end(WRITE);
    res.statusCode = 404; res.end('nope');
  };
}

const HTTPS_PORT = +(process.env.HTTPS_PORT || 8443);
const HTTP_PORT = +(process.env.HTTP_PORT || 8480);
const dir = __dirname;
https.createServer({ key: fs.readFileSync(path.join(dir, 'key.pem')), cert: fs.readFileSync(path.join(dir, 'cert.pem')) }, handler('https'))
  .listen(HTTPS_PORT, '127.0.0.1');
http.createServer(handler('http')).listen(HTTP_PORT, '127.0.0.1');
module.exports = { HTTPS_PORT, HTTP_PORT };
