// Probe pages for the holder-approval design (HOLDER_APPROVAL.md §3).
// One HTTPS server for every name (reached through proxy.js, as in
// ../diligence/). The group origin serves:
//   /            the group page (top level; modes below)
//   /sign.html   the framed signer: reports what it can reach from inside an app
//   /approve.html a top-level popup that reads first-party storage
// any other host is an app: it frames /sign.html at the group origin.
const https = require('https');
const fs = require('fs');
const path = require('path');

const GROUP = 'group.svrnme.sh';
const G = `https://${GROUP}`;

const IDB = `
const db = (name) => new Promise((ok, no) => {
  const r = indexedDB.open(name, 1);
  r.onupgradeneeded = () => r.result.createObjectStore('kv');
  r.onsuccess = () => ok(r.result); r.onerror = () => no(r.error);
});
const put = async (name, k, v) => { const d = await db(name); await new Promise((ok, no) => { const t = d.transaction('kv', 'readwrite'); t.objectStore('kv').put(v, k); t.oncomplete = ok; t.onerror = () => no(t.error); }); };
const get = async (name, k) => { const d = await db(name); return new Promise((ok, no) => { const q = d.transaction('kv').objectStore('kv').get(k); q.onsuccess = () => ok(q.result ?? null); q.onerror = () => no(q.error); }); };
`;

// The group page. mode=opener: a button opens the app in a new tab with
// window.open (opener kept) and answers a framed signer's ask for authority.
// mode=nav: navigate this tab to the app (production's location.assign).
// mode=cookie: set __Host- cookies (Strict, Lax, None), then optionally go.
// mode=write: write a first-party IndexedDB secret (for the popup probe).
// mode=popctl: window.open at load, no gesture (the harness's popup control).
const GROUP_PAGE = `<!doctype html><meta charset=utf-8><title>group</title><button id=open style="width:300px;height:100px">open</button><script>
${IDB}
const q = new URLSearchParams(location.search);
const app = q.get('app');
window.__asked = [];
let opened = null;
addEventListener('message', (e) => {
  if (e.origin !== location.origin || !e.data?.want) return;
  let sameTab; try { sameTab = e.source.top === opened; } catch (x) { sameTab = 'THREW ' + x.name; }
  let parentOrigin = e.data.parentOrigin;
  window.__asked.push({ from: e.origin, sameTab, parentOrigin });
  e.source.postMessage({ authority: 'scoped-' + parentOrigin, sameTab }, location.origin);
});
try { const bc = new BroadcastChannel('h'); bc.onmessage = (e) => { if (e.data === 'ping') bc.postMessage('pong'); }; } catch {}
(async () => {
  const mode = q.get('mode');
  if (mode === 'opener') document.getElementById('open').onclick = () => { opened = window.open(app); window.__opened = !!opened; };
  if (mode === 'nav') location.assign(app);
  if (mode === 'cookie') {
    const v = 'from-group-' + Date.now();
    document.cookie = '__Host-s=' + v + '; Secure; Path=/; SameSite=Strict; Max-Age=43200';
    document.cookie = '__Host-l=' + v + '; Secure; Path=/; SameSite=Lax; Max-Age=43200';
    document.cookie = '__Host-n=' + v + '; Secure; Path=/; SameSite=None; Max-Age=43200';
    localStorage.setItem('v', v);
    window.__done = { v, cookie: document.cookie };
    if (q.get('go')) location.assign(app);
  }
  if (mode === 'write') { await put('first', 'v', 'first-party-secret'); window.__done = { wrote: true }; }
  if (mode === 'popctl') { const w = window.open(location.origin + '/approve.html'); window.__done = { popup: w ? 'opened' : 'blocked' }; }
})();
</script>`;

