// A stand-in for svrngs's route to a group's database, for Magnets' tests
// and for playing its face before the database exists. It never ships. It
// serves apps/magnets/ (tests/ apart) under a carried page's policy, read
// from the trust floor, and answers svrngs/sql from the app's own
// schema.sql in SQLite (sqlite.py), so the SQL is real.
//
// The shape it pins, as svrn-docs rings/current/APP_STATE.md §4 ("What a
// page can do") draws it. The real route answers the same, or Magnets
// changes with it: Magnets' tests run against it too, through
// web/tests/route.mjs, in the scene crates/svrngs/tests/scenes/magnets.rs.
//
//   POST svrngs/sql, a JSON body, one of:
//     {sql, params}       one read; `params` an array bound to its `?` in
//                         order, none when left out. Answers 200
//                         {rows: [{<column>: <value>, ...}]}, with
//                         `syncing: true` beside `rows` while this copy
//                         hasn't synced yet; the real route says `more`
//                         too, true past the most rows it gives. A
//                         statement that writes is refused, its words
//                         naming the write form.
//     {write: [op, ...]}  one transaction. Answers 200 {count: <n>}, the
//                         rows changed in all. A refusal keeps nothing of
//                         the list. Each op is one of:
//       {insert: <table>, row: {<column>: <value>, ...}}
//           naming every key column: makes the row, or, if a row has that
//           key, sets the columns named (an upsert of the named columns)
//       {update: <table>, key: {<every key column>: <value>},
//           set: {<one or more non-key columns>: <value>}}
//       {delete: <table>, key: {<every key column>: <value>}}
//   A value is null, a number, a string, or {base64: "..."} for a BLOB.
//   A write is this one form, by key, the same on every device, and never
//   SQL: a phone's worker checks what it signs without a SQL parser, an app
//   that works on a laptop must work on a phone, and a conditional or
//   computed write holds only on the copy that ran it.
//   Or it refuses, keeping nothing of what it was given:
//     4xx {refused: <a short name>, words: <a sentence a person reads>}
//         among them an unknown table or column, a key that doesn't name
//         every key column, a key column in `set`, an empty list, or any
//         other value, in words a developer can act on
//     503 {unreachable: true, words}   no computer of the group answers
//
//   GET svrngs/sql?q=<one read>, with Accept: text/event-stream, answers 200
//   with an event stream whose every event is one `data:` line holding a
//   read's answer, {rows} (and `syncing: true` while not synced): at once,
//   then again each time those rows change. A refusal is answered as above,
//   in JSON, in place of the stream.
//
// What it doesn't stand in for: who asks, the authorizer of who may read
// or write what, limits and the write refill, change sets and sync. It
// tells a read apart by asking SQLite's authorizer what the statement does
// (sqlite.py), so it also refuses a PRAGMA or a BEGIN there. Its refusals
// are 400 {refused: 'form'}, a body or an op the form doesn't take;
// {refused: 'not-a-read'}, a read that does more than read; {refused: 'sql'},
// SQLite's own error; 500 {refused: 'stand-in'}, a fault of its own; and
// its switches. Each switch is for a test to see the page say that state
// (`set`):
//   syncing      every read's answer says `syncing: true`
//   unreachable  every request answered 503, and every open stream cut
//   refusing     {status, refused, words}: every POST answered with it
//
// Alone, `node apps/magnets/tests/standin.mjs [port] [address]` prints where
// it serves: on 127.0.0.1 unless given an address of this computer's, such
// as its WiFi's, for a phone on the same WiFi to play (demo/magnets.md,
// proof 1).

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

/** A request the route won't answer: `refused`, its short name, and `message`, its words. */
export class Refused extends Error {
  constructor(refused, words) {
    super(words);
    this.refused = refused;
  }
}

/** The write form, as every refusal that names it ends. */
const FORM =
  'A write goes as {write: [op, ...]}, each op one of {insert: <table>, row: {<column>: <value>, ...}}, ' +
  '{update: <table>, key: {<every key column>: <value>}, set: {<column>: <value>, ...}} ' +
  'or {delete: <table>, key: {<every key column>: <value>}}.';
const NOT_A_READ = `A read is one statement that only reads, and SQLite finds this one does more. ${FORM}`;
const BASE64 = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;
/** Each table's columns, from pragma_table_info: `name` and, for a key column, its place in the key. */
const TABLES =
  'SELECT t.name AS tbl, c.name, c.pk FROM sqlite_schema AS t, pragma_table_info(t.name) AS c ' +
  "WHERE t.type = 'table' AND substr(t.name, 1, 7) <> 'sqlite_'";

