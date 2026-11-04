// The door and who is visiting, both pages' (page-members (a);
// crates/svrngs-node/src/home/ui/door.js and visitors.js), with no browser:
// each drawing held to question.js `door` and `visiting`, which they move,
// byte for byte, written out as shelf.test.mjs writes either.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { Door } = await import('../../../crates/svrngs-node/src/home/ui/door.js?v=__VERSION__');
const { Visiting } =
  await import('../../../crates/svrngs-node/src/home/ui/visitors.js?v=__VERSION__');
import { v1 } from './before.mjs';

// A document of plain objects, enough for v1's `el` and `door`'s judge,
// which sets a cell's class and its last child's words.
class Node {
  constructor(tag) {
    Object.defineProperty(this, 'tag', { value: tag });
    Object.defineProperty(this, 'kids', { value: [] });
  }
  append(...kids) {
    this.kids.push(...kids);
  }
  get lastChild() {
    return this.kids.at(-1);
  }
  set textContent(words) {
    this.kids.splice(0, this.kids.length, words);
  }
}
globalThis.document = { createElement: (tag) => new Node(tag) };
// v1's `className` and v2's `class` are one attribute; an empty one is none.
const attrs = (o) =>
  Object.entries(o)
    .filter(
      ([k, v]) =>
        !['children', 'key', 'ref'].includes(k) &&
        !/^on/i.test(k) &&
        v != null &&
        v !== false &&
        v !== '',
    )
    .map(([k, v]) => [k === 'className' ? 'class' : k.toLowerCase(), v])
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
const drawn = (P, props) => out(P(props));

// ada started it; bo's computer; two cys told apart; pia only from a phone; dee on both.
const ROWS = [
  { person: 'ada', person_key: 'a', key: 'a', device: 'computer', joined: 'started' },
  { person: 'bo', person_key: 'b', key: 'b', device: 'computer', joined: { at: 1_791_000_000 } },
  { person: 'cy', person_key: 'c1', key: 'c1', device: 'computer', joined: { at: 1_791_000_100 } },
  { person: 'cy', person_key: 'c2', key: 'c2', device: 'computer', joined: { at: 1_791_000_200 } },
  { person: 'pia', person_key: 'p', key: 'p', device: 'browser', joined: { at: 1_791_000_300 } },
  { person: 'dee', person_key: 'd', key: 'd', device: 'computer', joined: { at: 1_791_000_400 } },
  { person: 'dee', person_key: 'd', key: 'd2', device: 'browser', joined: { at: 1_791_000_500 } },
];
const judged = (lends, lent) => {
  const [ul, judge] = v1.door(ROWS, lent);
  if (lends !== undefined) judge(lends ?? undefined);
  return out(ul);
};

test('the door as question.js `door` draws it: unjudged, judged, not known now', () => {
  for (const lends of [
    undefined,
    { unanswered: [{ person: 'bo', person_key: 'b' }] },
    { unanswered: ['cy'] },
    null,
  ]) {
    assert.equal(drawn(Door, { rows: ROWS, lends }), judged(lends), JSON.stringify(lends));
  }
  const v2 = drawn(Door, {
    rows: ROWS,
    lends: { unanswered: [{ person: 'bo', person_key: 'b' }] },
  });
  assert.ok(
    v2.startsWith(
      '<ul class="door"><li class="on"><strong>ada</strong> <small>on now</small></li><li class="off"><strong>bo</strong> <small>did not answer</small></li>',
    ),
    v2,
  );
  assert.ok(
    v2.includes(
      '<li class="phone"><strong>pia</strong> <small>in from a phone</small></li><li class="on"><strong>dee</strong> <small>on now, a phone too</small></li>',
    ),
    v2,
  );
  assert.ok(
    drawn(Door, { rows: ROWS }).includes('<li><strong>bo</strong> <small>asking…</small></li>'),
  );
});

test('given what is lent, each person’s own lends in their window, the group’s in nobody’s', () => {
  const lent = [
    { key: 'b', name: 'radio', origin: 'http://radio.example/' },
    { key: 'd2', name: 'pad', origin: 'http://pad.example/' },
    { key: 'a', group: 'g1', kind: 'Money', name: 'Money', origin: 'http://m.example/' },
  ];
  const lends = { unanswered: [] };
  assert.equal(drawn(Door, { rows: ROWS, lent, lends }), judged(lends, lent));
  assert.ok(
    drawn(Door, { rows: ROWS, lent, lends }).includes(
      '<strong>bo</strong> <a href="http://radio.example/" target="_blank">radio</a> <small>on now</small>',
    ),
  );
});

test('who is visiting as question.js `visiting` draws it, by host', () => {
  const vs = [
    { guest: 'sam', host: 'cy', host_key: 'c2', until: 1_791_100_000 },
    { guest: 'kim', host: 'ada', host_key: 'a', until: 1_791_200_000 },
    { guest: 'lu', host: 'bo', until: 1_791_300_000 },
  ];
  assert.equal(drawn(Visiting, { vs, rows: ROWS }), out(v1.visiting(vs, ROWS)));
  assert.ok(
    drawn(Visiting, { vs, rows: ROWS }).startsWith(
      '<ul class="visiting"><li>kim, ada’s guest, until ',
    ),
  );
  assert.equal(drawn(Visiting, { vs: [] }), '<ul class="visiting"></ul>');
});
