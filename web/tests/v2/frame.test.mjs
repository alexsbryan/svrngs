// The phone's v2 frame (page-scaffold (e); web/v2/page.js), with no DOM:
// PAGE composed from its flows refuses what two flows claim and a take
// that moves what its flow does not own; its heading is the first a slot
// gives, in ORDER, and its response every slot's lines in that order; and
// the flows compose, none left a stub, their places drawing
// web/index.html's body, word for word, in its order.

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { compose, ORDER } = await import('../../v2/page.js?v=__VERSION__');
const { machine, go } = await import('../../../crates/svrngs-node/src/home/ui/machine.js?v=__VERSION__');
const { Copy, drawn } = await import('../../../crates/svrngs-node/src/home/ui/copy.js?v=__VERSION__');
const FLOWS = ['enter', 'computer', 'join', 'ask', 'index', 'door', 'shelf', 'visit', 'among', 'acts', 'uses'];
const stubs = Object.fromEntries(await Promise.all(FLOWS.map(async (f) => [f, await import(`../../v2/${f}.js?v=__VERSION__`)])));

// A flow of `parts`, the rest left empty, as PAGE composes them.
const flow = (parts) => parts;
const lines = [];
globalThis.__trace = (l) => lines.push(l);

test('a message type taken by two flows is refused as PAGE composes', () => {
  const take = { read: (s) => go(s, {}) };
  assert.throws(() => compose({ list: flow({ takes: take }), shelf: flow({ takes: take }) }), /message type read is claimed by list and by shelf/);
});

test('a part’s own message is taken by its name, so two flows each draw a Fold', () => {
  const folds = (fold) => ({ owns: [fold], takes: { [`fold ${fold}`]: (s, m) => go(s, {}, { [fold]: m.open }) } });
  const parts = compose({ shelf: folds('shelf'), acts: folds('acts') });
  const s = parts.step(parts.first({}), { type: 'fold', fold: 'acts', open: true }).state;
  assert.equal(s.acts, true);
  assert.equal(parts.step(s, { type: 'fold', fold: 'uses', open: true }).refused, 'nothing here takes fold');
});

