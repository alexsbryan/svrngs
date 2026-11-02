// "What acts as you", what apps use, and an app's new version, both pages'
// shared parts (page-acts (a); crates/svrngs-node/src/home/ui/{acts,
// changes,uses}.js), with no browser: each drawing held to the question.js
// builder it moves, `acts`, `changes` and `uses`, byte for byte, written
// out as shelf.test.mjs writes either.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { Acts } = await import('../../../crates/svrngs-node/src/home/ui/acts.js?v=__VERSION__');
const { Changes } = await import('../../../crates/svrngs-node/src/home/ui/changes.js?v=__VERSION__');
const { Uses } = await import('../../../crates/svrngs-node/src/home/ui/uses.js?v=__VERSION__');
import { v1 } from './before.mjs';

// A document of plain objects, enough for v1's `el` (shelf.test.mjs), and
// `children`, which v1's `acts` counts.
class Node {
  constructor(tag) {
    Object.defineProperty(this, 'tag', { value: tag });
    Object.defineProperty(this, 'kids', { value: [] });
  }
  get children() {
    return this.kids.filter((k) => k instanceof Node);
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
// Each element of a v2 drawing, its own vnodes walked (`Act`'s drawn).
const each = (v, f) => (Array.isArray(v) ? v.forEach((k) => each(k, f)) : v && typeof v === 'object' && (f(v), each(typeof v.type === 'function' ? v.type(v.props) : v.props?.children, f)));
const sent = [];
const send = (m) => sent.push(m);

const house = { id: 'g1', name: 'house', person: 'ada' };
const rows = [
  { person: 'ada', person_key: 'pa', key: 'ka', joined: 'started' },
  { person: 'cy', person_key: 'pc', key: 'kc', joined: { at: 1_790_000_000 } },
];

test('"What acts as you" drawn as question.js `acts` draws it: folded, each part headed once it has a line, its note after', () => {
  const p = (w) => v1.el('p', w);
  for (const [notes, lines] of [
    [{}, {}],
    [{ opens: 'Each Stop holds from the app’s next request.' }, { knows: ['What you open knows your name.'] }],
    [{ opens: 'A Stop here holds within 12 hours.', play: 'Any program reaches these.' }, { reads: ['Money reads and writes who owes whom, when you open it.', 'Events too.'], opens: ['cy’s notes opens your friends’ films as you.'], decides: ['This phone.'] }],
  ]) {
    const [section, add] = v1.acts(notes);
    for (const [k, ws] of Object.entries(lines)) add(k, ...ws.map(p));
    const parts = Object.fromEntries(Object.entries(lines).map(([k, ws]) => [k, ws.map((w) => ({ type: 'p', props: { children: w } }))]));
    assert.equal(out(Acts({ parts, notes, open: false, send })), out(section), JSON.stringify(lines));
  }
  const drawing = out(Acts({ parts: {}, open: true, send }));
  assert.ok(drawing.startsWith('<section id="acts"><details open><summary><h2>What acts as you</h2></summary><div></div>'), drawing);
});

test('an app’s new version drawn as question.js `changes` draws it: unknown, none, by you and by another, of this group only', () => {
  const at = 1_792_000_000;
  for (const [g, changed] of [
    [house, null],
    [{ id: 'g2' }, null],
    [house, []],
    [house, [{ group: 'g1', app: 'House-rules', by_key: 'pa', at }]],
    [house, [{ group: 'g1', app: 'Money', by_key: 'pc', at }, { group: 'g1', app: 'Events', by: 'dee', at }, { group: 'g9', app: 'Money', by: 'eve', at }]],
    [house, [{ group: 'g1', app: 'Money', by: 'ada', at }]],
  ]) {
    assert.equal(out(Changes({ g, changed, rows, me: 'ka', send })), out(v1.changes(g, { changed }, async () => {}, rows, 'ka')), JSON.stringify(changed));
  }
});

test('a yes to a new version: sent by its button, its line said in its place once done, a refusal beside the button', async () => {
  const changed = [{ group: 'g1', app: 'House-rules', by_key: 'pc', at: 1_792_000_000 }];
  sent.length = 0;
  each(Changes({ g: house, changed, rows, me: 'ka', send }), (v) => v.type === 'button' && v.props.onClick());
  assert.deepEqual(sent, [{ type: 'yes', app: 'House-rules' }]);
  const [folded] = v1.changes(house, { changed }, async () => {}, rows, 'ka');
  folded.kids.at(-1).kids.at(-1).onclick();
  await new Promise((r) => setTimeout(r));
  assert.equal(out(Changes({ g: house, changed, rows, me: 'ka', doing: { 'House-rules': { state: 'done' } }, send })), out([folded]));
  assert.ok(out(folded).includes('<p>House rules opens at its new version for you now.</p>'), out(folded));
  const [refused] = v1.changes(house, { changed }, async () => Promise.reject(new Error('cy’s computer did not answer.')), rows, 'ka');
  refused.kids.at(-1).kids.at(-1).onclick();
  await new Promise((r) => setTimeout(r));
  assert.equal(out(Changes({ g: house, changed, rows, me: 'ka', doing: { 'House-rules': { state: 'refused', said: ' cy’s computer did not answer.' } }, send })), out([refused]));
  const busy = out(Changes({ g: house, changed, rows, me: 'ka', doing: { 'House-rules': { state: 'busy' } }, send }));
  assert.ok(busy.includes('<button disabled>Open the new version</button></p>'), busy);
});

test('what apps use drawn as question.js `uses` draws it, a line each, as `li` and as `p`; Stop sent with both labels', async () => {
  const film = { person: 'cy', name: 'shelf', kind: 'k', asks: 'It lends your shelf.', key: 'kc' };
  const page = { person: 'ada', name: 'page', kind: 'k', key: 'ka' };
  const plain = { person: 'cy', name: 'notes', kind: 'notes', key: 'kc' };
  const other = { person: 'dee', name: 'box', kind: 'box', asks: 'It lends your box.', key: 'kd' };
  const lent = [plain, film, page, other];
  const labelOf = (l) => `${l.name}-${l.key}`;
  const answers = { [labelOf(film)]: { choices: { [labelOf(page)]: { said: 'yes' }, 'gone-kx': { said: 'stop' } } }, [labelOf(other)]: { why: 'answered 503' } };
  for (const tag of ['li', 'p']) {
    const ul = v1.el('ul');
    const line = (w) => ((e) => (e.append(w), ul.append(e), e))(document.createElement(tag));
    const choicesOf = async (l) => (answers[labelOf(l)].why ? Promise.reject(new Error(answers[labelOf(l)].why)) : answers[labelOf(l)].choices);
    await v1.uses(lent, labelOf, choicesOf, line, () => {});
    assert.equal(`<ul>${out(Uses({ lent, labelOf, choices: answers, tag, send }))}</ul>`, out(ul), tag);
  }
  const drawing = out(Uses({ lent, labelOf, choices: answers, tag: 'li', send }));
  assert.ok(drawing.startsWith('<li>ada’s page (k) uses cy’s shelf (k). <button>Stop</button></li><li>An app was stopped using cy’s shelf (k). </li>'), drawing);
  assert.equal(out(Uses({ lent, labelOf, choices: {}, tag: 'li', send })), '', 'nothing answered yet, nothing drawn');
  sent.length = 0;
  each(Uses({ lent, labelOf, choices: answers, tag: 'li', send }), (v) => v.type === 'button' && v.props.onClick());
  assert.deepEqual(sent, [{ type: 'stop', lend: labelOf(film), app: labelOf(page) }]);
  const stopped = out(Uses({ lent, labelOf, choices: answers, tag: 'li', said: { [`${labelOf(film)} ${labelOf(page)}`]: 'cy’s computer did not keep your answer.' }, send }));
  assert.ok(stopped.startsWith('<li>ada’s page (k) uses cy’s shelf (k). <button>Stop</button>cy’s computer did not keep your answer.</li>'), stopped);
});

// The phone's PAGE composed from its flows (enter.test.mjs), each effect read as data.
const { compose } = await import('../../page/page.js?v=__VERSION__');
const FLOWS = ['enter', 'computer', 'join', 'ask', 'index', 'door', 'shelf', 'visit', 'among', 'acts', 'uses'];
const parts = compose(Object.fromEntries(await Promise.all(FLOWS.map(async (f) => [f, await import(`../../page/${f}.js?v=__VERSION__`)]))));

test('a group read holding unsent lines answers their carry, then nothing once none is left (page-acts (d))', () => {
  const line = { seq: 0, id: 'l0' };
  const s = parts.first({ group: { ...house, token: 'TOKEN', unsent: [line] } });
  const r = parts.step(s, { type: 'carry' });
  assert.equal(r.refused, undefined, r.refused);
  assert.deepEqual(r.effects.map((f) => [f.run, f.group?.unsent]), [['carrying', [line]]]);
  assert.equal(parts.step(r.state, { type: 'carry' }).refused, 'carry is under way, and is answered first');
  const refused = parts.step(r.state, { type: 'unacted', why: 'ada’s computer did not answer.' });
  assert.deepEqual(refused.effects, [], 'a carry refused asks no read again, so it is tried once a read');
  assert.equal(parts.step(parts.first({ group: { ...house, unsent: [] } }), { type: 'carry' }).refused, 'no line is unsent');
});
