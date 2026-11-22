// Magnets' board, as MAGNETS.md's "The whole app" has it, run under Node
// against the stand-in's route in SQLite (standin.mjs `posted`, sqlite.py),
// or, where SVRNGS_ROUTE names one, the real route on a computer's svrngs
// (web/tests/route.mjs; the scene crates/svrngs/tests/scenes/magnets.rs
// runs it so): each read and each tap sent as the text the page sends. Each
// law is a check that returns why it broke, or null, and the second test
// plants one broken board, or one broken route, per law and watches its
// check catch it, always on the stand-in, whose route a plant can break.

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { isDeepStrictEqual } from 'node:util';
import { test } from 'node:test';

import * as board from '../board.js';
import * as route from '../../../web/tests/route.mjs';
import { database, posted, Refused, SCHEMA } from './standin.mjs';

const PACK = JSON.parse(readFileSync(new URL('../pack.json', import.meta.url), 'utf8'));
// The board, its pack and schema, and the stand-in's database and route, so
// a plant can break any one of them; and the board on the route the run
// names, the real one where it names one.
const STANDIN = { ...board, pack: PACK, schema: SCHEMA, database, posted };
const APP = route.real ? { ...STANDIN, database: route.database, posted: route.posted } : STANDIN;
const differ = (got, want, what) =>
  isDeepStrictEqual(got, want)
    ? null
    : `${what}: ${JSON.stringify(got)}, not ${JSON.stringify(want)}`;

/** `bodies` posted to a fresh board of `app`'s schema, each as the page sends one; the last one's answer. */
async function on(app, bodies) {
  const db = app.database(app.schema);
  try {
    let last;
    for (const body of bodies) last = await app.posted(db, JSON.stringify(body));
    return last;
  } finally {
    db.stop();
  }
}
/** `sql` on a fresh board of `app`'s schema, run as the stand-in runs its own, unchecked: its rows. */
async function unchecked(app, sql) {
  const db = app.database(app.schema);
  try {
    return (await db.run([{ sql }]))[0].rows;
  } finally {
    db.stop();
  }
}
const ids = (answer) => answer.rows.map((r) => r.id);

