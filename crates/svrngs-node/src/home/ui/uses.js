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

export function Uses({ lent, labelOf, choices, tag, said = {}, send }) {
  const app = (o) => lent.find((l) => labelOf(l) === o), line = (words, ...after) => html`<${tag}>${words}${after}</${tag}>`;
  return lent.filter((l) => l.asks && choices[labelOf(l)]).flatMap((l) => ((a, lend) => (a.why !== undefined ? [line(`${unasked(l)} ${a.why}`.trim())]
    : Object.entries(a.choices).map(([o, c]) => line(`${app(o) ? named(app(o)) : 'An app'} ${c.said === 'yes' ? 'uses' : 'was stopped using'} ${named(l)}. `, c.said === 'yes' && html`<button onClick=${() => send({ type: 'stop', lend, app: o })}>Stop</button>`, said[`${lend} ${o}`]))))(choices[labelOf(l)], labelOf(l)));
}
