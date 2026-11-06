// SPDX-License-Identifier: AGPL-3.0-or-later
// The open handed to its lend's origin: the edge both a member's open
// (web/page/index.js `io`) and a guest's (web/page/visit.js `call`) end
// in. A module of its own, as both flows import it and index.js draws
// visit.js's `listed`.
import { named } from '../question.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { handoffCookie, sealOpen } from '../authority.js?v=__VERSION__';
import { trace, wasm } from './dial.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Handoff } from '../authority.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Lent, Whom } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage } from './page.js?v=__VERSION__' */

/**
 * What an open hands: the group it is opened in, what it lends, whose
 * computers did not answer, the lend's label, and the path to open it at.
 * A guest's open names its link's token alone (web/page/visit.js `call`).
 * @typedef {object} Hand
 * @property {Partial<Group> & { token: string }} group
 * @property {Lent[]} lent
 * @property {Whom[]} unanswered
 * @property {string} label
 * @property {string} [at]
 */

// How long a pass opens a lend, in seconds (app.js@19b63b2b:75): its origin asks for
// another after, which the page signs without a word to the person.
export const PASS_LIFE = 12 * 60 * 60;

// The page's own address, which the edge sends and index.js keeps.
export const here = () => `${location.origin}${location.pathname}`;

// The edge (app.js@19b63b2b:234-284): the yes the open is, kept with the group when the
// lend reads others' and none was kept for a `#open=`; a pass to the lend's
// origin for it and the lends of each kind it reads, signed by `key`, which
// never enters state; and the hand-off in the fragment, which no origin is
// sent. The holder is the lend origin's one key (capability-signed (H)),
// its bytes sent only when it holds none or a lend is on an older svrngs.
/**
 * @param {Hand} hand
 * @param {Uint8Array} key the person's key, unsealed
 * @param {number} [until] when the pass ends, in seconds
 * @returns {Promise<Extract<PageMessage, { type: 'handed' | 'unopened' }>>}
 */
export async function handing(hand, key, until = Math.floor(Date.now() / 1000) + PASS_LIFE) {
  const { group, lent, unanswered, label, at } = hand;
  // The lend named is one listed: `open` refuses any other.
  const core = await wasm();
  const lend = /** @type {Lent} */ (lent.find((one) => one.label === label));
  if (!('serviceWorker' in navigator))
    return { type: 'unopened', said: `This browser cannot open ${named(lend)}.` };
  const opened = await withYes(hand, lend);
  const lends = passLends(core, hand, lend, opened);
  const { holder, seed, raw } = await holderOf(core, label, lends);
  const start = `${store.lendOrigin(location.protocol, label, location.host)}/open.html?v=__VERSION__${at ? `&back=${encodeURIComponent(at)}` : ''}`;
  // This open's approver (HOLDER_APPROVAL §3.D): named in both passes, its
  // seed sealed into the open and its sealing key left in a cookie for the
  // group's signer alone, which approves the app's entries with it.
  const approver = crypto.getRandomValues(new Uint8Array(32));
  const approves = core.actor(approver);
  // Each pass of this open: signed by `key`, to `holder`, until `until`,
  // naming this open's approver.
  /** @type {Signed} */
  const signed = (grant, reads) => core.pass(key, holder, grant, reads, until, approves);
  /** @type {Handoff} */
  const handoff = {
    label,
    lends,
    pass: signed(
      label,
      lends.slice(1).map((one) => one.label),
    ),
    writes: writesOf(core, lend, group, signed),
    group: group.id,
    holder,
    seed: seed && store.hex(seed),
    raw,
    relays: core.relays_of(group.token, group.meets),
    page: here(),
    unanswered: unanswered.map((who) => who.person),
  };
  trace(`PAGE: opening ${label} at ${start}, a pass to ${lends.length} lend(s) until ${until}`);
  const sealed = await sealOpen(handoff, approver);
  handoff.proof = sealed.proof;
  document.cookie = `${handoffCookie(label)}=${sealed.w}; Secure; Path=/; SameSite=Strict; Max-Age=${until - Math.floor(Date.now() / 1000)}`;
  const fragment = btoa(String.fromCharCode(...new TextEncoder().encode(JSON.stringify(handoff))))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  location[at === undefined ? 'assign' : 'replace'](`${start}#${fragment}`);
  return { type: 'handed', why: `to ${start}` };
}
// The yes the open is, kept with the group when the lend reads others'
// and none was kept for a `#open=`: the group as it is kept now.
/**
 * @param {Hand} hand
 * @param {Lent} lend
 */