export const LAWS = {
  /** The sentence is the rows' magnets in their order; the tray the pack's others in its order; a row the pack doesn't name, neither. */
  'the board is drawn from the rows'(app) {
    const pack = { prompt: 'p', magnets: ['a', 'b', 'c', 'd'].map((id) => ({ id, word: id })) };
    const drawn = app.order(pack, [{ id: 'c' }, { id: 'gone' }, { id: 'a' }]);
    return differ(
      [drawn.sentence.map((m) => m.id), drawn.tray.map((m) => m.id), drawn.unknown],
      [['c', 'a'], ['b', 'd'], ['gone']],
      'sentence, tray and unknown',
    );
  },
  /**
   * A fresh board is an empty table; a tap appends at the position `next`
   * reads from the board the page holds, and a tap back leaves the row with
   * no position.
   */
  async 'a tap appends, and a tap back returns to the tray'(app) {
    const db = app.database(app.schema);
    const post = (body) => app.posted(db, JSON.stringify(body));
    const read = async () => post({ sql: app.BOARD });
    // A tap as app.js `move` sends one: one op, a put at the position `next`
    // reads from the board the page holds, or a back.
    const tap = async (id) => post({ write: [app.put(id, app.next((await read()).rows))] });
    const all = async () =>
      (await db.run([{ sql: 'SELECT id, position FROM magnets ORDER BY id' }]))[0].rows;
    try {
      const fresh = ids(await read());
      const changed = [await tap('moon'), await tap('goose')];
      const two = ids(await read());
      changed.push(await post({ write: [app.back('moon')] }));
      const out = await all();
      changed.push(await tap('moon'));
      const again = await all();
      return (
        differ(fresh, [], 'a fresh board') ??
        differ(two, ['moon', 'goose'], 'two taps') ??
        differ(
          out,
          [
            { id: 'goose', position: 2 },
            { id: 'moon', position: null },
          ],
          'moon back in the tray',
        ) ??
        differ(
          again,
          [
            { id: 'goose', position: 2 },
            { id: 'moon', position: 3 },
          ],
          'moon back, then on again at the end',
        ) ??
        differ(changed, Array(4).fill({ count: 1 }), 'the rows each move changed')
      );
    } finally {
      db.stop();
    }
  },
  /** Two moves made apart that chose one position read by magnet ID. */
  async 'a tie is read by ID'(app) {
    const tie = (id) => ({ write: [app.put(id, 1)] });
    return differ(
      ids(await on(app, [tie('moon'), tie('goose'), { sql: app.BOARD }])),
      ['goose', 'moon'],
      'two at one position',
    );
  },
  /**
   * A write sent as SQL, as the page sent its moves before the write form,
   * is refused, its words naming that form, and nothing of it kept. The
   * route asks SQLite what a statement does, so a write behind a WITH or
   * before a RETURNING is refused as well as one that begins INSERT.
   */
  async 'a write sent as SQL is refused'(app) {
    const db = app.database(app.schema);
    const post = (body) => app.posted(db, JSON.stringify(body)).catch((e) => e);
    try {
      for (const [sql, ...params] of [
        [
          'INSERT INTO magnets (id, position) VALUES (?, ?) ON CONFLICT (id) DO UPDATE SET position = excluded.position',
          'moon',
          1,
        ],
        ['UPDATE magnets SET position = NULL WHERE id = ?', 'moon'],
        ['INSERT INTO magnets (id, position) VALUES (?, ?) RETURNING id', 'moon', 1],
        [
          'WITH placed (id, position) AS (VALUES (?, ?)) INSERT INTO magnets SELECT * FROM placed',
          'moon',
          1,
        ],
      ]) {
        const answer = await post({ sql, params });
        if (
          answer.refused !== 'not-a-read' ||
          !String(answer.message).includes('{write: [op, ...]}')
        )
          return `${sql}: answered ${answer instanceof Error ? `${answer.refused}, ${answer.message}` : JSON.stringify(answer)}, not refused naming the write form`;
      }
      const kept = await db.run([{ sql: 'SELECT id FROM magnets' }]);
      return differ(kept[0].rows, [], 'what the refused writes kept');
    } finally {
      db.stop();
    }
  },
  /** Every magnet has an ID of its own and a word, under one prompt. */
  'the pack names each magnet once'(app) {
    const seen = new Set();
    for (const m of app.pack.magnets) {
      if (typeof m.id !== 'string' || !/^[a-z0-9-]+$/.test(m.id))
        return `an ID ${m.id} is not a slug`;
      if (typeof m.word !== 'string' || !m.word.trim()) return `${m.id} has no word`;
      if (seen.has(m.id)) return `${m.id} is in the pack twice`;
      seen.add(m.id);
    }
    return typeof app.pack.prompt === 'string' && app.pack.prompt.trim() && seen.size
      ? null
      : 'no prompt, or no magnets';
  },
  // cr-sqlite's own limits (superfly/cr-sqlite core/rs/core/src/tableinfo.rs),
  // read here from SQLite's pragmas until `svrngs check` loads cr-sqlite and
  // asks it (MAGNETS_BUILD S5), the one decider; this law goes then.
  /** Each table has a NOT NULL primary key, no other UNIQUE and no foreign key. */
  async 'the schema is one cr-sqlite takes'(app) {
    // Read unchecked, as the stand-in reads its own schema: the route takes
    // no PRAGMA as a read. cr-sqlite's own tables, beside the app's in a
    // real database, are not the app's.
    const tables = (
      await unchecked(
        app,
        "SELECT name AS id FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE '%crsql%'",
      )
    ).map((r) => r.id);
    if (!tables.length) return 'the schema makes no table';
    for (const t of tables) {
      const [columns, indexes, keys] = await Promise.all(
        [`table_info('${t}')`, `index_list('${t}')`, `foreign_key_list('${t}')`].map((p) =>
          unchecked(app, `SELECT * FROM pragma_${p}`),
        ),
      );
      const pk = columns.filter((c) => c.pk > 0);
      if (!pk.length) return `${t} has no primary key`;
      if (pk.some((c) => !c.notnull)) return `${t}'s primary key may be NULL`;
      if (indexes.some((i) => i.unique && i.origin !== 'pk'))
        return `${t} has a UNIQUE besides its key`;
      if (keys.length) return `${t} has a foreign key`;
    }
    return null;
  },
};

