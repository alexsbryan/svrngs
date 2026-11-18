// A stand-in for svrngs's route to a group's database, for Magnets' tests
// and for playing its face before the database exists. It never ships. It
// serves apps/magnets/ (tests/ apart) under a carried page's policy, read
// from the trust floor, and answers svrngs/sql from the app's own
// schema.sql in SQLite (sqlite.py), so the SQL is real.
//
// The shape it pins, as svrn-docs rings/current/APP_STATE.md §4 ("What a
// page can do") draws it. The real route answers the same, or Magnets
// changes with it:
//
//   POST svrngs/sql, a JSON body:
//     {sql, params}          one statement; `params` an array bound to its
//                            `?` in order, none when left out
//     [{sql, params}, ...]   a list, run as one transaction
//   answers 200 with
//     {rows: [{<column>: <value>, ...}]}  a statement that gives rows (a read)
//     {count: <n>}                         one that gives none (a write): the
//                                          rows it changed
//     [<answer>, ...]                      a list's: one a statement, in order
//     and `syncing: true` beside `rows` while this copy hasn't synced yet;
//   or refuses, keeping nothing of what it was given:
//     4xx {refused: <a short name>, words: <a sentence a person reads>}
//     503 {unreachable: true, words}   no computer of the group answers
//
//   GET svrngs/sql?q=<one read>, with Accept: text/event-stream, answers 200
//   with an event stream whose every event is one `data:` line holding a
//   read's answer, {rows} (and `syncing: true` while not synced): at once,
//   then again each time those rows change. A refusal is answered as above,
//   in JSON, in place of the stream.
//
// What it doesn't stand in for: who asks, the authorizer, limits and the
// write refill, change sets and sync. Its refusals are SQLite's own error,
// as 400 {refused: 'sql'}, and its switches. Each switch is for a test to
// see the page say that state (`set`):
//   syncing      every read's answer says `syncing: true`
//   unreachable  every request answered 503, and every open stream cut
//   refusing     {status, refused, words}: every POST answered with it
//
// Alone, `node apps/magnets/tests/standin.mjs [port]` prints where it serves.

import { spawn } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import http from 'node:http';
import { extname, join, normalize } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath, pathToFileURL } from 'node:url';

const APP = fileURLToPath(new URL('..', import.meta.url));
const ROOT = fileURLToPath(new URL('../../..', import.meta.url));
export const SCHEMA = join(APP, 'schema.sql');
/** What the stand-in says unreachable with, as a route would. */
export const AWAY = 'None of your group’s computers is on.';

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
};

/** A carried page's policy as the floor pins it: its directives under `page carried`. */
async function carriedPolicy() {
  const floor = await readFile(join(ROOT, 'scripts/trust_floor.txt'), 'utf8');
  const block = floor.split('\npage carried ')[1].split('\n').slice(1);
  const lines = block.slice(
    0,
    block.findIndex((l) => !l.startsWith('  ')),
  );
  return lines.map((l) => l.trim().split(' | ')[0]).join('; ');
}

/**
 * `schema` in SQLite, in a process of its own: `run(statements)` gives each
 * statement's answer, one transaction a call, in the order asked, or rejects
 * with SQLite's words.
 */
export function database(schema = SCHEMA) {
  const child = spawn('python3', ['-I', join(APP, 'tests/sqlite.py'), schema], {
    stdio: ['pipe', 'pipe', 'inherit'],
  });
  const stop = () => (process.off('exit', stop), child.kill());
  process.once('exit', stop);
  const waiting = [];
  createInterface({ input: child.stdout }).on('line', (line) => {
    const answer = JSON.parse(line);
    const { done, fail } = waiting.shift();
    if ('error' in answer) fail(new Error(answer.error));
    else done(answer.answers);
  });
  child.on('exit', (code) => {
    for (const w of waiting.splice(0)) w.fail(new Error(`the database ended (${code})`));
  });
  return {
    run: (statements) =>
      new Promise((done, fail) => {
        waiting.push({ done, fail });
        child.stdin.write(`${JSON.stringify(statements)}\n`);
      }),
    stop,
  };
}

/** The stand-in on 127.0.0.1 at `port`: where it serves, its database, its switches (`set`), and its end. */
export async function serve({ port = 0 } = {}) {
  const policy = await carriedPolicy();
  const db = database();
  const switches = { syncing: false, unreachable: false, refusing: null };
  /** Each open stream, to its query and the last answer it was sent. */
  const streams = new Map();
  const read = async (sql) => {
    const [answer] = await db.run([{ sql }]);
    if (!answer.rows) throw new Error('a subscription takes a read');
    return switches.syncing ? { ...answer, syncing: true } : answer;
  };
  const push = async () => {
    for (const [res, stream] of streams) {
      const data = JSON.stringify(await read(stream.q));
      if (data !== stream.last) res.write(`data: ${(stream.last = data)}\n\n`);
    }
  };

  const server = http.createServer(async (req, res) => {
    res.setHeader('content-security-policy', policy);
    const url = new URL(req.url, 'http://app');
    const json = (status, body) =>
      res.writeHead(status, { 'content-type': 'application/json' }).end(JSON.stringify(body));
    if (url.pathname === '/svrngs/sql') {
      if (switches.unreachable) return json(503, { unreachable: true, words: AWAY });
      try {
        if (req.method === 'GET' && /text\/event-stream/.test(req.headers.accept ?? '')) {
          const q = url.searchParams.get('q') ?? '';
          const last = JSON.stringify(await read(q));
          res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-store' });
          res.write(`data: ${last}\n\n`);
          streams.set(res, { q, last });
          res.on('close', () => streams.delete(res));
          return;
        }
        if (req.method !== 'POST')
          return json(405, {
            refused: 'method',
            words: 'Ask with POST, or GET with Accept: text/event-stream.',
          });
        if (switches.refusing) {
          const { status, refused, words } = switches.refusing;
          return json(status, { refused, words });
        }
        let body = '';
        for await (const chunk of req) body += chunk;
        const asked = JSON.parse(body);
        const list = (Array.isArray(asked) ? asked : [asked]).map(({ sql, params = [] }) => ({
          sql,
          params,
        }));
        const answers = (await db.run(list)).map((a) =>
          a.rows && switches.syncing ? { ...a, syncing: true } : a,
        );
        json(200, Array.isArray(asked) ? answers : answers[0]);
        if (answers.some((a) => 'count' in a)) await push();
        return;
      } catch (e) {
        return json(400, { refused: 'sql', words: e.message });
      }
    }
    const file = join(APP, normalize(url.pathname === '/' ? 'index.html' : url.pathname));
    if (!file.startsWith(APP) || file.startsWith(join(APP, 'tests')))
      return res.writeHead(404).end();
    try {
      const bytes = await readFile(file);
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(bytes);
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise((done) => server.listen(port, '127.0.0.1', done));
  return {
    at: `http://127.0.0.1:${server.address().port}/`,
    db,
    /** Change a switch: open streams are sent the board again, or cut while unreachable. */
    async set(change) {
      Object.assign(switches, change);
      if (switches.unreachable) for (const res of streams.keys()) res.destroy();
      else await push();
    },
    stop() {
      server.close();
      server.closeAllConnections();
      db.stop();
    },
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const standin = await serve({ port: Number(process.argv[2] ?? 0) });
  console.log(standin.at);
}
