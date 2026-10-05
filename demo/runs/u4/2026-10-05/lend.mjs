// A service to lend for U4's bar: a directory's files on 127.0.0.1, with
// byte ranges, so a browser's <video> plays and seeks a file in it. `/` is
// the directory's index.html, else a page holding a <video> for each video
// file there. The dry run lends it (dry.sh); so does the operator, on the
// computer the phone reaches (NEEDS_HUMAN.md). Node alone, nothing to
// install: Python's http.server answers no ranges, which a <video> needs
// to seek.
//
//   node lend.mjs <dir> [port]     prints `port <n>` once it listens
import http from 'node:http';
import { createReadStream, readdirSync, statSync } from 'node:fs';
import { extname, join, normalize, resolve } from 'node:path';

const [dir, port = '0'] = process.argv.slice(2);
if (!dir) {
  console.error('usage: node lend.mjs <dir> [port]');
  process.exit(2);
}
const root = resolve(dir);
const TYPES = {
  '.html': 'text/html; charset=utf-8', '.mp4': 'video/mp4', '.m4v': 'video/mp4',
  '.mov': 'video/quicktime', '.webm': 'video/webm', '.vtt': 'text/vtt', '.jpg': 'image/jpeg',
};
const VIDEO = new Set(['.mp4', '.m4v', '.mov', '.webm']);
const esc = (s) => s.replace(/[&<>"]/g, (c) => `&#${c.charCodeAt(0)};`);

function index() {
  const videos = readdirSync(root).filter((f) => VIDEO.has(extname(f).toLowerCase())).sort();
  return '<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">' +
    '<style>body{font:16px system-ui;margin:1rem}video{width:100%}</style>' +
    videos.map((f) => `<h2>${esc(f)}</h2><video src="${encodeURIComponent(f)}" controls playsinline preload="metadata"></video>`).join('');
}

http.createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://lend').pathname);
  if (path === '/') {
    let page;
    try { statSync(join(root, 'index.html')); } catch { page = index(); }
    if (page) return res.writeHead(200, { 'content-type': TYPES['.html'] }).end(page);
  }
  const file = join(root, normalize(path === '/' ? '/index.html' : path));
  let size;
  try {
    if (!file.startsWith(root)) throw new Error('outside');
    size = statSync(file).size;
  } catch {
    return res.writeHead(404).end();
  }
  const head = { 'content-type': TYPES[extname(file).toLowerCase()] ?? 'application/octet-stream', 'accept-ranges': 'bytes' };
  // One range, as a <video> asks: `bytes=a-b`, `bytes=a-` or `bytes=-n`.
  const m = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range ?? '');
  let [from, to, status] = [0, size - 1, 200];
  if (m && (m[1] || m[2])) {
    [from, to] = m[1] ? [Number(m[1]), m[2] ? Math.min(Number(m[2]), size - 1) : size - 1] : [Math.max(size - Number(m[2]), 0), size - 1];
    if (from > to || from >= size) {
      return res.writeHead(416, { 'content-range': `bytes */${size}` }).end();
    }
    status = 206;
    head['content-range'] = `bytes ${from}-${to}/${size}`;
  }
  head['content-length'] = to - from + 1;
  res.writeHead(status, head);
  // Each answer as it ends, a line: when it was asked (ms since 1970),
  // what, its status, how many bytes went and how long it took.
  const asked = Date.now();
  let sent = 0;
  res.on('close', () =>
    console.log(`${asked} ${req.method} ${path} ${req.headers.range ?? '-'} ${status} ${sent}/${to - from + 1} ${Date.now() - asked}ms`));
  if (req.method === 'HEAD' || size === 0) return res.end();
  createReadStream(file, { start: from, end: to }).on('data', (c) => (sent += c.length)).pipe(res);
}).listen(Number(port), '127.0.0.1', function () {
  console.log(`port ${this.address().port}`);
});
