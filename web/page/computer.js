// SPDX-License-Identifier: AGPL-3.0-or-later
// On a computer, the line that gets svrngs and joins it by this link
// (DESIGN §7), drawn in #computer (computer-line.js). It is composed from
// the link's `#` part, which no request carries. With no token, it is a
// founder's line, which joins nothing and opens the home page. A phone's
// pointer is coarse and a phone has no terminal, so no line is offered
// there. #how (how-it-reaches.js) says how this page reaches you.
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { ComputerLine } from './computer-line.js?v=__VERSION__';
import { HowItReaches } from './how-it-reaches.js?v=__VERSION__';
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, Said, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the line offered, what it does and
 * its text.
 * @typedef {{ line?: ['joins' | 'founds' | 'opens' | 'installs', string] }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'offer', token: string, name?: string, destination?: string }
 *   | { type: 'opens', wants?: Wants }
 *   | { type: 'offered', token?: string, page: string, href?: string, name?: string, destination?: string }} Message
 */

/**
 * The effect its steps answer: where this page is, for a line that joins
 * by `token` or one that `opens` the link.
 * @typedef {{ run: 'where', token?: string, opens?: boolean, name?: string, destination?: string, failed: string }} Effect
 */

export const owns = ['line'];
export const states = /** @type {const} */ ({ offer: ['unoffered', 'offered'] });
const quote = (/** @type {string} */ text) => `'${text.replaceAll("'", "'\\''")}'`;

/** @type {Takes<State, Message, Effect>} */
export const takes = {
  offer: (state, message) =>
    state.fine && /^[A-Za-z0-9_-]*$/.test(message.token)
      ? go(
          state,
          {},
          {},
          {
            run: 'where',
            token: message.token,
            name: message.name,
            destination: message.destination,
            failed: 'failed',
          },
        )
      : no(state, 'no line offered: not a computer, or not a token'),
  // A link into a lend on a computer keeping no group here, its person's
  // svrngs elsewhere on it: the link itself, to open from its home page.
  opens: (state) =>
    state.fine
      ? go(state, {}, {}, { run: 'where', opens: true, failed: 'failed' })
      : no(state, 'no link offered: not a computer'),
  offered: (state, { token, page, href, name, destination }) => {
    const source = `${page}install.sh`;
    const download = /^[\w:/.-]+$/.test(source) ? source : quote(source);
    const more = name === undefined ? '' : ` --at ${quote(destination ?? '/')} --as ${quote(name)}`;
    const install = `curl -fsSL ${download} | sh -s -- ${quote(`${page}${token && `#${token}`}`)}${more}`;
    const joined = token ? 'joins' : 'founds';
    const installs = name !== undefined ? 'installs' : joined;
    /** @type {Fields['line']} */
    const line = href ? ['opens', href] : [installs, install];
    return go(state, { offer: 'offered' }, { line });
  },
};

/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  elsewhere: (state) =>
    state.line?.[0] === 'opens' && ['This link opens in svrngs on your computer.', 'good'],
};

// The page's own address, which only the edge reads.
/** @type {Io<Effect, PageMessage>} */
export const io = {
  where: ({ token, opens, name, destination }) => ({
    type: 'offered',
    token,
    name,
    destination,
    page: `${location.origin}${location.pathname}`,
    href: opens ? location.href : undefined,
  }),
};

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = { computer: ComputerLine, how: HowItReaches };
