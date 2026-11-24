// Saturday's board under Node, against the stand-in's route in SQLite
// (standin.mjs `posted`, sqlite.py), each read and write sent as the text
// the page sends, as Magnets' laws are (apps/magnets/tests/board.test.mjs).
// Each law is a check that returns why it broke, or null; the second test
// plants one broken board per law and watches its check catch it.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isDeepStrictEqual } from 'node:util';

import * as board from '../board.js';
import { database, PEOPLE, posted } from './standin.mjs';

const APP = { ...board, database, posted };
const { Ana, Bo } = PEOPLE;
const NAMES = new Map([
  [Ana, 'Ana'],
  [Bo, 'Bo'],
]);
const differ = (got, want, what) =>
  isDeepStrictEqual(got, want)
    ? null
    : `${what}: ${JSON.stringify(got)}, not ${JSON.stringify(want)}`;

/** A fresh board of `app`'s schema, posted to as `who`, read as `you` would draw it; stopped by `stop`. */
function fresh(app) {
  const db = app.database();
  const as = (who) => (body) =>
    app
      .posted(db, JSON.stringify(body), who)
      .catch((e) => ({ refused: e.refused, status: e.status }));
  const drawn = async (you) =>
    app.board({
      plans: (await as(you)({ sql: app.PLANS })).rows,
      things: (await as(you)({ sql: app.THINGS })).rows,
      mine: (await as(you)({ sql: app.MINE, params: [you] })).rows,
      you,
      names: NAMES,
    });
  return { db, as, drawn, stop: db.stop };
}