const FRAME = `<!doctype html><meta charset=utf-8><title>signer</title><button id=b style="width:300px;height:150px">approve</button><script>
${IDB}
const q = new URLSearchParams(location.search);
const out = { origin: location.origin, ancestor: location.ancestorOrigins ? [...location.ancestorOrigins] : 'n/a' };
const send = (extra) => parent.postMessage({ ...out, ...extra }, '*');
const wait = (pred, ms) => new Promise((ok) => { const t = setTimeout(() => ok(null), ms); addEventListener('message', function f(e) { if (pred(e)) { clearTimeout(t); removeEventListener('message', f); ok(e); } }); });
let parentOrigin = null;
addEventListener('message', (e) => { if (e.source === parent) parentOrigin = e.origin; });
async function popup(how) {
  const t0 = performance.now();
  const w = window.open(location.origin + '/approve.html');
  out['popup_' + how] = w ? 'opened' : 'blocked';
  if (!w) return;
  const e = await wait((e) => e.origin === location.origin && e.data?.firstParty !== undefined, 5000);
  out['popup_' + how + '_read'] = e ? e.data.firstParty : 'no answer in 5 s';
  out['popup_' + how + '_ms'] = Math.round(performance.now() - t0);
}
(async () => {
  out.ancestorTop = Array.isArray(out.ancestor) ? out.ancestor.at(-1) : null;
  let o = null;
  try { o = top.opener; out.opener = o ? (o.closed ? 'closed' : 'present') : 'null'; } catch (x) { out.opener = 'THREW ' + x.name; }
  try { out.stored = await get('h', 'authority'); } catch (x) { out.stored = 'THREW ' + x.name; }
  out.cookie = document.cookie || null;
  try { out.ls = localStorage.getItem('v'); } catch (x) { out.ls = 'THREW ' + x.name; }
  try {
    const bc = new BroadcastChannel('h');
    const pong = new Promise((ok) => { const t = setTimeout(() => ok('no pong'), 800); bc.onmessage = (e) => { if (e.data === 'pong') { clearTimeout(t); ok('pong'); } }; });
    bc.postMessage('ping'); out.bc = await pong; bc.close();
  } catch (x) { out.bc = 'THREW ' + x.name; }
  try {
    const seed = crypto.getRandomValues(new Uint8Array(32));
    const pk8 = new Uint8Array([0x30,0x2e,0x02,0x01,0x00,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x04,0x22,0x04,0x20, ...seed]);
    const k = await crypto.subtle.importKey('pkcs8', pk8, 'Ed25519', false, ['sign']);
    out.ed25519 = (await crypto.subtle.sign('Ed25519', k, new Uint8Array(8))).byteLength + 'B sig from a raw seed';
  } catch (x) { out.ed25519 = 'THREW ' + x.name; }
  if (q.get('ask') === 'opener') {
    if (!o) out.reply = 'no opener: Open this app again from your group page.';
    else {
      const t0 = performance.now();
      o.postMessage({ want: 'authority', parentOrigin: out.ancestorTop }, location.origin);
      const e = await wait((e) => e.origin === location.origin && e.source === o, 3000);
      out.reply = e ? e.data : 'no reply in 3 s';
      out.reply_ms = Math.round(performance.now() - t0);
      if (e) { await put('h', 'authority', e.data.authority); out.storedNow = await get('h', 'authority'); }
    }
  }
  const p = q.get('popup');
  if (p === 'load') await popup('load');
  if (p === 'click') document.getElementById('b').onclick = async () => { await popup('click'); send({ final: true }); };
  if (p === 'msg') addEventListener('message', async (e) => { if (e.source === parent && e.data?.go) { await popup('msg'); send({ final: true }); } });
  send({ final: !['click', 'msg'].includes(p) });
})();
</script>`;

const APPROVE = `<!doctype html><meta charset=utf-8><title>approve</title><script>
${IDB}
(async () => {
  let v; try { v = await get('first', 'v'); } catch (x) { v = 'THREW ' + x.name; }
  window.__read = v;
  if (window.opener) window.opener.postMessage({ firstParty: v }, location.origin);
  setTimeout(() => window.close(), 50);
})();
</script>`;

const APP = (src, toss) => `<!doctype html><meta charset=utf-8><title>app</title><button id=top style="width:300px;height:100px">write</button><script>
window.__results = [];
addEventListener('message', (e) => { if (e.origin === '${G}') { window.__results.push(e.data); if (e.data.final) window.__result = e.data; } });
if (${toss}) {
  const t = {};
  const tries = {
    hostPrefixHere: '__Host-s=evil-host-only; Secure; Path=/',
    hostPrefixParent: '__Host-s=evil-parent; Secure; Path=/; Domain=${GROUP}',
    plainParent: 'tossed=evil-parent; Secure; Path=/; Domain=${GROUP}',
  };
  for (const [k, c] of Object.entries(tries)) { document.cookie = c; }
  window.__toss = { appSees: document.cookie };
}
document.getElementById('top').onclick = () => document.querySelector('iframe').contentWindow.postMessage({ go: 1 }, '${G}');
const popctl = new URLSearchParams(location.search).get('popctl');
if (popctl) { const w = window.open('${G}/approve.html'); window.__popctl = w ? 'opened' : 'blocked'; }
</script><iframe src="${src}" style="width:320px;height:170px"></iframe>`;

const server = https.createServer({ key: fs.readFileSync(path.join(__dirname, 'key.pem')), cert: fs.readFileSync(path.join(__dirname, 'cert.pem')) }, (req, res) => {
  const host = req.headers.host.split(':')[0];
  const u = new URL(req.url, 'https://' + host);
  if (process.env.LOG) console.log(host, u.pathname + u.search, req.headers.cookie ?? '');
  res.setHeader('content-type', 'text/html; charset=utf-8');
  if (host === GROUP) {
    if (u.pathname === '/sign.html') return res.end(FRAME);
    if (u.pathname === '/approve.html') return res.end(APPROVE);
    if (u.pathname === '/') return res.end(GROUP_PAGE);
  } else if (u.pathname === '/') {
    return res.end(APP(`${G}/sign.html${u.search}`, u.searchParams.has('toss')));
  }
  res.statusCode = 404; res.end('nope');
});
const HTTPS_PORT = +(process.env.HTTPS_PORT || 9443);
server.listen(HTTPS_PORT, '127.0.0.1');
module.exports = { HTTPS_PORT, GROUP };
