// Seeds with a leading zero byte (and some without), imported into WebKit
// by pkcs8 and by JWK {d, x}; each JWK key's signature compared with
// Node's for the same seed and message (Ed25519 signs deterministically).
import { createPrivateKey, sign as nodeSign, randomBytes } from 'node:crypto';
import http from 'node:http';
const server = http.createServer((q, r) => r.end('<title>x</title>')).listen(0, '127.0.0.1');
await new Promise((r) => server.once('listening', r));
const pw = await import('/work/web/node_modules/playwright/index.mjs');
const head = Buffer.from('302e020100300506032b657004220420', 'hex');
const cases = [];
for (let i = 0; i < 300; i++) {
  const seed = randomBytes(32);
  if (i < 200) seed[0] = 0;
  if (i < 20) seed[1] = 0;
  const der = Buffer.concat([head, seed]);
  const key = createPrivateKey({ key: der, format: 'der', type: 'pkcs8' });
  const jwk = key.export({ format: 'jwk' });
  const msg = randomBytes(32);
  cases.push({ seed: seed.toString('hex'), d: jwk.d, x: jwk.x, msg: msg.toString('hex'), sig: nodeSign(null, msg, key).toString('hex') });
}
const browser = await pw[process.argv[2] ?? 'webkit'].launch();
const page = await browser.newPage();
await page.goto(`http://127.0.0.1:${server.address().port}/`);
const out = await page.evaluate(async (cases) => {
  const hex = (b) => [...new Uint8Array(b)].map((x) => x.toString(16).padStart(2, '0')).join('');
  const bytes = (h) => Uint8Array.from(h.match(/../g), (p) => parseInt(p, 16));
  const head = bytes('302e020100300506032b657004220420');
  const tally = { pkcs8_failed: 0, jwk_failed: 0, jwk_wrong_signature: 0, jwk_ok: 0, errors: {} };
  for (const c of cases) {
    try { await crypto.subtle.importKey('pkcs8', Uint8Array.from([...head, ...bytes(c.seed)]), 'Ed25519', false, ['sign']); }
    catch { tally.pkcs8_failed++; }
    try {
      const key = await crypto.subtle.importKey('jwk', { kty: 'OKP', crv: 'Ed25519', d: c.d, x: c.x }, 'Ed25519', false, ['sign']);
      const sig = hex(await crypto.subtle.sign('Ed25519', key, bytes(c.msg)));
      if (sig === c.sig) tally.jwk_ok++; else tally.jwk_wrong_signature++;
    } catch (e) { tally.jwk_failed++; tally.errors[e.message] = (tally.errors[e.message] ?? 0) + 1; }
  }
  return tally;
}, cases);
console.log(JSON.stringify({ engine: process.argv[2] ?? 'webkit', cases: cases.length, leading_zero: 200, ...out }));
await browser.close();
server.close();
