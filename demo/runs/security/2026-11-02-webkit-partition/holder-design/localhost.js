// The tests' layout (web/tests/page.mjs:260): the group page at
// http://localhost:<port>/, apps at http://<label>.localhost:<port>/.
// Does a cookie the group page sets reach its framed signer under an app?
// usage: node localhost.js
const http = require('http');
const pw = require('/Users/alexsbryan/dev/svrngs/web/node_modules/playwright');
const PORT = 9480;
const L = 'a'.repeat(31) + '1'.repeat(31);
const GROUP = `http://localhost:${PORT}`;
const page = (body) => `<!doctype html><meta charset=utf-8><script>${body}</script>`;
http.createServer((req, res) => {
  const host = req.headers.host.split(':')[0];
  const u = new URL(req.url, 'http://x');
  res.setHeader('content-type', 'text/html; charset=utf-8');
  if (host === 'localhost' && u.pathname === '/') return res.end(page(`
    document.cookie = '__Host-s=hostpfx; Secure; Path=/; SameSite=Strict; Max-Age=600';
    document.cookie = 'p=plain; Path=/; SameSite=Strict; Max-Age=600';
    document.cookie = 'q=secure; Secure; Path=/; SameSite=Strict; Max-Age=600';
    window.__done = { top: document.cookie, header: ${JSON.stringify(req.headers.cookie ?? null)} };`));
  if (host === 'localhost' && u.pathname === '/sign.html') return res.end(page(`parent.postMessage({ frame: document.cookie, header: ${JSON.stringify(req.headers.cookie ?? null)} }, '*');`));
  if (host === `${L}.localhost`) return res.end(page(`addEventListener('message', (e) => (window.__result = e.data));`) + `<iframe src="${GROUP}/sign.html"></iframe>`);
  res.statusCode = 404; res.end();
}).listen(PORT, '127.0.0.1');
(async () => {
  for (const [name, type] of [['webkit', pw.webkit], ['chromium', pw.chromium]]) {
    const b = await type.launch();
    const ctx = await b.newContext();
    const g = await ctx.newPage();
    await g.goto(GROUP + '/');
    const top = await g.waitForFunction(() => window.__done).then((h) => h.jsonValue());
    const a = await ctx.newPage();
    await a.goto(`http://${L}.localhost:${PORT}/`);
    const fr = await a.waitForFunction(() => window.__result, null, { timeout: 8000 }).then((h) => h.jsonValue()).catch((e) => ({ error: String(e).split('\n')[0] }));
    console.log(JSON.stringify({ engine: name, version: b.version(), groupPageSees: top.top, framedSignerSees: fr.frame, sentWithFrameLoad: fr.header, error: fr.error }));
    await b.close();
  }
  process.exit(0);
})();