async function withYes({ group, label, at }, lend) {
  const yes = lend.reads && lend.reads.length > 0 && (at === undefined || !group.reads?.[label]);
  // A guest's lends read none (web/page/visit.js `stay`), so the yes is a group's.
  if (!yes) return group;
  return store.opened({
    .../** @type {Group} */ (group),
    reads: { ...group.reads, [label]: { said: 'yes', at: Math.floor(Date.now() / 1000) } },
  });
}

// The lends the pass names: the lend, and the lends of each kind it
// reads, unless the group stopped its reads. A relay the page's policy
// refuses goes as none, so the worker dials the taken routes instead of a
// blocked one (a worker has no document to judge that itself).
/**
 * @param {typeof Wasm} core
 * @param {Hand} hand
 * @param {Lent} lend
 * @param {Partial<Group>} opened
 */
function passLends(core, { lent, label }, lend, opened) {
  const stopped = opened.reads?.[label]?.said === 'stop';
  if (stopped) trace(`PAGE: ${lend.name}'s reads were stopped here, so its pass names none`);
  return [
    lend,
    ...(stopped
      ? []
      : lent.filter((other) => other !== lend && (lend.reads ?? []).includes(other.kind))),
  ].map((other) => ({
    label: other.label,
    key: other.key,
    relay: taken(core, other),
    name: other.name,
    kind: other.kind,
    group: other.group ?? null,
    person: other.person,
    asks: other.asks,
    version: other.version ?? null,
  }));
}

// The relay the record names for `lend` when this page's policy takes it,
// else none; one refused is said by name, so the worker's "names no relay"
// is read rightly (review F6).
/**
 * @param {typeof Wasm} core
 * @param {Lent} lend
 */
function taken(core, { name, relay }) {
  if (!relay || core.relay_taken(relay)) return relay ?? null;
  trace(`PAGE: ${name}'s relay ${relay} is one this page's policy refuses, so its pass names none`);
  return null;
}

// The holder of the lend origin's open, kept for `label` unless a lend of
// it is on an older svrngs (`raw`), when it goes as bytes, its `seed`; a
// new one when none is kept.
/**
 * @param {typeof Wasm} core
 * @param {string} label
 * @param {{ version: number | null }[]} lends
 */
async function holderOf(core, label, lends) {
  const raw = lends.some((one) => !core.links(one.version));
  const had = raw ? undefined : await store.holder(label);
  const seed = had ? null : crypto.getRandomValues(new Uint8Array(32));
  // With no holder had, a seed was made.
  const holder = had ?? core.actor(/** @type {Uint8Array} */ (seed));
  if (raw)
    trace(
      'PAGE: a lend of this open is at a computer on an older svrngs, so its holder goes as bytes',
    );
  if (!had && !raw) await store.holder(label, holder);
  return { holder, seed, raw };
}

/** @typedef {(grant: string, reads: string[]) => string} Signed */
// An app on a port writes its kind's part of the record as you by naming
// the act (DESIGN §3, "Lent apps"): the pass that lets it, `signed`. A
// group's own lend, or a guest's open, naming no group, writes none.
/**
 * @param {typeof Wasm} core
 * @param {Lent} lend
 * @param {Hand['group']} group
 * @param {Signed} signed
 */
const writesOf = (core, lend, group, signed) =>
  lend.group || !group.id ? null : signed(core.carried_label(lend.key, lend.kind, group.id), []);
