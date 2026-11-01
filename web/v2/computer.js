// SPDX-License-Identifier: AGPL-3.0-or-later
// On a computer, the line that gets svrngs and joins it by this link
// (v1-onboarding (c); DESIGN §7), with Copy (ui/copy.js), composed from the
// link's `#` part, which no request carries; with no token, a founder's
// line, which joins nothing and opens the home page (start-group). A phone's
// pointer is coarse, and it has no terminal: no line there (web/app.js
// `offer`, `line`, :595-614). page-open's, then page-join's; how this reaches
// you is drawn as web/index.html:88-96 has it, word for word.
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { Copy } from '../ui/copy.js?v=__VERSION__';

export const owns = ['line'];
export const states = { offer: ['unoffered', 'offered'] };
export const takes = {
  offer: (s, m) => (s.fine && /^[A-Za-z0-9_-]*$/.test(m.token) ? go(s, {}, {}, { run: 'where', token: m.token, failed: 'failed' }) : no(s, 'no line offered: not a computer, or not a token')),
  offered: (s, { token, page }) => go(s, { offer: 'offered' }, { line: [token ? 'joins' : 'founds', `curl -fsSL ${page}install.sh | sh -s -- '${page}${token && `#${token}`}'`] }),
};
// The page's own address, which only the edge reads.
export const io = { where: ({ token }) => ({ type: 'offered', token, page: `${location.origin}${location.pathname}` }) };

const SAYS = {
  joins: 'On this computer, paste this line in a terminal: it gets svrngs, joins you, and opens your home page, with the group’s folder on your disk. If svrngs is on this computer already, paste the link you opened on your home page instead, under “Join a group from a link”.',
  founds: 'To make your group, paste this line in a terminal on your computer: it gets svrngs and opens your home page, where you make your group and invite people to it.',
  opens: 'Copy this link, open your svrngs home page, and paste it under “Open a link from your group”: it opens there. If svrngs is not on this computer, open the link on your phone.',
};
const WORDS = { copy: 'Copy', copied: 'Copied', unwritten: 'Select the line and copy it' };
export const view = {
  computer: ({ s: { line: [says, text = ''] = [] } }) =>
    html`<section id="computer" hidden=${!says}>${Object.entries(SAYS).map(([id, w]) => html`<p id=${id} hidden=${id !== says}>${w}</p>`)}<pre id="line">${text}</pre><${Copy} id="copy" text=${text} words=${WORDS} /></section>`,
  how: () => html`<details id="how" hidden><summary>How this reaches you</summary><p>The link from the chat keeps its code after the #, which your browser never sends to the site this page comes from.</p><p>This browser keeps who you are. There is no account anywhere.</p><p>The site this page comes from sends only files. It does see which thing’s address you open, as each has a name under it. Your group can serve this page from a site of its own.</p><p>Between you and the computer lending a thing, a server passes it along sealed, and can’t read it. Your group can run its own.</p><p>That computer is someone’s in your group. It checks it’s you before it answers, and tells the app there who you are: some apps let you straight in, others ask you to sign in once.</p><p>What your group keeps together is on its people’s own computers, and can leave with the group, whole.</p></details>`,
};
