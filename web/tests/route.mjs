// A carried app's own database on a real svrngs, in the shape an app's
// stand-in for it takes in the app's tests (apps/magnets/tests/standin.mjs):
// `database()`, `posted(db, text)` and `serve()`, so the same tests run
// against the real route as against the stand-in. It names no app's table.
//
// Which of the file's tables are the app's is svrngs's to say, so it is
// asked (`tables`).
//
// The route is named by SVRNGS_ROUTE, JSON `{origin, port, token, db}`: the
// app's origin on a computer, that computer's svrngs's port and token, and
// the app's database file there, as crates/svrngs/tests/scenes/magnets.rs
// names Ana's. Unset, `real` is null and nothing here is asked.
//
// A real app has one database in its group, where a stand-in makes one
// fresh each time: so each `database()` first deletes every row of every
// table of the app's, through the route, by key, and each test starts on an
// empty one, so tests on it run one at a time, never two files at once (the
// scene runs them under --test-concurrency=1). Its `run` reads the file itself, nothing checked, as a
// stand-in's runs its own: what the route kept, never through it.
// A browser opens the app's origin opened by svrngs first (SVRNGS_OPENS,
// page.mjs `launch`), never `unopened`.

import { DatabaseSync } from 'node:sqlite';

import { asYou } from './as-you.mjs';

/** The real route the run names, or null. */
export const real = process.env.SVRNGS_ROUTE ? JSON.parse(process.env.SVRNGS_ROUTE) : null;

let client = null;
/** svrngs, asked as its person, as a program on its computer asks it. */
const svrngs = () => (client ??= asYou(real.port, real.token));

/** A request the route refused: `refused`, its short name, and its words as the message. */
export class Refused extends Error {
  constructor(refused, words) {
    super(words);
    this.refused = refused;
  }
}

/** `text` posted to the app's svrngs/sql, as its page posts it: the answer, or a Refused with the route's words. */
export async function posted(db, text) {
  await db?.ready;
  const res = await svrngs().fetch(new URL('svrngs/sql', real.origin).href, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: text,
  });
  const body = await res.json().catch(() => ({}));
  // svrngs-db's refusals say `refused` and `words`; the door's, `kind` and `error`.
  if (!res.ok) throw new Refused(body.refused ?? body.kind, body.words ?? body.error);
  return body;
}

/**
 * Each of the app's tables in `file`, and its key's columns. svrngs lets a
 * page read only the tables svrngs-db's `tables_of` names, the ones its
 * write form takes (crates/svrngs-db/src/lib.rs, `page_rule`'s `own`), and
 * none of cr-sqlite's, SQLite's or its own beside them (`internal`). So the
 * route is asked of each table in the file, never a copy of that rule: one
 * refused as not-allowed isn't the app's, and any other refusal is thrown.
 */
async function tables(file) {
  const every = file.prepare("SELECT name FROM sqlite_schema WHERE type = 'table'").all();
  const mine = [];
  for (const { name } of every) {
    const read = JSON.stringify({ sql: `SELECT * FROM "${name}" LIMIT 0` });
    const asked = await posted(null, read).catch((e) => e);
    if (asked instanceof Refused && asked.refused === 'not-allowed') continue;
    if (asked instanceof Error) throw asked;
    const columns = file.prepare('SELECT name FROM pragma_table_info(?) WHERE pk > 0').all(name);
    mine.push({ name, key: columns.map((c) => c.name) });
  }
  return mine;
}

/** Every row of every table of the app's deleted through the route, by key: those tables. */
async function emptied(file) {
  const mine = await tables(file);
  for (const { name, key } of mine) {
    const keys = file.prepare(`SELECT ${key.map((k) => `"${k}"`).join(', ')} FROM "${name}"`).all();
    const write = keys.map((k) => ({ delete: name, key: { ...k } }));
    if (write.length) await posted(null, JSON.stringify({ write }));
  }
  return mine;
}

/**
 * The app's database, emptied: `run(statements)` runs each on the file,
 * nothing checked, and gives each one's {rows}, once it is; `tables()`,
 * the names of the app's tables there, as svrngs names them; `stop()`.
 */
export function database() {
  const file = new DatabaseSync(real.db, { readOnly: true });
  const ready = emptied(file);
  return {
    ready,
    tables: async () => (await ready).map((t) => t.name),
    async run(statements) {
      await ready;
      return statements.map(({ sql, params = [] }) => ({
        rows: file
          .prepare(sql)
          .all(...params)
          .map((row) => ({ ...row })),
      }));
    },
    stop: () => file.close(),
  };
}

/** The app at its origin on the real svrngs, its database emptied: where it serves, its database, and its end. It has no switches: a stand-in's `set` is the stand-in's alone. */
export async function serve() {
  const db = database();
  await db.ready;
  const set = () => {
    throw new Error('the real route has no switches: a test that sets them is the stand-in’s');
  };
  return { at: real.origin, db, set, stop: () => db.stop() };
}
