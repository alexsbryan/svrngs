// SPDX-License-Identifier: AGPL-3.0-or-later
// On a computer, the line that gets svrngs, and how this reaches you:
// page-open's, then page-join's, a stub until then (web/v2/page.js `compose`),
// its places as web/index.html:63-69 and :88-96 draw them, word for word.
import { html } from '../ui/view.js?v=__VERSION__';

export const unbuilt = ['offer'];
export const view = {
  computer: () => html`<section id="computer" hidden><p id="joins" hidden>On this computer, paste this line in a terminal: it gets svrngs, joins you, and opens your home page, with the group’s folder on your disk. If svrngs is on this computer already, paste the link you opened on your home page instead, under “Join a group from a link”.</p><p id="founds" hidden>To make your group, paste this line in a terminal on your computer: it gets svrngs and opens your home page, where you make your group and invite people to it.</p><p id="opens" hidden>Copy this link, open your svrngs home page, and paste it under “Open a link from your group”: it opens there. If svrngs is not on this computer, open the link on your phone.</p><pre id="line"></pre><button id="copy">Copy</button></section>`,
  how: () => html`<details id="how" hidden><summary>How this reaches you</summary><p>The link from the chat keeps its code after the #, which your browser never sends to the site this page comes from.</p><p>This browser keeps who you are. There is no account anywhere.</p><p>The site this page comes from sends only files. It does see which thing’s address you open, as each has a name under it. Your group can serve this page from a site of its own.</p><p>Between you and the computer lending a thing, a server passes it along sealed, and can’t read it. Your group can run its own.</p><p>That computer is someone’s in your group. It checks it’s you before it answers, and tells the app there who you are: some apps let you straight in, others ask you to sign in once.</p><p>What your group keeps together is on its people’s own computers, and can leave with the group, whole.</p></details>`,
};