export const LAWS = {
  /** Plans are read by their time, then their title, and each starts with no votes. */
  async 'plans land, read in time order'(app) {
    const b = fresh(app);
    try {
      await b.as(Ana)({
        write: app.addPlan({ title: 'Picnic', at: '14:00', place: 'Park' }, Ana, 'p2'),
      });
      await b.as(Bo)({
        write: app.addPlan({ title: 'Breakfast', at: '09:30', place: '' }, Bo, 'p1'),
      });
      const drawn = await b.drawn(Ana);
      return differ(
        drawn.map((p) => [p.title, p.at, p.place, p.votes]),
        [
          ['Breakfast', '09:30', null, 0],
          ['Picnic', '14:00', 'Park', 0],
        ],
        'the plans',
      );
    } finally {
      b.stop();
    }
  },
  /** A total counts one vote a person however often they press, and drops when one is taken back. */
  async 'a total is one vote a person'(app) {
    const b = fresh(app);
    const votes = async () => (await b.drawn(Ana)).map((p) => [p.votes, p.yours]);
    try {
      await b.as(Ana)({ write: app.addPlan({ title: 'Picnic' }, Ana, 'p1') });
      await b.as(Ana)({ write: app.vote('p1', Ana, true) });
      await b.as(Ana)({ write: app.vote('p1', Ana, true) });
      await b.as(Bo)({ write: app.vote('p1', Bo, true) });
      const two = await votes();
      await b.as(Ana)({ write: app.vote('p1', Ana, false) });
      return (
        differ(two, [[2, true]], 'Ana twice and Bo once') ??
        differ(await votes(), [[1, false]], 'Ana’s taken back')
      );
    } finally {
      b.stop();
    }
  },
  /** A claim is shown as its claimer's alone, and only the claimer changes or takes it back: anyone else is refused, nothing kept. */
  async 'a claim is its claimer’s own'(app) {
    const b = fresh(app);
    const claims = async (you) =>
      (await b.drawn(you))[0].others.map((c) => [c.who, c.what, c.yours]);
    try {
      await b.as(Ana)({ write: app.addPlan({ title: 'Picnic' }, Ana, 'p1') });
      await b.as(Ana)({ write: app.claim('p1', 'ice', Ana, 'c1') });
      const shown =
        differ(await claims(Ana), [['Ana', 'ice', true]], 'Ana’s claim, to Ana') ??
        differ(await claims(Bo), [['Ana', 'ice', false]], 'Ana’s claim, to Bo');
      if (shown) return shown;
      // Bo's page shows him no Change on Ana's claim; sent anyway, it is refused.
      const theirs = (await b.drawn(Bo))[0].others[0];
      const tried =
        differ(
          [
            await b.as(Bo)({ write: app.change(theirs, 'nothing') }),
            await b.as(Bo)({ write: app.withdraw(theirs) }),
          ],
          Array(2).fill({ refused: 'not-yours', status: 403 }),
          'Bo changing Ana’s claim',
        ) ?? differ(await claims(Ana), [['Ana', 'ice', true]], 'Ana’s claim after Bo tried');
      if (tried) return tried;
      const mine = (await b.drawn(Ana))[0].others[0];
      await b.as(Ana)({ write: app.change(mine, 'two bags of ice') });
      const changed = await claims(Ana);
      await b.as(Ana)({ write: app.withdraw(mine) });
      return (
        differ(changed, [['Ana', 'two bags of ice', true]], 'Ana’s claim changed') ??
        differ(await claims(Ana), [], 'Ana’s claim taken back')
      );
    } finally {
      b.stop();
    }
  },
  /** A pasted list lands as one write, a row a thing, however it is spelled or bulleted, and pasting a thing again makes no second row. */
  async 'a pasted list lands all at once'(app) {
    const b = fresh(app);
    const listed = async () => (await b.drawn(Ana))[0].things.map((t) => t.words);
    try {
      await b.as(Ana)({ write: app.addPlan({ title: 'Picnic' }, Ana, 'p1') });
      const landed = await b.as(Ana)({
        write: app.paste('p1', 'Ice\n- chips\n\n  ICE \n1. Paper  plates\n'),
      });
      const first = await listed();
      await b.as(Bo)({ write: app.unlist('p1', 'chips') });
      const again = await b.as(Bo)({ write: app.paste('p1', 'chips\nice') });
      return (
        differ(landed, { count: 3 }, 'the rows one paste changed') ??
        differ(first, ['chips', 'Ice', 'Paper plates'], 'the list') ??
        differ(again, { count: 2 }, 'a paste of two things already listed') ??
        differ(await listed(), ['chips', 'ice', 'Paper plates'], 'the list, chips back')
      );
    } finally {
      b.stop();
    }
  },
  /** A plan taken off goes from the board, and its things and claims from what the board reads. */
  async 'a plan taken off is read no more'(app) {
    const b = fresh(app);
    try {
      await b.as(Ana)({ write: app.addPlan({ title: 'Picnic' }, Ana, 'p1') });
      await b.as(Ana)({ write: app.addPlan({ title: 'Bowling' }, Ana, 'p2') });
      await b.as(Ana)({ write: app.paste('p1', 'ice') });
      await b.as(Bo)({ write: app.claim('p1', 'chips', Bo, 'c1') });
      await b.as(Bo)({ write: app.takeOff('p1') });
      const things = (await b.as(Ana)({ sql: app.THINGS })).rows;
      return (
        differ(
          (await b.drawn(Ana)).map((p) => p.title),
          ['Bowling'],
          'the plans',
        ) ?? differ(things, [], 'the things and claims read')
      );
    } finally {
      b.stop();
    }
  },
  /** A claim brings the listed thing its words name, spelled any way; one naming nothing listed stands on its own. */
  'a claim brings the thing it names'(app) {
    const drawn = app.board({
      plans: [{ id: 'p1', title: 'Picnic', at: null, place: null, votes: 0 }],
      things: [
        { kind: 'thing', plan: 'p1', id: 'ice', words: 'Ice', person_key: null },
        { kind: 'claim', plan: 'p1', id: 'c1', words: ' ICE ', person_key: Bo },
        { kind: 'claim', plan: 'p1', id: 'c2', words: 'napkins', person_key: Ana },
        { kind: 'claim', plan: 'gone', id: 'c3', words: 'ice', person_key: Ana },
      ],
      mine: [],
      you: Ana,
      names: NAMES,
    })[0];
    return differ(
      [
        drawn.things.map((t) => [t.words, t.claims.map((c) => c.who)]),
        drawn.others.map((c) => [c.who, c.what]),
      ],
      [[['Ice', ['Bo']]], [['Ana', 'napkins']]],
      'the things and the other claims',
    );
  },
};

