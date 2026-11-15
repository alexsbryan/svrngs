// WebKit's Ed25519 on random seeds: pkcs8 import, sign, raw public import,
// verify; which steps throw, and on what bytes.
const pw = await import('/work/web/node_modules/playwright/index.mjs');
const browser = await pw[process.argv[2] ?? 'webkit'].launch();
const page = await browser.newPage();
await page.goto('https://example.com/').catch(() => {});
await page.setContent('<title>x</title>');
const out = await page.evaluate(async (n) => {
  const hex = (b) => [...new Uint8Array(b)].map((x) => x.toString(16).padStart(2, '0')).join('');
  const head = Uint8Array.from([0x30,0x2e,0x02,0x01,0x00,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x04,0x22,0x04,0x20]);
  const fails = {}; const samples = [];
  for (let i = 0; i < n; i++) {
    const seed = crypto.getRandomValues(new Uint8Array(32));
    let step = 'import pkcs8';
    try {
      const key = await crypto.subtle.importKey('pkcs8', Uint8Array.from([...head, ...seed]), 'Ed25519', true, ['sign']);
      step = 'export jwk';
      const jwk = await crypto.subtle.exportKey('jwk', key);
      step = 'sign';
      const msg = crypto.getRandomValues(new Uint8Array(32));
      const sig = new Uint8Array(await crypto.subtle.sign('Ed25519', key, msg));
      if (sig.length !== 64) throw new Error(`signature of ${sig.length} bytes`);
      step = 'import raw public';
      const x = Uint8Array.from(atob(jwk.x.replace(/-/g, '+').replace(/_/g, '/')), (c) => c.charCodeAt(0));
      if (x.length !== 32) throw new Error(`public of ${x.length} bytes`);
      const pub = await crypto.subtle.importKey('raw', x, 'Ed25519', false, ['verify']);
      step = 'verify';
      if (!(await crypto.subtle.verify('Ed25519', pub, sig, msg))) throw new Error('verify false');
    } catch (e) {
      const k = `${step}: ${e.name} ${e.message}`;
      fails[k] = (fails[k] ?? 0) + 1;
      if (samples.length < 6) samples.push({ k, seed: hex(seed) });
    }
  }
  return { n, fails, samples };
}, Number(process.argv[3] ?? 3000));
console.log(JSON.stringify(out));
await browser.close();
