#!/usr/bin/env node
// Ana's taste: a small program on her computer that keeps the films she
// rated and guesses which films in her friends' libraries she'll like. Its
// page is its own, lent by her, so it runs at her own origin on her own
// computer: what she rated, and what taste guesses from it, reaches no page
// of anyone else's (apps/taste/README.md). Started by her, it lends itself,
// as Give does (`svrngs share`):
//
//   node taste.mjs
//
// so the door refuses /diary to everyone, and its page reads her groups'
// Jellyfins by the account each lender keeps for apps (`--reads jellyfin`).
// Only its own page is answered: a page another app serves runs at its
// lender's origin, and could send what taste guesses back to that lender
// (seen in 25f9f707's planted fetch). Its ratings are kept in
// `$SVRNGS_DIR/taste`, else `~/.svrngs/taste`.
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { SAID, diary, imported, letterboxd, rate, same } from './diary.mjs';
import { catalog, numbers } from './model.mjs';
import { scored } from './score.mjs';
import * as svrngs from './svrngs.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
// A plant the check names, else none: `bo`, the owner check off; `subset`,
// only her rated films sent to the model, as her diary names them;
// `rated-first`, her rated films first in the list; `constant`, a scorer
// that knows nothing.
const PLANT = process.env.TASTE_PLANT || null;

const dir = join(svrngs.dataDir(), 'taste');
mkdirSync(dir, { recursive: true, mode: 0o700 });
// Every decision taste makes, written to `taste.log` in its dir: the
// terminal Ana pasted the line into shows only what is said to her.
const log = (...words) =>
  appendFileSync(
    join(dir, 'taste.log'),
    `${[new Date().toISOString(), ...words.map((w) => (typeof w === 'string' ? w : String(w?.stack ?? w)))].join(' ')}\n`,
  );

// Read by taste's own page alone, at its own origin: no other origin's
// script may read an answer.
function answer(res, status, body) {
  res.writeHead(status, { 'content-type': 'application/json' });
  res.end(JSON.stringify(body));
}

async function read(req) {
  const parts = [];
  let n = 0;
  for await (const part of req) {
    n += part.length;
    if (n > 8 * 1024 * 1024) throw new Error('too large');
    parts.push(part);
  }
  const text = Buffer.concat(parts).toString();
  return text ? JSON.parse(text) : {};
}

// The person this computer is, asked of svrngs on each request (svrngs.mjs
// `ownerKeys`): only they rank, rate or import. A key svrngs cannot name is
// nobody's, never assumed hers.
async function owned(key) {
  if (PLANT === 'bo') return true;
  if (!key) return false;
  try {
    return (await svrngs.ownerKeys()).has(key);
  } catch (e) {
    log('owner: svrngs could not say who this computer is:', e.message);
    return false;
  }
}

// Ana's own name, for the words a refusal says.
async function whose() {
  try {
    const { groups } = await svrngs.groups();
    return groups.find((g) => g.person)?.person ?? null;
  } catch {
    return null;
  }
}

// The model taste asks: each friend's llama-server her groups lend, in the
// record's order, reached as her through svrngs with this computer's
// token, which svrngs takes off before the model is asked (pantry.mjs
// `probe`).
async function models() {
  const { token } = svrngs.record();
  const lent = await svrngs.lendsOf('llama-server');
  return lent.map((l) => ({
    at: l.origin.replace(/\/+$/, ''),
    token,
    who: `${l.person}’s ${l.offered?.name ?? l.name ?? 'llama-server'}`,
  }));
}

// Numbers for every film of `list` from the first model that answers, or
// why none did: never a guess made up in their place.
async function ask(list, rows) {
  let tried = [];
  try {
    tried = await models();
  } catch (e) {
    return { why: `svrngs couldn’t say who shares a model (${e.message}).` };
  }
  if (!tried.length)
    return { why: 'nobody in your groups shares a model taste can use. Your ratings are kept.' };
  for (const model of tried) {
    try {
      if (PLANT === 'subset') {
        // The plant: her rated films, as her diary names them, sent alone.
        const theirs = rows.map((r, i) => ({
          library: 'diary',
          id: String(i),
          title: r.title,
          year: r.year,
        }));
        await numbers(catalog(theirs).catalog, model, log);
        return { model, vectors: null };
      }
      return { model, vectors: await numbers(list, model, log) };
    } catch (e) {
      log('model:', model.who, 'did not answer:', e.message);
    }
  }
  return { why: `${tried.map((m) => m.who).join(', ')} didn’t answer. Try again later.` };
}

// The films handed, scored, with her ratings, the guesses and why, for
// taste's own page.
async function rank(body) {
  const made = catalog(body.films);
  if (!made.catalog) return { status: 400, body: { error: made.why } };
  const list = made.catalog;
  const rows = diary(dir);
  if (PLANT === 'rated-first') {
    // The plant: the films she rated first in what the model is sent.
    const mine = (f) => rows.some((r) => same(f, r));
    const films = [...list.films.filter(mine), ...list.films.filter((f) => !mine(f))];
    const texts = films.map((f) => list.texts[list.films.indexOf(f)]);
    Object.assign(list, { films, texts });
  }
  const { model, vectors, why } = await ask(list, rows);
  const ranked = scored(list.films, vectors, rows, PLANT);
  log('ranked', ranked.length, 'films');
  const elsewhere = rows.filter((r) => !list.films.some((f) => same(f, r))).length;
  return {
    status: 200,
    body: {
      films: ranked.map((r) => ({
        library: r.film.library,
        id: r.film.id,
        title: r.film.title,
        year: r.film.year,
        ids: r.film.ids,
        mine: r.mine,
        guess: r.mine === null ? r.score : null,
        because: r.because,
      })),
      model: model?.who ?? null,
      unguessed: why ?? null,
      elsewhere,
    },
  };
}