test('Saturday’s board keeps every law', async () => {
  for (const [name, law] of Object.entries(LAWS)) assert.equal(await law(APP), null, name);
});

test('each law catches its plant', async () => {
  const PLANTS = {
    'plans land, read in time order': {
      why: /^the plans: \[\["Picnic"/,
      // Read in the order they were added, not by time.
      PLANS: board.PLANS.replace('ORDER BY p.at, p.title, p.id', 'ORDER BY p.rowid'),
    },
    'a total is one vote a person': {
      why: /^Ana’s taken back: \[\[2,false\]\]/,
      // A total of every vote row, a vote taken back among them.
      PLANS: board.PLANS.replace(' AND v.yes = 1', ''),
    },
    'a claim is its claimer’s own': [
      {
        why: /^Ana’s claim, to Bo: \[\["Ana","ice",true\]\]/,
        // Every claim drawn as the asker's, so Bo is shown Change on Ana's.
        board: (read) =>
          board
            .board(read)
            .map((p) => ({ ...p, others: p.others.map((c) => ({ ...c, yours: true })) })),
      },
      {
        why: /^Bo changing Ana’s claim: \[\{"count":1\}/,
        // A route that doesn't know who asks, taking every write as Ana's.
        posted: (db, text) => posted(db, text, Ana),
      },
    ],
    'a pasted list lands all at once': {
      why: /^the rows one paste changed: \{"count":4\}/,
      // A thing keyed as typed, so "Ice" and "ICE" make two rows.
      paste: (plan, text) =>
        text
          .split('\n')
          .map((l) => l.replace(/^\s*(?:[-*]|\d+[.)])\s+/, '').trim())
          .filter(Boolean)
          .map((words) => ({ insert: 'things', row: { plan, thing: words, words, off: 0 } })),
    },
    'a plan taken off is read no more': {
      why: /^the things and claims read: \[\{"kind":"thing","plan":"p1"/,
      // The things and claims of every plan, on the board or not.
      THINGS: board.THINGS.replaceAll(
        ' AND plan IN (SELECT id FROM plans WHERE off IS NOT 1)',
        '',
      ).replaceAll(
        '\nWHERE withdrawn IS NOT 1 AND plan IN (SELECT id FROM plans WHERE off IS NOT 1)',
        ' WHERE withdrawn IS NOT 1',
      ),
    },
    'a claim brings the thing it names': {
      why: /^the things and the other claims: \[\[\["Ice",\[\]\]\]/,
      // Claims never matched to a thing: each stands on its own.
      board: (read) =>
        board.board(read).map((p) => ({
          ...p,
          things: p.things.map((t) => ({ ...t, claims: [] })),
          others: [...p.things.flatMap((t) => t.claims), ...p.others],
        })),
    },
  };
  for (const [name, plants] of Object.entries(PLANTS))
    for (const { why, ...plant } of [plants].flat())
      assert.match(String(await LAWS[name]({ ...APP, ...plant })), why, `${name}, planted`);
});

test('a pasted list is its lines, bullets, blanks and repeats left out', () => {
  assert.deepEqual(board.lines('  Ice\r\n* chips\n• Ice\n\n2) Paper   plates\n-dash kept'), [
    { thing: 'ice', words: 'Ice' },
    { thing: 'chips', words: 'chips' },
    { thing: 'paper plates', words: 'Paper plates' },
    { thing: '-dash kept', words: '-dash kept' },
  ]);
});
