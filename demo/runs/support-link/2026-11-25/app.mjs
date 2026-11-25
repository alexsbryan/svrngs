// The player app: the customer's page on one port, and on another the
// diagnostic listener that is lent to support. The listener only reads, and
// only the incident the customer captured and the replays they ran; nothing
// on it starts a replay or changes the player (SUPPORT_LINK, "The app owns
// the data boundary").
//
//   node app.mjs <out dir> [player port] [diagnostic port]
import { appendFileSync, readFileSync } from 'node:fs';
import http from 'node:http';
import { join } from 'node:path';

const [out, pport = '8402', dport = '8403'] = process.argv.slice(2);
if (!out) throw new Error('usage: node app.mjs <out dir> [player port] [diagnostic port]');
const here = import.meta.dirname;
const attempts = []; // every attempt this app ran, the operator's
let incident = null; // the attempt the customer chose to share
let ended = null;

const body = (req) => new Promise((ok) => { let b = ''; req.on('data', (c) => (b += c)).on('end', () => ok(b)); });
const json = (res, code, v) => res.writeHead(code, { 'content-type': 'application/json' }).end(JSON.stringify(v, null, 1));
const clock = (s) => (s == null ? null : `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`);
const log = (row) => appendFileSync(join(out, 'diagnostic-calls.jsonl'), JSON.stringify({ at: Date.now(), ...row }) + '\n');

http
  .createServer(async (req, res) => {
    const { pathname } = new URL(req.url, 'http://x');
    if (req.method === 'GET' && pathname === '/') return res.writeHead(200, { 'content-type': 'text/html' }).end(readFileSync(join(here, 'player.html')));
    if (req.method === 'GET' && pathname === '/vendor/hls-1.7.4.min.js') return res.writeHead(200, { 'content-type': 'text/javascript' }).end(readFileSync(join(here, pathname)));
    if (req.method === 'POST' && pathname === '/report') {
      const a = { id: attempts.length + 1, ...JSON.parse(await body(req)) };
      attempts.push(a);
      appendFileSync(join(out, 'attempts.jsonl'), JSON.stringify(a) + '\n');
      return json(res, 200, { id: a.id });
    }
    if (req.method === 'POST' && pathname === '/capture') { incident = attempts.at(-1); ended = null; return json(res, 200, { id: incident.id }); }
    if (req.method === 'GET' && pathname === '/shared') return json(res, 200, { shared: !!incident && !ended });
    if (req.method === 'POST' && pathname === '/end') { ended = Date.now(); return json(res, 200, { ended }); }
    json(res, 404, { error: 'no such route' });
  })
  .listen(Number(pport), '127.0.0.1', () => console.log(`player on http://127.0.0.1:${pport}/`));

// --- What support can read -------------------------------------------------

const CHECKS = {
  incident: { says: 'The title, position, player build and rendition when it froze', params: [] },
  timeline: { says: "The player's events between two media times, in seconds (default 12:30 to 12:55)", params: ['from', 'to'] },
  replay: { says: "Whether the customer's latest test replay played past the failure, and with what change", params: [] },
};

function incidentView() {
  const ev = incident.events;
  const stall = ev.find((e) => e.e === 'verdict');
  const before = ev.filter((e) => e.e === 'rendition' && e.media <= (stall.evidence?.worst_gap_at_media_s ?? Infinity)).at(-1);
  return {
    title: 'Night Train',
    started_at: clock(750),
    froze_at: clock(stall.evidence?.worst_gap_at_media_s),
    froze_at_media_s: stall.evidence?.worst_gap_at_media_s ?? null,
    player_build: incident.build,
    browser: incident.browser,
    rendition_when_it_froze: before?.now ?? null,
    renditions: ev.find((e) => e.e === 'renditions')?.all ?? null,
    what_the_viewer_saw: stall,
  };
}

function read(name, q) {
  if (name === 'incident') return incidentView();
  if (name === 'timeline') {
    const from = Number(q.get('from') ?? 750), to = Number(q.get('to') ?? 775);
    if (!Number.isFinite(from) || !Number.isFinite(to)) return { verdict: 'could-not-judge', why: 'from and to are media seconds' };
    return {
      note: 'wall is seconds since this playback began; media is the playback position in seconds; segment media times are from/to',
      events: incident.events.filter((e) => e.e === 'verdict' || (e.media >= from - 2 && e.media <= to) || (e.from != null && e.from < to && e.to > from)),
    };
  }
  const tests = attempts.filter((a) => a.id > incident.id && a.kind !== 'viewing');
  if (!tests.length) return { verdict: 'never-ran', why: 'the customer has not pressed Test fix' };
  return tests.map((a) => ({ kind: a.kind, change: a.cap ? `rendition capped at ${a.cap}, for this replay only` : 'none: normal settings', verdict: a.verdict, why: a.why ?? null, evidence: a.evidence, renditions_requested: [...new Set(a.events.filter((e) => e.e === 'request' && e.from >= 758 && e.from <= 766).map((e) => `${e.rendition} segment ${e.segment}`))], errors: a.events.filter((e) => e.e === 'error') }));
}

http
  .createServer((req, res) => {
    const u = new URL(req.url, 'http://x');
    const name = u.pathname.slice(1);
    const refuse = (code, error) => { log({ method: req.method, path: u.pathname, status: code, error }); json(res, code, { error }); };
    if (req.method !== 'GET' && req.method !== 'HEAD') return refuse(405, 'support can only read');
    if (name === '' || name === 'svrngs-support') return json(res, 200, { desk: 'Acme Player support', about: 'Playback froze around 12:43 in Night Train', checks: Object.entries(CHECKS).map(([n, c]) => ({ name: n, says: c.says, params: c.params })) });
    const check = CHECKS[name];
    if (!check) return refuse(404, `no check called ${name}; the checks are ${Object.keys(CHECKS).join(', ')}`);
    const extra = [...u.searchParams.keys()].filter((k) => !check.params.includes(k));
    if (extra.length) return refuse(400, `${name} takes no ${extra.join(', ')}`);
    if (ended) return refuse(410, 'the customer ended support access');
    if (!incident) return refuse(409, 'the customer has shared no incident');
    const v = read(name, u.searchParams);
    log({ method: req.method, path: u.pathname, query: u.search, status: 200 });
    json(res, 200, v);
  })
  .listen(Number(dport), '127.0.0.1', () => console.log(`diagnostics on 127.0.0.1:${dport}`));
