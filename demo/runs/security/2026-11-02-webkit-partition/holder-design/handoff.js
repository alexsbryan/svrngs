// The one-time cookie handoff (HOLDER_APPROVAL.md §3.D): the group page seals
// a fresh Ed25519 approver seed under W, sets `__Host-svrngs-<label>=W` and
// navigates (same tab) to the app with the seal in the fragment. The signer
// frame, on the app's first ask, opens the seal with W, stores the approver
// key in its own IndexedDB keyed by the label of its parent's origin, and
// only then deletes the cookie. Later asks read the store.
// usage: node handoff.js [webkit|chromium|chromium-part ...]
// Needs key.pem/cert.pem (README) and proxy.js beside it.
process.env.HTTPS_PORT ??= '9445';
process.env.PROXY_PORT ??= '9889';
const https = require('https');
const fs = require('fs');
const os = require('os');
const path = require('path');
const nodeCrypto = require('crypto');
const pw = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright');
const { PROXY_PORT } = require('./proxy.js');

const GROUP = 'group.svrnme.sh';
const G = `https://${GROUP}`;
const L1 = 'a'.repeat(31) + '1'.repeat(31);
const L2 = 'b'.repeat(31) + '2'.repeat(31);
const APP1 = `https://${L1}.${GROUP}/`;
const APP2 = `https://${L2}.${GROUP}/`;

const COMMON = `
const enc = new TextEncoder();
const b64u = (b) => btoa(String.fromCharCode(...new Uint8Array(b))).replace(/\\+/g, '-').replace(/\\//g, '_').replace(/=+$/, '');
const unb64u = (s) => Uint8Array.from(atob(s.replace(/-/g, '+').replace(/_/g, '/')), (c) => c.charCodeAt(0));
const openMessage = (h) => JSON.stringify({ label: h.label, pub: h.pub, n: h.n });
`;

// The group page: mint, seal, set the cookie, go (or, with &stay, hand back the open).
const GROUP_PAGE = `<!doctype html><meta charset=utf-8><title>group</title><script>${COMMON}
(async () => {
  const q = new URLSearchParams(location.search);
  if (q.has('peek')) return (window.__done = { cookie: document.cookie });
  const app = new URL(q.get('go'));
  const label = app.hostname.split('.')[0];
  const pair = await crypto.subtle.generateKey('Ed25519', true, ['sign', 'verify']);
  const seed = new Uint8Array(await crypto.subtle.exportKey('pkcs8', pair.privateKey)).slice(-32);
  const pub = b64u(await crypto.subtle.exportKey('raw', pair.publicKey));
  const W = crypto.getRandomValues(new Uint8Array(32));
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const open = { label, pub, n: q.get('n') };
  const k = await crypto.subtle.importKey('raw', W, 'AES-GCM', false, ['encrypt']);
  const ct = await crypto.subtle.encrypt({ name: 'AES-GCM', iv, additionalData: enc.encode(openMessage(open)) }, k, seed);
  document.cookie = '__Host-svrngs-' + label + '=' + b64u(W) + '; Secure; Path=/; SameSite=Strict; Max-Age=43200';
  const handed = { ...open, proof: b64u(iv) + '.' + b64u(ct) };
  window.__done = { handed };
  if (!q.has('stay')) location.assign(app.href + '#' + encodeURIComponent(JSON.stringify(handed)));
})().catch((e) => (window.__done = { error: String(e) }));
</script>`;

