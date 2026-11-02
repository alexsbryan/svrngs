// SPDX-License-Identifier: AGPL-3.0-or-later
// Copy, the first component, both pages' (page-open (a)): a button that
// writes `text` to the clipboard, and its answer. Its machine, COPY, is the
// part's own (ui/view.js `useMachine`), so each Copy on a page answers for
// itself and its trace lines say `COPY:`. The clipboard is its io, the edge,
// and a page may be refused it, so `unwritten` says how to copy by hand.
import { html, useMachine } from './view.js?v=__VERSION__';
import { go, machine, no } from './machine.js?v=__VERSION__';

const AGAIN = ['idle', 'copied', 'unwritten'];
// `said` is each answer in turn, by state: a second " Copied." after the
// first, as home.js's `p.append` leaves both (home.js@19b63b2b:137, :202).
const answered = (s, m) => (s.at.copy === 'copying' ? go(s, { copy: m.type }, { said: [...s.said, m.type] }) : no(s, `${m.type} answers no copy, as none is being written`));
export const COPY = {
  states: { copy: ['idle', 'copying', 'copied', 'unwritten'] },
  step: (s, m) => ({
    copy: () => (AGAIN.includes(s.at.copy) ? go(s, { copy: 'copying' }, {}, { run: 'write', text: m.text, failed: 'unwritten' }) : no(s, 'a copy is being written already')),
    copied: () => answered(s, m),
    unwritten: () => answered(s, m),
  })[m.type]?.() ?? no(s, `a copy takes no ${m.type}`),
  io: { write: ({ text }) => navigator.clipboard.writeText(text).then(() => ({ type: 'copied' })) },
};

const FIRST = { at: { copy: 'idle' }, said: [] };
export const Copy = (props) => drawn(...useMachine(() => machine('COPY', COPY, FIRST)), props);

// `words` is the button's and each answer's: { copy, copied, unwritten }.
// Without `beside`, the answer is the button's word in its place
// (web/app.js@19b63b2b:611); with it, each answer is said after the button, past
// what `beside` draws there (`true`, nothing), as home.js appends them.
export function drawn(s, send, { id, text, words, beside }) {
  const press = () => send({ type: 'copy', text });
  if (!beside) return html`<button id=${id} onClick=${press}>${words[s.at.copy] ?? words.copy}</button>`;
  return html`<button id=${id} onClick=${press}>${words.copy}</button>${beside}${s.said.map((k) => words[k])}`;
}
