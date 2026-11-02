// As localhost.js, over HTTPS (lcert.pem: localhost, *.localhost, *.page.localhost):
// would a TLS test layout let WebKit's framed signer read the group page's cookie?
// usage: node localhost-tls.js
const https = require('https');
const fs = require('fs');
const pw = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright');
const PORT = 9444;
const L = 'a'.repeat(31) + '1'.repeat(31);
const page = (body) => `<!doctype html><meta charset=utf-8><script>${body}</script>`;
const GROUPS = ['localhost', 'page.localhost'];
https.createServer({ key: fs.readFileSync(__dirname + '/lkey.pem'), cert: fs.readFileSync(__dirname + '/lcert.pem') }, (req, res) => {
  const host = req.headers.host.split(':')[0];
  const u = new URL(req.url, 'https://x');
  res.setHeader('content-type', 'text/html; charset=utf-8');
  if (GROUPS.includes(host) && u.pathname === '/') return res.end(page(`
    document.cookie = '__Host-s=hostpfx; Secure; Path=/; SameSite=Strict; Max-Age=600';
    window.__done = document.cookie;`));
  if (GROUPS.includes(host) && u.pathname === '/sign.html') return res.end(page(`parent.postMessage({ frame: document.cookie }, '*');`));
  if (host.startsWith(L + '.')) return res.end(page(`addEventListener('message', (e) => (window.__result = e.data));`) + `<iframe src="https://${host.slice(L.length + 1)}:${PORT}/sign.html"></iframe>`);
  res.statusCode = 404; res.end();
}).listen(PORT, '127.0.0.1');
(async () => {
  for (const [name, type] of [['webkit', pw.webkit], ['chromium', pw.chromium]]) {
    const b = await type.launch();
    for (const group of GROUPS) {
      const ctx = await b.newContext({ ignoreHTTPSErrors: true });
      const g = await ctx.newPage();
      await g.goto(`https://${group}:${PORT}/`);
      const top = await g.waitForFunction(() => window.__done).then((h) => h.jsonValue());
      const a = await ctx.newPage();
      await a.goto(`https://${L}.${group}:${PORT}/`);
      const fr = await a.waitForFunction(() => window.__result, null, { timeout: 8000 }).then((h) => h.jsonValue()).catch((e) => ({ error: String(e).split('\n')[0] }));
      console.log(JSON.stringify({ engine: name, group, groupPageSees: top, framedSignerSees: fr.frame, error: fr.error }));
      await ctx.close();
    }
    await b.close();
  }
  process.exit(0);
})();
