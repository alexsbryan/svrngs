// SPDX-License-Identifier: AGPL-3.0-or-later
// "This computer’s svrngs": what UPDATE (update.js) says, its one button
// when the page offers a new one, and the page’s word for each answer. It
// is drawn above the groups, so a new svrngs is seen as the page opens, and
// says nothing while it is current or before it has asked.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { START, UPDATE } from './update.js?v=__VERSION__';
/** @import { State } from './update.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

const SHOWN = new Set(['available', 'updating', 'updated', 'stuck', 'refused']);

/** @param {State} state */
const said = (state) => {
  switch (state.at.update) {
    case 'available':
      return 'A new svrngs is ready. It comes from the page this computer was installed from, and is checked against that page’s checksum before it replaces this one.';
    case 'updating':
      return 'Updating svrngs. This page restarts it when the new one is in place.';
    case 'updated':
      return 'Updated. Restarting svrngs…';
    case 'stuck':
      return 'Updated. If this page does not come back, open svrngs from your Applications, or on Linux your applications menu.';
    case 'refused':
      return `The update did not finish: ${state.why ?? 'the page’s answer could not be used'}.`;
    case 'unread':
      return `Couldn’t check for updates: ${state.why ?? 'the page did not answer'}.`;
    case 'checking':
    case 'current':
    case 'dismissed':
      return '';
  }
};

export function Update() {
  const [state, send] = useMachine(() =>
    machine('UPDATE', UPDATE, { at: { update: 'checking' } }, START),
  );
  const at = state.at.update;
  if (!SHOWN.has(at) && !(at === 'unread' && state.tried)) return false;
  return html`
    <section id="update">
      <h2>This computer’s svrngs</h2>
      <p>${said(state)}</p>
      ${
        at === 'available' &&
        html`
        <p>
          <button onClick=${() => send({ type: 'update' })}>Update and restart</button>
          <button onClick=${() => send({ type: 'later' })}>Later</button>
        </p>
      `
      }
      ${
        at === 'refused' &&
        html`
        <p>
          <button onClick=${() => send({ type: 'update' })}>Try again</button>
          <button onClick=${() => send({ type: 'later' })}>Later</button>
        </p>
      `
      }
      ${
        at === 'unread' &&
        state.tried &&
        html`<p><button onClick=${() => send({ type: 'retry' })}>Try again</button></p>`
      }
    </section>
  `;
}

/** @type {{ update: () => Drawn }} */
export const view = { update: Update };
