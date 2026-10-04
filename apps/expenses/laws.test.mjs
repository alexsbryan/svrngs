// five-laws (RING_APP_LIBRARY §4, and DESIGN at tag e0-bot, C12): Expenses holds
// determinism, environment independence, non-interference, idempotence,
// totality (RING_APP_LIBRARY §4) and the sixth, no module globals. Each law is
// a check that returns why it broke, or null; the second test plants one
// broken reducer per law and watches its check catch it.
//
// These run the app under Node, as its developer does.

import assert from 'node:assert/strict';
import { isDeepStrictEqual } from 'node:util';
import { test } from 'node:test';

import * as expenses from './expenses.js';

const [SAM, AMA, BO, CY, DI] = ['discord:1', 'discord:2', 'discord:3', 'discord:4', 'discord:5'];
const DAY = 86400;

const act = (id, ts, by, op) => ({ id, ts, by, op });
/** Five people split a pizza, confirm (one twice), pay back, and buy drinks. */
const JOURNAL = [
  act('a1', 1000, SAM, { kind: 'expense', amount_cents: 6000, description: 'pizza' }),
  act('a2', 1010, AMA, { kind: 'share', expense: 'a1' }),
  act('a3', 1020, BO, { kind: 'share', expense: 'a1' }),
  act('a4', 1030, CY, { kind: 'share', expense: 'a1' }),
  act('a5', 1040, DI, { kind: 'share', expense: 'a1' }),
  act('a6', 1050, BO, { kind: 'share', expense: 'a1' }),
  act('a7', 2000, AMA, { kind: 'settle', to: SAM, amount_cents: 1200 }),
  act('a8', 2100, AMA, { kind: 'settle', to: SAM, amount_cents: 1200 }),
  act('a9', 3000, CY, { kind: 'expense', amount_cents: 1001, description: 'drinks' }),
  act('a10', 3010, DI, { kind: 'share', expense: 'a9' }),
  act('a11', 3020, SAM, { kind: 'expense', amount_cents: -5, description: 'nothing' }),
  act('a12', 9 * DAY, null, { kind: 'reminded', due: `remind:${BO}:1050` }),
];
/** Acts a duplicate of which must change nothing: the kinds Expenses keys. */
const KEYED = new Set(['share', 'settle', 'reminded']);

const clone = (v) => structuredClone(v);
const fold = (app, acts) => acts.reduce((s, a) => app.reduce(s, clone(a)), app.init());
/** Everything the app answers: the fold, copied, so a later fold that shares its object cannot change it. */
function answers(app, acts = JOURNAL) {
  return { state: clone(fold(app, acts)) };
}
const differ = (a, b, what) => (isDeepStrictEqual(a, b) ? null : `${what} differ`);

/** Run `fn` with the clock, the dice and the locale all changed. */
function elsewhere(fn) {
  const saved = { Date: globalThis.Date, random: Math.random, compare: String.prototype.localeCompare };
  const RealDate = saved.Date;
  const at = 4_102_444_800_000; // 2100-01-01
  globalThis.Date = class extends RealDate {
    constructor(...a) {
      super(...(a.length ? a : [at]));
    }
    static now() {
      return at;
    }
  };
  Math.random = () => 0.999;
  String.prototype.localeCompare = function (other) {
    return -saved.compare.call(this, other);
  };
  try {
    return fn();
  } finally {
    globalThis.Date = saved.Date;
    Math.random = saved.random;
    String.prototype.localeCompare = saved.compare;
  }
}

