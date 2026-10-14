// five-laws (RING_APP_LIBRARY §4, as apps/expenses/laws.test.mjs holds them):
// Events holds determinism, environment independence, non-interference,
// idempotence, totality and the sixth, no module globals; and two of its own
// (EVENTS.md §1): a correction counts only from the person whose act it
// corrects, and nothing records "no". Each law is a check that returns why
// it broke, or null; the second test plants one broken fold per law and
// watches its check catch it.

import assert from 'node:assert/strict';
import { isDeepStrictEqual } from 'node:util';
import { test } from 'node:test';

import * as events from './events.js';

const [SAM, AMA, BO, CY, DI] = ['sam', 'ama', 'bo', 'cy', 'di'];
const E1 = 'e-dinner';

const act = (id, ts, by, op, corrects = null) => ({ id, ts, by, op, corrects });
/** Sam posts a dinner; Bo brings Maya; Cy changes her mind; Di tries to
 * change Ama's; Di says "no" three ways; Bo and his Maya came; Sam moves it;
 * Cy tries to post it as her own. */
const JOURNAL = [
  act('a1', 1000, SAM, { kind: 'event', event: E1, what: 'dinner', when: 'Friday 7', where: 'the house' }),
  act('a2', 1010, BO, { kind: 'coming', event: E1, guests: ['Maya'] }),
  act('a3', 1020, CY, { kind: 'coming', event: E1 }),
  act('a4', 1030, AMA, { kind: 'coming', event: E1, guests: [] }),
  act('a5', 1040, BO, { kind: 'coming', event: E1, guests: ['Maya'] }),
  act('a6', 1050, CY, null, 'a3'),
  act('a7', 1060, DI, null, 'a4'),
  act('a8', 1070, DI, { kind: 'not_coming', event: E1 }),
  act('a9', 1080, DI, { kind: 'coming', event: E1, coming: false }),
  act('a10', 1090, DI, { kind: 'came', event: E1, came: false }),
  act('a11', 5000, BO, { kind: 'came', event: E1 }),
  act('a12', 5010, BO, { kind: 'came', event: E1, guest: 'Maya' }),
  act('a13', 5020, AMA, { kind: 'came', event: E1, guest: 'Maya' }),
  act('a14', 900, SAM, { kind: 'event', event: E1, what: 'dinner', when: 'Friday 8', where: 'the house' }, 'a1'),
  act('a15', 1100, CY, { kind: 'event', event: E1, what: 'my dinner', when: '', where: '' }),
];
/** The acts that try to say "no", each in its own way. */
const NOES = new Set(['a8', 'a9', 'a10']);
/** Acts a duplicate of which must change nothing: the answers. */
const KEYED = new Set(['coming', 'came']);
/** Everything an event says, and nothing else. */
const SAYS = ['at', 'came', 'coming', 'guests_came', 'host', 'what', 'when', 'where'];