// The signer. The label is the one its parent's origin names, never the ask's.
const FRAME = `<!doctype html><meta charset=utf-8><title>signer</title><script>${COMMON}
const db = () => new Promise((ok, no) => {
  const r = indexedDB.open('svrngs-approvers', 1);
  r.onupgradeneeded = () => r.result.createObjectStore('approvers', { keyPath: 'label' });
  r.onsuccess = () => ok(r.result); r.onerror = () => no(r.error);
});
const tx = async (mode, f) => { const d = await db(); return new Promise((ok, no) => { const t = d.transaction('approvers', mode); const r = f(t.objectStore('approvers')); t.oncomplete = () => ok(r?.result); t.onerror = () => no(t.error); }); };
const digest = async (s) => b64u(await crypto.subtle.digest('SHA-256', enc.encode(s)));
addEventListener('message', async (e) => {
  const port = e.ports[0];
  if (e.source !== parent || !port) return;
  const out = {};
  try {
    const host = new URL(e.origin).host, suffix = '.' + location.host;
    if (!host.endsWith(suffix) || !/^[a-z0-9]{62}$/.test(host.slice(0, -suffix.length))) throw new Error('another origin');
    const label = host.slice(0, -suffix.length);
    out.visible = (await tx('readonly', (s) => s.getAllKeys())).map((k) => k.slice(0, 4) + '…');
    const h = e.data?.open;
    if (!h || h.label !== label) throw new Error('an open for another app');
    const name = '__Host-svrngs-' + label;
    const jar = document.cookie.split('; ').filter((c) => c.startsWith(name + '='));
    out.cookieSeen = jar.length;
    if (jar.length === 1) {
      const W = unb64u(jar[0].slice(name.length + 1));
      const [iv, ct] = h.proof.split('.').map(unb64u);
      const k = await crypto.subtle.importKey('raw', W, 'AES-GCM', false, ['decrypt']);
      const seed = new Uint8Array(await crypto.subtle.decrypt({ name: 'AES-GCM', iv, additionalData: enc.encode(openMessage(h)) }, k, ct));
      const key = await crypto.subtle.importKey('pkcs8', new Uint8Array([0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20, ...seed]), 'Ed25519', false, ['sign']);
      // Store first, then delete: a second frame finds the cookie or the record, never neither.
      const d = await digest(openMessage(h));
      await tx('readwrite', (s) => s.put({ label, pub: h.pub, digest: d, key }));
      out.adopted = true;
    }
    const rec = await tx('readonly', (s) => s.get(label));
    if (!rec) throw new Error('no approver stored for this app');
    if (rec.digest !== await digest(openMessage(h))) throw new Error('not the open this signer adopted');
    if (jar.length === 1) {
      document.cookie = name + '=; Max-Age=0; Path=/; Secure';
      out.cookieAfterDelete = document.cookie.includes(name);
    }
    out.pub = rec.pub;
    out.keyExtractable = rec.key.extractable;
    if (e.data.sign) out.sig = b64u(await crypto.subtle.sign('Ed25519', rec.key, enc.encode(e.data.sign)));
    out.ok = true;
  } catch (x) { out.ok = false; out.why = x.message || String(x); out.said = 'Open this app again from your group’s page.'; }
  port.postMessage(out);
});
</script>`;

// The app: the open from its fragment, else the one it kept; asks the signer.
const APP = `<!doctype html><meta charset=utf-8><title>app</title><script>
let handed;
const q = new URLSearchParams(location.search);
if (location.hash.length > 1) {
  handed = JSON.parse(decodeURIComponent(location.hash.slice(1)));
  localStorage.setItem('handed', JSON.stringify(handed));
  history.replaceState(null, '', location.pathname + location.search);
} else handed = JSON.parse(localStorage.getItem('handed') || 'null');
if (q.has('claim')) handed = JSON.parse(q.get('claim'));
window.__handed = handed;
const frame = document.createElement('iframe');
frame.src = '${G}/sign.html';
const ask = (data) => new Promise((ok) => { const c = new MessageChannel(); c.port1.onmessage = (e) => ok(e.data); frame.contentWindow.postMessage(data, '${G}', [c.port2]); });
window.__sign = (text) => ask({ open: handed, sign: text });
frame.onload = async () => { window.__result = await ask({ open: handed }); };
document.documentElement.append(frame);
</script>`;

