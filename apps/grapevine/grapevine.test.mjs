// Grapevine's ground and its answers (GRAPEVINE_DEMO §2-§4, §7), one rule a
// test. The laws of §8 are laws.test.mjs's.

import assert from 'node:assert/strict';
import { cpSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { pins, unpinned } from '../pinned.mjs';

import * as grapevine from './grapevine.js';
import { LISTINGS, MAX_FIELDS, MAX_TEXT, askAct, formGaps, kindsOf, msgAct } from './grapevine.js';
import askKind from './kinds/ask.js';
import tool from './kinds/tool.js';
import { GROUPS, KINDS, NOW, TILE_SAW, scene, world } from './scene.mjs';
import * as sealing from './seal.mjs';
import { ACT_MOST, BODY_MOST, keyPair, open, seal } from './seal.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const VENDOR = join(HERE, 'vendor');

test('tweetnacl is the copy npm published, every file as pinned', () => {
  assert.equal(pins(VENDOR).length, 3, 'vendor/README.md pins nacl-fast.js, package.json and LICENSE');
  assert.deepEqual(unpinned(VENDOR), []);
});

test('one byte changed in the vendored nacl-fast.js is caught', () => {
  const dir = mkdtempSync(join(tmpdir(), 'grapevine-vendor-'));
  cpSync(VENDOR, dir, { recursive: true });
  const f = join(dir, 'tweetnacl/nacl-fast.js');
  const bytes = readFileSync(f);
  bytes[1000] ^= 1;
  writeFileSync(f, bytes);
  assert.deepEqual(unpinned(VENDOR, dir).map((s) => s.split(' ')[0]), ['tweetnacl/nacl-fast.js']);
});

/** Every string `body` holds, at any depth. */
const words = (v) => (typeof v === 'string' ? [v] : v && typeof v === 'object' ? Object.values(v).flatMap(words) : []);

test('a sealed message opens at either end, and nowhere else', () => {
  const [sam, kim, jo] = [keyPair(), keyPair(), keyPair()];
  const body = { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } };
  const sealed = seal(body, sam.public, kim.secret);
  assert.deepEqual(open(sealed, kim.public, sam.secret), body);
  assert.deepEqual(open(sealed, sam.public, kim.secret), body, 'the sender reads back what they sealed');
  assert.equal(open(sealed, kim.public, jo.secret), null, 'a carrier holding every byte cannot open it');
  const box = Buffer.from(sealed.box, 'base64');
  box[3] ^= 1;
  assert.equal(open({ ...sealed, box: box.toString('base64') }, kim.public, sam.secret), null, 'one byte changed does not open');
});

test('a sealed message holds none of its words', () => {
  const [sam, kim] = [keyPair(), keyPair()];
  const body = { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } };
  const wire = JSON.stringify(seal(body, sam.public, kim.secret));
  for (const w of [...words(body), JSON.stringify(body), Buffer.from(JSON.stringify(body)).toString('base64')]) {
    assert.ok(!wire.includes(w), `the sealed message holds ${JSON.stringify(w)}`);
  }
});

test("the most an act may be is svrngs-core's, and the largest sealed message is under it", () => {
  const core = readFileSync(join(HERE, '../../crates/svrngs-core/src/payload.rs'), 'utf8');
  const [, a, b] = core.match(/pub const MAX_PAYLOAD_BYTES: usize = (\d+) \* (\d+);/) ?? [];
  assert.equal(ACT_MOST, Number(a) * Number(b), 'ACT_MOST is payload.rs\'s MAX_PAYLOAD_BYTES');
  const [sam, kim] = [keyPair(), keyPair()];
  const most = { say: 'text', text: 'x'.repeat(BODY_MOST - JSON.stringify({ say: 'text', text: '' }).length) };
  assert.equal(JSON.stringify(most).length, BODY_MOST);
  const msg = { app: 'grapevine', type: 'msg', handle: 'A'.repeat(22) + '==', dir: 'down', from: kim.public, to: sam.public, ...seal(most, sam.public, kim.secret) };
  assert.ok(Buffer.byteLength(JSON.stringify(msg)) <= ACT_MOST, `a msg sealing ${BODY_MOST} bytes is ${Buffer.byteLength(JSON.stringify(msg))}`);
  assert.throws(() => seal({ ...most, text: `${most.text}x` }, sam.public, kim.secret), new RegExp(`at most ${BODY_MOST} bytes sealed, and this one is ${BODY_MOST + 1}`));
});

const APP = { ...grapevine, ...sealing };
const OFFER = { say: 'offer', fields: { when: 'Saturday morning', where: 'Waller and Pierce' } };

