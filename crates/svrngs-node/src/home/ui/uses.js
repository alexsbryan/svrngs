// SPDX-License-Identifier: AGPL-3.0-or-later
// What apps use of each lend of `lent` that asks, both pages' (page-acts
// (a); question.js `uses`, :72-86, which it moves): a line each, a `tag`
// (the phone's `li`, the home's `p`), from its lender's choices as the page
// read them at its edge, `choices[labelOf(l)]`, `{choices}` or `{why}`; the
// app named when `lent` lists it by `labelOf`; each yes with Stop, which
// sends `{type: 'stop', lend, app}`, labels both; a lend whose choices
// could not be asked said so, with why; one not answered yet, nothing.
// What a Stop's refusal said is `said[`${lend} ${app}`]`, after the button,
// as v1 appends it.
import { html } from './view.js?v=__VERSION__';
import { named, unasked } from '../question.js?v=__VERSION__';
/** @import { Drawn } from './view.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */

/**
 * A lender's choices as a page read them: by each app's label, what the
 * person said of it using the lend; or why they could not be read.
 * @typedef {{ choices: Record<string, { said: 'yes' | 'stop' }>, why?: undefined } | { why: string, choices?: undefined }} Asked
 */

/**
 * The lines' props: the lends, each named by `labelOf`; each lender's
 * choices as the page read them, by that label; the element each line is;
 * what each Stop's refusal said, by `${lend} ${app}`; and where a Stop is
 * sent.
 * @template {Lend} L
 * @typedef {object} UsesProps
 * @property {L[]} lent
 * @property {(lend: L) => string} labelOf
 * @property {Record<string, Asked>} choices
 * @property {'li' | 'p'} tag
 * @property {Record<string, string>} [said]
 * @property {(message: { type: 'stop', lend: string, app: string }) => void} send
 */

/**
 * @template {Lend} L
 * @param {UsesProps<L>} props
 * @returns {Drawn[]}
 */
export function Uses({ lent, labelOf, choices, tag, said = {}, send }) {
  const app = (/** @type {string} */ o) => lent.find((l) => labelOf(l) === o),
    line = (/** @type {string} */ words, /** @type {Drawn[]} */ ...after) =>
      html`<${tag}>${words}${after}</${tag}>`;
  // A lend past the filter has its choices, and an app named is found, as
  // the test before its `named` says: tsc narrows neither lookup.
  return lent
    .filter((l) => l.asks && choices[labelOf(l)])
    .flatMap((l) =>
      ((/** @type {Asked} */ a, /** @type {string} */ lend) =>
        a.why !== undefined
          ? [line(`${unasked(l)} ${a.why}`.trim())]
          : Object.entries(a.choices).map(([o, c]) =>
              line(
                `${app(o) ? named(/** @type {L} */ (app(o))) : 'An app'} ${c.said === 'yes' ? 'uses' : 'was stopped using'} ${named(l)}. `,
                c.said === 'yes' &&
                  html`<button onClick=${() => send({ type: 'stop', lend, app: o })}>Stop</button>`,
                said[`${lend} ${o}`],
              ),
            ))(/** @type {Asked} */ (choices[labelOf(l)]), labelOf(l)),
    );
}
