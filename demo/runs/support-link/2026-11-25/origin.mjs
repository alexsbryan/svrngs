// The test origin: serves media/ uncached, breaking one segment by mode.
// Its log is the operator's independent witness; the agent never sees it.
//
//   node origin.mjs <healthy|broken|lookalike> <log file> [port]
//
// broken:    only 1080p's segment 381 (12:42-12:44) answers 404, every time.
// lookalike: segment 381 answers 404 at every rendition.
// truncated: 1080p's segment 381 answers 200 with only its first half.
// hang:      1080p's segment 381 answers 200 headers, then never a byte.
import { createReadStream, appendFileSync, statSync, readFileSync } from 'node:fs';
import http from 'node:http';
import { join, normalize } from 'node:path';

const [mode, log, port = '8401'] = process.argv.slice(2);
const BAD = '/v1080p/seg381.m4s';
const MODES = { healthy: () => false, broken: (p) => p === BAD, lookalike: (p) => /^\/v\d+p\/seg381\.m4s$/.test(p), truncated: () => false, hang: () => false };
if (!MODES[mode] || !log) throw new Error(`usage: node origin.mjs <${Object.keys(MODES).join('|')}> <log> [port]`);
const root = join(import.meta.dirname, 'media');
const rev = readFileSync(join(root, 'REV'), 'utf8').trim();
const TYPES = { '.m3u8': 'application/vnd.apple.mpegurl', '.m4s': 'video/iso.segment', '.mp4': 'video/mp4' };

const note = (row) => appendFileSync(log, JSON.stringify({ at: Date.now(), mode, rev, ...row }) + '\n');
note({ started: true });

http
  .createServer((req, res) => {
    const path = normalize(decodeURIComponent(new URL(req.url, 'http://x').pathname));
    const head = { 'access-control-allow-origin': '*', 'cache-control': 'no-store' };
    if (req.method === 'OPTIONS') return res.writeHead(204, head).end();
    if (MODES[mode](path)) {
      note({ path, status: 404 });
      return res.writeHead(404, head).end('missing');
    }
    let size;
    try {
      size = statSync(join(root, path)).size;
    } catch {
      note({ path, status: 404, why: 'no such file' });
      return res.writeHead(404, head).end();
    }
    const type = TYPES[path.slice(path.lastIndexOf('.'))] ?? 'application/octet-stream';
    if (path === BAD && mode === 'hang') {
      note({ path, status: 200, hung: true });
      return res.writeHead(200, { ...head, 'content-type': type, 'content-length': size }).flushHeaders();
    }
    if (path === BAD && mode === 'truncated') {
      const half = readFileSync(join(root, path)).subarray(0, Math.floor(size / 2));
      note({ path, status: 200, bytes: half.length, truncated: true });
      return res.writeHead(200, { ...head, 'content-type': type, 'content-length': half.length }).end(half);
    }
    res.writeHead(200, { ...head, 'content-type': type, 'content-length': size });
    createReadStream(join(root, path)).pipe(res).on('finish', () => note({ path, status: 200, bytes: size }));
  })
  .listen(Number(port), '127.0.0.1', () => console.log(`origin ${mode} rev ${rev} on 127.0.0.1:${port}`));
