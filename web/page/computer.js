// SPDX-License-Identifier: AGPL-3.0-or-later
// On a computer, the line that gets svrngs and joins it by this link
// (v1-onboarding (c); DESIGN §7), with Copy (ui/copy.js), composed from the
// link's `#` part, which no request carries; with no token, a founder's
// line, which joins nothing and opens the home page (start-group). A phone's
// pointer is coarse, and it has no terminal: no line there (web/app.js
// `offer`, `line`, :595-614). page-open's, then page-join's; how this reaches
// you is drawn as web/index.html@19b63b2b:88-96 has it, word for word.
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { Copy } from '../ui/copy.js?v=__VERSION__';
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, Said, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the line offered, what it does and
 * its text.
 * @typedef {{ line?: ['joins' | 'founds' | 'opens', string] }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'offer', token: string }
 *   | { type: 'opens', wants?: Wants }
 *   | { type: 'offered', token?: string, page: string, href?: string }} Message
 */

/**
 * The effect its steps answer: where this page is, for a line that joins
 * by `token` or one that `opens` the link.
 * @typedef {{ run: 'where', token?: string, opens?: boolean, failed: string }} Effect
 */

export const owns = ['line'];
export const states = /** @type {const} */ ({ offer: ['unoffered', 'offered'] });
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  offer: (state, message) =>
    state.fine && /^[A-Za-z0-9_-]*$/.test(message.token)
      ? go(state, {}, {}, { run: 'where', token: message.token, failed: 'failed' })
      : no(state, 'no line offered: not a computer, or not a token'),
  // A link into a lend on a computer keeping no group here, its person's
  // svrngs elsewhere on it: the link itself, to open from its home page
  // (events-links (c); app.js@19b63b2b:702).
  opens: (state) =>
    state.fine
      ? go(state, {}, {}, { run: 'where', opens: true, failed: 'failed' })
      : no(state, 'no link offered: not a computer'),
  offered: (state, { token, page, href }) =>
    go(
      state,
      { offer: 'offered' },
      {
        line: href
          ? ['opens', href]
          : [
              token ? 'joins' : 'founds',
              `curl -fsSL ${page}install.sh | sh -s -- '${page}${token && `#${token}`}'`,
            ],
      },
    ),
};
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  elsewhere: (state) =>
    state.line?.[0] === 'opens' && ['This link opens in svrngs on your computer.', 'good'],
};
// The page's own address, which only the edge reads.
/** @type {Io<Effect, PageMessage>} */
export const io = {
  where: ({ token, opens }) => ({
    type: 'offered',
    token,
    page: `${location.origin}${location.pathname}`,
    href: opens ? location.href : undefined,
  }),
};

const SAYS = {
  joins:
    'On this computer, paste this line in a terminal: it gets svrngs, joins you, and opens your home page, with the group’s folder on your disk. If svrngs is on this computer already, paste the link you opened on your home page instead, under “Join a group from a link”.',
  founds:
    'To make your group, paste this line in a terminal on your computer: it gets svrngs and opens your home page, where you make your group and invite people to it.',
  opens:
    'Copy this link, open your svrngs home page, and paste it under “Open a link from your group”: it opens there. If svrngs is not on this computer, open the link on your phone.',
};
const WORDS = { copy: 'Copy', copied: 'Copied', unwritten: 'Select the line and copy it' };
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  // No line offered draws nothing: no words, and an empty line.
  computer: ({
    s: { line: [says, text = ''] = /** @type {Partial<Fields['line'] & {}>} */ ([]) },
  }) =>
    html`<section id="computer" hidden=${!says}>${Object.entries(SAYS).map(([id, words]) => html`<p id=${id} hidden=${id !== says}>${words}</p>`)}<pre id="line">${text}</pre><${Copy} id="copy" text=${text} words=${WORDS} /></section>`,
  // For someone at a computer who likes to see how a thing works, once in;
  // on a phone left out of the first look (app.js@19b63b2b:522-525; join_page.rs).
  how: ({ s: state }) =>
    html`<details id="how" hidden=${!(state.fine && state.lent)}><summary>How this reaches you</summary><p>The link from the chat keeps its code after the #, which your browser never sends to the site this page comes from.</p><p>This browser keeps who you are. There is no account anywhere.</p><p>The site this page comes from sends only files. It does see which thing’s address you open, as each has a name under it. Your group can serve this page from a site of its own.</p><p>Between you and the computer lending a thing, a server passes it along sealed, and can’t read it. Your group can run its own.</p><p>That computer is someone’s in your group. It checks it’s you before it answers, and tells the app there who you are: some apps let you straight in, others ask you to sign in once.</p><p>What your group keeps together is on its people’s own computers, and can leave with the group, whole.</p></details>`,
};
