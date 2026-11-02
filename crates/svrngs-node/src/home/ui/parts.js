// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' seven primitives, both pages' (page-scaffold (d); ralph/next/page/
// CHARTER.md, "The parts"). Each is a pure view: props in, vnodes out, and
// `send` for what it asks. Each draws what v1 draws in its place, words,
// ids, classes and element kinds byte for byte (web/tests/parts.test.mjs).
import { html } from './view.js?v=__VERSION__';

const IDLE = { state: 'idle', said: '' };

// A button and its answer, a text node beside it as v1's are (home.js's
// `p.append(' Copied.')`), or in its place (`answer: 'instead'`, as the
// shelf's line is replaced). While `doing` is busy it is disabled and says
// nothing, and a press sends nothing. With no `on` it sends nothing itself:
// a form's own button, whose form says what is asked.
export function Act({ words, doing = IDLE, on, type, answer = 'beside', send }) {
  const busy = doing.state === 'busy', said = busy ? '' : (doing.said ?? '');
  if (answer === 'instead' && said) return said;
  return html`<button type=${type} disabled=${busy} onClick=${on && (() => busy || send(on))}>${words}</button>${said}`;
}

// Words from one region, in one element.
export const Said = ({ tag = 'p', id, class: tone, words }) => html`<${tag} id=${id} class=${tone}>${words}</${tag}>`;

// A <details> whose open is in the store: a toggle, a driver's `open =`
// too (web/tests/home.mjs), is said once, and the open it was drawn with
// is not said back.
export const Fold = ({ id, summary, heading, open = false, fold, send, children }) =>
  html`<details id=${id} open=${open} onToggle=${(e) => e.currentTarget.open === open || send({ type: 'fold', fold, open: e.currentTarget.open })}><summary>${heading ? html`<h2>${summary}</h2>` : summary}</summary>${children}</details>`;

// A box to type in: `for`, a label beside its input, which takes the focus
// as it is drawn (web/app.js@19b63b2b:136), or `wrap`, the label around it, or a
// `select` of `options`, [value, words]. Each change is sent as `typed`.
const focus = (box) => box?.focus();
export function Field({ shape = 'wrap', id, name = id, words, value, attrs = {}, options, send }) {
  const typed = (e) => send({ type: 'typed', field: name, value: e.currentTarget.value });
  const box = options
    ? html`<select name=${name} value=${value} onChange=${typed}>${options.map(([v, w]) => html`<option value=${v}>${w}</option>`)}</select>`
    : html`<input ref=${shape === 'for' ? focus : undefined} id=${id} name=${name} value=${value} onInput=${typed} ...${attrs} />`;
  return shape === 'for' ? html`<label for=${id}>${words}</label>${box}` : html`<label>${words}${box}</label>`;
}

// A row of a table, `th` for its head, or one cell across it (`span`).
export const Row = ({ cells, tag = 'td', span }) => html`<tr>${cells.map((c) => html`<${tag} colSpan=${span}>${c}</${tag}>`)}</tr>`;

// The address to keep for a lend, as question.js `keeping` says it.
export const Keep = ({ at }) => html`<small class="keep"><a href=${at}>${at}</a> Bookmark this.</small>`;

// The phone page's heading, `tone` good or bad.
export const Heading = ({ words, tone }) => html`<h1 id="status" class=${tone}>${words}</h1>`;
