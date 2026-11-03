// SPDX-License-Identifier: AGPL-3.0-or-later
// What `#open=` names, and a lend's address opened cold (page-join (b);
// DESIGN §3, "Lent apps", call 1; svrn-docs rings/current/
// IN_THE_BROWSER.md §4). Once the group is read (enter.js `kept` sends
// `wanted`), `#open=` names a lend by its label, else by its alias
// (question.js `alias`), one of what the group lends now, and it opens
// (index.js `open`) at `#at=`; none, or more than one, is said, never a
// nearest match (app.js@19b63b2b:546-563). Lent in more than one group held here, each
// is offered in #door, opened in the one the person picks (app.js@19b63b2b:581-587). A
// lend's address with nothing serving it in this browser goes to the
// group's page, at most once a minute from one tab (app.js@19b63b2b:638-646).
import { aliased, go, html, no, whose } from '../ui/kit.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lent, Wants } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, Said, Send, Sender, State } from './page.js?v=__VERSION__' */

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
const titled = (/** @type {Group} */ g) => `${g.name}${g.maker ? ` (${whose(g)})` : ''}`,
  send = /** @type {Sender} */ ((type, m) => ({ run: 'send', m: { type, ...m } }));
/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  among: (s, { groups, wants }) => go(s, { among: 'offered' }, { offered: { groups, wants } }),
  wanted: (s, { wants: { open, at = '/', anew } }) => {
    if (s.at.among === 'named') return no(s, `#open=${open} was named already`);
    // Wanted once the group is read (enter.js `kept`), so its list is.
    const byLabel = /** @type {Lent[]} */ (s.lent).filter((l) => l.label === open),
      by = byLabel.length ? byLabel : aliased(open, /** @type {Lent[]} */ (s.lent), s.people),
      name = s.group.name;
    const why = `#open=${open} names ${by.length} of what ${name} lends${byLabel.length ? ', by its label' : ', by its alias'}`;
    // A lend's origin holding no key its pass names asks for a new one (open.js `anew`).
    // Named by one, `by[0]` is it.
    if (by.length === 1)
      return go(
        s,
        { among: 'named' },
        {},
        anew === undefined
          ? send('open', { label: /** @type {Lent} */ (by[0]).label, at, why })
          : { run: 'forget', label: /** @type {Lent} */ (by[0]).label, at, why, failed: 'failed' },
      );
    if (by.length)
      return go(
        s,
        { among: 'named' },
        {
          missed: `${by.length} things in ${name} go by ${open}. Open the one you mean from the list.`,
        },
      );
    return go(
      s,
      { among: 'named' },
      {},
      { run: 'others', open, id: s.group.id, why, failed: 'failed' },
    );
  },
  // Naming none here, it is said of the other groups held here too, as each was last listed.
  others: (
    s,
    { open, label, others },
    nor = others.length
      ? `, nor in ${others.map(titled).join(' or ')} when you last opened ${others.length > 1 ? 'them' : 'it'}`
      : '',
  ) =>
    go(
      s,
      {},
      {
        missed: label
          ? `What that address opens is not lent in ${s.group.name} now${nor}.`
          : `Nothing in ${s.group.name} goes by ${open} now${nor}.`,
      },
    ),
  // Picked, the offer goes in the click's own draw, as v1's did (web/app.js@19b63b2b:586), before `hold` is answered.
  pick: (s, { group, wants }) =>
    s.at.among === 'offered'
      ? go(s, { among: 'picked' }, {}, send('hold', { group, wants }))
      : no(s, `nothing is offered: among is ${s.at.among}`),
  cold: (s) => go(s, { among: 'sent' }, {}, { run: 'cold', failed: 'failed' }),
  sent: (s) => go(s, {}),
};
const choosing = (/** @type {State} */ s) => s.at.among === 'offered' && s.at.tab === 'unheld';
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  among: (s) =>
    choosing(s) && [
      `What that address opens is lent in ${s.offered.groups.map(titled).join(' and in ')}.`,
      'good',
    ],
};
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  among: (s) => (choosing(s) ? ['Open it in the one you mean.'] : s.missed && [s.missed]),
};
// Each group offered, drawn in #door (enter.js `chosen`).
/**
 * @param {State} s
 * @param {Props['send']} send
 */
export const offered = (s, send) =>
  choosing(s) &&
  s.offered.groups.map(
    (g) =>
      html`<p><button onClick=${() => send({ type: 'pick', group: g, wants: s.offered.wants })}>Open it in ${titled(g)}</button></p>`,
  );

// The edge. `forget`: never from a window another page opened, which could
// read its bytes as they arrive. `cold`: the group's page is this host past
// its first name, marked in this origin's session, so a lend whose worker
// never takes its pass comes back here and is said, never sent on again.
const ONCE = 60_000,
  wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__'),
  trace = (/** @type {string} */ l) => globalThis.__trace?.(l);
/** @type {Io<Effect, PageMessage>} */
export const io = {
  forget: ({ label, at, why }) =>
    (window.opener ? Promise.resolve() : store.holder(label, null)).then(() => ({
      type: 'open',
      label,
      at,
      why,
    })),
  others: ({ open, id }) =>
    Promise.all([wasm(), store.groups()]).then(([w, kept]) => ({
      type: 'others',
      open,
      label: w.is_label(open),
      others: kept.filter((g) => g.id !== id && g.addresses),
    })),
  cold: () => {
    const at = location.href.slice(location.origin.length),
      last = Number(sessionStorage.getItem('svrngs cold')),
      page = `${location.protocol}//${location.host.slice(location.host.indexOf('.') + 1)}/`;
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
    return (
      trace(
        `PAGE: a lend's address with nothing serving it: to the group's page, ${page}, to come back at ${at}`,
      ),
      { type: 'sent', why: `to ${page}` }
    );
  },
};
