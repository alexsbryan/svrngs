// SPDX-License-Identifier: AGPL-3.0-or-later
// The box for a link from a group's page, and its button (open-link.js
// OPEN). The machine is the box's own (ui/view.js `useMachine`), so HOME's
// step never sees it.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { OPEN } from './open-link.js?v=__VERSION__';

/**
 * The box, on the home page at `origin`.
 * @typedef {{ origin: string }} OpenProps
 */

/** @param {OpenProps} props */
export function Open({ origin }) {
  const [state, send] = useMachine(() =>
    machine('OPEN', OPEN, {
      at: { open: 'typing' },
      link: '',
      said: '',
      port: new URL(origin).port,
    }),
  );
  return html`
    <section id="open">
      <h2>Open a link from your group</h2>
      <p>
        <${Field} words="The link: " value=${state.link} send=${send} />
        <${Act} words="Open it" doing=${{ said: state.said }} on=${{ type: 'open' }} send=${send} />
      </p>
    </section>
  `;
}
