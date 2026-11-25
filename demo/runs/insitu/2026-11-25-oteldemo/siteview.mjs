// The site's view of the services it runs, for the vendor's agent: it
// follows the collector's file, projects each record once as it arrives
// (project.mjs), and keeps the last WINDOW minutes. Every route summarizes
// only what was already projected, so no aggregate can carry what the
// projection dropped. GET only.
//
//   node siteview.mjs <raw.jsonl> --port P [--raw] [--fresh]
//
// --raw serves the same routes with nothing projected: the benchmark's
// control arm, which measures what the projection costs. Never a site's.
import { createHmac, randomBytes } from 'node:crypto';
import { closeSync, existsSync, openSync, readSync, statSync } from 'node:fs';
import http from 'node:http';
import { RAW, contract, project } from './project.mjs';

const [file] = process.argv.slice(2);
const port = Number(process.argv[process.argv.indexOf('--port') + 1]);
const raw = process.argv.includes('--raw');
const key = raw ? RAW : randomBytes(32).toString('hex');
const C = contract();
const WINDOW = 30 * 60_000;
const events = [];
// --fresh starts at the file's end, so a window holds only what came after.
let at = process.argv.includes('--fresh') && existsSync(file) ? statSync(file).size : 0, partial = '', unreadable = 0, inode = existsSync(file) ? statSync(file).ino : null;

function follow() {
  if (!existsSync(file)) return;
  const st = statSync(file);
  if (st.ino !== inode || st.size < at) { inode = st.ino; at = 0; partial = ''; }
  if (st.size === at) return;
  const fd = openSync(file, 'r');
  try {
    while (at < st.size) {
      const buf = Buffer.alloc(Math.min(8 << 20, st.size - at));
      const n = readSync(fd, buf, 0, buf.length, at);
      at += n;
      const lines = (partial + buf.subarray(0, n).toString('utf8')).split('\n');
      partial = lines.pop();
      for (const l of lines) if (l) { try { for (const e of project(l, C, key)) { if (e.kind === 'metric') e.t = Math.max(0, ...e.points.map((p) => p.t ?? 0)) || null; events.push(e); } } catch { unreadable++; } }
    }
  } finally { closeSync(fd); }
  const cut = Date.now() - WINDOW;
  let i = 0;
  while (i < events.length && (events[i].t ?? Date.now()) < cut) i++;
  if (i) events.splice(0, i);
}
setInterval(follow, 1000);
follow();

const q = (u, k) => u.searchParams.get(k);
const since = (u) => Date.now() - Math.min(Number(q(u, 'since') ?? 600), 1800) * 1000;
const limit = (u, d) => Math.min(Number(q(u, 'limit') ?? d) || d, 500);
const pct = (xs, p) => { if (!xs.length) return null; const s = [...xs].sort((a, b) => a - b); return s[Math.min(s.length - 1, Math.floor((p / 100) * s.length))]; };
const label = (v) => (v && typeof v === 'object' && v.shape ? `<shape len ${v.len} ${v.classes.join('+')} #${v.h}>` : String(v));
const matches = (e, u) => (!q(u, 'service') || e.service === q(u, 'service')) && (!q(u, 'name') || label(e.name) === q(u, 'name')) && (e.t ?? 0) >= since(u);

const routes = {
  '/': () => ({ mode: raw ? 'raw (the control arm: nothing projected)' : 'projected', window_minutes: WINDOW / 60_000, routes: Object.keys(routes), params: 'service, name, status=error, severity, since (seconds, default 600), limit', events: events.length, unreadable }),
  '/services': (u) => {
    const by = {};
    for (const e of events) {
      if ((e.t ?? 0) < since(u)) continue;
      const s = (by[e.service ?? '?'] ??= { spans: 0, errors: 0, logs: {}, metrics: new Set(), names: {} });
      if (e.kind === 'span') {
        s.spans++; if (e.status === 'error') s.errors++;
        const n = (s.names[label(e.name)] ??= { kind: e.span_kind, count: 0, errors: 0, ms: [] });
        n.count++; if (e.status === 'error') n.errors++; if (e.ms != null) n.ms.push(e.ms);
      } else if (e.kind === 'log') s.logs[e.severity ?? '?'] = (s.logs[e.severity ?? '?'] ?? 0) + 1;
      else if (e.kind === 'metric') s.metrics.add(label(e.name));
    }
    return Object.fromEntries(Object.entries(by).map(([k, s]) => [k, { spans: s.spans, errors: s.errors, logs: s.logs, metrics: s.metrics.size, operations: Object.fromEntries(Object.entries(s.names).sort((a, b) => b[1].count - a[1].count).slice(0, 25).map(([n, x]) => [n, { kind: x.kind, count: x.count, errors: x.errors, p50_ms: pct(x.ms, 50), p95_ms: pct(x.ms, 95), max_ms: pct(x.ms, 100) }])) }]));
  },
  '/errors': (u) => {
    const g = {};
    for (const e of events) if (e.kind === 'span' && e.status === 'error' && matches(e, u)) {
      const ex = e.events.find((x) => x.name === 'exception');
      const k = [e.service, label(e.name), ex ? label(ex.type) : '-', ex?.at?.[0] ?? '-'].join(' | ');
      (g[k] ??= { count: 0, example_trace: e.trace, ...(raw && ex ? { message: ex.message } : {}) }).count++;
    }
    return Object.entries(g).sort((a, b) => b[1].count - a[1].count).slice(0, limit(u, 50)).map(([k, v]) => ({ 'service | operation | exception | at': k, ...v }));
  },
  '/spans': (u) => events.filter((e) => e.kind === 'span' && matches(e, u) && (!q(u, 'status') || e.status === q(u, 'status'))).slice(-limit(u, 50)),
  '/trace': (u) => events.filter((e) => (e.kind === 'span' || e.kind === 'log') && e.trace === q(u, 'id')),
  '/logs': (u) => events.filter((e) => e.kind === 'log' && matches(e, u) && (!q(u, 'severity') || e.severity === q(u, 'severity'))).slice(-limit(u, 50)),
  '/metrics': (u) => {
    const series = {};
    for (const e of events) if (e.kind === 'metric' && matches(e, u)) for (const p of e.points) {
      const k = `${e.service} ${label(e.name)} ${JSON.stringify(Object.fromEntries(Object.entries(p.attrs).map(([a, v]) => [a, label(v)])))}`;
      (series[k] ??= { service: e.service, name: e.name, unit: e.unit, type: e.type, attrs: p.attrs, points: [] }).points.push({ t: p.t, ...(p.value != null ? { value: p.value } : { count: p.count, sum: p.sum }) });
    }
    return Object.values(series).slice(0, limit(u, 100)).map((s) => ({ ...s, points: s.points.slice(-Number(q(u, 'points') ?? 12)) }));
  },
};

http.createServer((req, res) => {
  const say = (code, v) => res.writeHead(code, { 'content-type': 'application/json' }).end(JSON.stringify(v));
  if (req.method !== 'GET') return say(405, { error: 'this view only reads' });
  const u = new URL(req.url, 'http://x');
  const route = routes[u.pathname];
  if (!route) return say(404, { error: 'no such view', routes: Object.keys(routes) });
  try { say(200, route(u)); } catch (e) { say(500, { error: 'the view could not read this' }); }
}).listen(port, '127.0.0.1', () => console.log(`${raw ? 'raw' : 'projected'} view of ${file} on 127.0.0.1:${port}`));
