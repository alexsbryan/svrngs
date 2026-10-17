// The film Ana lends in relay-n0-off's run (run.py): a page that plays it
// and the film itself, with byte ranges, as a lent player answers.
//
//   node film.mjs <film.mp4>     prints `port <n>`, then a line per answer
import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';

const [film] = process.argv.slice(2);
const size = statSync(film).size;
const PAGE = '<!doctype html><title>film</title><video src="film.mp4" playsinline muted autoplay controls></video>';

const server = createServer((req, res) => {
  const path = new URL(req.url, 'http://x').pathname;
  if (path === '/' || path === '/index.html') {
    res.writeHead(200, { 'content-type': 'text/html' });
    res.end(PAGE);
    console.log(`200 ${path} - ${PAGE.length}`);
    return;
  }
  if (path !== '/film.mp4') {
    res.writeHead(404);
    res.end();
    console.log(`404 ${path} - 0`);
    return;
  }
  const range = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range ?? '');
  let [start, end] = [0, size - 1];
  if (range) {
    if (range[1] === '') start = Math.max(0, size - Number(range[2]));
    else {
      start = Number(range[1]);
      if (range[2] !== '') end = Math.min(Number(range[2]), size - 1);
    }
    if (start > end || start >= size) {
      res.writeHead(416, { 'content-range': `bytes */${size}` });
      res.end();
      console.log(`416 ${path} ${req.headers.range} 0`);
      return;
    }
  }
  const headers = { 'content-type': 'video/mp4', 'accept-ranges': 'bytes', 'content-length': end - start + 1 };
  if (range) headers['content-range'] = `bytes ${start}-${end}/${size}`;
  res.writeHead(range ? 206 : 200, headers);
  createReadStream(film, { start, end }).pipe(res);
  console.log(`${range ? 206 : 200} ${path} ${req.headers.range ?? '-'} ${end - start + 1}`);
});
server.listen(0, '127.0.0.1', () => console.log(`port ${server.address().port}`));
