// The loop's laws (GRAPEVINE_DEMO §5, §8): bar 5, a withdrawn ask's carries
// corrected within one loop; and sealed means sealed in the trace, no
// message's words and no secret key in anything GRAPEVINE_DEBUG prints. The
// scene runs on a file of the app's own (store.mjs `file`), three people in
// one process. Each law's plant is one line of a copy of relay.mjs, as
// laws.test.mjs plants the fold, and a plant that no longer applies fails
// by name.

import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { pathToFileURL } from 'node:url';

import * as relay from './relay.mjs';
import { KINDS, TILE_SAW } from './scene.mjs';
import { file } from './store.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const OFFER = { when: 'Saturday morning', where: 'Waller and Pierce' };
const NAMES = ['Sam Reyes', 'Kim Okafor'];

/** §10's two groups on a file of their own, and Sam, Jo and Kim, each a `person` of `mod`, every trace kept. */
function town(mod) {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-relay-'));
  const path = join(dir, 'town.json');
  writeFileSync(path, JSON.stringify({ groups: { page: { name: '1400 Page', people: ['sam', 'jo'] }, waller: { name: 'Waller block', people: ['jo', 'kim'] } } }));
  const store = file(path);
  const traced = [];
  const trace = (what, fields) => traced.push(JSON.stringify({ what, ...fields }));
  const p = {};
  for (const id of ['sam', 'jo', 'kim']) p[id] = mod.person({ id, store, held: mod.heldIn(store, id), kinds: KINDS, state: mod.stateFile(join(dir, `${id}.json`)), trace });
  return { p, store, traced, dir };
}

/** Sam asks, Jo carries, Kim offers, both say yes; each message relayed by Jo's loop. */
async function scene({ p }) {
  const ask = await p.sam.ask({ kind: 'tool', fields: TILE_SAW, group: 'page', hops: 1, expires: Math.floor(Date.now() / 1000) + 3600 });
  const carry = await p.jo.carry({ ask, group: 'waller' });
  await p.kim.reply({ ask, group: 'waller', fields: OFFER });
  await p.jo.pass();
  const them = (await p.sam.view()).threads.find((t) => t.role === 'asker')?.them;
  await p.sam.yes({ ask, them, name: NAMES[0] });
  await p.jo.pass();
  const back = (await p.kim.view()).threads.find((t) => t.role === 'replier')?.them;
  await p.kim.yes({ ask, them: back, name: NAMES[1] });
  await p.jo.pass();
  return { ask, carry };
}

const LAWS = {
  /** Bar 5: Sam withdraws, and one pass of Jo's loop corrects the carry in Waller. */
  async 'bar 5: a withdrawn ask\'s carry is corrected within one loop'(mod) {
    const t = town(mod);
    const { ask, carry } = await scene(t);
    if (!(await t.p.kim.view()).asks.some((a) => a.id === ask)) return 'Kim never saw the ask';
    await t.p.sam.withdraw({ ask });
    await t.p.jo.pass();
    const corrected = (await t.store.log('waller')).ops.some((o) => o.corrects === carry && o.person === 'jo');
    const still = (await t.p.kim.view()).asks.length;
    return corrected && !still ? null : `one pass of Jo's loop after Sam withdrew: the carry ${corrected ? 'corrected' : 'not corrected'}, Kim sees ${still} asks`;
  },
  /** Sealed means sealed: nothing the loop or the person traces holds a message's words or a secret key. */
  async 'no plaintext or secret in any trace'(mod) {
    const t = town(mod);
    await scene(t);
    const secrets = ['sam', 'jo', 'kim'].flatMap((id) => {
      try {
        return Object.values(JSON.parse(readFileSync(join(t.dir, `${id}.json`), 'utf8')).keys);
      } catch {
        return [];
      }
    });
    if (secrets.length < 2) return `${secrets.length} secret keys kept, not one each for Sam and Kim`;
    const all = t.traced.join('\n');
    const leak = [...Object.values(OFFER), ...NAMES, ...secrets].find((s) => all.includes(s));
    return leak ? `a trace holds ${JSON.stringify(leak.slice(0, 20))}` : t.traced.length ? null : 'nothing was traced';
  },
};

/** Every law `mod` breaks, with why. */
async function broken(mod) {
  const out = [];
  for (const [law, check] of Object.entries(LAWS)) {
    try {
      const why = await check(mod);
      if (why) out.push([law, why]);
    } catch (e) {
      out.push([law, `it threw: ${e?.message ?? e}`]);
    }
  }
  return out;
}

/** relay.mjs with `from` changed to `to`, imported from a copy whose imports reach the real files. */
async function planted(from, to) {
  const src = readFileSync(join(HERE, 'relay.mjs'), 'utf8');
  assert.equal(src.split(from).length, 2, `the plant no longer applies to relay.mjs: ${from}`);
  const copy = join(mkdtempSync(join(tmpdir(), 'grapevine-plant-')), 'relay.mjs');
  writeFileSync(copy, src.replace(from, to).replaceAll("from './", `from '${pathToFileURL(HERE).href}`));
  return import(pathToFileURL(copy).href);
}

const PLANTS = {
  // the loop skips its step 3
  "bar 5: a withdrawn ask's carry is corrected within one loop": ["} else if (r.do === 'correct') {", "} else if (r.do === 'correct' && false) {"],
  // GRAPEVINE_DEBUG printing the opened body
  'no plaintext or secret in any trace': ["trace('folded', { person: id,", "trace('folded', { opened: v.threads, person: id,"],
};

test('the loop holds its laws', async () => {
  assert.deepEqual(await broken(relay), []);
});

test("each of the loop's laws catches the change planted to break it", async () => {
  assert.deepEqual(Object.keys(PLANTS).sort(), Object.keys(LAWS).sort(), 'every law has its plant');
  for (const [law, [from, to]] of Object.entries(PLANTS)) {
    const laws = (await broken(await planted(from, to))).map(([l]) => l);
    assert.ok(laws.includes(law), `the plant for ${law} broke only ${JSON.stringify(laws)}`);
  }
});

test("a group whose people can't be read keeps svrngs's answer, and says so", async () => {
  const api = http.createServer((req, res) => res.end(JSON.stringify({ groups: [{ id: 'page', name: '1400 Page', person: 'Jo' }] })));
  await new Promise((done) => api.listen(0, '127.0.0.1', done));
  const traced = [];
  const store = { people: () => Promise.reject(new Error('the record is away')) };
  try {
    const held = await relay.heldBy(`http://127.0.0.1:${api.address().port}`, store, (what, fields) => traced.push({ what, ...fields }))();
    assert.deepEqual(held, [{ id: 'page', name: '1400 Page', me: 'Jo' }]);
    assert.ok(traced.some((t) => t.group === 'page' && t.error === 'the record is away'), JSON.stringify(traced));
  } finally {
    api.close();
  }
});
