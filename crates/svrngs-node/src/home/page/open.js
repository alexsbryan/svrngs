// SPDX-License-Identifier: AGPL-3.0-or-later
// "Open a link from your group" (page-join (f); home.js@19b63b2b:394-400): a link
// from a group's page, `<page>#open=<label>&at=<path>`, pasted
// (events-links (c)): the lend it names opened at its path in a tab of its
// own, through `/to` (give-session), as the group's page opens it on a
// phone. Where it opens is `to`, pure, and only within the lend (`within`,
// #153); the tab is opened at the edge. Its machine, OPEN, is the place's
// own (ui/view.js `useMachine`). A link to nothing lent here is said beside
// the button once, the last in place of the one before, where v1 appended
// each (home.js@19b63b2b:397; page-join (f)'s fix).
import { Act, assertNever, Field, go, html, machine, useMachine } from '../ui/kit.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PlaceProps } from '../home.js?v=__VERSION__' */

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
// kept only if it is still there, as open.js judges its way back — a path
// `startsWith` let through, `/\evil.example`, left for another host (#153).
/**
 * @param {string | null} at
 * @param {string} origin
 * @returns {string | null}
 */
export const within = (at, origin) =>
  ((u) => (u.origin === new URL(origin).origin ? u.href : null))(new URL(at ?? '/', origin));
// Where `link` opens through `/to` from a home page on `port`, or null.
/**
 * @param {string} link
 * @param {string} port
 * @param {URLSearchParams} [q]
 * @param {string} [lend]
 */
const to = (
  link,
  port,
  q = new URLSearchParams(link.split('#')[1] ?? ''),
  lend = q.get('open') ?? '',
) =>
  ((at) =>
    at &&
    `/to?${new URLSearchParams({ lend, at: ((u) => u.pathname + u.search + u.hash)(new URL(at)) })}`)(
    /^[a-z0-9]+$/.test(lend) && within(q.get('at'), `http://${lend}.localhost:${port}`),
  );
/** @type {Machine<State, Message, Effect>} */
const OPEN = {
  states: { open: ['typing', 'opened', 'refused'] },
  io: { open: ({ at }) => (window.open(at, '_blank'), { type: 'opened', why: at }) },
  step: (s, m) => {
    switch (m.type) {
      case 'typed':
        return go(s, { open: 'typing' }, { link: m.value });
      case 'opened':
        return go(s, {});
      case 'open':
        return ((at) =>
          at
            ? go(s, { open: 'opened' }, {}, { run: 'open', at, failed: 'refused' })
            : go(
                s,
                { open: 'refused' },
                { said: ' That is not a link to something lent in your groups.' },
              ))(to(s.link, s.port));
      case 'refused':
        return go(s, { open: 'refused' }, { said: ` ${m.why}` });
      default:
        return assertNever(s, m, 'opening a link');
    }
  },
};

/**
 * The place's link box, on the home page at `origin`.
 * @typedef {{ origin: string }} OpenProps
 */

/** @param {OpenProps} props */
function Open({ origin }) {
  const [s, send] = useMachine(() =>
    machine('OPEN', OPEN, {
      at: { open: 'typing' },
      link: '',
      said: '',
      port: new URL(origin).port,
    }),
  );
  return html`<section id="open"><h2>Open a link from your group</h2><p><${Field} words="The link: " value=${s.link} send=${send} /><${Act} words="Open it" doing=${{ said: s.said }} on=${{ type: 'open' }} send=${send} /></p></section>`;
}
// Drawn once the groups are read (home.js `Home`), so `s.groups` is read
// then.
/** @type {{ open: (props: PlaceProps) => Drawn }} */
export const view = {
  open: ({ s }) =>
    /** @type {NonNullable<PlaceProps['s']['groups']>} */ (s.groups).length > 0 &&
    html`<${Open} origin=${s.origin} />`,
};