test('the three acts are §3\'s exactly, every one grapevine\'s', () => {
  const { w, ask } = scene(APP, KINDS);
  const shapes = {
    ask: ['app', 'expires', 'fields', 'handle', 'hops', 'kind', 'name', 'seal_to', 'type'],
    carry: ['app', 'expires', 'fields', 'handle', 'hops', 'kind', 'of', 'seal_to', 'type'],
    msg: ['app', 'box', 'dir', 'from', 'handle', 'nonce', 'to', 'type'],
  };
  const ours = Object.values(w.logs).flat().filter((o) => o.payload?.app === 'grapevine');
  assert.deepEqual([...new Set(ours.map((o) => o.payload.type))].sort(), ['ask', 'carry', 'msg']);
  for (const o of ours) assert.deepEqual(Object.keys(o.payload).sort(), shapes[o.payload.type], o.payload.type);
  assert.equal(w.logs.waller.find((o) => o.payload?.type === 'carry').payload.of, ask);
  assert.deepEqual(w.view('kim').gaps, []);
});

test('Jo sees a sealed message pass, its size and time, and nothing in it', () => {
  const { w, ask } = scene(APP, KINDS);
  const jo = w.view('jo');
  const [t] = jo.threads;
  assert.deepEqual({ role: t.role, ask: t.ask, from: t.from, to: t.to }, { role: 'carrier', ask, from: '1400 Page', to: 'Waller block' });
  assert.deepEqual(t.passed.map((p) => p.dir), ['up', 'down', 'up']);
  assert.ok(t.passed.every((p) => Object.keys(p).sort().join() === 'at,dir,size' && p.size > 16));
  assert.ok(!JSON.stringify(jo).includes('Waller and Pierce') && !JSON.stringify(jo).includes('Okafor'));
});

test('a message is relayed once, and down only to a replier whose message came up that way', () => {
  const w = world(APP, KINDS, GROUPS);
  const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
  w.carry('jo', ask, 'waller');
  w.reply('kim', ask, 'waller', OFFER);
  assert.deepEqual(w.view('jo').relay.map((r) => [r.do, r.group, r.payload.dir]), [['forward', 'page', 'up']]);
  w.tick();
  assert.deepEqual(w.view('jo').relay, [], 'once forwarded, not again');
  // A message down to a key that never wrote up through Waller stays in 1400 Page.
  const stranger = keyPair();
  const t = w.view('sam').threads[0];
  w.append('page', 'sam', grapevine.msgAct({ ...t.send, to: stranger.public }, seal({ say: 'text', text: 'hello?' }, stranger.public, w.states.sam.keys[t.send.from])));
  assert.deepEqual(w.view('jo').relay, []);
});

test('a carry of a carry cites the first ask, and a reply walks back two rings', () => {
  const w = world(APP, KINDS, { a: { name: 'A', people: ['sam', 'jo'] }, b: { name: 'B', people: ['jo', 'lee'] }, c: { name: 'C', people: ['lee', 'kim'] } });
  const ask = w.ask('sam', 'a', 'tool', TILE_SAW, { hops: 2 });
  w.carry('jo', ask, 'b');
  w.carry('lee', ask, 'c');
  assert.equal(w.logs.c[0].payload.of, ask);
  assert.equal(w.logs.c[0].payload.hops, 0);
  assert.deepEqual(w.view('kim').asks.map((a) => [a.id, a.carriers]), [[ask, ['lee']]]);
  w.reply('kim', ask, 'c', OFFER);
  w.tick();
  w.tick();
  assert.deepEqual(w.view('sam').threads[0].messages.map((m) => [m.say, m.mine, m.fields?.where]), [['offer', false, 'Waller and Pierce']]);
  w.say('sam', ask, { say: 'text', text: 'Saturday works' });
  w.tick();
  w.tick();
  assert.deepEqual(w.view('kim').threads[0].messages.map((m) => [m.say, m.mine]), [['offer', true], ['text', false]]);
});

test('an ask past its end is gone with no act, and saying so is the decider\'s', () => {
  const w = world(APP, KINDS, GROUPS);
  const ask = w.ask('sam', 'page', 'tool', TILE_SAW, { expires: NOW + 60 });
  assert.equal(w.view('jo').asks.length, 1);
  const later = w.view('jo', NOW + 60);
  assert.deepEqual(later.asks, []);
  assert.deepEqual(later.carriable.not.find((n) => n.group === 'waller'), { ask, group: 'waller', why: 'expired', words: 'it has ended' });
  assert.equal(Object.values(w.logs).flat().length, 1, 'no act was written');
});