const clone = (v) => structuredClone(v);
const fold = (app, acts) => acts.reduce((s, a) => app.reduce(s, clone(a)), app.init());
/** Everything the app answers: the state and what it comes to, copied. */
function said(app, acts = JOURNAL) {
  const state = fold(app, acts);
  return clone({ state, answers: app.answers(state) });
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
  /** Folding one journal twice gives the same answers, and they are exactly JSON. */
  determinism(app) {
    const once = said(app);
    const exact = JSON.parse(JSON.stringify(once));
    return differ(once, said(app), 'two folds of one journal') ?? differ(once, exact, 'the answers and their JSON');
  },
  /** The same answers under another clock, random seed and locale. */
  'environment independence'(app) {
    return differ(said(app), elsewhere(() => said(app)), 'answers here and elsewhere');
  },
  /** One group's fold does not reach another's: no shared object between folds, and no act changed. */
  'non-interference'(app) {
    const alone = said(app, JOURNAL.slice(0, 6));
    fold(app, JOURNAL);
    const acts = clone(JOURNAL);
    app.answers(acts.reduce((s, a) => app.reduce(s, a), app.init()));
    return differ(alone, said(app, JOURNAL.slice(0, 6)), 'a fold alone and after another') ?? differ(acts, JOURNAL, 'the acts before and after a fold');
  },
  /** A second answer, the same person's, the same words, changes no one's answers. An
   * answer a correction names is skipped: the correction names one act, so its twin
   * stands until changed too, as `server.mjs` changes every live answer of a person's. */
  idempotence(app) {
    for (const [i, a] of JOURNAL.entries()) {
      if (!KEYED.has(a.op?.kind) || JOURNAL.some((c) => c.corrects === a.id)) continue;
      const doubled = [...JOURNAL.slice(0, i + 1), { ...a, id: `${a.id}-again`, ts: a.ts + 1 }, ...JOURNAL.slice(i + 1)];
      const why = differ(said(app).answers.events, said(app, doubled).answers.events, `the events with ${a.id} once and twice`);
      if (why) return why;
    }
    return null;
  },
  /** No act makes the fold throw, and what it says stays exactly JSON. */
  totality(app) {
    const ops = [null, 1, 'x', [], {}, { kind: 'event' }, { kind: 'toString' }, { kind: '__proto__' },
      { kind: 'event', event: '__proto__', what: 'x', when: '', where: '' }, { kind: 'event', event: 'e', what: 7, when: '', where: '' },
      { kind: 'coming', event: E1, guests: 'Maya' }, { kind: 'coming', event: E1, guests: [null] }, { kind: 'coming', event: E1, guests: ['__proto__'] },
      { kind: 'coming', event: 'constructor' }, { kind: 'came', event: E1, guest: {} }, { kind: 'came', event: ['a'] }, { kind: 'came' }];
    const bys = [null, '', 42, '__proto__', 'constructor', SAM];
    const acts = [null, 7, [], {}, { id: 5, ts: 'now', by: SAM, op: {} }, { id: '__proto__', ts: 1, by: SAM, op: JOURNAL[0].op },
      act('c1', 1, SAM, null), act('c2', 1, SAM, null, 'nothing'), act('c3', 1, SAM, null, ['a1']), act('c4', 1, '__proto__', null, 'a1')];
    for (const op of ops) for (const by of bys) acts.push(act(`x${acts.length}`, 1, by, op));
    let out;
    try {
      let state = fold(app, JOURNAL);
      for (const a of acts) state = app.reduce(state, a);
      out = { state, answers: app.answers(state) };
    } catch (e) {
      return `it threw: ${e?.message ?? e}`;
    }
    return differ(out, JSON.parse(JSON.stringify(out)), 'what it says and its JSON');
  },
  /** The fold depends only on the state and the act: the same pair gives the same answer, whatever ran before. */
  'no module globals'(app) {
    const before = clone(fold(app, JOURNAL.slice(0, 3)));
    const once = () => {
      const state = app.reduce(clone(before), clone(JOURNAL[3]));
      return clone({ state, answers: app.answers(state) });
    };
    const first = once();
    fold(app, JOURNAL);
    fold(app, JOURNAL);
    return differ(first, once(), 'one reduce before and after other folds');
  },
  /** A correction from anyone but the person whose act it changes changes no one's answers. */
  'only their own'(app) {
    const theirs = (a) => a.corrects == null || JOURNAL.find((b) => b.id === a.corrects)?.by === a.by;
    return differ(said(app).answers.events, said(app, JOURNAL.filter(theirs)).answers.events, "the events with others' corrections and without");
  },
  /** Nothing records "no": an act saying it changes no one's answers, and an event says only who answered. */
  'nothing records no'(app) {
    const { events: with_ } = said(app).answers;
    const odd = Object.values(with_).map((e) => Object.keys(e).sort()).find((k) => !isDeepStrictEqual(k, SAYS));
    if (odd) return `an event says ${odd.join(', ')}, not ${SAYS.join(', ')}`;
    return differ(with_, said(app, JOURNAL.filter((a) => !NOES.has(a.id))).answers.events, 'the events with a "no" and without');
  },
};

/** Every law `app` breaks, with why. */
export function broken(app) {
  return Object.entries(LAWS)
    .map(([law, check]) => [law, check(app)])
    .filter(([, why]) => why);
}

test('Events holds the five laws, the sixth and its own two', () => {
  assert.deepEqual(broken(events), []);
});

test('each law catches the fold planted to break it', () => {
  const { reduce, init, answers } = events;
  let calls = 0;
  const shared = init();
  const planted = {
    determinism: { reduce: (s, a) => Object.assign(reduce(s, a), { gone: new Set([a.id]) }) },
    'environment independence': { reduce: (s, a) => Object.assign(reduce(s, a), { at: Date.now() }) },
    'non-interference': { init: () => shared },
    idempotence: {
      reduce: (s, a) => (a?.op?.kind === 'came' && !a.op.guest ? Object.assign(s, { acts: { ...s.acts, [a.id]: { ...a, op: { ...a.op, guest: a.id } } }, order: [...s.order, a.id] }) : reduce(s, a)),
    },
    totality: { reduce: (s, a) => (a?.corrects != null && a.op == null ? Object.assign(s, { last: s.acts[a.corrects].by }) : reduce(s, a)) },
    'no module globals': { reduce: (s, a) => Object.assign(reduce(s, a), { calls: ++calls }) },
    // A correction counted from someone else: whoever corrects an act is taken as its person.
    'only their own': {
      reduce: (s, a) => {
        if (a?.corrects && s.acts?.[a.corrects]) s.acts[a.corrects].by = a.by;
        return reduce(s, a);
      },
    },
    // A "no" recorded: `not_coming` kept and said beside who is coming.
    'nothing records no': {
      reduce: (s, a) => (a?.op?.kind === 'not_coming' ? Object.assign(s, { declined: [...(s.declined ?? []), a.by] }) : reduce(s, a)),
      answers: (s) => {
        const out = answers(s);
        for (const e of Object.values(out.events)) e.declined = s.declined ?? [];
        return out;
      },
    },
  };
  for (const [law, override] of Object.entries(planted)) {
    const laws = broken({ ...events, ...override }).map(([l]) => l);
    assert.ok(laws.includes(law), `a fold planted to break ${law} broke only ${JSON.stringify(laws)}`);
  }
});
