// SPDX-License-Identifier: AGPL-3.0-or-later
// #how: how this page reaches you, for someone at a computer who likes to
// see how a thing works, once the group is read. A phone's pointer is
// coarse, so it is left out of a phone's first look; the browser test
// holds both, and that no word of the system's own is in it
// (crates/svrngs-node/tests/browser/main.rs, "How this reaches you").
import { html } from '../ui/view.js?v=__VERSION__';
/** @import { Props } from './page.js?v=__VERSION__' */

/** @param {Props} props */
export function HowItReaches({ s: state }) {
  // The group is read once `lent` is: an empty list is read too.
  const shown = state.fine && state.lent !== undefined;
  return html`
    <details id="how" hidden=${!shown}>
      <summary>How this reaches you</summary>
      <p>The link from the chat keeps its code after the #, which your browser never sends to the site this page comes from.</p>
      <p>This browser keeps who you are. There is no account anywhere.</p>
      <p>The site this page comes from sends only files. It does see which thing’s address you open, as each has a name under it. Your group can serve this page from a site of its own.</p>
      <p>Between you and the computer lending a thing, a server passes it along sealed, and can’t read it. Your group can run its own.</p>
      <p>That computer is someone’s in your group. It checks it’s you before it answers, and tells the app there who you are: some apps let you straight in, others ask you to sign in once.</p>
      <p>What your group keeps together is on its people’s own computers, and can leave with the group, whole.</p>
    </details>
  `;
}