const wire = [];
https.createServer({ key: fs.readFileSync(path.join(__dirname, 'key.pem')), cert: fs.readFileSync(path.join(__dirname, 'cert.pem')) }, (req, res) => {
  const host = req.headers.host.split(':')[0];
  const u = new URL(req.url, 'https://' + host);
  res.setHeader('content-type', 'text/html; charset=utf-8');
  if (host === GROUP) {
    wire.push({ path: u.pathname, cookie: req.headers.cookie ?? '' });
    if (u.pathname === '/sign.html') return res.end(FRAME);
    if (u.pathname === '/') return res.end(GROUP_PAGE);
  } else if (u.pathname === '/') return res.end(APP);
  res.statusCode = 404; res.end();
}).listen(+process.env.HTTPS_PORT, '127.0.0.1');

const { chromiumSwitches } = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright-core/lib/server/chromium/chromiumSwitches.js');
const PW_DISABLE = chromiumSwitches(false).find((a) => a.startsWith('--disable-features='));
const part = { ignoreDefaultArgs: [PW_DISABLE], args: [PW_DISABLE.replace(/,?ThirdPartyStoragePartitioning/, '')] };
const ENGINES = { webkit: [pw.webkit, {}], chromium: [pw.chromium, {}], 'chromium-part': [pw.chromium, part] };
const base = { ignoreHTTPSErrors: true, proxy: { server: `http://127.0.0.1:${PROXY_PORT}` } };

const pubDer = (pub) => Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), Buffer.from(pub, 'base64url')]);
const verifies = (pub, text, sig) => !!sig && nodeCrypto.verify(null, Buffer.from(text), { key: pubDer(pub), format: 'der', type: 'spki' }, Buffer.from(sig, 'base64url'));
const settled = (p) => p.waitForFunction(() => window.__result, null, { timeout: 10000 }).then((h) => h.jsonValue()).catch((e) => ({ error: String(e).split('\n')[0] }));
const sign = (p, text) => p.evaluate((t) => window.__sign(t), text);
const pick = ({ visible, cookieSeen, adopted, cookieAfterDelete, ok, why, keyExtractable, error }) => ({ visible, cookieSeen, adopted, cookieAfterDelete, ok, why, keyExtractable, error });
const svrngsIn = (s) => (s.match(/__Host-svrngs-[a-z0-9]{4}/g) ?? []).map((n) => n + '…');
const jar = async (ctx) => (await ctx.cookies(G)).filter((c) => c.name.startsWith('__Host-svrngs-')).length;
const log = (engine, step, r) => console.log(JSON.stringify({ engine, step, ...r }));

