// The shelf, both pages' (page-shelf (a); crates/svrngs-node/src/home/ui/
// shelf.js), with no browser: its drawing held to question.js `table`,
// which it moves, byte for byte, written out as parts.test.mjs writes
// either; and SHELF's steps, each effect read as data, the io run against
// a stand-in for the page's `bring`.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { SHELF, drawn } = await import('../../../crates/svrngs-node/src/home/ui/shelf.js?v=__VERSION__');
import { v1 } from './before.mjs';

// A document of plain objects, enough for v1's `el` (parts.test.mjs).
class Node {
  constructor(tag) {
    Object.defineProperty(this, 'tag', { value: tag });
    Object.defineProperty(this, 'kids', { value: [] });
  }
  append(...kids) {
    this.kids.push(...kids);
  }
  replaceChildren(...kids) {
    this.kids.splice(0, this.kids.length, ...kids);
  }
}
globalThis.document = { createElement: (tag) => new Node(tag) };
const attrs = (o) =>
  Object.entries(o)
    .filter(([k, v]) => !['children', 'key', 'ref'].includes(k) && !/^on/i.test(k) && v != null && v !== false)
    .map(([k, v]) => [k.toLowerCase(), v])
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([k, v]) => (v === true ? ` ${k}` : ` ${k}="${v}"`))
    .join('');
const out = (v) => {
  if (v == null || typeof v === 'boolean') return '';
  if (Array.isArray(v)) return v.map(out).join('');
  if (typeof v !== 'object') return String(v);
  if (v instanceof Node) return `<${v.tag}${attrs({ ...v })}>${out(v.kids)}</${v.tag}>`;
  if (typeof v.type === 'function') return out(v.type(v.props));
  return `<${v.type}${attrs(v.props)}>${out(v.props.children)}</${v.type}>`;
};

const SHELVED = [
  { name: 'Money', words: 'who owes whom, and for what.' },
  { name: 'Events', words: 'the get-togethers people post.' },
  { name: 'House-rules', words: 'the rules your house agrees.' },
];
const house = { id: 'g1', name: 'house' };
const money = { group: 'g1', kind: 'Money' };
const FIRST = { at: { bring: 'idle' }, ticked: [], brought: [], open: false, said: '' };
const sent = [];
const send = (m) => sent.push(m);
const step = SHELF(async () => []).step;

test('drawn as question.js `table` draws it: offered, one on the table, all on it, a group unnamed', () => {
  for (const [g, lent] of [[house, []], [house, [money]], [house, SHELVED.map((a) => ({ group: 'g1', kind: a.name }))], [{ id: 'g2' }, [money]]]) {
    assert.equal(out(drawn(FIRST, send, { g, lent, shelf: SHELVED })), out(v1.table(g, { lent, shelf: SHELVED }, () => {})), JSON.stringify([g, lent]));
  }
});

test('folded as "Bring more to <group>" for someone who just joined, as `table` folds it', () => {
  const v2 = out(drawn(FIRST, send, { g: house, lent: [], shelf: SHELVED, joined: true }));
  assert.equal(v2, out(v1.table(house, { lent: [], shelf: SHELVED }, () => {}, true)));
  assert.ok(v2.startsWith('<details><summary>Bring more to house</summary><section>'), v2);
});

test('a tick is kept, and the button brings the ticked ones still offered', () => {
  const ticked = [['Money', true], ['House-rules', true], ['Money', false]].reduce((s, [app, on]) => step(s, { type: 'tick', app, on }).state, FIRST);
  assert.deepEqual(ticked.ticked, ['House-rules']);
  const drawing = out(drawn(ticked, send, { g: house, lent: [], shelf: SHELVED }));
  assert.ok(drawing.includes('<label><input checked type="checkbox"></input> House rules: the rules your house agrees.</label>'), drawing);
  sent.length = 0;
  const act = drawn(ticked, send, { g: house, lent: [], shelf: SHELVED }).props.children.at(-1).props.children[0];
  act.type(act.props)[0].props.onClick();
  assert.deepEqual(sent, [{ type: 'bring', g: house, name: 'house', apps: ['House-rules'] }]);
});

test('each answer said beside the button, in v1’s words, the words above the boxes kept', () => {
  const bringing = step(FIRST, { type: 'bring', g: house, name: 'house', apps: ['Money', 'Events'] });
  assert.deepEqual(bringing.effects, [{ run: 'bring', g: house, apps: ['Money', 'Events'], failed: 'unbrought' }]);
  assert.equal(bringing.state.at.bring, 'bringing');
  const brought = step(bringing.state, { type: 'brought', answers: [{ app: 'Money' }, { app: 'Events', already: true }] }).state;
  assert.equal(brought.said, 'You brought Money to house: everyone in it can open it in a few seconds. Events is on the table already.');
  const drawing = out(drawn(brought, send, { g: house, lent: [], shelf: SHELVED }));
  assert.ok(drawing.endsWith(`<p><button>Bring the ticked ones to house</button> ${brought.said}</p></section>`), drawing);
  assert.ok(drawing.includes('<p>These come with svrngs.'), drawing);
  assert.equal(step(bringing.state, { type: 'brought', answers: [] }).state.said, 'Tick something to bring first.');
  const refused = step(bringing.state, { type: 'unbrought', why: 'the group is not held here' }).state;
  assert.deepEqual([refused.at.bring, refused.said], ['refused', 'Nothing was brought: the group is not held here']);
  assert.equal(step(FIRST, { type: 'yes' }).refused, 'the shelf takes no yes');
});

