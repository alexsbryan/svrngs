// su: own serve — Ana's taste (demo/runs/su/2026-10-09/README.md, su-0;
// cut to what svrngs carries, sup-4, demo/runs/sup/2026-10-10/): a small
// program on Ana's computer, on loopback alone, offering `rankCandidates`
// (POST /rank) beside her watch history and keeping `viewingDiary` (GET
// /diary) back. Lent by Ana `svrngs share <port> --name taste --kind taste
// --refuse /diary --ask "<what an app receives>"`, so the door refuses
// /diary to everyone who reaches the lend, and carries an ask only from an
// app its person said yes to and has not stopped (DESIGN §3, "What an app
// gets"). Who may rank, beyond that, is this program's: its owner, until a
// lend its lender's person alone reaches (DESIGN §9). Every line is marked
// by su-0's rule (§4).
//
//   node taste.mjs <port> <dir>
//
// <dir> holds `diary.json` (the planted rows) and `owners` (Ana's keys,
// actor hex, a line each: her computer's and each of her phones').
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';

const [port, dir] = process.argv.slice(2);
const at = (name) => path.join(dir, name);
const log = (...words) => console.log(new Date().toISOString(), ...words);
// su: plant switch — which of su-0's plants this program runs (§3): none
// unless the check names one.
const PLANT = process.env.TASTE_PLANT || null;

// su: own serve
function answer(res, status, body) {
  // A page at another origin reads the answer, as Jellyfin's lets it: on a
  // computer Tonight's Picks is at its own (DESIGN §3, "Lent apps", call 6).
  res.writeHead(status, { 'content-type': 'application/json', 'access-control-allow-origin': '*' });
  res.end(JSON.stringify(body));
}

async function read(req) {
  const parts = [];
  for await (const part of req) parts.push(part);
  const text = Buffer.concat(parts).toString();
  return text ? JSON.parse(text) : {};
}

// su: own diary — the rows, read where they are stored, on each ask.
const diary = () => JSON.parse(fs.readFileSync(at('diary.json'), 'utf8')).rows;

// su: own rank — a film the history names: by a provider id both name,
// else by title and year.
const plain = (s) => String(s ?? '').toLowerCase().replace(/[^a-z0-9]/g, '');
function same(film, row) {
  const ids = Object.keys(film.ids ?? {}).filter((k) => k in (row.ids ?? {}));
  if (ids.length) return ids.some((k) => String(film.ids[k]) === String(row.ids[k]));
  return plain(film.title) === plain(row.title) && film.year === row.year;
}

// The films given, in the order Ana's history gives them: the mean of her
// ratings of each, a film she never rated after every film she did, ties
// by title. Nothing but the films goes back: no score, no row.
function rank(films) {
  const rows = diary();
  const score = (f) => {
    const mine = rows.filter((r) => same(f, r));
    return mine.length ? mine.reduce((s, r) => s + r.rating, 0) / mine.length : -1;
  };
  return films
    .map((f) => ({ f, s: score(f) }))
    .sort((a, b) => b.s - a.s || plain(a.f.title).localeCompare(plain(b.f.title)))
    .map(({ f }) => f);
}

// su: enforce refuse — a refusal as one: 403, its kind, and words a person
// reads; logged with who asked and why.
function refuse(res, kind, words, who) {
  log(`refused ${kind}:`, JSON.stringify(who));
  answer(res, 403, { refused: kind, error: words });
}

// su: enforce owner — the asker is Ana: the key the door verified is one of
// hers. svrngs names no person across devices (DESIGN §9), so her keys
// are listed here, read on each ask.
const owners = () => fs.readFileSync(at('owners'), 'utf8').split(/\s+/).filter(Boolean);
const owned = (key) => owners().includes(key);
// su: plant bo — bar 2's plant: the owner check taken out
const isOwner = PLANT === 'bo' ? () => true : owned;

// su: own serve
const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, 'http://taste');
  // su: enforce refuse — who asks, as the door says: an absent header is
  // never defaulted.
  const who = {
    person: req.headers['x-svrngs-person'] ?? null,
    key: req.headers['x-svrngs-key'] ?? null,
  };
  // su: own serve — the app the door carried this for, logged: whether it
  // may ask is the door's (sup-2).
  const app = req.headers['x-svrngs-app'] ?? null;
  log(req.method, url.pathname, JSON.stringify({ ...who, app }));
  try {
    // su: own diary — on this port, loopback on Ana's computer: the door
    // refuses /diary to everyone who reaches the lend.
    if (req.method === 'GET' && url.pathname === '/diary') return answer(res, 200, { rows: diary() });
    // su: own serve
    if (req.method !== 'POST' || url.pathname !== '/rank') return answer(res, 404, { error: 'nothing here by that name' });
    // su: enforce owner
    if (!isOwner(who.key)) return refuse(res, 'not-yours', 'This taste is someone else’s.', who);
    // su: own rank
    const { films } = await read(req);
    if (!Array.isArray(films)) return answer(res, 400, { error: 'films: a list of films to rank' });
    const ranked = rank(films);
    log('ranked', ranked.length, 'films for', app);
    return answer(res, 200, { films: ranked });
    // su: own serve
  } catch (e) {
    log('failed:', e);
    return answer(res, 500, { error: 'Ana’s taste could not answer.' });
  }
});

server.listen(Number(port), '127.0.0.1', () => log(`taste: listening on 127.0.0.1:${port}, ${dir}`));