test('a region, a field, an io or a place two flows claim is refused as PAGE composes', () => {
  assert.throws(() => compose({ a: flow({ states: { list: ['unread'] } }), b: flow({ states: { list: ['read'] } }) }), /region list is claimed by a and by b/);
  assert.throws(() => compose({ a: flow({ owns: ['group'] }), b: flow({ owns: ['group'] }) }), /field group is claimed by a and by b/);
  assert.throws(() => compose({ a: flow({ owns: ['said'] }) }), /field said is claimed by frame and by a/, "the frame's own fields are claimed too");
  assert.throws(() => compose({ a: flow({ io: { read: () => {} } }), b: flow({ io: { read: () => {} } }) }), /io read is claimed by a and by b/);
  assert.throws(() => compose({ a: flow({ view: { door: () => '' } }), b: flow({ view: { door: () => '' } }) }), /place door is claimed by a and by b/);
  assert.throws(() => compose({ a: flow({ heading: { shelf: () => ['x'] } }) }), /a's heading or response names shelf, which is not one of failed, visit/);
});

test('a take that moves a field or region its flow does not own is refused, naming it, and nobody is told', () => {
  const list = flow({ owns: ['group'], states: { list: ['unread', 'read'] }, takes: { read: (s) => go(s, { list: 'read' }, { group: 'Bo’s' }) } });
  const shelf = flow({ owns: ['ticked'], states: { bring: ['idle', 'bringing'] },
    takes: { tick: (s) => go(s, {}, { ticked: ['Money'], group: 'mine' }), bring: (s) => go(s, { bring: 'bringing', list: 'read' }), again: (s) => go(s, {}, {}, { run: 'send', m: { type: 'read' } }) } });
  const parts = compose({ list, shelf });
  const page = machine('PAGE', parts, parts.first({}));
  let told = 0;
  page.subscribe(() => told++);
  lines.length = 0;
  page.dispatch({ type: 'tick' });
  page.dispatch({ type: 'bring' });
  page.dispatch({ type: 'nothing' });
  assert.deepEqual(lines, [
    "PAGE: tick refused in opening/unread/idle: shelf moved field group, which is list's",
    "PAGE: bring refused in opening/unread/idle: shelf moved region list, which is list's",
    'PAGE: nothing refused in opening/unread/idle: nothing here takes nothing',
  ]);
  assert.equal(told, 0);
  assert.deepEqual(page.get(), { at: { page: 'opening', list: 'unread', bring: 'idle' } });
  return new Promise((done) => {
    page.subscribe(() => page.get().group && (assert.equal(page.get().group, 'Bo’s'), done()));
    page.dispatch({ type: 'again' });
  }).then(() => assert.equal(lines.at(-1), 'PAGE: read: opening/unread/idle -> opening/read/idle', 'a flow asks another by a message, through the frame'));
});

test('the heading is the first a slot gives, in ORDER; the response every slot’s lines, joined in that order', () => {
  assert.deepEqual(ORDER, ['failed', 'visit', 'ask', 'among', 'elsewhere', 'join', 'enter']);
  const said = (words) => (s) => s.on && words;
  const parts = compose({
    enter: flow({ heading: { enter: said(['Reaching Bo’s…']), elsewhere: said(['You have Bo’s open in another tab', 'good']) }, response: { enter: said(['Nobody in Bo’s lends anything yet.']) } }),
    index: flow({ response: { enter: said('sam’s computer did not answer.') } }),
    ask: flow({ heading: { ask: said(['May Notes use Money?']) }, response: { ask: (s) => s.on && 'Money is not lent.' } }),
    join: flow({ heading: { join: said(['Join Bo’s']) } }),
  });
  const s = parts.first({ on: true });
  assert.deepEqual(parts.heading(s), ['May Notes use Money?']);
  assert.equal(parts.response(s), 'Money is not lent.\nNobody in Bo’s lends anything yet.\nsam’s computer did not answer.');
  const off = parts.first({ on: false });
  assert.deepEqual(parts.heading(off), ['Opening…'], 'web/v2/body.html’s, first and once');
  assert.equal(parts.response(off), '');
  const failed = parts.step(off, { type: 'failed', n: 1, why: 'no answer' }).state;
  assert.deepEqual(parts.heading({ ...failed, on: true }), ['This browser could not open the page.', 'bad'], 'a failed effect with no words (web/app.js:88), before every other slot');
  assert.deepEqual(parts.heading(parts.step(off, { type: 'failed', said: 'Open this page from your group’s link.' }).state), ['Open this page from your group’s link.', 'bad']);
});

// One way to write vnodes out, and web/index.html the same way: attributes
// sorted, a void element closed, the whitespace between elements dropped.
const sorted = (a) => [...a.matchAll(/([\w-]+)(?:="([^"]*)")?/g)].map(([, k, v]) => (v === undefined ? ` ${k}` : ` ${k}="${v}"`)).sort().join('');
const out = (v) => {
  if (v == null || typeof v === 'boolean') return '';
  if (Array.isArray(v)) return v.map(out).join('');
  if (typeof v !== 'object') return String(v);
  // A part with a machine of its own (ui/view.js `useMachine`) as it is first drawn.
  if (v.type === Copy) return out(drawn({ at: { copy: 'idle' }, said: [] }, () => {}, v.props));
  const a = Object.entries(v.props).filter(([k, x]) => k !== 'children' && x != null && x !== false && typeof x !== 'function').map(([k, x]) => (x === true ? k : `${k}="${x}"`));
  return `<${v.type}${sorted(a.join(' '))}>${out(v.props.children)}</${v.type}>`;
};

test('the flows compose, every flow built, their places web/index.html’s body in its order as first drawn', async () => {
  const parts = compose(stubs);
  assert.deepEqual(Object.entries(stubs).filter(([, f]) => 'unbuilt' in f).map(([n]) => n), [], 'no flow is left a stub (page-parity (e))');
  assert.deepEqual(['start', 'hold', 'reread'].filter((t) => !(t in (stubs.enter.takes ?? {}))), [], 'the frame’s `start`, among’s `hold`, the shelf’s `reread`, each enter’s');
  const page = await readFile(new URL('../../index.html', import.meta.url), 'utf8');
  const body = page.slice(page.indexOf('<section id="computer"'), page.indexOf('<pre id="response">'))
    .replace(/>\s*\n\s*</g, '><').trim().replace(/<input([^>]*)>/g, '<input$1></input>').replace(/<(\w+)([^>]*)>/g, (_, t, a) => `<${t}${sorted(a)}>`);
  assert.equal(parts.places.map((P) => out(P({ s: parts.first({}) }))).join(''), body);
});
