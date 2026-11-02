// v2's seven primitives (page-scaffold (d); crates/svrngs-node/src/home/ui/
// parts.js), each called as a function with no DOM, its vnodes written out
// as HTML and held to what v1 draws in its place: v1's own builders where
// one exists (question.js `keeping`, `row`, `el`; home.js's `button`), run
// against a document of plain objects, and v1's markup otherwise
// (web/index.html). Then what each sends.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./pages/layout.mjs', import.meta.url);
const { Act, Field, Fold, Heading, Keep, Row, Said } = await import('../../crates/svrngs-node/src/home/ui/parts.js?v=__VERSION__');
import { v1 } from './pages/before.mjs';

// A document of plain objects, enough for v1's `el` and its `Object.assign`s.
class Node {
  constructor(tag) {
    Object.defineProperty(this, 'tag', { value: tag });
    Object.defineProperty(this, 'kids', { value: [] });
  }
  append(...kids) {
    this.kids.push(...kids);
  }
}
globalThis.document = { createElement: (tag) => new Node(tag) };

// One way to write either out: attributes sorted, by their HTML names, a
// handler or an absent value left out, `true` as the bare name.
const NAMES = { className: 'class', htmlFor: 'for', colSpan: 'colspan' };
const attrs = (o) =>
  Object.entries(o)
    .filter(([k, v]) => !['children', 'key', 'ref'].includes(k) && !/^on/i.test(k) && v != null && v !== false)
    .map(([k, v]) => [NAMES[k] ?? k.toLowerCase(), v])
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
const { el } = v1;
const said = [];
const send = (m) => said.push(m);

test('Keep is question.js `keeping`, byte for byte', () => {
  const at = 'https://group.svrnme.sh/#open=money';
  assert.equal(out(Keep({ at })), out(v1.keeping(at)));
  assert.equal(out(Keep({ at })), `<small class="keep"><a href="${at}">${at}</a> Bookmark this.</small>`);
});

test('Row is question.js `row`, a head row too, and a line across the table as home.js draws it', () => {
  const cells = ['Money', el('strong', 'sam'), 'In a tab.'];
  assert.equal(out(Row({ cells })), out(v1.row(cells)));
  const head = ['What', 'Whose', 'How it opens', 'Now'];
  assert.equal(out(Row({ cells: head, tag: 'th' })), out(v1.row(head, 'th')));
  const words = 'Nobody lends anything here yet.';
  assert.equal(out(Row({ cells: [words], span: 4 })), out(el('tr', '', Object.assign(el('td', words), { colSpan: 4 }))));
});

test('Act is a button with its answer beside it, as home.js draws one; disabled and silent while busy', () => {
  const v1p = el('p', '', Object.assign(el('button', 'Copy the link'), { onclick: () => {} }));
  v1p.append(' Copied.');
  const done = { state: 'done', said: ' Copied.' };
  assert.equal(out(el('p', '', Act({ words: 'Copy the link', doing: done, on: { type: 'copy' }, send }))), out(v1p));
  const busy = Act({ words: 'Copy the link', doing: { state: 'busy', said: ' Copied.' }, on: { type: 'copy' }, send });
  assert.equal(out(busy), '<button disabled>Copy the link</button>', 'no answer while busy');
  said.length = 0;
  busy[0].props.onClick();
  assert.deepEqual(said, [], 'a press while busy sends nothing');
  Act({ words: 'Copy the link', on: { type: 'copy' }, send })[0].props.onClick();
  assert.deepEqual(said, [{ type: 'copy' }]);
});

test("Act's answer in its place, and a form's own button, which sends nothing itself", () => {
  const brought = { state: 'done', said: 'You brought Money to Bo’s: everyone in it can open it in a few seconds.' };
  assert.equal(out(Act({ words: 'Bring the ticked ones to Bo’s', doing: brought, answer: 'instead', on: { type: 'bring' }, send })), brought.said);
  const join = Act({ words: 'Join', send });
  assert.equal(out(join), '<button>Join</button>', 'web/index.html@19b63b2b:75');
  assert.equal(join[0].props.onClick, undefined);
});

test('Fold is the shelf’s and acts’ <details>, and says each toggle', () => {
  const shelf = el('section', '', el('h3', 'Bring things to the table'));
  const kept = el('details', '', el('summary', 'Bring more to Bo’s'), shelf);
  assert.equal(out(Fold({ summary: 'Bring more to Bo’s', fold: 'shelf', send, children: shelf })), out(kept));
  const acts = el('details', '', el('summary', '', el('h2', 'What acts as you')));
  assert.equal(out(Fold({ summary: 'What acts as you', heading: true, fold: 'acts', send })), out(acts));
  assert.equal(out(Fold({ id: 'uses', summary: 'What apps use', open: true, fold: 'uses', send })), '<details id="uses" open><summary>What apps use</summary></details>');
  said.length = 0;
  const f = Fold({ summary: 'What acts as you', heading: true, fold: 'acts', send });
  f.props.onToggle({ currentTarget: { open: true } });
  f.props.onToggle({ currentTarget: { open: false } });
  assert.deepEqual(said, [{ type: 'fold', fold: 'acts', open: true }], 'a driver’s `open =` is taken; the open it was drawn with is not said again');
});

test('Field: the join’s label and box, the home page’s wrapped ones and its select, and what each sends', () => {
  const group = el('span', 'Bo’s');
  group.id = 'group';
  const words = ['What’s your name? Everyone in ', group, ' sees it.'];
  const join = Field({ shape: 'for', id: 'name', words, attrs: { autocomplete: 'name', maxlength: '64', required: true }, send });
  assert.equal(out(join), '<label for="name">What’s your name? Everyone in <span id="group">Bo’s</span> sees it.</label><input autocomplete="name" id="name" maxlength="64" name="name" required></input>', 'web/index.html@19b63b2b:72-73');
  const wrap = Field({ name: 'link', words: 'The link someone sent you: ', send });
  assert.equal(out(wrap), out(el('label', 'The link someone sent you: ', Object.assign(el('input'), { name: 'link' }))));
  const hours = [48, 24, 6].map((h) => [h * 3600, `${h} hours`]);
  const v1length = el('label', 'For how long, from when they open the link: ', el('select', '', ...hours.map(([value, t]) => Object.assign(el('option', t), { value }))));
  assert.equal(out(Field({ name: 'length', words: 'For how long, from when they open the link: ', options: hours, send })).replace(' name="length"', ''), out(v1length));
  said.length = 0;
  wrap.props.children[1].props.onInput({ currentTarget: { value: 'https://x/#y' } });
  assert.deepEqual(said, [{ type: 'typed', field: 'link', value: 'https://x/#y' }]);
});

test('Heading and Said are the phone’s #status, #response and #why', () => {
  assert.equal(out(Heading({ words: 'Opening…' })), '<h1 id="status">Opening…</h1>', 'web/index.html@19b63b2b:62');
  assert.equal(out(Heading({ words: 'This browser could not open the page.', tone: 'bad' })), '<h1 class="bad" id="status">This browser could not open the page.</h1>');
  assert.equal(out(Said({ tag: 'pre', id: 'response', words: '' })), '<pre id="response"></pre>', 'web/index.html@19b63b2b:98');
  assert.equal(out(Said({ id: 'why', class: 'bad', words: 'That name is taken.' })), '<p class="bad" id="why">That name is taken.</p>');
});
