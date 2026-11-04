// SPDX-License-Identifier: AGPL-3.0-or-later
// A guest's visit (GUESTS §5, "How a visit is made"): a link a member made
// for a guest, `#visit=<token>`. The guest's name is asked once, in #join,
// by join.js's NameForm, which sends it here (`guest named`). The visit is
// asked as a key made fresh for it, never one kept for a group (§3.9), and
// the link is redeemed at the member's computer, which writes the visit.
// It is kept apart from every group (store.js `keepVisit`), so the link
// opened again shows it again, or says it has ended. What the visit opens
// is listed first in #lent (guest-lends.js), each open a pass that lasts
// past the visit (`call`), so the lender's door, not the page, says it has
// ended. The key never enters state: the edge holds it.
import { go, no } from '../ui/machine.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { patiently, saying, wasm } from './dial.js?v=__VERSION__';
import { PASS_LIFE, handing } from './hand.js?v=__VERSION__';
/** @import { Visit } from '../store.js?v=__VERSION__' */
/** @import { Io, Step, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { At, Message as PageMessage, Nothing, Said, State } from './page.js?v=__VERSION__' */

/**
 * A visit as this page holds it: whose guest, until when, in seconds, and
 * each lend it opens, at the host's computer.
 * @typedef {{ host: string, until: number, lent: Lent[] }} Stay
 */

/**
 * The fields this flow owns (`owns`): the link's token, once a visit is
 * asked, and the visit, once it is `on` or `ended`.
 * @typedef {{ invited: string, stay: Stay }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'visit', token: string }
 *   | { type: 'unkept' }
 *   | { type: 'guest named', guest: string }
 *   | { type: 'redeemed', stay: Stay, ended: boolean }
 *   | { type: 'ended' }
 *   | { type: 'called' }
 *   | { type: 'call', label: string }} Message
 */

/**
 * The effects its steps answer.
 * @typedef {{ run: 'recall', token: string, failed: string }
 *   | { run: 'redeem', token: string, guest: string, failed: string }
 *   | { run: 'over', until: number, failed: string }
 *   | { run: 'call', token: string, lent: Lent[], label: string, until: number, failed: string }} Effect
 */

export const owns = ['invited', 'stay'];
export const states = /** @type {const} */ ({
  visit: ['none', 'naming', 'redeeming', 'on', 'ended'],
});
// Each take in the states it is asked in, else refused saying where the visit is.
/**
 * @template M
 * @param {readonly At['visit'][]} regions
 * @param {(s: State, m: M) => Step<State, Effect>} listener
 * @returns {(s: State, m: M) => Step<State, Effect>}
 */
const at = (regions, listener) => (state, message) =>
  regions.includes(state.at.visit)
    ? listener(state, message)
    : no(state, `the visit is ${state.at.visit}`);
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  visit: at(['none'], (state, { token }) =>
    go(state, {}, { invited: token }, { run: 'recall', token, failed: 'failed' }),
  ),
  unkept: at(['none'], (state) => go(state, { visit: 'naming' })),
  'guest named': at(['naming'], (state, { guest }) =>
    go(
      state,
      { visit: 'redeeming' },
      {},
      { run: 'redeem', token: state.invited, guest, failed: 'failed' },
    ),
  ),
  // Whether it has ended is the edge's, which alone reads the clock. One
  // that has not ends on its timer (`over`), and its lends go from the
  // page with it.
  redeemed: at(['none', 'redeeming'], (state, { stay, ended }) => {
    if (ended) return go(state, { visit: 'ended' }, { stay });
    /** @satisfies {Effect} */
    const over = { run: 'over', until: stay.until, failed: 'failed' };
    return go(state, { visit: 'on' }, { stay }, over);
  }),
  ended: at(['on'], (state) => go(state, { visit: 'ended' })),
  called: (state) => go(state, {}),
  call: at(['on'], (state, { label }) =>
    go(
      state,
      {},
      {},
      {
        run: 'call',
        token: state.invited,
        lent: state.stay.lent,
        label,
        until: state.stay.until + PASS_LIFE,
        failed: 'failed',
      },
    ),
  ),
};
// Each heading's words are built whatever the visit's state, so `stay` is
// read before a link is redeemed too, absent then (page.js `first`).
const host = (/** @type {Partial<State>} */ state) => state.stay?.host;
const until = (/** @type {Partial<State>} */ state) =>
  state.stay && new Date(state.stay.until * 1000).toLocaleString();
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  visit: (state) =>
    /** @type {Partial<Record<At['visit'], Said>>} */ ({
      naming: ['You’re invited over as a guest'],
      redeeming: ['Opening your link…'],
      on: [`You’re ${host(state)}’s guest until ${until(state)}`, 'good'],
      ended: [`Your visit as ${host(state)}’s guest ended ${until(state)}.`, 'bad'],
    })[state.at.visit],
};
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  visit: (state) =>
    state.at.visit === 'on' && !state.stay.lent.length && [`${host(state)} lends nothing yet.`],
};
// The edge: the key, made here and kept sealed with the visit, or unsealed
// from it; what the visit opens, each by its label, at the link's
// computer; and the visit asked of it, dialled as the key, its refusal
// said by its status.
const failing = (/** @type {any} */ error) =>
  /** @satisfies {PageMessage} */ ({ type: 'failed', said: error.said, why: String(error) });
