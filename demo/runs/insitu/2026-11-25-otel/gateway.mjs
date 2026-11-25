// The gateway for a service's declared effects: the calls it makes that
// change something outside it (here, a claim sent to a clearinghouse). The
// service is pointed at the gateway, never at the outside, by the variable
// its recipe names.
//
//   node gateway.mjs --port P --route <name>=<upstream> --record <file>
//   node gateway.mjs --port P --route <name>= --replay <file> --unrecorded <file>
//
// Recording, it forwards each call and keeps the exchange on this machine,
// owner-only: request and answer whole, since a trial must answer as the
// outside did. Replaying, in a trial with no network, it answers a call with
// the answer recorded for that exact request (method, path, body). A request
// it has no answer for gets 599 and a line in the unrecorded file, so the
// trial says it can't judge that input: no claim is ever sent from a trial,
// and no answer is invented.
import { createHash } from 'node:crypto';
import { appendFileSync, existsSync, readFileSync } from 'node:fs';
import http from 'node:http';

const arg = (k) => { const i = process.argv.indexOf(k); return i < 0 ? null : process.argv[i + 1]; };
const routes = Object.fromEntries(process.argv.flatMap((a, i, all) => (all[i - 1] === '--route' ? [a.split(/=(.*)/s).slice(0, 2)] : [])));
const port = Number(arg('--port')), record = arg('--record'), replay = arg('--replay'), unrecorded = arg('--unrecorded');
export const requestHash = (method, path, body) => createHash('sha256').update(`${method} ${path}\n`).update(body).digest('hex');

const answers = new Map();
if (replay && existsSync(replay)) for (const l of readFileSync(replay, 'utf8').split('\n').filter(Boolean)) { const r = JSON.parse(l); answers.set(r.req_hash, r); }

http.createServer((req, res) => {
  const chunks = [];
  req.on('data', (c) => chunks.push(c)).on('end', async () => {
    const body = Buffer.concat(chunks);
    const [, name, ...rest] = req.url.split('/');
    const path = '/' + rest.join('/');
    if (!(name in routes)) return res.writeHead(404).end(`no declared effect called ${name}`);
    const h = requestHash(req.method, `/${name}${path}`, body);
    if (replay) {
      const a = answers.get(h);
      if (!a) { appendFileSync(unrecorded, JSON.stringify({ effect: name, path, req_hash: h }) + '\n'); return res.writeHead(599, { 'content-type': 'application/json' }).end('{"unrecorded":true}'); }
      return res.writeHead(a.status, { 'content-type': a.type ?? 'application/json' }).end(a.resp);
    }
    try {
      const r = await fetch(routes[name] + path, { method: req.method, headers: { 'content-type': req.headers['content-type'] ?? 'application/octet-stream' }, body: ['GET', 'HEAD'].includes(req.method) ? undefined : body, signal: AbortSignal.timeout(10_000) });
      const resp = await r.text();
      appendFileSync(record, JSON.stringify({ at: new Date().toISOString(), effect: name, method: req.method, path, req_hash: h, status: r.status, type: r.headers.get('content-type'), resp }) + '\n', { mode: 0o600 });
      res.writeHead(r.status, { 'content-type': r.headers.get('content-type') ?? 'application/json' }).end(resp);
    } catch (e) {
      appendFileSync(record, JSON.stringify({ at: new Date().toISOString(), effect: name, method: req.method, path, req_hash: h, status: 502, failed: e.cause?.code ?? e.name }) + '\n', { mode: 0o600 });
      res.writeHead(502).end(`the gateway couldn't reach ${name}`);
    }
  });
}).listen(port, '127.0.0.1');
