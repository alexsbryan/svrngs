// Magnets' board, as MAGNETS.md's "The whole app" has it, run under Node on
// its own SQL in SQLite (standin.mjs `database`, sqlite.py): each law a check
// that returns why it broke, or null, and the second test plants one broken
// board per law and watches its check catch it.

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { isDeepStrictEqual } from 'node:util';
import { test } from 'node:test';

import * as board from '../board.js';
import { database, SCHEMA } from './standin.mjs';

const PACK = JSON.parse(readFileSync(new URL('../pack.json', import.meta.url), 'utf8'));
const APP = { ...board, pack: PACK, schema: SCHEMA };
const differ = (got, want, what) =>
  isDeepStrictEqual(got, want)
    ? null
    : `${what}: ${JSON.stringify(got)}, not ${JSON.stringify(want)}`;

/** `steps` on a fresh board of `app`'s schema, each `[sql, ...params]`; the last one's answer. */
async function on(app, steps) {
  const db = database(app.schema);
  try {
    let last;
    for (const [sql, ...params] of steps) [last] = await db.run([{ sql, params }]);
    return last;
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
    const db = database(app.schema);
    const run = async (sql, ...params) => (await db.run([{ sql, params }]))[0];
    const tap = async (id) => run(app.PUT, id, app.next((await run(app.BOARD)).rows));
    try {
      const fresh = ids(await run(app.BOARD));
      await tap('moon');
      await tap('goose');
      const two = ids(await run(app.BOARD));
      await run(app.BACK, 'moon');
      await tap('moon');
      const again = (await run('SELECT id, position FROM magnets ORDER BY id')).rows;
      return (
        differ(fresh, [], 'a fresh board') ??
        differ(two, ['moon', 'goose'], 'two taps') ??
        differ(
          again,
          [
            { id: 'goose', position: 2 },
            { id: 'moon', position: 3 },
          ],
          'moon back, then on again at the end',
        )
      );
    } finally {
      db.stop();
    }
  },
  /** Two moves made apart that chose one position read by magnet ID. */
  async 'a tie is read by ID'(app) {
    const tie = 'INSERT INTO magnets (id, position) VALUES (?, 1)';
    return differ(
      ids(await on(app, [[tie, 'moon'], [tie, 'goose'], [app.BOARD]])),
      ['goose', 'moon'],
      'two at one position',
    );
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
    const tables = ids(
      await on(app, [["SELECT name AS id FROM sqlite_schema WHERE type = 'table'"]]),
    );
    if (!tables.length) return 'the schema makes no table';
    for (const t of tables) {
      const [columns, indexes, keys] = await Promise.all(
        [`table_info("${t}")`, `index_list("${t}")`, `foreign_key_list("${t}")`].map(
          async (p) => (await on(app, [[`SELECT * FROM pragma_${p}`]])).rows,
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
  // Each plant, and the words its law must break with.
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
    'a tap appends, and a tap back returns to the tray': {
      why: /^two taps: \["goose","moon"\], not \["moon","goose"\]$/,
      // Every tap at the first place.
      next: () => 1,
    },
    'a tie is read by ID': {
      why: /^two at one position: \["moon","goose"\]/,
      BOARD: 'SELECT id, position FROM magnets WHERE position IS NOT NULL ORDER BY position',
    },
    'the pack names each magnet once': {
      why: /^goose is in the pack twice$/,
      pack: { ...PACK, magnets: [...PACK.magnets, { id: 'goose', word: 'goose' }] },
    },
  };
  for (const [name, { why, ...plant }] of Object.entries(PLANTS))
    assert.match(String(await LAWS[name]({ ...APP, ...plant })), why, `${name}, planted`);
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
      String(await LAWS['the schema is one cr-sqlite takes']({ ...APP, schema: schema(planted) })),
      why,
      planted,
    );
});