// The visit's key, made or unsealed by `recall` or `redeem` before any
// other run reads it.
/** @type {Uint8Array<ArrayBuffer> | undefined} */
let key;
/**
 * @param {typeof Wasm} core
 * @param {string} token
 * @param {Visit} visit
 * @returns {PageMessage}
 */
const stay = (core, token, visit) => {
  const issuer = core.issuer_of(token);
  const lent = Object.entries(visit.lent).map(([name, kind]) => ({
    key: issuer,
    relay: core.relay_of(token),
    name,
    kind,
    person: visit.host,
    version: visit.version,
    label: core.label(issuer, name),
  }));
  return {
    type: 'redeemed',
    ended: visit.until <= Date.now() / 1000,
    why: `a guest of ${visit.host} until ${visit.until}, opening ${Object.keys(visit.lent).join(', ')}`,
    stay: { host: visit.host, until: visit.until, lent },
  };
};
/**
 * The visit, asked of the computer that made the link as `guest`.
 * @param {typeof Wasm} core
 * @param {string} token
 * @param {string} guest
 * @returns {Promise<Visit>}
 */
async function visiting(core, token, guest) {
  const ask = JSON.parse(core.ask(/** @type {Uint8Array} */ (key), token));
  const bytes = new TextEncoder().encode(JSON.stringify({ ask, guest }));
  const dialled = await patiently(
    core.connect_to_join(/** @type {Uint8Array} */ (key), token),
  ).catch(() => {
    throw saying('The computer that sent this link did not answer. Try again later.');
  });
  // The wasm's `fetch` answers the page's `Response` (svrngs_web.d.ts `Dialled`).
  /** @type {Response} */
  const res = await dialled
    .fetch(
      'POST',
      '/visit',
      [
        ['content-type', 'application/json'],
        ['content-length', String(bytes.length)],
      ],
      new Blob([bytes]).stream(),
    )
    .finally(() => dialled.close());
  if (res.ok) return res.json();
  if (res.status === 403) throw saying('This link lets nobody in. Ask for a new one.');
  throw saying('The computer that sent this link cannot let anyone in just now. Try again later.');
}
/**
 * The visit kept for this link, its key unsealed, or `unkept` when none is.
 * @param {string} token
 * @returns {Promise<PageMessage>}
 */
async function recalling(token) {
  const core = await wasm();
  if (!core.relay_of(token)) {
    throw saying('This link was made by an older svrngs. Ask whoever sent it for a new one.');
  }
  const kept = await store.visitOf(token);
  if (!kept) return { type: 'unkept', why: 'no visit is kept for this link' };
  key = kept.key;
  return stay(core, token, kept);
}
/**
 * The link redeemed as `guest` with a key made for it, and the visit kept
 * sealed with that key.
 * @param {string} token
 * @param {string} guest
 * @returns {Promise<PageMessage>}
 */
async function redeeming(token, guest) {
  const core = await wasm();
  key = crypto.getRandomValues(new Uint8Array(32));
  const visit = await visiting(core, token, guest);
  await store.keepVisit(token, visit, key);
  return stay(core, token, visit);
}
/** @type {Io<Effect, PageMessage>} */
export const io = {
  recall: ({ token }) => recalling(token).catch(failing),
  redeem: ({ token, guest }) => redeeming(token, guest).catch(failing),
  over: ({ until: ends }) =>
    new Promise((resolve) => setTimeout(resolve, ends * 1000 - Date.now())).then(() => ({
      type: 'ended',
      why: `the visit ended at ${ends}`,
    })),
  call: ({ token, lent, label, until: ends }) =>
    handing(
      { group: { token }, lent, unanswered: [], label },
      /** @type {Uint8Array} */ (key),
      ends,
    ).then((message) =>
      message.type === 'handed'
        ? /** @satisfies {PageMessage} */ ({ ...message, type: 'called' })
        : /** @satisfies {PageMessage} */ ({
            type: 'failed',
            said: message.said,
            why: message.said,
          }),
    ),
};