test("a withdrawal takes the ask away, and the carrier's relay corrects the carry", () => {
  const w = world(APP, KINDS, GROUPS);
  const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
  const carry = w.carry('jo', ask, 'waller');
  w.withdraw('sam', ask);
  assert.deepEqual(w.view('sam').asks, []);
  assert.deepEqual(w.view('kim').asks.map((a) => a.id), [ask], 'the carry stands until Jo corrects it');
  assert.deepEqual(w.view('jo').relay, [{ do: 'correct', group: 'waller', corrects: carry }]);
  w.tick();
  assert.deepEqual(w.view('kim').asks, []);
  assert.deepEqual(w.view('jo').relay, []);
});

test("someone else's correction, an ask that shows its asker, and another app's acts are each said or skipped", () => {
  const w = world(APP, KINDS, GROUPS);
  const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
  w.correct('page', 'jo', ask);
  const shown = { ...w.logs.page[0].payload, name: 'shown' };
  w.append('page', 'sam', shown);
  w.append('page', 'jo', { app: 'events', kind: 'event', event: 'e1', what: 'dinner', when: '', where: '' });
  const { asks, gaps } = w.view('jo');
  assert.deepEqual(asks.map((a) => a.id), [ask]);
  assert.deepEqual(gaps.map((g) => g.code), ['not_theirs', 'name_shown']);
  assert.match(gaps[0].message, /only sam can/);
});

test('the person\'s own records reach notice; Kim, who listed a tile saw, is told', () => {
  const { w } = scene(APP, KINDS);
  assert.equal(w.view('kim').asks[0].notice, true);
  assert.equal(w.view('jo').asks[0].notice, false);
  assert.equal(w.view('sam').asks[0].notice, false, 'never told of their own ask');
  assert.equal(w.view('kim').asks[0].show, 'tile saw, Saturday');
});

test('the two kinds keep the interface, and one that breaks it is refused, saying how', () => {
  assert.deepEqual(Object.keys(kindsOf([askKind, tool])), ['ask', 'tool']);
  const msg = (k) => {
    try {
      kindsOf([k]);
    } catch (e) {
      return e.message;
    }
    return 'accepted';
  };
  assert.match(msg({ ...tool, route: () => 'waller' }), /^the tool kind sets route; a kind writes no routing, scope, handle, sealing, carrying or name/);
  assert.match(msg({ ...tool, seal: () => null, handle: 'x' }), /sets seal, handle; a kind writes no/);
  assert.match(msg({ ...tool, ask: { fields: { what: 'html' } } }), /its ask field what is not one of text, date, number, choice/);
  const seven = Object.fromEntries([...'abcdefg'].map((c) => [c, 'text']));
  assert.match(msg({ ...tool, reply: { fields: seven } }), new RegExp(`its reply has 7 fields, and a kind has 1 to ${MAX_FIELDS}`));
  assert.match(msg({ ...tool, name: 'shown' }), /a name other than withheld/);
  assert.match(msg({ ...tool, show: undefined }), /has no `show`/);
  assert.match(msg({ ...tool, ask: { fields: { size: { choice: ['small', 'large'] }, on: 'date', many: 'number' } } }), /^accepted$/);
  assert.throws(() => kindsOf([tool, { ...tool }]), /the tool kind is a second kind of that word/);
  // kinds/README.md says the limits in words; they are grapevine.js's.
  const readme = readFileSync(join(HERE, 'kinds/README.md'), 'utf8').replace(/\s+/g, ' ');
  assert.ok(readme.includes(`1 to ${MAX_FIELDS} fields`) && readme.includes(`up to ${MAX_TEXT.toLocaleString('en-US')} characters`), 'kinds/README.md says the limits grapevine.js keeps');
  assert.ok(readme.indexOf('What a kind cannot touch') < readme.indexOf('## '), 'kinds/README.md says what a kind cannot touch first');
});

