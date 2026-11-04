// SPDX-License-Identifier: AGPL-3.0-or-later
// #computer: the line computer.js offers, in a terminal's words, with Copy
// (ui/copy.js). Each line's words are drawn once, hidden unless they are
// the line's, so the words a browser test reads (web/tests/computer.mjs,
// `#computer p:not([hidden])`) are the only ones shown.
import { html } from '../ui/view.js?v=__VERSION__';
import { Copy } from '../ui/copy.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

const SAYS = {
  installs:
    'Paste this line in a terminal on your Mac or Linux computer. It installs or updates svrngs, keeps your identity, joins the group and opens its local page with the shared folder’s exact path and Open this folder. It runs whenever you log in. If joining fails, wait a minute and retry this same line; your existing groups keep working.',
  joins:
    'On this computer, paste this line in a terminal: it gets svrngs, joins you, and opens your home page, with the group’s folder on your disk. If svrngs is on this computer already, paste the link you opened on your home page instead, under “Join a group from a link”.',
  founds:
    'To make your group, paste this line in a terminal on your computer: it gets svrngs and opens your home page, where you make your group and invite people to it.',
  opens:
    'Copy this link, open your svrngs home page, and paste it under “Open a link from your group”: it opens there. If svrngs is not on this computer, open the link on your phone.',
};

const WORDS = { copy: 'Copy', copied: 'Copied', unwritten: 'Select the line and copy it' };

/** @param {Props} props */
export function ComputerLine({ s: state }) {
  // No line offered draws nothing: no words, and an empty line.
  const [says, text = ''] = state.line ?? [];
  const words = Object.entries(SAYS)
    .filter(([id]) => id !== 'installs' || says === 'installs')
    .map(([id, said]) => html`<p id=${id} hidden=${id !== says}>${said}</p>`);
  return html`
    <section id="computer" hidden=${!says}>
      ${words}
      <pre id="line">${text}</pre>
      <${Copy} id="copy" text=${text} words=${WORDS} />
    </section>
  `;
}
