// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' eight primitives, both pages' (ralph/next/page/CHARTER.md,
// "The parts"). Each is a pure view: props in, vnodes out, and `send` for
// what it asks. Their words, ids, classes and element kinds are pinned byte
// for byte (web/tests/parts.test.mjs).
import { html } from './view.js?v=__VERSION__';
/** @import { Drawn, Targeted } from './view.js?v=__VERSION__' */

/**
 * What a page is doing with a button's ask, and what its answer said:
 * `busy` is the one state `Act` reads; a page's others pass through it
 * (home/page/acts.js `Acted`'s `done`, the shelf's `refused`).
 * @typedef {{ state?: string, said?: string }} Doing
 */

const IDLE = { state: 'idle', said: '' };

// A button and its answer, a text node beside it (" Copied."), or in its place (`answer: 'instead'`, as the
// shelf's line is replaced). While `doing` is busy it is disabled and says
// nothing, and a press sends nothing. With no `on` it sends nothing itself:
// a form's own button, whose form says what is asked.
/**
 * A button's props: its words; what its ask is doing, idle when not
 * given; the message it sends, none for a form's own; its `type`; its
 * answer beside it or in its place; and where its message is sent.
 * @template Message
 * @typedef {object} ActProps
 * @property {Drawn} words
 * @property {Doing} [doing]
 * @property {Message} [on]
 * @property {'submit' | 'button'} [type]
 * @property {'beside' | 'instead'} [answer]
 * @property {(message: Message) => void} send
 */
/**
 * @template Message
 * @param {ActProps<Message>} props
 */
export function Act({ words, doing = IDLE, on, type, answer = 'beside', send }) {
  const busy = doing.state === 'busy';
  const said = busy ? '' : (doing.said ?? '');
  if (answer === 'instead' && said) return said;
  const press = () => {
    if (!busy && on !== undefined) send(on);
  };
  return html`
    <button type=${type} disabled=${busy} onClick=${on === undefined ? undefined : press}>
      ${words}
    </button>
    ${said}
  `;
}

// A <details> whose open is in the store: a toggle, a driver's `open =`
// too (web/tests/home.mjs), is said once, and the open it was drawn with
// is not said back.
/**
 * A fold's props: its id; its summary, a heading when `heading`; whether
 * it is open, in the store; its name in the `fold` message a toggle sends;
 * and what it holds.
 * @typedef {object} FoldProps
 * @property {string} [id]
 * @property {Drawn} summary
 * @property {boolean} [heading]
 * @property {boolean} [open]
 * @property {string} fold
 * @property {(message: { type: 'fold', fold: string, open: boolean }) => void} send
 * @property {Drawn} [children]
 */
/** @param {FoldProps} props */
export function Fold({ id, summary, heading, open = false, fold, send, children }) {
  /** @param {Targeted<HTMLDetailsElement>} toggled */
  const toggle = (toggled) => {
    const opened = toggled.currentTarget.open;
    if (opened !== open) send({ type: 'fold', fold, open: opened });
  };
  const head = heading ? html`<h2>${summary}</h2>` : summary;
  return html`
    <details id=${id} open=${open} onToggle=${toggle}>
      <summary>${head}</summary>
      ${children}
    </details>
  `;
}

// A tick box and its words, a line of a list: the group a lend goes to, an
// app brought, a lend an app may read.
/**
 * @typedef {object} TickProps
 * @property {boolean} on
 * @property {boolean} [disabled]
 * @property {(ticked: Targeted<HTMLInputElement>) => void} change
 * @property {Drawn} [children]
 */
/** @param {TickProps} props */
export function Tick({ on, disabled, change, children }) {
  return html`
    <p>
      <label>
        <input type="checkbox" disabled=${disabled} checked=${on} onChange=${change} />
        ${children}
      </label>
    </p>
  `;
}

// A box to type in: `for`, a label beside its input, which takes the focus
// as it is drawn, or `wrap`, the label around it, or a
// `select` of `options`, [value, words]. Each change is sent as `typed`.
const focus = (/** @type {HTMLInputElement | null} */ box) => box?.focus();
/**
 * A field's props: its `typed` message names the field `name`, `id` when
 * not given, so the machine taking it is checked for that field.
 * @template {string} Name
 * @typedef {object} FieldProps
 * @property {'wrap' | 'for'} [shape]
 * @property {Name} id
 * @property {Name} [name]
 * @property {import('./view.js?v=__VERSION__').Drawn} words
 * @property {string} [value]
 * @property {Record<string, string | number | boolean>} [attrs] the box's own attributes
 * @property {[value: string, words: string][]} [options] a `select`'s, else a text box
 * @property {(message: { type: 'typed', field: Name, value: string }) => void} send
 */
/**
 * @template {string} Name
 * @param {FieldProps<Name>} props
 */
export function Field({ shape = 'wrap', id, name = id, words, value, attrs = {}, options, send }) {
  const typed = (/** @type {Targeted<HTMLInputElement | HTMLSelectElement>} */ changed) =>
    send({ type: 'typed', field: name, value: changed.currentTarget.value });
  const choices = options?.map(
    ([optionValue, optionWords]) => html`<option value=${optionValue}>${optionWords}</option>`,
  );
  const box = options
    ? html`
        <select name=${name} value=${value} onChange=${typed}>
          ${choices}
        </select>
      `
    : html`
        <input
          ref=${shape === 'for' ? focus : undefined}
          id=${id}
          name=${name}
          value=${value}
          onInput=${typed}
          ...${attrs}
        />
      `;
  return shape === 'for'
    ? html`<label for=${id}>${words}</label>${box}`
    : html`<label>${words}${box}</label>`;
}

// A row of a table, `th` for its head, or one cell across it (`span`).
/**
 * @typedef {object} RowProps
 * @property {Drawn[]} cells
 * @property {'td' | 'th'} [tag]
 * @property {number} [span]
 */
/** @param {RowProps} props */
export function Row({ cells, tag = 'td', span }) {
  const drawn = cells.map((cell) => html`<${tag} colSpan=${span}>${cell}</${tag}>`);
  return html`<tr>${drawn}</tr>`;
}

// The address to keep for a lend, as question.js `keeping` says it.
/** @param {{ at: string }} props */
export const Keep = ({ at }) =>
  html`<small class="keep"><a href=${at}>${at}</a> Bookmark this.</small>`;

// The phone page's heading, `tone` good or bad.
/** @param {{ words: Drawn, tone?: 'good' | 'bad' }} props */
export const Heading = ({ words, tone }) => html`<h1 id="status" class=${tone}>${words}</h1>`;