test('every kind in kinds/ imports nothing: the routing, handles, sealing and carrying it could reach are the app\'s', () => {
  const kinds = readdirSync(join(HERE, 'kinds')).filter((f) => f.endsWith('.js'));
  assert.ok(kinds.length >= 2, `kinds/ holds ${kinds.length} kinds`);
  for (const f of kinds) {
    const reach = readFileSync(join(HERE, 'kinds', f), 'utf8').split('\n').map((l, n) => [n + 1, l]).filter(([, l]) => /^\s*import\b|\bimport\s*\(|\brequire\s*\(/.test(l));
    assert.deepEqual(reach.map(([n, l]) => `kinds/${f}:${n}: ${l.trim()}`), [], `kinds/${f} imports: a kind is a plain object, and writes no routing, scope, handle, sealing, carrying or name (kinds/README.md)`);
  }
});

test("a listing is a plain record in its owner's own group, never an ask, so never carried", () => {
  const { w } = scene(APP, KINDS);
  const listing = w.logs.waller.find((o) => o.payload.app === LISTINGS);
  // Quoted: the id is `a1`, which a random base64 key or handle holds by chance.
  const named = (x) => JSON.stringify(x).includes(JSON.stringify(listing.id));
  assert.deepEqual(listing.payload, { app: LISTINGS, kind: 'tool', listing: { what: 'tile saw' } });
  for (const who of ['sam', 'jo', 'kim']) {
    const v = w.view(who);
    assert.ok(!named(v.asks) && !named(v.carriable), who);
    assert.deepEqual(v.gaps, [], `${who} reads the listing as no gap`);
  }
  assert.ok(!Object.values(w.logs).flat().some((o) => o.payload?.type === 'carry' && o.payload.fields?.what === 'tile saw' && o.payload.of === listing.id));
});

test('a longer text is refused at the form, with the limit said, and never cut', () => {
  const long = 'x'.repeat(MAX_TEXT + 1);
  assert.deepEqual(formGaps(tool, 'reply', { when: long, where: 'Waller and Pierce' }), [`when is ${MAX_TEXT + 1} characters, and the most is ${MAX_TEXT}`]);
  assert.deepEqual(formGaps(tool, 'reply', { when: long.slice(1), where: 'Waller and Pierce' }), []);
  assert.deepEqual(formGaps(tool, 'text', long), [`your message is ${MAX_TEXT + 1} characters, and the most is ${MAX_TEXT}`]);
  assert.deepEqual(formGaps(tool, 'text', '  '), ['your message is empty']);
  assert.deepEqual(formGaps(tool, 'listing', { what: '' }), ['what is empty']);
  assert.deepEqual(formGaps(askKind, 'ask', { question: 'Anyone have a ladder?', extra: 1 }), ['extra is not asked for here']);
});

test("the largest a kind allows stays under an act's most: its sealed message and its ask", () => {
  // Each character a control one, six bytes in JSON, and the longest field names.
  const names = Array.from({ length: MAX_FIELDS }, (_, i) => String.fromCharCode(97 + i).repeat(40));
  const widest = Object.fromEntries(names.map((n) => [n, '\u0001'.repeat(MAX_TEXT)]));
  const [kind] = Object.values(kindsOf([{ ...tool, ask: { fields: Object.fromEntries(names.map((n) => [n, 'text'])) }, reply: { fields: Object.fromEntries(names.map((n) => [n, 'text'])) } }]));
  const body = { say: 'offer', fields: widest, text: '\u0001'.repeat(MAX_TEXT) };
  assert.deepEqual(formGaps(kind, 'reply', widest), []);
  const bytes = Buffer.byteLength(JSON.stringify(body));
  assert.ok(bytes <= BODY_MOST, `the largest body a kind allows is ${bytes} bytes, past BODY_MOST ${BODY_MOST}`);
  const [sam, kim] = [keyPair(), keyPair()];
  const msg = msgAct({ handle: 'A'.repeat(22) + '==', dir: 'up', from: kim.public, to: sam.public }, seal(body, sam.public, kim.secret));
  assert.ok(Buffer.byteLength(JSON.stringify(msg)) <= ACT_MOST);
  const ask = askAct({ kind: 'tool', fields: widest, hops: 3, expires: NOW, seal_to: sam.public, handle: 'A'.repeat(22) + '==' });
  assert.ok(Buffer.byteLength(JSON.stringify(ask)) <= ACT_MOST, `the largest ask is ${Buffer.byteLength(JSON.stringify(ask))} bytes`);
});

test('which computer wrote it is kept: `via` is its person when it wrote for another, else null; every answer unchanged', () => {
  // ralph/FINDING_ON_BEHALF_OF.md's scene: Sam's ask written through Jo's
  // computer, Jo's carry through Kim's; then Sam's withdrawal through Jo's.
  const w = world(APP, KINDS, GROUPS);
  const ask = w.ask('sam', 'page', 'tool', TILE_SAW);
  Object.assign(w.logs.page.at(-1), { person: 'jo', on_behalf_of: 'sam' });
  const carry = w.carry('jo', ask, 'waller');
  Object.assign(w.logs.waller.at(-1), { person: 'kim', on_behalf_of: 'jo' });
  const untold = (who) => {
    const kept = structuredClone(w.logs);
    for (const o of Object.values(w.logs).flat()) Object.assign(o, { person: o.on_behalf_of ?? o.person, on_behalf_of: undefined });
    const v = w.view(who);
    Object.assign(w.logs, kept);
    return v;
  };
  const jo = w.view('jo');
  assert.deepEqual(jo.asks[0].in.map((c) => [c.act, c.by, c.via]), [[ask, 'sam', 'jo'], [carry, 'jo', 'kim']]);
  assert.deepEqual(jo.asks[0].carriers, ['jo']);
  const strip = (v) => ({ ...v, asks: v.asks.map((a) => ({ ...a, in: a.in.map(({ via, ...c }) => c) })) });
  for (const who of ['sam', 'jo', 'kim']) assert.deepEqual(strip(w.view(who)), strip(untold(who)), `${who}'s view: via changes no answer`);
  assert.deepEqual(untold('jo').asks[0].in.map((c) => c.via), [null, null], 'written by the person\'s own computer, via is null');
  const withdrawal = w.withdraw('sam', ask);
  Object.assign(w.logs.page.at(-1), { person: 'jo', on_behalf_of: 'sam' });
  assert.deepEqual(w.view('sam').asks, [], "Sam's withdrawal through Jo's computer counts, as Sam's");
  assert.deepEqual(w.view('jo').relay, [{ do: 'correct', group: 'waller', corrects: carry }]);
  assert.deepEqual(grapevine.actsOf(w.logs.page).map((a) => [a.id, a.by, a.via]), [[ask, 'sam', 'jo'], [withdrawal, 'sam', 'jo']]);
});

test("every gap a page may list, and every refusal it may say, is in the page's words (fence 9)", () => {
  // The page lists the fold's gaps under "couldn't be read" and says the validator's first gap when it refuses a write.
  const NOT_SAID = /\b(keys?|members?|records?|acts?|hash(es)?|handles?|hops?|seal)\b/i;
  const w = world(APP, KINDS, GROUPS);
  w.ask('sam', 'page', 'tool', TILE_SAW);
  const good = w.logs.page[0].payload;
  const other = keyPair();
  const junk = [null, 'x', { app: 'grapevine', type: 'toString' }, { ...good, fields: null }, { ...good, kind: 'rooms' }, { ...good, hops: -1 }, { ...good, seal_to: 'x' },
    { ...good, handle: 'x' }, { ...good, type: 'carry', of: null }, { ...good, expires: 'soon' },
    { app: 'grapevine', type: 'msg', handle: good.handle, dir: 'sideways', from: 1, to: null, nonce: [], box: {} },
    // A message to Sam's ask that does not open with what Sam keeps.
    msgAct({ handle: good.handle, dir: 'up', from: other.public, to: good.seal_to }, seal(OFFER, other.public, other.secret))];
  for (const payload of junk) w.logs.page.push({ id: `j${w.logs.page.length}`, ts_unix: 5, person: 'jo', payload });
  w.logs.page.push({ id: 'k1', ts_unix: 5, person: 'jo', corrects: ['x'] }, { id: 'k2', ts_unix: 'now', person: null, payload: good }, { id: 'k3', ts_unix: 6, person: 'jo', corrects: 'a1' });
  const odd = APP.fold({ groups: [{ id: 'page', name: '1400 Page', me: 'sam', ops: 'none' }], state: { keys: { x: 1 }, routes: [null] }, kinds: KINDS, now: NOW });
  const said = [...w.view('sam').gaps, ...w.view('jo').gaps, ...odd.gaps].map((g) => g.message);
  for (const p of junk) said.push(...grapevine.validate(p, KINDS).map((g) => g.message));
  for (const code of ['unopened', 'bad_state', 'not_theirs']) assert.ok([...w.view('sam').gaps, ...odd.gaps].some((g) => g.code === code), `the gaps include ${code}`);
  assert.deepEqual([...new Set(said.filter((m) => NOT_SAID.test(m)))], [], 'a gap or refusal says a word a reader must look up');
});

test('the README says the shortcut first: who sees that a sealed message went by, its size, its handle and its time', () => {
  const first = readFileSync(join(HERE, 'README.md'), 'utf8').split(/\n\n+/)[1].replace(/\s+/g, ' ');
  for (const w of ['sealed message went by', 'its size', 'its handle', 'its time', 'never see what it says']) assert.ok(first.includes(w), `the README's first paragraph says ${JSON.stringify(w)}`);
});
