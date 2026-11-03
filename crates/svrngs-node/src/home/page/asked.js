// SPDX-License-Identifier: AGPL-3.0-or-later
// The ask at /ask (asking.js ASK): svrngs's question in its one wording
// (question.js `question`) around the lender's words, Yes and Not now, and
// what was last said; or why it is not asked (`unlent`). The machine is
// the ask's own (ui/view.js `useMachine`), so HOME's step never sees it.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { question, unlent } from '../question.js?v=__VERSION__';
import { ASK } from './asking.js?v=__VERSION__';
/** @import { Group, Lent } from './group.js?v=__VERSION__' */
/** @import { PlaceProps } from '../home.js?v=__VERSION__' */

// HOME's state, `home`, past its groups' read, as `Home` draws the ask only
// then (home.js `Home`, past its first `return`), so `home.groups` is read.
/** @param {PlaceProps} props */
export function Asked({ s: home }) {
  const [state, send] = useMachine(() =>
    machine('ASK', ASK(home.q), { at: { ask: 'reading' }, said: '' }, [
      { run: 'read', groups: /** @type {Group[]} */ (home.groups), failed: 'lent' },
    ]),
  );
  if (state.at.ask === 'reading') return null;
  if (state.at.ask === 'unlent') return html`<p>${unlent}</p>`;
  // Past `reading` and `unlent`, the ask is `asking` or `sending`, so its
  // `lend` and `app` are set (ASK's `step`).
  const lend = /** @type {Lent} */ (state.lend);
  const app = /** @type {Lent} */ (state.app);
  return html`
    <section>
      <h2>${question(app, lend)}</h2>
      <blockquote>${lend.asks}</blockquote>
      <button onClick=${() => send({ type: 'yes' })}>Yes</button>
      <button onClick=${() => send({ type: 'not' })}>Not now</button>
      ${state.said}
    </section>
  `;
}
