// The laws of GRAPEVINE_DEMO §8, the numbers GRAPEVINE.md §7's bars, and two
// the fold holds besides (totality, and the same answer everywhere). Each
// law is a check that returns why it broke, or null; the second test plants
// each law's breaking change into a copy of the real source, one line, and
// watches its check catch it. A plant that no longer applies to the source
// fails by name, so a plant never goes stale unseen. Bar 5 is the loop's,
// with relay.mjs.

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { pathToFileURL } from 'node:url';
import { isDeepStrictEqual } from 'node:util';

import * as grapevine from './grapevine.js';
import { GROUPS, KINDS, NOW, TILE_SAW, scene, world } from './scene.mjs';
import * as sealing from './seal.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const OFFER = { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } };
/** Every string a value holds, at any depth. */
const words = (v) => (typeof v === 'string' ? [v] : v && typeof v === 'object' ? [...Object.keys(v), ...Object.values(v).flatMap(words)] : []);
const ops = (w) => Object.entries(w.logs).flatMap(([group, log]) => log.map((o) => ({ group, ...o })));
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
  /** Bar 1: a carry names its carrier, and holds neither the asker, the group it came from, nor that group's handle. */
  'bar 1: a carry never names the asker'(app) {
    const w = world(app, KINDS, GROUPS);
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    w.carry('jo', ask, 'waller');
    const upstream = w.logs.page.find((o) => o.id === ask).payload.handle;
    const never = ['sam', 'page', GROUPS.page.name, upstream];
    const carries = ops(w).filter((o) => o.payload?.type === 'carry');
    if (carries.length !== 1) return `${carries.length} carries written, not 1`;
    const [c] = carries;
    if (c.person !== 'jo') return `the carry is written by ${c.person}, not its carrier`;
    const keys = Object.keys(c.payload).sort();
    const want = ['app', 'expires', 'fields', 'handle', 'hops', 'kind', 'of', 'seal_to', 'type'];
    if (!isDeepStrictEqual(keys, want)) return `a carry says ${keys.join(', ')}, not ${want.join(', ')}`;
    const leak = words(c.payload).find((s) => never.includes(s));
    return leak ? `a carry holds ${JSON.stringify(leak)}` : null;
  },
  /** Bar 2: each pairing left out of `carriable` says its reason, and no other. */
  'bar 2: carriable refuses each by its reason'(app) {
    const w = world(app, KINDS, GROUPS);
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    const ended = w.ask('sam', 'page', 'tool', TILE_SAW, { expires: NOW - 1 });
    w.carry('jo', ask, 'waller');
    const why = (who, a, g) => w.view(who).carriable.not.find((n) => n.ask === a && n.group === g)?.why ?? 'carriable';
    const want = [
      ['kim', ask, 'yard', 'no_hops'],
      ['jo', ended, 'waller', 'expired'],
      ['jo', ask, 'yard', 'not_in_both'],
      ['jo', ask, 'page', 'already_held'],
      ['jo', ask, 'waller', 'already_held'],
      ['sam', ask, 'club', 'would_name_asker'],
    ];
    for (const [who, a, g, reason] of want) {
      const got = why(who, a, g);
      if (got !== reason) return `${who}'s ${a} to ${g} is ${got}, not ${reason}`;
    }
    return null;
  },
  /** Bar 3: no record in any group holds a word of a sealed message. */
  'bar 3: no record holds a plaintext'(app) {
    const w = world(app, KINDS, GROUPS);
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    w.carry('jo', ask, 'waller');
    w.reply('kim', ask, 'waller', OFFER);
    w.tick();
    const all = JSON.stringify(w.logs);
    for (const body of w.sealed) {
      const leak = [...words(body).filter((s) => s.length >= 8), JSON.stringify(body)].find((s) => all.includes(s));
      if (leak) return `a record holds ${JSON.stringify(leak)}`;
    }
    return w.sealed.length ? null : 'nothing was sealed';
  },
  /** Bar 6: one ask carried into a group twice folds as one ask with two carriers. */
  'bar 6: two carries are one ask'(app) {
    const w = world(app, KINDS, { page: { name: '1400 Page', people: ['sam', 'jo', 'lee'] }, waller: { name: 'Waller block', people: ['jo', 'lee', 'kim'] } });
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    // Both tap before either carry has reached the other's computer.
    const before = w.view('lee');
    w.carry('jo', ask, 'waller');
    w.carry('lee', ask, 'waller', before);
    const { asks } = w.view('kim');
    if (asks.length !== 1) return `Kim sees ${asks.length} asks, not 1`;
    return differ(asks[0].carriers, ['jo', 'lee'], 'its carriers and Jo and Lee') ?? (asks[0].in.length === 2 ? null : `it is held ${asks[0].in.length} times, not 2`);
  },
  /** A name is shown only once both ends have said yes. */
  'names wait for two yeses'(app) {
    const w = world(app, KINDS, GROUPS);
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    w.carry('jo', ask, 'waller');
    w.reply('kim', ask, 'waller', OFFER);
    w.tick();
    w.say('sam', ask, { say: 'yes', name: 'Sam Reyes' });
    w.tick();
    const named = (who) => w.view(who).threads.find((t) => t.ask === ask && t.role !== 'carrier');
    for (const who of ['sam', 'kim']) {
      const t = named(who);
      if (!t) return `${who} has no thread`;
      if (t.names !== null || words(t).some((s) => s.includes('Reyes'))) return `${who}'s thread names someone after one yes`;
    }
    w.say('kim', ask, { say: 'yes', name: 'Kim Okafor' });
    w.tick();
    return differ(named('sam').names, { me: 'Sam Reyes', them: 'Kim Okafor' }, "Sam's names after two yeses and the two") ??
      differ(named('kim').names, { me: 'Kim Okafor', them: 'Sam Reyes' }, "Kim's names after two yeses and the two");
  },
  /** A kind's `notice` is given only what the person wrote in groups they stand in. */
  'notice sees only its own groups'(app) {
    const w = world(app, KINDS, GROUPS);
    w.append('club', 'lee', app.listingAct('tool', { what: 'tile saw' }));
    w.append('waller', 'kim', app.listingAct('tool', { what: 'ladder' }));
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    w.carry('jo', ask, 'waller');
    const given = [];
    const spy = { ...KINDS, tool: { ...KINDS.tool, notice: (a, mine) => (given.push(...mine.records), KINDS.tool.notice(a, mine)) } };
    const gs = Object.entries(GROUPS).map(([id, g]) => ({ id, name: g.name, me: g.people.includes('kim') ? 'kim' : null, ops: w.logs[id] }));
    const { asks } = app.fold({ groups: gs, state: { keys: {}, routes: [] }, kinds: spy, now: NOW });
    const hers = Object.entries(GROUPS).filter(([, g]) => g.people.includes('kim')).flatMap(([id]) => w.logs[id].filter((o) => o.person === 'kim').map((o) => o.payload));
    const foreign = given.find((r) => !hers.some((h) => isDeepStrictEqual(h, r)));
    if (foreign) return `notice was given ${JSON.stringify(foreign)}, from a group Kim is not in`;
    return asks[0]?.notice === false ? null : 'Kim is told she might have a tile saw she never listed';
  },
  /** A correction counts only from the author of what it corrects (§13.4). */
  'a withdrawal is only its asker\'s'(app) {
    const w = world(app, KINDS, GROUPS);
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    w.carry('jo', ask, 'waller');
    w.correct('page', 'jo', ask);
    w.tick();
    for (const who of ['sam', 'jo', 'kim']) if (!w.view(who).asks.some((a) => a.id === ask)) return `Jo's correction took Sam's ask from ${who}`;
    w.withdraw('sam', ask);
    w.tick();
    return w.view('kim').asks.length ? 'Sam\'s own withdrawal left the carry standing' : null;
  },
  /** No act makes the fold throw, and what it says is exactly JSON. */
  totality(app) {
    const w = world(app, KINDS, GROUPS);
    const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
    const good = w.logs.page[0].payload;
    const junk = [null, 1, 'x', [], {}, { app: 'grapevine' }, { app: 'grapevine', type: 'toString' }, { ...good, fields: null }, { ...good, fields: [] },
      { ...good, fields: { what: 7, when: '' } }, { ...good, kind: '__proto__' }, { ...good, hops: 1.5 }, { ...good, hops: -1 }, { ...good, seal_to: 'x' },
      { ...good, type: 'carry', of: null }, { ...good, type: 'carry', of: ask, name: undefined }, { app: 'grapevine', type: 'msg', handle: good.handle, dir: 'up', from: good.seal_to, to: good.seal_to, nonce: 'n', box: 'b' },
      { app: 'grapevine', type: 'msg', handle: good.handle, dir: 'sideways', from: 1, to: null, nonce: [], box: {} }];
    for (const payload of junk) w.logs.page.push({ id: `j${w.logs.page.length}`, ts_unix: 5, person: 'jo', payload });
    w.logs.page.push({ id: 'k1', ts_unix: 5, person: 'jo', corrects: ['x'] }, { id: '__proto__', ts_unix: 5, person: 'jo', payload: good }, { id: 'k2', ts_unix: 'now', person: null, payload: good }, null, 7);
    let out;
    try {
      out = w.view('jo');
      out = { ...out, odd: app.fold({ groups: [{ id: 'page', name: 'P', me: 'jo', ops: 'none' }, { id: 'x', me: 'jo', unread: 'gone' }], state: { keys: { x: 1 }, routes: [null] }, kinds: KINDS, now: NOW }) };
    } catch (e) {
      return `it threw: ${e?.message ?? e}`;
    }
    return differ(out, JSON.parse(JSON.stringify(out)), 'what it says and its JSON');
  },
  /** The same answers twice, and under another clock, random seed and locale: the time is the caller's. */
  'the same everywhere'(app) {
    const { w } = scene(app, KINDS);
    const once = w.view('kim');
    return differ(once, w.view('kim'), 'two folds of one record') ?? differ(once, elsewhere(() => w.view('kim')), 'answers here and elsewhere');
  },
};

