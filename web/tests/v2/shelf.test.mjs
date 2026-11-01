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
const v1 = await import('../../../crates/svrngs-node/src/home/question.js?v=__VERSION__');

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
const FIRST = { at: { bring: 'idle' }, ticked: [], open: false, said: '' };
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
  const act = drawn(ticked, send, { g: house, lent: [], shelf: SHELVED }).props.children.at(-1).props.children;
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

test('moved with v1’s faults: a second tap while bringing brings again', () => {
  const bringing = step(FIRST, { type: 'bring', g: house, name: 'house', apps: ['Money'] }).state;
  assert.equal(step(bringing, { type: 'bring', g: house, name: 'house', apps: ['Money'] }).effects.length, 1);
});

test('the io answers brought, with what it brought as why', async () => {
  const asked = [];
  const io = SHELF(async (g, apps) => (asked.push([g.id, apps]), [{ app: 'Money' }, { app: 'Events', already: true }])).io;
  assert.deepEqual(await io.bring({ g: house, apps: ['Money', 'Events'] }), { type: 'brought', answers: [{ app: 'Money' }, { app: 'Events', already: true }], why: 'Money, Events already' });
  assert.deepEqual(asked, [['g1', ['Money', 'Events']]]);
});
