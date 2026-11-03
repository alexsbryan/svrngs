// SPDX-License-Identifier: AGPL-3.0-or-later
// A guest's visit (page-members (c); web/app.js `visit`, :198-226;
// GUESTS §5, "How a visit is made"): a link a member made for a guest,
// `#visit=<token>`. The guest's name is asked once, in #join, by
// web/page/join.js's NameForm, which sends it here (`guest named`), a key made fresh for this visit, never one kept for a group
// (§3.9), and the link redeemed at the member's computer, which writes the
// visit. It is kept apart from every group (store.js `keepVisit`), so the
// link opened again shows it again, or says it has ended. What the visit
// opens is listed in #lent (`listed`, web/page/index.js's place), each open
// a pass that lasts past the visit (app.js@19b63b2b:211), so the lender's door says it
// has ended. The key never enters state: the edge holds it.
import { go, html, named, no, Row } from '../ui/kit.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { PASS_LIFE, handing } from './index.js?v=__VERSION__';
/** @import { Visit } from '../store.js?v=__VERSION__' */
/** @import { Io, Step, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { At, Message as PageMessage, Nothing, Props, Said, State } from './page.js?v=__VERSION__' */

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
 * @param {(s: State, m: M) => Step<State, Effect>} f
 * @returns {(s: State, m: M) => Step<State, Effect>}
 */
const at = (regions, f) => (s, m) =>
  regions.includes(s.at.visit) ? f(s, m) : no(s, `the visit is ${s.at.visit}`);
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  visit: at(['none'], (s, { token }) =>
    go(s, {}, { invited: token }, { run: 'recall', token, failed: 'failed' }),
  ),
  unkept: at(['none'], (s) => go(s, { visit: 'naming' })),
  'guest named': at(['naming'], (s, { guest }) =>
    go(s, { visit: 'redeeming' }, {}, { run: 'redeem', token: s.invited, guest, failed: 'failed' }),
  ),
  // Whether it has ended is the edge's, which alone reads the clock; one
  // that has not ends on its timer (`over`), and its lends go from the
  // page with it, where web/app.js listed them until the link was opened
  // again (:221-224).
  redeemed: at(['none', 'redeeming'], (s, { stay, ended }) =>
    go(
      s,
      { visit: ended ? 'ended' : 'on' },
      { stay },
      ...(ended
        ? []
        : [/** @satisfies {Effect} */ ({ run: 'over', until: stay.until, failed: 'failed' })]),
    ),
  ),
  ended: at(['on'], (s) => go(s, { visit: 'ended' })),
  called: (s) => go(s, {}),
  call: at(['on'], (s, { label }) =>
    go(
      s,
      {},
      {},
      {
        run: 'call',
        token: s.invited,
        lent: s.stay.lent,
        label,
        until: s.stay.until + PASS_LIFE,
        failed: 'failed',
      },
    ),
  ),
};
const host = (/** @type {State} */ s) => s.stay?.host,
  until = (/** @type {State} */ s) => s.stay && new Date(s.stay.until * 1000).toLocaleString();
/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  visit: (s) =>
    /** @type {Partial<Record<At['visit'], Said>>} */ ({
      naming: ['You’re invited over as a guest'],
      redeeming: ['Opening your link…'],
      on: [`You’re ${host(s)}’s guest until ${until(s)}`, 'good'],
      ended: [`Your visit as ${host(s)}’s guest ended ${until(s)}.`, 'bad'],
    })[s.at.visit],
};
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  visit: (s) => s.at.visit === 'on' && !s.stay.lent.length && [`${host(s)} lends nothing yet.`],
};
/**
 * @param {State} s
 * @param {Props['send']} send
 */
export const listed = (s, send) =>
  s.at.visit === 'on' &&
  s.stay.lent.map(
    (l) =>
      html`<${Row} key=${l.label} cells=${[html`<button onClick=${() => send({ type: 'call', label: l.label })}>${named(l)}</button>`]} />`,
  );