/** Refused as a body or an op the form doesn't take, in `words`. */
const refuse = (words) => {
  throw new Refused('form', words);
};
const isObject = (v) => typeof v === 'object' && v !== null && !Array.isArray(v);
const quote = (name) => `"${name.replaceAll('"', '""')}"`;
const listed = (names) => names.map((n) => JSON.stringify(n)).join(', ') || 'none';
const names = (entries) => entries.map(([column]) => column);
const values = (entries) => entries.map(([, v]) => v);
const same = (a, b) => JSON.stringify([...a].sort()) === JSON.stringify([...b].sort());

/** `v` as a value the route binds: null, a number, a string, or {base64} for a BLOB; else refused, naming `where`. */
function value(v, where) {
  const blob =
    isObject(v) &&
    same(Object.keys(v), ['base64']) &&
    typeof v.base64 === 'string' &&
    BASE64.test(v.base64);
  if (v === null || typeof v === 'string' || Number.isFinite(v) || blob) return v;
  refuse(
    `${where} is ${JSON.stringify(v)}: a value is null, a number, a string, or {base64: "..."} for a BLOB.`,
  );
}

/** `named`, columns of `table` to values, as its entries; refused for a column the table lacks or a value the route doesn't take. */
function columns(named, table, where) {
  if (!isObject(named))
    refuse(`${where} is ${JSON.stringify(named)}, not an object of columns to values.`);
  for (const [column, v] of Object.entries(named)) {
    if (!table.columns.includes(column))
      refuse(
        `${where} names ${JSON.stringify(column)}, but ${table.name} has no such column; its columns are ${listed(table.columns)}.`,
      );
    value(v, `${where}'s ${column}`);
  }
  return Object.entries(named);
}

/** `named` as a key of `table`: every key column, and no other. */
function keyOf(named, table, where) {
  const key = columns(named, table, `${where}'s key`);
  if (!same(names(key), table.key))
    refuse(
      `${where}'s key names ${listed(names(key))}: a key of ${table.name} names every key column, ${listed(table.key)}, and no other.`,
    );
  return key;
}

/** Each op's fields, by its kind. */
const OPS = {
  insert: ['insert', 'row'],
  update: ['update', 'key', 'set'],
  delete: ['delete', 'key'],
};

/** One op as one statement on `tables`, its names quoted and every value bound; refused naming what to change. */
function statement(op, tables, where) {
  const kind = Object.keys(OPS).find((k) => isObject(op) && k in op);
  if (!kind || !same(Object.keys(op), OPS[kind]))
    refuse(`${where} is ${JSON.stringify(op)}. ${FORM}`);
  const table =
    tables.get(op[kind]) ??
    refuse(
      `${where} names the table ${JSON.stringify(op[kind])}, but there is no such table; the tables are ${listed([...tables.keys()])}.`,
    );
  if (!table.key.length)
    refuse(`${table.name} has no key column, so no write can name a row of it.`);
  const [name, key] = [quote(table.name), table.key.map(quote).join(', ')];

  if (kind === 'insert') {
    const row = columns(op.row, table, `${where}'s row`);
    const missing = table.key.filter((k) => !names(row).includes(k));
    if (missing.length)
      refuse(
        `${where}'s row leaves out ${listed(missing)}: an insert names every key column of ${table.name}, ${listed(table.key)}.`,
      );
    const rest = names(row).filter((c) => !table.key.includes(c));
    const upsert = rest.length
      ? `DO UPDATE SET ${rest.map((c) => `${quote(c)} = excluded.${quote(c)}`).join(', ')}`
      : 'DO NOTHING';
    const holes = row.map(() => '?').join(', ');
    return {
      sql: `INSERT INTO ${name} (${names(row).map(quote).join(', ')}) VALUES (${holes}) ON CONFLICT (${key}) ${upsert}`,
      params: values(row),
    };
  }

  const keyed = keyOf(op.key, table, where);
  const matched = names(keyed).map((c) => `${quote(c)} = ?`);
  if (kind === 'delete')
    return { sql: `DELETE FROM ${name} WHERE ${matched.join(' AND ')}`, params: values(keyed) };

  const set = columns(op.set, table, `${where}'s set`);
  const moved = names(set).filter((c) => table.key.includes(c));
  if (moved.length)
    refuse(
      `${where}'s set changes ${listed(moved)}, a key column of ${table.name}: to give a row another key, delete it and insert it.`,
    );
  if (!set.length)
    refuse(
      `${where}'s set names no column: an update sets one or more of the other columns of ${table.name}.`,
    );
  const assigned = names(set).map((c) => `${quote(c)} = ?`);
  return {
    sql: `UPDATE ${name} SET ${assigned.join(', ')} WHERE ${matched.join(' AND ')}`,
    params: [...values(set), ...values(keyed)],
  };
}