test('Magnets’ board keeps every law', async () => {
  for (const [name, law] of Object.entries(LAWS)) assert.equal(await law(APP), null, name);
});

/** A schema written to a file of its own, for a plant. */
function schema(sql) {
  const file = join(mkdtempSync(join(tmpdir(), 'magnets-schema-')), 'schema.sql');
  writeFileSync(file, sql);
  return file;
}

test('each law catches its plant', async () => {
  // Each plant, or plants, and the words its law must break with.
  const PLANTS = {
    'the board is drawn from the rows': {
      why: /^sentence, tray and unknown: \[\["a","c"\]/,
      // The sentence in the pack's order, not the rows'.
      order: (pack, rows) => {
        const placed = new Set(rows.map((r) => r.id));
        return {
          sentence: pack.magnets.filter((m) => placed.has(m.id)),
          tray: pack.magnets.filter((m) => !placed.has(m.id)),
          unknown: [],
        };
      },
    },
    'a tap appends, and a tap back returns to the tray': [
      {
        why: /^two taps: \["goose","moon"\], not \["moon","goose"\]$/,
        // Every tap at the first place.
        next: () => 1,
      },
      {
        why: /^moon back in the tray: \[\{"id":"goose","position":2\}\], not /,
        // A tap back that deletes the row in place of keeping it unplaced.
        back: (id) => ({ delete: 'magnets', key: { id } }),
      },
    ],
    'a tie is read by ID': {
      why: /^two at one position: \["moon","goose"\]/,
      BOARD: 'SELECT id, position FROM magnets WHERE position IS NOT NULL ORDER BY position',
    },
    'a write sent as SQL is refused': {
      why: /^WITH placed .*: answered \{"count":-?\d+\}, not refused naming the write form$/,
      // A route that tells a read by its first word, as a keyword check
      // would: a write behind a WITH gets through, run as given.
      database: (schema) => {
        const db = database(schema);
        const read = async (sql, params) => {
          if (/^\s*(INSERT|UPDATE|DELETE|REPLACE)\b/i.test(sql))
            throw new Refused('not-a-read', 'A write goes as {write: [op, ...]}.');
          return (await db.run([{ sql, params }]))[0];
        };
        return { ...db, read };
      },
    },
    'the pack names each magnet once': {
      why: /^goose is in the pack twice$/,
      pack: { ...PACK, magnets: [...PACK.magnets, { id: 'goose', word: 'goose' }] },
    },
  };
  for (const [name, plants] of Object.entries(PLANTS))
    for (const { why, ...plant } of [plants].flat())
      assert.match(String(await LAWS[name]({ ...STANDIN, ...plant })), why, `${name}, planted`);
  for (const [planted, why] of [
    ['CREATE TABLE magnets (id TEXT PRIMARY KEY, position INTEGER);', /key may be NULL/],
    [
      'CREATE TABLE magnets (id TEXT NOT NULL PRIMARY KEY, position INTEGER UNIQUE);',
      /UNIQUE besides its key/,
    ],
    [
      'CREATE TABLE packs (id TEXT NOT NULL PRIMARY KEY);\nCREATE TABLE magnets (id TEXT NOT NULL PRIMARY KEY, pack TEXT REFERENCES packs (id));',
      /^magnets has a foreign key$/,
    ],
  ])
    assert.match(
      String(
        await LAWS['the schema is one cr-sqlite takes']({ ...STANDIN, schema: schema(planted) }),
      ),
      why,
      planted,
    );
});
