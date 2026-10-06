// The conformance canary (DESIGN §8, "The canaries"; u5-2): a localhost
// service with one route per thing a self-hosted app leans on, which the
// infra must carry whole to whoever opens it — on a computer, and on a
// phone at an address of its own or under one for every app (DESIGN §7).
// Its page (`/`) loads `checks.js`, which runs each route's check in the
// browser; crates/svrngs-node/tests/conformance.rs lends two of it, Sam's
// and Bo's, and judges each route in each way.
//
//   node apps/web-conformance/serve.mjs <who>
//
// Listens on 127.0.0.1, any port, and prints `{"port": N}`. No dependency.
//
// Routes:
//   GET  /            its page, under COOP and COEP and a script policy by
//                     nonce, asking `/abs.js` by a path from the root and
//                     `checks.js` by one from itself, each with the nonce
//   GET  /abs.js      a script found only at the root of its own address
//   GET  /checks.js   the checks
//   GET  /ws          a WebSocket echo
//   GET  /big.bin     64 MiB, by Range; byte i is (i * 7 + (i >> 10)) & 255
//   POST /upload      read as it streams; answers `<bytes> <sha-256> <its Content-Length>`
//   GET  /app-sw.js   its own service worker, answering `sw-probe`
//   GET  /sw-probe    `from the server`, when no worker of its own answers
//   GET  /redirect    refused unless asked on its own address,
//                     `localhost:<port>`; else an absolute redirect there
//   GET  /landed      where that lands
//   POST /signin      sets `canary-<who>`, and an HttpOnly `canary-<who>-hidden`
//   POST /guarded     refused when an `Origin` is not its own address, as
//                     an app checking a post's origin against its Host does
//   GET  /me          the cookies it was sent
//   GET  /seen        every request it took: path, cookies, person and key
import { createHash, randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import http from 'node:http';

const who = process.argv[2] ?? 'canary';
const SIZE = 64 * 1024 * 1024;
const byte = (i) => (i * 7 + (i >> 10)) & 255;
const seen = [];
const SAME = { 'cross-origin-resource-policy': 'same-origin', 'cache-control': 'no-store' };
const checks = readFileSync(new URL('./checks.js', import.meta.url));
// Under `script-src 'nonce-…' 'strict-dynamic'`, as Nextcloud's pages are:
// a script the page did not name with its nonce does not run.
const page = (nonce) => `<!doctype html><html><head><meta charset="utf-8"><title>conformance</title>
<script nonce="${nonce}" src="/abs.js"></script>
<script nonce="${nonce}" src="checks.js"></script>
</head><body>the conformance canary, ${who}'s</body></html>`;

const sends = (res, status, type, body, more = {}) => {
  res.writeHead(status, { ...SAME, 'content-type': type, ...more });
  res.end(body);
};

function bytes(from, to) {
  const out = Buffer.alloc(to - from + 1);
  for (let i = from; i <= to; i++) out[i - from] = byte(i);
  return out;
}

function big(req, res) {
  const m = /^bytes=(\d+)-(\d*)$/.exec(req.headers.range ?? '');
  if (!m) return sends(res, 200, 'application/octet-stream', bytes(0, 1023), { 'content-length': 1024 });
  const from = Number(m[1]);
  const to = Math.min(m[2] === '' ? SIZE - 1 : Number(m[2]), SIZE - 1);
  if (from >= SIZE || to < from) return sends(res, 416, 'text/plain', '', { 'content-range': `bytes */${SIZE}` });
  res.writeHead(206, { ...SAME, 'content-type': 'application/octet-stream', 'content-length': to - from + 1, 'content-range': `bytes ${from}-${to}/${SIZE}` });
  for (let at = from; at <= to; at += 1 << 20) res.write(bytes(at, Math.min(at + (1 << 20) - 1, to)));
  res.end();
}

const worker = `self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));
self.addEventListener('fetch', (e) => {
  if (new URL(e.request.url).pathname.endsWith('/sw-probe')) e.respondWith(new Response("from the app's worker"));
});`;

const server = http.createServer((req, res) => {
  const path = new URL(req.url, 'http://x').pathname;
  seen.push({ path, cookie: req.headers.cookie ?? '', person: req.headers['x-svrngs-person'] ?? '', key: req.headers['x-svrngs-key'] ?? '', secret: Boolean(req.headers.authorization) });
  const own = [`localhost:${server.address().port}`, `127.0.0.1:${server.address().port}`];
  if (req.method === 'POST' && path === '/upload') {
    const sha = createHash('sha256');
    let n = 0;
    req.on('data', (d) => ((n += d.length), sha.update(d)));
    // With the length it was told, as a parser that needs it reads (Django's multipart).
    return req.on('end', () => sends(res, 200, 'text/plain', `${n} ${sha.digest('hex')} ${req.headers['content-length'] ?? 'no length'}`));
  }
  if (req.method === 'POST' && path === '/guarded') {
    const origin = req.headers.origin;
    if (origin && !own.some((h) => origin === `http://${h}`)) return sends(res, 403, 'text/plain', `origin ${origin} is not ${own[0]}`, { 'cross-origin-embedder-policy': 'require-corp' });
    // Its answer may be a frame's page in the canary's, which COEP holds to its own.
    return sends(res, 200, 'text/plain', `taken from ${origin ?? 'no origin'}`, { 'cross-origin-embedder-policy': 'require-corp' });
  }
  if (req.method === 'POST' && path === '/signin') {
    const token = randomBytes(6).toString('hex');
    return sends(res, 200, 'application/json', JSON.stringify({ who }), {
      'set-cookie': [`canary-${who}=${token}; Path=/`, `canary-${who}-hidden=1; Path=/; HttpOnly`],
    });
  }
  switch (path) {
    case '/': {
      const nonce = randomBytes(12).toString('base64');
      return sends(res, 200, 'text/html; charset=utf-8', page(nonce), {
        'cross-origin-opener-policy': 'same-origin',
        'cross-origin-embedder-policy': 'require-corp',
        'content-security-policy': `script-src 'nonce-${nonce}' 'strict-dynamic'; worker-src 'self'`,
      });
    }
    case '/abs.js':
      return sends(res, 200, 'text/javascript', "window.__abs = 'ran';");
    case '/checks.js':
      return sends(res, 200, 'text/javascript', checks);
    case '/big.bin':
      return big(req, res);
    case '/app-sw.js':
      return sends(res, 200, 'text/javascript', worker);
    case '/sw-probe':
      return sends(res, 200, 'text/plain', 'from the server');
    case '/redirect':
      if (!own.includes(req.headers.host)) return sends(res, 403, 'text/plain', `host ${req.headers.host} is not allowed`);
      return sends(res, 302, 'text/plain', '', { location: `http://${req.headers.host}/landed` });
    case '/landed':
      return sends(res, 200, 'text/plain', 'landed');
    case '/me':
      return sends(res, 200, 'application/json', JSON.stringify({ who, cookie: req.headers.cookie ?? '' }));
    case '/seen':
      return sends(res, 200, 'application/json', JSON.stringify(seen));
    default:
      return sends(res, 404, 'text/plain', 'no such route');
  }
});

// A WebSocket echo (RFC 6455), by hand: each text or binary message back.
server.on('upgrade', (req, sock, head) => {
  seen.push({ path: req.url, cookie: req.headers.cookie ?? '', person: req.headers['x-svrngs-person'] ?? '', key: req.headers['x-svrngs-key'] ?? '', secret: Boolean(req.headers.authorization) });
  if (req.url !== '/ws' || !req.headers['sec-websocket-key']) return sock.destroy();
  const accept = createHash('sha1').update(`${req.headers['sec-websocket-key']}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`).digest('base64');
  sock.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);
  let buf = head;
  const frame = (op, data) => {
    const n = data.length;
    const len = n < 126 ? Buffer.from([0x80 | op, n]) : Buffer.from([0x80 | op, 126, n >> 8, n & 255]);
    return Buffer.concat([len, data]);
  };
  sock.on('data', (d) => {
    buf = Buffer.concat([buf, d]);
    while (buf.length >= 2) {
      let n = buf[1] & 127;
      let at = 2;
      if (n === 126) [n, at] = [buf.readUInt16BE(2), 4];
      const masked = buf[1] & 128;
      if (buf.length < at + (masked ? 4 : 0) + n) return;
      const mask = masked ? buf.subarray(at, at + 4) : null;
      at += masked ? 4 : 0;
      const data = Buffer.from(buf.subarray(at, at + n));
      if (mask) for (let i = 0; i < n; i++) data[i] ^= mask[i & 3];
      const op = buf[0] & 15;
      buf = buf.subarray(at + n);
      if (op === 1 || op === 2) sock.write(frame(op, data));
      if (op === 9) sock.write(frame(10, data));
      if (op === 8) return sock.end(frame(8, data));
    }
  });
  sock.on('error', () => {});
});

server.listen(0, '127.0.0.1', () => console.log(JSON.stringify({ port: server.address().port })));
