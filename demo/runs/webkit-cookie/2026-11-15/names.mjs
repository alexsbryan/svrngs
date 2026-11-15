// Which names let the group's signer, a frame at the page's name under an
// app's name below it, write its cookie: production's under .sh, the tests'
// under .test, and names under example.com. Recorded in README.md here.
import http from 'node:http';
import net from 'node:net';
import tls from 'node:tls';
import fs from 'node:fs';
import { createRequire } from 'node:module';
const req = createRequire('/work/web/package.json');
const pw = req('playwright');
const ctx = tls.createSecureContext({ key: fs.readFileSync('/work/target/tls/page/key.pem'), cert: fs.readFileSync('/work/target/tls/page/cert.pem') });
const NAME = '__Host-svrngs-money';
const DEL = `${NAME}=; Max-Age=0; Path=/; Secure; SameSite=Strict`;
const server = http.createServer((q, res) => {
  const u = new URL(q.url, 'http://x');
  res.setHeader('content-type', 'text/html');
  const parent = u.searchParams.get('p');
  if (u.pathname === '/set') return res.end(`<script>document.cookie='${NAME}=abc; Secure; Path=/; SameSite=Strict; Max-Age=600';document.title='set'</script>`);
  if (u.pathname === '/app') return res.end(`<script>addEventListener('message',e=>{document.title=JSON.stringify(e.data)});</script><iframe src="https://${parent}:${server.address().port}/sign"></iframe>`);
  if (u.pathname === '/sign') return res.end(`<script>(async()=>{const access=document.hasStorageAccess?await document.hasStorageAccess():'none';const before=document.cookie;document.cookie=${JSON.stringify(DEL)};document.cookie='probe=1; Secure; Path=/; SameSite=None';parent.postMessage({access,before,after:document.cookie},'*')})()</script>`);
  res.end('?');
});
await new Promise((d) => server.listen(0, '127.0.0.1', d));
const port = server.address().port;
const proxy = http.createServer();
proxy.on('connect', (r, sock, head) => {
  sock.write('HTTP/1.1 200 Connection Established\r\n\r\n');
  if (head?.length) sock.unshift(head);
  sock.on('error', () => sock.destroy());
  const t = new tls.TLSSocket(sock, { isServer: true, secureContext: ctx });
  const n = net.connect(port, '127.0.0.1');
  t.pipe(n).pipe(t);
  [t, n].forEach((s) => s.on('error', () => (t.destroy(), n.destroy())));
});
await new Promise((d) => proxy.listen(0, '127.0.0.1', d));
const pairs = [
  ['group.svrnme.test', 'money.group.svrnme.test'],
  ['group.svrnme.sh', 'money.group.svrnme.sh'],
  ['group.example.com', 'money.group.example.com'],
  ['group.svrnme.example.com', 'money.group.svrnme.example.com'],
  ['group.svrnme.sh', 'group.svrnme.sh'],
];
for (const engine of ['webkit', 'chromium']) for (const [parent, top] of pairs) {
  const browser = await pw[engine].launch();
  const c = await browser.newContext({ ignoreHTTPSErrors: true, proxy: { server: `http://127.0.0.1:${proxy.address().port}` } });
  const p = await c.newPage();
  await p.goto(`https://${parent}:${port}/set`);
  await p.goto(`https://${top}:${port}/app?p=${parent}`);
  await p.waitForFunction(() => document.title.startsWith('{'), null, { timeout: 10000 }).catch(() => {});
  const said = await p.title();
  const jar = (await c.cookies(`https://${parent}:${port}/`)).map((k) => k.name);
  console.log(JSON.stringify({ engine, parent, top, said, jar }));
  await browser.close();
}
process.exit(0);