/**
 * The write form on `db`: each op checked against the tables and columns
 * pragma_table_info gives, made one statement, and the list run as one
 * transaction; the rows changed in all, or refused with nothing kept.
 */
export async function write(db, ops) {
  if (!Array.isArray(ops) || !ops.length)
    refuse(`write takes a list of one or more ops, not ${JSON.stringify(ops)}. ${FORM}`);
  const tables = new Map();
  for (const { tbl, name, pk } of (await db.run([{ sql: TABLES }]))[0].rows) {
    if (!tables.has(tbl)) tables.set(tbl, { name: tbl, columns: [], key: [] });
    tables.get(tbl).columns.push(name);
    if (pk > 0) tables.get(tbl).key[pk - 1] = name;
  }
  const answers = await db.run(ops.map((op, i) => statement(op, tables, `write[${i}]`)));
  return { count: answers.reduce((n, a) => n + a.count, 0) };
}

/** `text` as JSON, or undefined where it isn't. */
function parsed(text) {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

/** A POST's body, as the text a page sends, answered from `db`: {rows} for {sql, params}, {count} for {write}, or refused. */
export async function posted(db, text) {
  const asked = parsed(text);
  const fields = isObject(asked) ? Object.keys(asked) : [];
  if (same(fields, ['write'])) return write(db, asked.write);
  if ((same(fields, ['sql']) || same(fields, ['sql', 'params'])) && typeof asked.sql === 'string') {
    const params = asked.params ?? [];
    if (!Array.isArray(params))
      refuse(`params is ${JSON.stringify(params)}, not a list of values.`);
    return db.read(
      asked.sql,
      params.map((v, i) => value(v, `params[${i}]`)),
    );
  }
  const sent = asked === undefined ? 'a body that isn’t JSON' : text.slice(0, 200);
  refuse(`svrngs/sql takes {sql, params}, one read, or {write: [op, ...]}, not ${sent}. ${FORM}`);
}

/**
 * `schema` in SQLite, in a process of its own (sqlite.py), asked one thing
 * at a time: `run(statements)` runs them as one transaction, nothing
 * checked, and gives each one's answer in order; `read(sql, params)` runs
 * one statement only if SQLite finds it only reads, and gives its {rows};
 * `tables()` names the app's tables. Either rejects with a Refused,
 * nothing of it kept.
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
    if (answer.refused === 'not-a-read') fail(new Refused('not-a-read', NOT_A_READ));
    else if ('refused' in answer) fail(new Refused(answer.refused, answer.words));
    else done(answer);
  });
  child.on('exit', (code) => {
    for (const w of waiting.splice(0)) w.fail(new Error(`the database ended (${code})`));
  });
  const ask = (asked) =>
    new Promise((done, fail) => {
      waiting.push({ done, fail });
      child.stdin.write(`${JSON.stringify(asked)}\n`);
    });
  const run = async (statements) => (await ask({ run: statements })).answers;
  return {
    run,
    read: (sql, params = []) => ask({ read: { sql, params } }),
    /** The app's tables, as the write form reads them (TABLES), the route's `tables` stood in for. */
    tables: async () => [...new Set((await run([{ sql: TABLES }]))[0].rows.map((r) => r.tbl))],
    stop,
  };
}

/** The stand-in on 127.0.0.1 at `port`: where it serves, its database, its switches (`set`), and its end. */
export async function serve({ port = 0, host = '127.0.0.1' } = {}) {
  const policy = await carriedPolicy();
  const db = database();
  const switches = { syncing: false, unreachable: false, refusing: null };
  /** Each open stream, to its query and the last answer it was sent. */
  const streams = new Map();
  /** An answer as this copy gives it: `syncing: true` beside its rows while that switch is on. */
  const given = (answer) =>
    answer.rows && switches.syncing ? { ...answer, syncing: true } : answer;
  const read = async (sql) => given(await db.read(sql));
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
        const answer = await posted(db, body);
        json(200, given(answer));
        if ('count' in answer) await push();
        return;
      } catch (e) {
        // A Refused is the route's answer; anything else is the stand-in's
        // own fault, said as one, never as the asker's.
        if (e instanceof Refused) return json(400, { refused: e.refused, words: e.message });
        return json(500, { refused: 'stand-in', words: e.message });
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
  await new Promise((done) => server.listen(port, host, done));
  return {
    at: `http://${host}:${server.address().port}/`,
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
  const [port = 0, host] = process.argv.slice(2);
  const standin = await serve({ port: Number(port), ...(host ? { host } : {}) });
  console.log(standin.at);
}
