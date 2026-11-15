// The page's own authority.js `ed25519`, in each engine, on seeds with a
// leading zero byte (200 of 300): imported, signing as Node does, and the
// holder `holderMatches` names.
import { createPrivateKey, sign as nodeSign, randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import http from 'node:http';
const pw = await import('/work/web/node_modules/playwright/index.mjs');
const server = http.createServer((q, r) => {
  const path = new URL(q.url, 'http://x').pathname;
  if (path === '/') return r.end('<title>x</title>');
  try { r.writeHead(200, { 'content-type': 'text/javascript' }); r.end(readFileSync(`/work/web${path}`)); }
  catch { r.writeHead(404); r.end(); }
}).listen(0, '127.0.0.1');
await new Promise((r) => server.once('listening', r));
const head = Buffer.from('302e020100300506032b657004220420', 'hex');
const cases = [];
for (let i = 0; i < 300; i++) {
  const seed = randomBytes(32);
  if (i < 200) seed[0] = 0;
  const key = createPrivateKey({ key: Buffer.concat([head, seed]), format: 'der', type: 'pkcs8' });
  const pub = Buffer.from(key.export({ format: 'jwk' }).x, 'base64url').toString('hex');
  const msg = randomBytes(32);
  cases.push({ seed: seed.toString('hex'), pub, msg: msg.toString('hex'), sig: nodeSign(null, msg, key).toString('hex') });
}
for (const engine of ['webkit', 'chromium']) {
  const browser = await pw[engine].launch();
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const out = await page.evaluate(async (cases) => {
    const { ed25519, fromHex, holderMatches } = await import('/authority.js?v=__VERSION__');
    const hex = (b) => [...new Uint8Array(b)].map((x) => x.toString(16).padStart(2, '0')).join('');
    const t = { ok: 0, failed: 0, wrong: 0, notHolder: 0, errors: {} };
    for (const c of cases) {
      try {
        const key = await crypto.subtle.importKey('jwk', ed25519(c.seed, c.pub), 'Ed25519', false, ['sign']);
        if (hex(await crypto.subtle.sign('Ed25519', key, fromHex(c.msg))) !== c.sig) t.wrong++;
        else if (!(await holderMatches(key, c.pub))) t.notHolder++;
        else t.ok++;
      } catch (e) { t.failed++; t.errors[e.message] = (t.errors[e.message] ?? 0) + 1; }
    }
    return t;
  }, cases);
  console.log(JSON.stringify({ engine, cases: 300, leading_zero: 200, ...out }));
  await browser.close();
}
server.close();
