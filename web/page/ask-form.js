// SPDX-License-Identifier: AGPL-3.0-or-later
// #ask: the lender's words in #words, then Yes and Not now, while
// something is asked (ask.js). Hidden while the yes is being sent, and
// shown again when it is refused, so the person can say it again. While
// the address asks, it holds the part that sends the ask once the group's
// list is read (ask-when-read.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { AskWhenRead } from './ask-when-read.js?v=__VERSION__';
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Props, State } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function AskForm({ s: state, send }) {
  const asking = state.asking;
  const words = asking && state.lent?.find((lend) => lend.label === asking.lend)?.asks;
  const sender =
    state.wants?.ask &&
    html`<${AskWhenRead} s=${/** @type {State & { wants: Wants }} */ (state)} send=${send} />`;
  /** @param {Event} event */
  const say = (event) => {
    event.preventDefault();
    return send({ type: 'say' });
  };
  return html`
    <form id="ask" hidden=${!asking || state.at.ask === 'sending'} onSubmit=${say}>
      <blockquote id="words">${words}</blockquote>
      <button>Yes</button>
      <button type="button" id="not" onClick=${() => send({ type: 'not' })}>Not now</button>
      ${sender}
    </form>
  `;
}
