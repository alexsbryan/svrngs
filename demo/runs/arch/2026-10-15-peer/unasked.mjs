// A page Chromium takes to be on the internet, NOT given leave to reach the
// local network, asks a stand-in listener on 127.0.0.1:30999 (listen.mjs).
import { chromium } from '/Users/alexsbryan/dev/svrngs/web/node_modules/playwright/index.mjs';
import http from 'node:http';
const server = http.createServer((_, res) => res.writeHead(200, {'content-type':'text/html'}).end('<title>x</title>'));
await new Promise((d) => server.listen(0, '127.0.0.1', d));
const at = server.address().port;
const b = await chromium.launch({ args: [`--ip-address-space-overrides=127.0.0.1:${at}=public`] });
const c = await b.newContext();
const logs = []; c.on('console', (m) => logs.push(m.text()));
const p = await c.newPage();
await p.goto(`http://127.0.0.1:${at}/`);
const r = await p.evaluate(async () => { try { const r = await fetch('http://127.0.0.1:30999/choices/book', {method:'POST', mode:'no-cors', body:'unasked'}); return r.type; } catch (e) { return String(e); } });
console.log(JSON.stringify({ chromium: b.version(), page: `http://127.0.0.1:${at}/`, fetch: r, logs }));
await b.close(); server.close();