/** Every law `app` breaks, with why; a law that throws is broken by its throw. */
export function broken(app) {
  return Object.entries(LAWS)
    .map(([law, check]) => {
      try {
        return [law, check(app)];
      } catch (e) {
        return [law, `it threw: ${e?.message ?? e}`];
      }
    })
    .filter(([, why]) => why);
}

/** `file` with `from` changed to `to`, imported from a copy whose imports reach the real files. */
async function planted(file, from, to) {
  const src = readFileSync(join(HERE, file), 'utf8');
  assert.equal(src.split(from).length, 2, `the plant no longer applies to ${file}: ${from}`);
  const copy = join(mkdtempSync(join(tmpdir(), 'grapevine-plant-')), file.replace(/\.m?js$/, '.mjs'));
  writeFileSync(copy, src.replace(from, to).replaceAll("from './", `from '${pathToFileURL(HERE).href}`));
  return import(pathToFileURL(copy).href);
}

const APP = { ...grapevine, ...sealing };

/** Each law's plant: the one change to the source that breaks it (§8's plants). */
const PLANTS = {
  // the carry spreads the ask
  'bar 1: a carry never names the asker': ['grapevine.js', 'payload: { app: APP, type: CARRY, kind: ask.kind,', 'payload: { ...ask, app: APP, type: CARRY, kind: ask.kind,'],
  // hops not decremented
  'bar 2: carriable refuses each by its reason': ['grapevine.js', 'carriable.can.push({ ask: a.id, group: g.id, name, hops: up.hops - 1,', 'carriable.can.push({ ask: a.id, group: g.id, name, hops: up.hops,'],
  // `seal` returns the body
  'bar 3: no record holds a plaintext': ['seal.mjs', 'return { nonce: b64(nonce), box: b64(nacl.box(text, nonce, them, mine)) };', 'return { nonce: b64(nonce), box: JSON.stringify(body) };'],
  // folded by act id, not `of`
  'bar 6: two carries are one ask': ['grapevine.js', 'type: op.type, of: op.type === ASK ? id : op.of,', 'type: op.type, of: id,'],
  // names shown on the first yes
  'names wait for two yeses': ['grapevine.js', 'if (t.yes.me !== undefined && t.yes.them !== undefined)', 'if (t.yes.me !== undefined || t.yes.them !== undefined)'],
  // `notice` given every log
  'notice sees only its own groups': ['grapevine.js', 'const mine = { records: read.flatMap((r) => r.records) };', 'const mine = { records: groups.flatMap((g) => (actsOf(g.ops) ?? []).map((a) => a.op).filter((op) => isObject(op) && op.app !== APP)) };'],
  // any person's correction counted
  "a withdrawal is only its asker's": ['grapevine.js', 'if (was?.by === c.by) voided[c.corrects] = true;', 'if (was) voided[c.corrects] = true;'],
  // a field read before it is known to be an object
  totality: ['grapevine.js', "  if (!isObject(values)) return ['nothing filled in'];\n", ''],
  // the fold reads the clock
  'the same everywhere': ['grapevine.js', 'const live = cs.filter((c) => c.expires > now);', 'const live = cs.filter((c) => c.expires > Date.now() / 1000);'],
};

test('grapevine holds every law', () => {
  assert.deepEqual(broken(APP), []);
});

test('each law catches the change planted to break it', async () => {
  assert.deepEqual(Object.keys(PLANTS).sort(), Object.keys(LAWS).sort(), 'every law has its plant');
  for (const [law, [file, from, to]] of Object.entries(PLANTS)) {
    const mod = await planted(file, from, to);
    const laws = broken({ ...APP, ...mod }).map(([l]) => l);
    assert.ok(laws.includes(law), `the plant for ${law} broke only ${JSON.stringify(laws)}`);
  }
});
