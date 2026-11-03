// SPDX-License-Identifier: AGPL-3.0-or-later
// What `#open=` names, and a lend's address opened cold (DESIGN §3, "Lent
// apps", call 1; svrn-docs rings/current/IN_THE_BROWSER.md §4). Once the
// group is read (enter.js `kept` sends `wanted`), `#open=` names a lend by
// its label, else by its alias (question.js `alias`), one of what the
// group lends now, and it opens (index.js `open`) at `#at=`. None, or more
// than one, is said, never a nearest match, so an address never opens a
// thing the person did not name. Lent in more than one group held here,
// each is offered in #door (offered-groups.js), opened in the one the
// person picks. A lend's address with nothing serving it in this browser
// goes to the group's page, at most once a minute from one tab, so a lend
// that never takes its pass cannot send the tab back and forth.
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { aliased, titled } from '../question.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { trace, wasm } from './dial.js?v=__VERSION__';
import { sending } from './page.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lent, Wants } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Said, Send, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the groups offered to open what the
 * address asks in, once `among` is `offered`, and why it did not open.
 * @typedef {{ offered: { groups: Group[], wants: Wants }, missed?: string }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'among', groups: Group[], wants: Wants }
 *   | { type: 'wanted', wants: Wants & { open: string } }
 *   | { type: 'others', open: string, label: boolean, others: Group[] }
 *   | { type: 'pick', group: Group, wants: Wants }
 *   | { type: 'cold' }
 *   | { type: 'sent' }} Message
 */

/**
 * The effects its steps answer.
 * @typedef {{ run: 'forget', label: string, at: string, why: string, failed: string }
 *   | { run: 'others', open: string, id: string, why: string, failed: string }
 *   | { run: 'cold', failed: string }} Effect
 */

export const owns = ['offered', 'missed'];
export const states = /** @type {const} */ ({
  among: ['unasked', 'offered', 'picked', 'named', 'sent'],
});
/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  among: (state, { groups, wants }) =>
    go(state, { among: 'offered' }, { offered: { groups, wants } }),
  wanted: (state, { wants: { open, at = '/', anew } }) => {
    if (state.at.among === 'named') return no(state, `#open=${open} was named already`);
    // Wanted once the group is read (enter.js `kept`), so its list is.
    const byLabel = /** @type {Lent[]} */ (state.lent).filter((lend) => lend.label === open);
    const by = byLabel.length
      ? byLabel
      : aliased(open, /** @type {Lent[]} */ (state.lent), state.people);
    const name = state.group.name;
    const how = byLabel.length ? 'by its label' : 'by its alias';
    const why = `#open=${open} names ${by.length} of what ${name} lends, ${how}`;
    if (by.length === 1) {
      const { label } = /** @type {Lent} */ (by[0]);
      // A lend's origin holding no key its pass names asks for a new one
      // (open.js `anew`), so the one it holds is forgotten first.
      const opening =
        anew === undefined
          ? sending('open', { label, at, why })
          : /** @satisfies {Effect} */ ({ run: 'forget', label, at, why, failed: 'failed' });
      return go(state, { among: 'named' }, {}, opening);
    }
    if (by.length) {
      const missed = `${by.length} things in ${name} go by ${open}. Open the one you mean from the list.`;
      return go(state, { among: 'named' }, { missed });
    }
    return go(
      state,
      { among: 'named' },
      {},
      { run: 'others', open, id: state.group.id, why, failed: 'failed' },
    );
  },
  // Naming none here, it is said of the other groups held here too, as
  // each was last listed.
  others: (state, { open, label, others }) => {
    const them = others.length > 1 ? 'them' : 'it';
    const nor = others.length
      ? `, nor in ${others.map(titled).join(' or ')} when you last opened ${them}`
      : '';
    const group = state.group.name;
    const missed = label
      ? `What that address opens is not lent in ${group} now${nor}.`
      : `Nothing in ${group} goes by ${open} now${nor}.`;
    return go(state, {}, { missed });
  },
  // Picked, the offer goes in the click's own draw, before `hold` is
  // answered, so a second click finds nothing offered.
  pick: (state, { group, wants }) => {
    if (state.at.among !== 'offered') {
      return no(state, `nothing is offered: among is ${state.at.among}`);
    }
    return go(state, { among: 'picked' }, {}, sending('hold', { group, wants }));
  },
  cold: (state) => go(state, { among: 'sent' }, {}, { run: 'cold', failed: 'failed' }),
  sent: (state) => go(state, {}),
};
// Offered more than one group while no tab here holds one: group-door.js
// draws the offer in #door (offered-groups.js) only then.
export const choosing = (/** @type {State} */ state) =>
  state.at.among === 'offered' && state.at.tab === 'unheld';
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  among: (state) =>
    choosing(state) && [
      `What that address opens is lent in ${state.offered.groups.map(titled).join(' and in ')}.`,
      'good',
    ],
};
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  among: (state) =>
    choosing(state) ? ['Open it in the one you mean.'] : state.missed && [state.missed],
};
// The edge. `forget`: never from a window another page opened, which could
// read its bytes as they arrive. `cold`: the group's page is this host past
// its first name, marked in this origin's session, so a lend whose worker
// never takes its pass comes back here and is said, never sent on again.
const ONCE = 60_000;
/** @type {Io<Effect, PageMessage>} */
export const io = {
  forget: async ({ label, at, why }) => {
    const opener = /** @type {Window | null} */ (window.opener);
    if (!opener) await store.holder(label, null);
    return { type: 'open', label, at, why };
  },
  others: ({ open, id }) =>
    Promise.all([wasm(), store.groups()]).then(([core, kept]) => ({
      type: 'others',
      open,
      label: core.is_label(open),
      others: kept.filter((group) => group.id !== id && group.addresses !== undefined),
    })),
  cold: () => {
    const at = location.href.slice(location.origin.length);
    const last = Number(sessionStorage.getItem('svrngs cold'));
    const page = `${location.protocol}//${location.host.slice(location.host.indexOf('.') + 1)}/`;
    if (Date.now() - last < ONCE)
      return {
        type: 'failed',
        said: 'This could not be opened. Open it again from your group’s page.',
        why: `a lend's address with nothing serving it, sent to the group's page ${Date.now() - last} ms ago, so not again`,
      };
    sessionStorage.setItem('svrngs cold', String(Date.now()));
    location.replace(
      `${page}#${new URLSearchParams({ open: store.labelOf(location.hostname), at })}&cold`,
    );
    trace(
      `PAGE: a lend's address with nothing serving it: to the group's page, ${page}, to come back at ${at}`,
    );
    return { type: 'sent', why: `to ${page}` };
  },
};