const PAGE = { '/': 'index.html', '/page.js': 'page.js', '/page.css': 'page.css' };
const TYPES = { html: 'text/html; charset=utf-8', js: 'text/javascript', css: 'text/css' };

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, 'http://taste');
  // Who asks and for which app, as the door says: an absent header is
  // nobody, never defaulted to her.
  const key = req.headers['x-svrngs-key'] ?? null;
  const app = req.headers['x-svrngs-app'] ?? null;
  // Its own page: the app the door names is this lend itself.
  const own = app !== null && app === (req.headers['x-svrngs-label'] ?? null);
  log(
    req.method,
    url.pathname,
    JSON.stringify({ person: req.headers['x-svrngs-person'] ?? null, key, app, own }),
  );
  try {
    if (req.method === 'GET' && PAGE[url.pathname]) {
      const file = PAGE[url.pathname];
      res.writeHead(200, {
        'content-type': TYPES[file.split('.').pop()],
        'cache-control': 'no-store',
      });
      return res.end(readFileSync(join(HERE, 'page', file)));
    }
    // On loopback alone: the door refuses /diary to everyone the lend reaches.
    if (req.method === 'GET' && url.pathname === '/diary')
      return answer(res, 200, { rows: diary(dir) });
    if (req.method !== 'POST' || !['/rank', '/rate', '/import'].includes(url.pathname))
      return answer(res, 404, { error: 'Nothing here by that name.' });
    if (!(await owned(key))) {
      const name = await whose();
      log('refused: not the owner', JSON.stringify({ key, app }));
      const words = name
        ? `This is ${name}’s taste. Only ${name} can rate films here or see what it guesses.`
        : 'This taste is someone else’s.';
      return answer(res, 403, { refused: 'not-yours', error: words });
    }
    // Her taste is told and written by its own page alone, never another app's.
    if (!own) {
      log('refused: not its own page', JSON.stringify({ key, app }));
      return answer(res, 403, {
        refused: 'not-its-page',
        error: 'Only taste’s own page can ask this.',
      });
    }
    const body = await read(req);
    if (url.pathname === '/rank') {
      const ranked = await rank(body);
      return answer(res, ranked.status, ranked.body);
    }
    if (url.pathname === '/rate') {
      const f = body.film ?? {};
      if (typeof f.title !== 'string' || !(body.said === null || body.said in SAID))
        return answer(res, 400, { error: 'say loved, liked or not-for-me of a film with a title' });
      const film = {
        title: f.title,
        year: Number.isInteger(f.year) ? f.year : null,
        ids: f.ids ?? {},
      };
      rate(dir, film, body.said === null ? null : SAID[body.said]);
      log('rated a film:', body.said);
      return answer(res, 200, { rated: true });
    }
    let got;
    try {
      got = letterboxd(String(body.text ?? ''));
    } catch {
      return answer(res, 400, {
        error: 'That file isn’t a Letterboxd ratings file. Choose ratings.csv from your export.',
      });
    }
    imported(dir, got.rows);
    log('imported', got.rows.length, 'ratings from Letterboxd;', got.unrated, 'lines without one');
    return answer(res, 200, { kept: got.rows.length });
  } catch (e) {
    log('failed:', e);
    return answer(res, 500, { error: 'Taste couldn’t answer. Try again.' });
  }
});

// Where taste listens on this computer: the place it took last time, kept
// in its dir, so its lend still points at it after a restart.
function place() {
  try {
    return Number(readFileSync(join(dir, 'port'), 'utf8').trim()) || 0;
  } catch {
    return 0;
  }
}

function listen(port) {
  return new Promise((done, fail) => {
    server.once('error', fail);
    server.listen(port, '127.0.0.1', () => done(server.address().port));
  });
}

async function start() {
  let port;
  try {
    port = await listen(place());
  } catch (e) {
    if (e.code !== 'EADDRINUSE') throw e;
    console.log(
      `Another program is using taste’s place on this computer (127.0.0.1:${place()}). If taste is already running in another window, use that one.`,
    );
    process.exit(1);
  }
  log(`listening on 127.0.0.1:${port}, ratings in ${dir}`);
  writeFileSync(join(dir, 'port'), `${port}\n`);
  let held;
  try {
    held = await svrngs.groups();
  } catch (e) {
    console.log(
      `Taste couldn’t reach svrngs on this computer. Start svrngs, then paste the line again. (${e.message})`,
    );
    process.exit(1);
  }
  if (!held.groups.length) {
    console.log(
      'Join a group first: taste shows films from your friends’ libraries. Then paste the line again.',
    );
    process.exit(1);
  }
  const groups = held.groups.flatMap((g) => ['--group', g.id]);
  const args = [
    'share',
    String(port),
    '--name',
    'taste',
    '--kind',
    'taste',
    '--reads',
    'jellyfin',
    '--refuse',
    '/diary',
    ...groups,
  ];
  const shared = await svrngs.run(args);
  log('svrngs share:', shared.ok ? 'lent' : 'failed', shared.said);
  if (!shared.ok) {
    console.log(`svrngs didn’t share taste: ${shared.said}`);
    process.exit(1);
  }
  // The home page names a lend by its lender, its own lender's too
  // (home/question.js `named`).
  const me = held.groups.find((g) => g.person)?.person;
  const opener = me ? `“Open ${me}’s taste”` : 'Open beside taste';
  const names = held.groups
    .map((g) => g.name)
    .filter(Boolean)
    .join(' and ');
  console.log(
    `Taste is on. To open it, go to your svrngs home page and press ${opener} under ${names || 'your group'}.`,
  );
  console.log(
    'Keep this window open while you use it: closing it turns taste off. Your ratings stay on this computer.',
  );
}

start();
