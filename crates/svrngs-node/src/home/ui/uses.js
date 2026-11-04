// SPDX-License-Identifier: AGPL-3.0-or-later
// What apps use of each lend of `lent` that asks, both pages': a line each, a `tag`
// (the phone's `li`, the home's `p`), from its lender's choices as the page
// read them at its edge, `choices[labelOf(l)]`, `{choices}` or `{why}`; the
// app named when `lent` lists it by `labelOf`; each yes with Stop, which
// sends `{type: 'stop', lend, app}`, labels both; a lend whose choices
// could not be asked said so, with why; one not answered yet, nothing.
// What a Stop's refusal said is `said[`${lend} ${app}`]`, after the button.
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
  /** @param {string} label */
  const app = (label) => lent.find((lend) => labelOf(lend) === label);
  /**
   * @param {string} words
   * @param {Drawn[]} after
   */
  const line = (words, ...after) => html`<${tag}>${words}${after}</${tag}>`;
  const asking = lent.filter((lend) => Boolean(lend.asks) && choices[labelOf(lend)] !== undefined);
  return asking.flatMap((asks) => {
    const lend = labelOf(asks);
    // A lend past the filter has its choices: tsc does not narrow the lookup.
    const asked = /** @type {Asked} */ (choices[lend]);
    if (asked.why !== undefined) return [line(`${unasked(asks)} ${asked.why}`.trim())];
    return Object.entries(asked.choices).map(([appLabel, choice]) => {
      const using = app(appLabel);
      const who = using ? named(using) : 'An app';
      const yes = choice.said === 'yes';
      const stop = () => send({ type: 'stop', lend, app: appLabel });
      return line(
        `${who} ${yes ? 'uses' : 'was stopped using'} ${named(asks)}. `,
        yes && html`<button onClick=${stop}>Stop</button>`,
        said[`${lend} ${appLabel}`],
      );
    });
  });
}
