// Copy, v2's first component (page-open (a); crates/svrngs-node/src/home/ui/
// copy.js), with no DOM: COPY's every move and refusal, its trace, an io
// that fails each way it can, and what it draws, held to what v1 draws in
// its place: the phone's #copy, its word replaced (web/app.js:611), and
// the home page's answers appended after the button (home.js:137, :202),
// the second " Copied." kept.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./v2/layout.mjs', import.meta.url);
const { COPY, drawn } = await import('../../crates/svrngs-node/src/home/ui/copy.js?v=__VERSION__');
const { machine } = await import('../../crates/svrngs-node/src/home/ui/machine.js?v=__VERSION__');

const FIRST = { at: { copy: 'idle' }, said: [] };
const settled = () => new Promise((r) => setTimeout(r, 0));
// A COPY over `write`, each text it was asked kept, its trace kept.
function copy(write) {
  const lines = [], texts = [];
  globalThis.__trace = (l) => lines.push(l);
  const m = machine('COPY', { ...COPY, io: { write: (f) => (texts.push(f.text), write(f)) } }, FIRST);
  return { m, lines, texts, at: () => m.get().at.copy };
}
const step = (at, m, said = []) => COPY.step({ at: { copy: at }, said }, m);

test('a copy is written, then said copied, and traced as COPY', async () => {
  const { m, lines, texts, at } = copy(async () => ({ type: 'copied' }));
  m.dispatch({ type: 'copy', text: 'curl -fsSL x | sh' });
  assert.equal(at(), 'copying');
  await settled();
  assert.deepEqual([at(), m.get().said, texts], ['copied', ['copied'], ['curl -fsSL x | sh']]);
  assert.deepEqual(lines, ['COPY: copy: idle -> copying, running write', 'COPY: copied: copying -> copied']);
});

test('a copy the clipboard refuses is unwritten, with why; one whose io answers nothing too', async () => {
  const refused = copy(() => Promise.reject(new Error('NotAllowedError')));
  refused.m.dispatch({ type: 'copy', text: 'x' });
  await settled();
  assert.deepEqual([refused.at(), refused.m.get().said], ['unwritten', ['unwritten']]);
  assert.equal(refused.lines[1], 'COPY: unwritten (NotAllowedError): copying -> unwritten');
  const thrown = copy(() => {
    throw new Error('no clipboard');
  });
  thrown.m.dispatch({ type: 'copy', text: 'x' });
  await settled();
  assert.equal(thrown.lines[1], 'COPY: unwritten (no clipboard): copying -> unwritten');
  const silent = copy(async () => undefined);
  silent.m.dispatch({ type: 'copy', text: 'x' });
  await settled();
  assert.equal(silent.lines[1], 'COPY: unwritten (write answered nothing): copying -> unwritten');
});

test('a copy while one is copying is refused, and once answered a copy is taken again, each answer kept in turn', async () => {
  let answer = { type: 'copied' };
  const { m, lines, texts, at } = copy(async () => answer);
  m.dispatch({ type: 'copy', text: 'a' });
  m.dispatch({ type: 'copy', text: 'b' });
  assert.equal(lines[1], 'COPY: copy refused in copying: a copy is being written already');
  await settled();
  answer = { type: 'unwritten', why: 'NotAllowedError' };
  m.dispatch({ type: 'copy', text: 'a' });
  await settled();
  m.dispatch({ type: 'copy', text: 'a' });
  await settled();
  assert.deepEqual([at(), m.get().said, texts], ['unwritten', ['copied', 'unwritten', 'unwritten'], ['a', 'a', 'a']]);
});

test('an answer with no copy being written is refused, and so is a message COPY does not take', () => {
  for (const at of ['idle', 'copied', 'unwritten']) {
    assert.equal(step(at, { type: 'copied' }).refused, 'copied answers no copy, as none is being written');
    assert.equal(step(at, { type: 'unwritten' }).refused, 'unwritten answers no copy, as none is being written');
  }
  assert.equal(step('idle', { type: 'paste' }).refused, 'a copy takes no paste');
  assert.deepEqual(step('copying', { type: 'copied' }).state.at, { copy: 'copied' });
  assert.deepEqual(step('idle', { type: 'copy', text: 'a' }).effects, [{ run: 'write', text: 'a', failed: 'unwritten' }]);
});

// What it draws, written out as HTML: attributes sorted, handlers left out.
const out = (v) => {
  if (v == null || typeof v === 'boolean') return '';
  if (Array.isArray(v)) return v.map(out).join('');
  if (typeof v !== 'object') return String(v);
  const a = Object.entries(v.props).filter(([k, x]) => k !== 'children' && !k.startsWith('on') && x != null).map(([k, x]) => ` ${k}="${x}"`).join('');
  return `<${v.type}${a}>${out(v.props.children)}</${v.type}>`;
};
const PHONE = { copy: 'Copy', copied: 'Copied', unwritten: 'Select the line and copy it' };
const HOME = { copy: 'Copy the link', copied: ' Copied.', unwritten: ' Select it above to copy it.' };

test('without beside, the answer is the button’s word, as the phone’s #copy is; a press sends the copy of its text', () => {
  const sent = [], send = (m) => sent.push(m);
  const at = (copy) => out(drawn({ at: { copy }, said: [] }, send, { id: 'copy', text: 'curl', words: PHONE }));
  assert.equal(at('idle'), '<button id="copy">Copy</button>', 'web/index.html:95');
  assert.equal(at('copying'), '<button id="copy">Copy</button>');
  assert.equal(at('copied'), '<button id="copy">Copied</button>', 'web/app.js:611');
  assert.equal(at('unwritten'), '<button id="copy">Select the line and copy it</button>');
  drawn({ at: { copy: 'idle' }, said: [] }, send, { id: 'copy', text: 'curl', words: PHONE }).props.onClick();
  assert.deepEqual(sent, [{ type: 'copy', text: 'curl' }]);
});

test('with beside, each answer is said after the button and what beside draws, the second " Copied." kept', () => {
  const s = { at: { copy: 'copied' }, said: ['copied', 'unwritten', 'copied'] };
  assert.equal(out(drawn(s, () => {}, { text: 'l', words: HOME, beside: true })), '<button>Copy the link</button> Copied. Select it above to copy it. Copied.');
  const keep = { type: 'small', props: { children: 'Bookmark this.' } };
  assert.equal(out(drawn({ at: { copy: 'idle' }, said: [] }, () => {}, { text: 'l', words: HOME, beside: [' ', keep] })), '<button>Copy the link</button> <small>Bookmark this.</small>');
});