async function run(name, [type, opts]) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ho-'));
  let ctx = await type.launchPersistentContext(dir, { ...opts, ...base });
  // 1. Open, adopt, delete.
  const t1 = await ctx.newPage();
  await t1.goto(`${G}/?n=1&go=${encodeURIComponent(APP1)}`);
  await t1.waitForURL((u) => u.host.startsWith(L1));
  log(name, 'H1 first ask under the app: adopt, then delete the cookie', pick(await settled(t1)));
  const open1 = await t1.evaluate(() => window.__handed);
  const sig1 = (await sign(t1, 'entry-1')).sig;
  log(name, 'H1 a signature with the adopted key verifies under the open\'s public key', { verifies: verifies(open1.pub, 'entry-1', sig1) });
  const peek = await ctx.newPage();
  await peek.goto(`${G}/?peek`);
  const seen = await peek.waitForFunction(() => window.__done).then((h) => h.jsonValue());
  await peek.close();
  log(name, 'H1 after the delete: group page\'s document.cookie, the browser\'s jar', { groupPage: svrngsIn(seen.cookie), jar: await jar(ctx) });
  // 2. Later visits: reload, restart.
  const before = wire.length;
  await t1.goto(APP1);
  const r2 = await settled(t1);
  const sig2 = (await sign(t1, 'entry-2')).sig;
  log(name, 'H2 app reloaded, no cookie', { ...pick(r2), verifies: verifies(open1.pub, 'entry-2', sig2), signHtmlCookieOnWire: wire.slice(before).filter((w) => w.path === '/sign.html').map((w) => svrngsIn(w.cookie)) });
  await ctx.close();
  ctx = await type.launchPersistentContext(dir, { ...opts, ...base });
  const t1b = await ctx.newPage();
  await t1b.goto(APP1);
  const r3 = await settled(t1b);
  const sig3 = (await sign(t1b, 'entry-3')).sig;
  log(name, 'H2 browser restarted, app from a bookmark', { ...pick(r3), verifies: verifies(open1.pub, 'entry-3', sig3) });
  // 3. A second open while the first is stored; the first's page still open.
  const t2 = await ctx.newPage();
  await t2.goto(`${G}/?n=2&go=${encodeURIComponent(APP1)}`);
  await t2.waitForURL((u) => u.host.startsWith(L1));
  log(name, 'H3 second open of the same app: adopt the new one', pick(await settled(t2)));
  const open2 = await t2.evaluate(() => window.__handed);
  const s5 = await sign(t2, 'entry-5');
  log(name, 'H3 new open signs under its own key', { newKey: open2.pub !== open1.pub, verifies: verifies(open2.pub, 'entry-5', s5.sig) });
  log(name, 'H3 the page still holding the first open asks again', pick(await sign(t1b, 'entry-4')));
  log(name, 'H3 entry-1, approved under the first open, still verifies', { verifies: verifies(open1.pub, 'entry-1', sig1) });
  // 4. A second app's frame.
  const a2 = await ctx.newPage();
  await a2.goto(`${APP2}?claim=${encodeURIComponent(JSON.stringify(open2))}`);
  log(name, 'H4 another app presents the first app\'s open', pick(await settled(a2)));
  await a2.goto(`${APP2}?claim=${encodeURIComponent(JSON.stringify({ ...open2, label: L2 }))}`);
  log(name, 'H4 another app presents it relabelled as its own', pick(await settled(a2)));
  // 5. Two tabs of the same app race on one cookie, 10 rounds.
  for (const [how, gap] of [['at once', null], ['30 ms apart', 30], ['second after the first settled', 'settled']]) {
  const tally = {};
  for (let i = 0; i < 10; i++) {
    const g = await ctx.newPage();
    await g.goto(`${G}/?n=r${i}&stay&go=${encodeURIComponent(APP1)}`);
    const { handed } = await g.waitForFunction(() => window.__done).then((h) => h.jsonValue());
    await g.close();
    const [x, y] = [await ctx.newPage(), await ctx.newPage()];
    const at = `${APP1}#${encodeURIComponent(JSON.stringify(handed))}`;
    if (gap === null) await Promise.all([x.goto(at), y.goto(at)]);
    else if (gap === 'settled') { await x.goto(at); await settled(x); await y.goto(at); }
    else await Promise.all([x.goto(at), new Promise((r) => setTimeout(r, gap)).then(() => y.goto(at))]);
    const rs = await Promise.all([settled(x), settled(y)]);
    const sigs = await Promise.all([sign(x, 'race'), sign(y, 'race')]);
    const k = rs.map((r) => (r.adopted ? 'adopted' : r.ok ? 'reused' : 'refused: ' + r.why)).sort().join(' + ') + ` | both verify: ${sigs.every((s) => verifies(handed.pub, 'race', s.sig))} | jar after: ${await jar(ctx)}`;
    tally[k] = (tally[k] ?? 0) + 1;
    await x.close(); await y.close();
  }
  log(name, `H5 two tabs, one cookie, ${how}, 10 rounds`, { tally });
  }
  await ctx.close();
}

(async () => {
  const want = process.argv.slice(2).length ? process.argv.slice(2) : ['webkit', 'chromium', 'chromium-part'];
  for (const name of want) {
    const [type, opts] = ENGINES[name];
    const b = await type.launch(opts);
    console.log(`# ${name} ${b.version()} (${new Date().toISOString()})`);
    await b.close();
    try { await run(name, ENGINES[name]); } catch (e) { log(name, 'THREW', { error: String(e).split('\n')[0] }); }
  }
  process.exit(0);
})();