export const LAWS = {
  /** Folding one journal twice gives the same state, and it is exactly JSON. */
  determinism(app) {
    const once = answers(app);
    const exact = JSON.parse(JSON.stringify(once.state));
    return differ(once, answers(app), 'two folds of one journal') ?? differ(once.state, exact, 'the state and its JSON');
  },
  /** The same answers under another clock, random seed and locale. */
  'environment independence'(app) {
    return differ(answers(app), elsewhere(() => answers(app)), 'answers here and elsewhere');
  },
  /** One group's fold does not reach another's: no shared object between folds, and no act changed. */
  'non-interference'(app) {
    const alone = answers(app, JOURNAL.slice(0, 6));
    fold(app, JOURNAL);
    const acts = clone(JOURNAL);
    acts.reduce((s, a) => app.reduce(s, a), app.init());
    return differ(alone, answers(app, JOURNAL.slice(0, 6)), 'a fold alone and after another') ?? differ(acts, JOURNAL, 'the acts before and after a fold');
  },
  /** A second act of a keyed kind, with the same author, op and day, changes nothing. */
  idempotence(app) {
    for (const [i, a] of JOURNAL.entries()) {
      if (!KEYED.has(a.op.kind)) continue;
      const doubled = [...JOURNAL.slice(0, i + 1), { ...a, id: `${a.id}-again`, ts: a.ts + 1 }, ...JOURNAL.slice(i + 1)];
      const why = differ(fold(app, JOURNAL), fold(app, doubled), `the fold with ${a.id} once and twice`);
      if (why) return why;
    }
    return null;
  },
  /** No act makes `reduce` throw, and the state stays exactly JSON. */
  totality(app) {
    const ops = [null, 1, 'x', [], {}, { kind: 'expense' }, { kind: 'toString' }, { kind: '__proto__' },
      { kind: 'expense', amount_cents: 1.5, description: 'x' }, { kind: 'expense', amount_cents: 1e300, description: 'x' },
      { kind: 'expense', amount_cents: 5, description: 7 }, { kind: 'share', expense: '__proto__' },
      { kind: 'share', expense: 'constructor' }, { kind: 'share', expense: ['a1'] }, { kind: 'settle', to: null, amount_cents: 5 },
      { kind: 'settle', to: '__proto__', amount_cents: 5 }, { kind: 'reminded' }, { kind: 'reminded', due: {} }];
    const bys = [null, '', 42, '__proto__', 'constructor', SAM];
    const acts = [null, 7, [], {}, { id: 5, ts: 'now', by: SAM, op: {} }, { id: '__proto__', ts: 1, by: SAM, op: JOURNAL[0].op }];
    for (const op of ops) for (const by of bys) acts.push(act(`x${acts.length}`, 1, by, op));
    let state = fold(app, JOURNAL);
    try {
      for (const a of acts) state = app.reduce(state, a);
    } catch (e) {
      return `it threw: ${e?.message ?? e}`;
    }
    return differ(state, JSON.parse(JSON.stringify(state)), 'the state and its JSON');
  },
  /** `reduce` depends only on the state and the act: the same pair gives the same answer, whatever ran before. */
  'no module globals'(app) {
    const before = clone(fold(app, JOURNAL.slice(0, 3)));
    const first = app.reduce(clone(before), clone(JOURNAL[3]));
    fold(app, JOURNAL);
    fold(app, JOURNAL);
    return differ(first, app.reduce(clone(before), clone(JOURNAL[3])), 'one reduce before and after other folds');
  },
};

/** Every law `app` breaks, with why. */
export function broken(app) {
  return Object.entries(LAWS)
    .map(([law, check]) => [law, check(app)])
    .filter(([, why]) => why);
}

test('Expenses holds the five laws and the sixth', () => {
  assert.deepEqual(broken(expenses), []);
});

test('each law catches the reducer planted to break it', () => {
  const { reduce, init } = expenses;
  let calls = 0;
  const shared = init();
  const planted = {
    determinism: { reduce: (s, a) => Object.assign(reduce(s, a), { gone: new Set([a.id]) }) },
    'environment independence': { reduce: (s, a) => Object.assign(reduce(s, a), { at: Date.now() }) },
    'non-interference': { init: () => shared },
    idempotence: {
      reduce: (s, a) => (a?.op?.kind === 'share' ? Object.assign(s, { counted: s.counted + 1 }) : reduce(s, a)),
    },
    totality: { reduce: (s, a) => (a.op.kind === 'expense' ? reduce(s, a) : reduce(s, a)) },
    'no module globals': { reduce: (s, a) => Object.assign(reduce(s, a), { calls: ++calls }) },
  };
  for (const [law, override] of Object.entries(planted)) {
    const laws = broken({ ...expenses, ...override }).map(([l]) => l);
    assert.ok(laws.includes(law), `a reducer planted to break ${law} broke only ${JSON.stringify(laws)}`);
  }
});
