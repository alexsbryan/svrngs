// SPDX-License-Identifier: AGPL-3.0-or-later
// OPEN, the machine of "Open a link from your group" (open.js `Open`): a
// link from a group's page, `<page>#open=<label>&at=<path>`, pasted, opens
// the lend it names at its path in a tab of its own, through `/to`, as the
// group's page opens it on a phone. Where it opens is `to`, pure, and only
// within the lend (`within`, #153); the tab is opened at the edge, in `io`.
// A link to nothing lent here is said beside the button, the last in place
// of the one before.
import { assertNever, go } from '../ui/machine.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */

/**
 * The link's state: where its open is, the link as typed, what was last
 * said of it, and the home page's port, which `to` opens a lend's address
 * on.
 * @typedef {object} State
 * @property {{ open: 'typing' | 'opened' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} link
 * @property {string} said
 * @property {string} port
 */

/**
 * The messages the link takes: typed, in the Field, which names no field
 * here; Open pressed; its tab opened at `why`, or refused, with why.
 * @typedef {{ type: 'typed', value: string }
 *   | { type: 'open' }
 *   | { type: 'opened', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effect its step answers: a tab opened at `at`.
 * @typedef {{ run: 'open', at: string, failed: 'refused' }} Effect
 */

// `at` (`/` when none) as an address at `origin`, or null: resolved, and
// kept only if it is still there, as web/open.js judges its way back. A
// path `startsWith` let through, `/\evil.example`, resolves to another
// host (#153).
/**
 * @param {string | null} at
 * @param {string} origin
 * @returns {string | null}
 */
export const within = (at, origin) => {
  const url = new URL(at ?? '/', origin);
  return url.origin === new URL(origin).origin ? url.href : null;
};

// Where `link` opens through `/to` from a home page on `port`, or null. A
// lend's label is lowercase letters and digits, so any other `open` names
// nothing lent here.
/**
 * @param {string} link
 * @param {string} port
 * @returns {string | null}
 */
const to = (link, port) => {
  const params = new URLSearchParams(link.split('#')[1] ?? '');
  const lend = params.get('open') ?? '';
  if (!/^[a-z0-9]+$/.test(lend)) return null;
  const at = within(params.get('at'), `http://${lend}.localhost:${port}`);
  if (!at) return at;
  const url = new URL(at);
  return `/to?${new URLSearchParams({ lend, at: url.pathname + url.search + url.hash })}`;
};

/** @type {Machine<State, Message, Effect>} */
export const OPEN = {
  states: { open: ['typing', 'opened', 'refused'] },
  io: {
    open: ({ at }) => {
      window.open(at, '_blank');
      return { type: 'opened', why: at };
    },
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed':
        return go(state, { open: 'typing' }, { link: message.value });
      case 'opened':
        return go(state, {});
      case 'open': {
        const at = to(state.link, state.port);
        if (at) return go(state, { open: 'opened' }, {}, { run: 'open', at, failed: 'refused' });
        return go(
          state,
          { open: 'refused' },
          { said: ' That is not a link to something lent in your groups.' },
        );
      }
      case 'refused':
        return go(state, { open: 'refused' }, { said: ` ${message.why}` });
      default:
        return assertNever(state, message, 'opening a link');
    }
  },
};
