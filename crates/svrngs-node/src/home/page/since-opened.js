// SPDX-License-Identifier: AGPL-3.0-or-later
// Beside a group on a screen: who joined and what was lent since the
// screen opened, newest first, the newest marked new (on-screen.js
// SCREEN's `since`).
import { html } from '../ui/view.js?v=__VERSION__';
/** @import { Since } from './on-screen.js?v=__VERSION__' */

/**
 * What arrived since the screen opened, newest first.
 * @typedef {{ since: Since[] }} SinceOpenedProps
 */

/** @param {SinceOpenedProps} props */
export function SinceOpened({ since }) {
  const lines = since.map(
    ([time, words], i) =>
      html`<li><time>${time}</time> ${words} ${i === 0 && html`<mark>new</mark>`}</li>`,
  );
  return html`
    <aside>
      <h2>Since this screen opened</h2>
      <p>Kept on this screen only: it starts over when the screen is opened again.</p>
      <ol>${lines}</ol>
      <p>Nothing new yet. Who joins, and what they lend, shows here as it happens.</p>
    </aside>
  `;
}
