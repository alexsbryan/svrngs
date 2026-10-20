// A stand-in for Jellyfin, for run.py here (PREDICTIONS.md §1): a page
// with one <video> and the film at `title.mp4`, served in ranges as a
// <video> asks for it. Every request's svrngs headers are kept, a JSON
// line each in `heads`, so the run reads who the door said asked.
//
//   node jellyfin.mjs <film> <heads>      prints `port <n>`
import http from 'node:http';
import { appendFileSync, createReadStream, statSync } from 'node:fs';

const [film, heads] = process.argv.slice(2);
const size = statSync(film).size;
const PAGE = '<!doctype html><title>Jellyfin</title><video src="title.mp4" playsinline preload="auto"></video>';

const server = http.createServer((req, res) => {
  const path = new URL(req.url, 'http://lend').pathname;
  const kept = { t: Date.now() / 1000, path, range: req.headers.range ?? null };
  for (const h of ['x-svrngs-person', 'x-svrngs-key', 'x-svrngs-user']) kept[h] = req.headers[h] ?? null;
  appendFileSync(heads, `${JSON.stringify(kept)}\n`);
  if (path !== '/title.mp4') return res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' }).end(PAGE);
  const asked = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range ?? '');
  if (!asked) {
    res.writeHead(200, { 'content-type': 'video/mp4', 'content-length': size, 'accept-ranges': 'bytes' });
    return createReadStream(film).pipe(res);
  }
  const start = asked[1] ? Number(asked[1]) : size - Number(asked[2]);
  const end = asked[1] && asked[2] ? Math.min(Number(asked[2]), size - 1) : size - 1;
  res.writeHead(206, {
    'content-type': 'video/mp4',
    'content-length': end - start + 1,
    'content-range': `bytes ${start}-${end}/${size}`,
    'accept-ranges': 'bytes',
  });
  createReadStream(film, { start, end }).pipe(res);
});
server.listen(0, '127.0.0.1', () => console.log(`port ${server.address().port}`));
