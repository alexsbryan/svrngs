// The conformance canary (DESIGN §5, "The canaries"; u5-2): a localhost
// service with one route per thing a self-hosted app leans on, which the
// infra must carry whole to whoever opens it — on a computer, and on a
// phone at an address of its own or under one for every app (DESIGN §7).
// Its page (`/`) loads `checks.js`, which runs each route's check in the
// browser; crates/svrngs-node/tests/conformance.rs lends two of it, Sam's
// and Bo's, and judges each route in each way.
//
//   node apps/web-conformance/serve.mjs <who> [<lender's key>]
//
// Listens on 127.0.0.1, any port, and prints `{"port": N}`. No dependency.
// Given the lender's key, it keeps accounts of its own, a token each, as an
// app that signs its people in does (mwp-4; `answer.mjs` signs each person
// in to them through the door).
//
// Routes:
//   GET  /            its page, under COOP and COEP and a script policy by
//                     nonce, asking `/abs.js` by a path from the root and
//                     `checks.js` by one from itself, each with the nonce
//   GET  /abs.js      a script found only at the root of its own address
//   GET  /checks.js   the checks
//   GET  /ws          a WebSocket echo
//   GET  /big.bin     64 MiB, by Range; byte i is (i * 7 + (i >> 10)) & 255
//   GET  /big.wav     the same 64 MiB under a WAV head, its first 44 bytes:
//                     sound a media element plays and seeks by Range, sent
//                     as it is read
//   POST /upload      read as it streams; answers `<bytes> <sha-256> <its Content-Length>`
//   GET  /app-sw.js   its own service worker, answering `sw-probe`
//   GET  /sw-probe    `from the server`, when no worker of its own answers
//   GET  /redirect    refused unless asked on its own address,
//                     `localhost:<port>`; else an absolute redirect there
//   GET  /landed      where that lands
//   POST /signin      sets `canary-<who>`, and an HttpOnly `canary-<who>-hidden`
//   POST /guarded     refused unless its `Origin` is its own address, as
//                     an app checking a post's origin against its Host does
//   GET  /me          the cookies it was sent
//   GET  /seen        every request it took: path, cookies, person, user, key and
//                     the account it was signed in as
//   POST /accounts    `{"name"}`, by the lender's key as a bearer: that
//                     account's token, the account made if it is new
//   GET  /whoami      the account a request's `?token=`, else its bearer
//                     `authorization`, names, and that token, in its body
//   GET  /account     a redirect to `/whoami?token=<the bearer's token>`
import { createHash, randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import http from 'node:http';
import { Readable } from 'node:stream';

const who = process.argv[2] ?? 'canary';
const lender = process.argv[3];
const accounts = new Map();
// The token a request signs in with, by its query or its bearer, and the
// account it names, or ''.
const tokenOf = (req) => new URL(req.url, 'http://x').searchParams.get('token') ?? req.headers.authorization?.replace(/^Bearer /, '') ?? '';
const nameOf = (token) => [...accounts].find(([, t]) => t === token)?.[0] ?? '';
const SIZE = 64 * 1024 * 1024;
const byte = (i) => (i * 7 + (i >> 10)) & 255;
const seen = [];
// What `/seen` gives of a request, a page's or a WebSocket's, read one way.
const saw = (req, path) => seen.push({ path, cookie: req.headers.cookie ?? '', person: req.headers['x-svrngs-person'] ?? '', user: req.headers['x-svrngs-user'] ?? '', key: req.headers['x-svrngs-key'] ?? '', secret: Boolean(req.headers.authorization), signed: nameOf(tokenOf(req)) });
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

// A WAV head for the 64 MiB: 16-bit stereo at 96 kHz, 384,000 bytes a
// second, the order of a film's (Demo A's converted one, 466,319), so
// the file plays for 174 s.
const RATE = 96_000;
const head = Buffer.alloc(44);
head.write('RIFF', 0);
head.writeUInt32LE(SIZE - 8, 4);
head.write('WAVEfmt ', 8);
head.writeUInt32LE(16, 16);
head.writeUInt16LE(1, 20);
head.writeUInt16LE(2, 22);
head.writeUInt32LE(RATE, 24);
head.writeUInt32LE(RATE * 4, 28);
head.writeUInt16LE(4, 32);
head.writeUInt16LE(16, 34);
head.write('data', 36);
head.writeUInt32LE(SIZE - 44, 40);

// The bytes a request's Range asks of the 64 MiB, `[from, to]`, its end
// held to the file's; null when it asks no range.
function span(req) {
  const m = /^bytes=(\d+)-(\d*)$/.exec(req.headers.range ?? '');
  return m && [Number(m[1]), Math.min(m[2] === '' ? SIZE - 1 : Number(m[2]), SIZE - 1)];
}

// The WAV, whole or by Range, a MiB at a time as the asker reads it, so a
// seek's abandoned answer costs the canary nothing it has not sent.
function played(req, res) {
  const asked = span(req);
  const [from, to] = asked ?? [0, SIZE - 1];
  if (from >= SIZE || to < from) return sends(res, 416, 'text/plain', '', { 'content-range': `bytes */${SIZE}` });
  const ranged = asked ? { 'content-range': `bytes ${from}-${to}/${SIZE}` } : {};
  res.writeHead(asked ? 206 : 200, { ...SAME, 'content-type': 'audio/wav', 'accept-ranges': 'bytes', 'content-length': to - from + 1, ...ranged });
  Readable.from(
    (function* () {
      for (let at = from; at <= to; at += 1 << 20) {
        const part = bytes(at, Math.min(at + (1 << 20) - 1, to));
        if (at < head.length) head.copy(part, 0, at);
        yield part;
      }
    })(),
  ).pipe(res);
}

function big(req, res) {
  const asked = span(req);
  if (!asked) return sends(res, 200, 'application/octet-stream', bytes(0, 1023), { 'content-length': 1024 });
  const [from, to] = asked;
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
  saw(req, path);
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
    // Refused with no Origin too, as Nextcloud's sign-in is (u5-catalogue-dry).
    if (!origin || !own.some((h) => origin === `http://${h}`)) return sends(res, 403, 'text/plain', `origin ${origin} is not ${own[0]}`, { 'cross-origin-embedder-policy': 'require-corp' });
    // Its answer may be a frame's page in the canary's, which COEP holds to its own.
    return sends(res, 200, 'text/plain', `taken from ${origin}`, { 'cross-origin-embedder-policy': 'require-corp' });
  }
  if (req.method === 'POST' && path === '/signin') {
    const token = randomBytes(6).toString('hex');
    return sends(res, 200, 'application/json', JSON.stringify({ who }), {
      'set-cookie': [`canary-${who}=${token}; Path=/`, `canary-${who}-hidden=1; Path=/; HttpOnly`],
    });
  }
  if (req.method === 'POST' && path === '/accounts') {
    if (!lender || req.headers.authorization !== `Bearer ${lender}`) return sends(res, 403, 'text/plain', 'not the lender');
    let body = '';
    req.on('data', (d) => (body += d));
    return req.on('end', () => {
      const { name } = JSON.parse(body);
      if (!accounts.has(name)) accounts.set(name, randomBytes(16).toString('hex'));
      sends(res, 200, 'application/json', JSON.stringify({ token: accounts.get(name) }));
    });
  }
  switch (path) {
    case '/whoami': {
      const token = tokenOf(req);
      return sends(res, 200, 'application/json', JSON.stringify({ who: nameOf(token), token }));
    }
    case '/account':
      return sends(res, 302, 'text/plain', '', { location: `/whoami?token=${tokenOf(req)}` });
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
    case '/big.wav':
      return played(req, res);
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
  saw(req, req.url);
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