// The edge: the key, made here and kept sealed with the visit, or unsealed
// from it; what the visit opens, each by its label, at the link's computer;
// and the visit asked of it, dialled as the key, as web/app.js `post` does
// (:110-117), its refusal by status (`joining`, :106).
const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__'),
  said = (/** @type {string} */ words) =>
    Promise.reject(Object.assign(new Error(words), { said: words })),
  failing = (/** @type {any} */ e) =>
    /** @satisfies {PageMessage} */ ({ type: 'failed', said: e.said, why: String(e) });
// The visit's key, made or unsealed by `recall` or `redeem` before any
// other run reads it.
/** @type {Uint8Array<ArrayBuffer> | undefined} */
let key;
/**
 * @param {typeof Wasm} w
 * @param {string} token
 * @param {Visit} v
 * @param {string} [at]
 * @returns {PageMessage}
 */
const stay = (w, token, v, at = w.issuer_of(token)) => ({
  type: 'redeemed',
  ended: v.until <= Date.now() / 1000,
  why: `a guest of ${v.host} until ${v.until}, opening ${Object.keys(v.lent).join(', ')}`,
  stay: {
    host: v.host,
    until: v.until,
    lent: Object.entries(v.lent).map(([name, kind]) => ({
      key: at,
      relay: w.relay_of(token),
      name,
      kind,
      person: v.host,
      version: v.version,
      label: w.label(at, name),
    })),
  },
});
/**
 * @param {typeof Wasm} w
 * @param {string} token
 * @param {string} guest
 * @param {Uint8Array<ArrayBuffer>} [bytes]
 * @returns {Promise<Visit>}
 */
async function visiting(
  w,
  token,
  guest,
  bytes = new TextEncoder().encode(
    JSON.stringify({ ask: JSON.parse(w.ask(/** @type {Uint8Array} */ (key), token)), guest }),
  ),
) {
  const d = await Promise.race([
    w.connect_to_join(/** @type {Uint8Array} */ (key), token),
    new Promise((_, n) => setTimeout(n, 60_000)),
  ]).catch(() => said('The computer that sent this link did not answer. Try again later.'));
  const res = await d
    .fetch(
      'POST',
      '/visit',
      [
        ['content-type', 'application/json'],
        ['content-length', String(bytes.length)],
      ],
      new Blob([bytes]).stream(),
    )
    .finally(() => d.close());
  return res.ok
    ? res.json()
    : said(
        res.status === 403
          ? 'This link lets nobody in. Ask for a new one.'
          : 'The computer that sent this link cannot let anyone in just now. Try again later.',
      );
}
/** @type {Io<Effect, PageMessage>} */
export const io = {
  recall: ({ token }) =>
    wasm()
      .then((w) =>
        (w.relay_of(token)
          ? store.visitOf(token)
          : said('This link was made by an older svrngs. Ask whoever sent it for a new one.')
        ).then((kept) =>
          kept
            ? ((key = kept.key), stay(w, token, kept))
            : /** @satisfies {PageMessage} */ ({
                type: 'unkept',
                why: 'no visit is kept for this link',
              }),
        ),
      )
      .catch(failing),
  redeem: ({ token, guest }) =>
    wasm()
      // `v` is a local, held as a parameter no caller passes.
      .then(
        /** @type {(w: typeof Wasm) => Promise<PageMessage>} */ (
          async (/** @type {typeof Wasm} */ w, /** @type {Visit | undefined} */ v) =>
            (
              (key = crypto.getRandomValues(new Uint8Array(32))),
              (v = await visiting(w, token, guest)),
              await store.keepVisit(token, v, /** @type {Uint8Array<ArrayBuffer>} */ (key)),
              stay(w, token, v)
            )
        ),
      )
      .catch(failing),
  over: ({ until }) =>
    new Promise((r) => setTimeout(r, until * 1000 - Date.now())).then(() => ({
      type: 'ended',
      why: `the visit ended at ${until}`,
    })),
  call: ({ token, lent, label, until }) =>
    handing(
      { group: { token }, lent, unanswered: [], label },
      /** @type {Uint8Array} */ (key),
      until,
    ).then((m) =>
      m.type === 'handed'
        ? /** @satisfies {PageMessage} */ ({ ...m, type: 'called' })
        : /** @satisfies {PageMessage} */ ({ type: 'failed', said: m.said, why: m.said }),
    ),
};