test('while it brings, a second bring and a tick are refused, the button and the boxes held', () => {
  const bringing = step(FIRST, { type: 'bring', g: house, name: 'house', apps: ['Money'] }).state;
  assert.equal(step(bringing, { type: 'bring', g: house, name: 'house', apps: ['Money'] }).refused, 'a bring is under way, and is answered first');
  assert.equal(step(bringing, { type: 'tick', app: 'Events', on: true }).refused, 'a bring is under way, and is answered first');
  const drawing = out(drawn(bringing, send, { g: house, lent: [], shelf: SHELVED }));
  assert.equal(drawing.match(/<input disabled type="checkbox">/g)?.length, 3, drawing);
  assert.ok(drawing.endsWith('<p><button disabled>Bring the ticked ones to house</button> Bringing…</p></section>'), drawing);
  const brought = step(bringing, { type: 'brought', answers: [{ app: 'Money' }] }).state;
  assert.equal(step(brought, { type: 'bring', g: house, name: 'house', apps: [] }).effects.length, 1, 'answered, it brings again');
});

test('what it brought is offered no more, its tick gone; all brought, the answer stands alone', () => {
  const ticked = ['Money', 'Events'].reduce((s, app) => step(s, { type: 'tick', app, on: true }).state, FIRST);
  const bringing = step(ticked, { type: 'bring', g: house, name: 'house', apps: ['Money'] }).state;
  const brought = step(bringing, { type: 'brought', answers: [{ app: 'Money' }] }).state;
  assert.deepEqual([brought.brought, brought.ticked], [['Money'], ['Events']]);
  const drawing = out(drawn(brought, send, { g: house, lent: [], shelf: SHELVED }));
  assert.ok(!drawing.includes('Money: who owes') && !drawing.includes('Money is on the table.'), drawing);
  const all = step(step(brought, { type: 'bring', g: house, name: 'house', apps: ['Events', 'House-rules'] }).state, { type: 'brought', answers: [{ app: 'Events' }, { app: 'House-rules', already: true }] }).state;
  const alone = out(drawn(all, send, { g: house, lent: [], shelf: SHELVED }));
  assert.ok(alone.endsWith('<p></p><p> You brought Events to house: everyone in it can open it in a few seconds. House rules is on the table already.</p></section>'), alone);
});

test('brought, the page reads its group again, by the `after` it gives', async () => {
  const bringing = step(FIRST, { type: 'bring', g: house, name: 'house', apps: ['Money'] }).state;
  const brought = step(bringing, { type: 'brought', answers: [{ app: 'Money' }] });
  assert.deepEqual(brought.effects, [{ run: 'after', ms: 0, failed: 'told' }]);
  assert.deepEqual(step(bringing, { type: 'unbrought', why: 'no' }).effects, [], 'refused, nothing was written to read');
  let asked = 0;
  assert.deepEqual(await SHELF(null, () => asked++).io.after({ run: 'after', ms: 0 }), { type: 'told', why: 'the group is read again, 0 ms on' });
  assert.equal(asked, 1);
  assert.equal(step(brought.state, { type: 'told' }).state.at.bring, 'brought');
  // Each read not listing Money yet asks another, a second on, then two, four, eight; then no more, said why.
  let s = brought.state;
  const waits = [];
  for (let i = 0; i < 4; i++) {
    const r = step(s, { type: 'listed', missing: ['Money'], why: 'Money' });
    waits.push(r.effects[0].ms);
    s = r.state;
  }
  assert.deepEqual(waits, [1000, 2000, 4000, 8000]);
  assert.equal(step(s, { type: 'listed', missing: ['Money'], why: 'Money' }).refused, 'Money not listed after 4 reads again; the next read says it');
  assert.deepEqual(step(brought.state, { type: 'listed', missing: [], why: 'all listed' }).effects, [], 'listed: nothing more asked');
});

test('the io answers brought, with what it brought as why', async () => {
  const asked = [];
  const io = SHELF(async (g, apps) => (asked.push([g.id, apps]), [{ app: 'Money' }, { app: 'Events', already: true }])).io;
  assert.deepEqual(await io.bring({ g: house, apps: ['Money', 'Events'] }), { type: 'brought', answers: [{ app: 'Money' }, { app: 'Events', already: true }], why: 'Money, Events already' });
  assert.deepEqual(asked, [['g1', ['Money', 'Events']]]);
});
