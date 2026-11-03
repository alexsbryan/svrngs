// SPDX-License-Identifier: AGPL-3.0-or-later
// The index of what the group lends, and opening one (page-index (a)):
// web/app.js's list (:530-541), a line each, and `open` (:227-284). Each
// line is its open, named, then what opening it agrees to (`agreed`),
// " Full now." when it is, and on a fine pointer the address to keep;
// drawn from state's `lent`, so each read draws it again. `open` takes a
// lend's label and, from `#open=` (page-join), the path to go to, never a
// closure's lend. The pass, the holder and the hand-off are the edge's.
// A guest's lends are drawn here too (web/page/visit.js `listed`).
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { Keep, Row } from '../ui/parts.js?v=__VERSION__';
import { agreed, alias, aliased, named, once, opens } from '../question.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { handoffCookie, sealOpen } from '../authority.js?v=__VERSION__';
import { listed } from './visit.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Handoff } from '../authority.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lent, Whom } from './enter.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the label of the lend opening, and
 * why it did not open.
 * @typedef {{ opening?: string, unopened?: string }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'open', label: string, at?: string }
 *   | { type: 'handed' }
 *   | { type: 'unopened', said?: string, why?: string }} Message
 */

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

/**
 * The effect its steps answer: the open, handed.
 * @typedef {Hand & { run: 'hand', group: Group, failed: string }} Effect
 */

// How long a pass opens a lend, in seconds (app.js@19b63b2b:75): its origin asks for
// another after, which the page signs without a word to the person.
export const PASS_LIFE = 12 * 60 * 60;

export const owns = ['opening', 'unopened'];
export const states = /** @type {const} */ ({ open: ['shut', 'opening', 'handed', 'refused'] });
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  open: (state, { label, at }, lend = state.lent?.find((one) => one.label === label)) => {
    if (!lend) return no(state, `nothing ${state.group?.name ?? 'here'} lends is ${label}`);
    if (state.at.open === 'opening') return no(state, `${state.opening} is opening already`);
    return go(
      state,
      { open: 'opening' },
      { opening: label, unopened: '' },
      {
        run: 'hand',
        group: state.group,
        // `lend` is found in it, so the list is read.
        lent: /** @type {Lent[]} */ (state.lent),
        unanswered: state.unanswered ?? [],
        label,
        at,
        failed: 'unopened',
      },
    );
  },
  handed: (state) =>
    state.at.open === 'opening' ? go(state, { open: 'handed' }) : no(state, 'nothing is opening'),
  // Opening, the lend opening is one listed (`open`).
  unopened: (
    state,
    message,
    lend = /** @type {Lent} */ (
      /** @type {Lent[]} */ (state.lent).find((one) => one.label === state.opening)
    ),
  ) =>
    state.at.open === 'opening'
      ? go(
          state,
          { open: 'refused' },
          { unopened: message.said ?? `${named(lend)} did not open: ${message.why}` },
        )
      : no(state, 'nothing is opening'),
};
// "Opening …" until the page has gone, or why it did not (app.js@19b63b2b:238).
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  // Refused, it says why (`unopened`).
  enter: (state, lend = state.lent?.find((one) => one.label === state.opening)) =>
    state.at.open === 'refused'
      ? [/** @type {string} */ (state.unopened)]
      : lend && state.at.open !== 'shut' && [`Opening ${named(lend)}…`],
};

// The page's own address, which only the edge reads; and the address to
// keep for `lend` there (svrn-docs rings/current/IN_THE_BROWSER.md §4): its
// alias, or, where two share one, its label, which never collides; the
// alias as the page reads it, a space as `+` (app.js@19b63b2b:530).
const here = () => `${location.origin}${location.pathname}`;
// Drawn only once the list is read (`view`).
/**
 * @param {State} state
 * @param {Lent} lend
 * @param {string} [name]
 */
const kept = (state, lend, name = alias(lend, /** @type {Lent[]} */ (state.lent), state.people)) =>
  `${here()}#open=${aliased(name, /** @type {Lent[]} */ (state.lent), state.people).length === 1 ? encodeURIComponent(name).replaceAll('%20', '+').replaceAll('%40', '@') : lend.label}`;
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  lent: ({ s: state, send }) =>
    html`<table class="index"><tbody id="lent">${listed(state, send)}${once(state.lent ?? []).map((lend) => html`<${Row} key=${lend.label} cells=${[html`<span><button onClick=${() => send({ type: 'open', label: lend.label })}>${named(lend)}</button>${` ${agreed(lend, state.shelf, state.group)}${opens(lend)[1] === 'full' ? ' Full now.' : ''}`}${state.fine && html`${' '}<${Keep} at=${kept(state, lend)} />`}</span>`]} />`)}</tbody></table>`,
};

// The edge (app.js@19b63b2b:234-284): the yes the open is, kept with the group when the
// lend reads others' and none was kept for a `#open=`; a pass to the lend's
// origin for it and the lends of each kind it reads, signed by `key`, which
// never enters state; and the hand-off in the fragment, which no origin is
// sent. The holder is the lend origin's one key (capability-signed (H)),
// its bytes sent only when it holds none or a lend is on an older svrngs.
const trace = (/** @type {string} */ line) => globalThis.__trace?.(line);
const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');
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
  const lends = passLends(hand, lend, opened);
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
    relay: core.relay_of(group.token) ?? null,
    page: here(),
    unanswered: unanswered.map((who) => who.person ?? who),
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
  const yes = lend.reads?.length && (at === undefined || !group.reads?.[label]);
  // A guest's lends read none (web/page/visit.js `stay`), so the yes is a group's.
  if (!yes) return group;
  return store.opened({
    .../** @type {Group} */ (group),
    reads: { ...group.reads, [label]: { said: 'yes', at: Math.floor(Date.now() / 1000) } },
  });
}

// The lends the pass names: the lend, and the lends of each kind it
// reads, unless the group stopped its reads.
/**
 * @param {Hand} hand
 * @param {Lent} lend
 * @param {Partial<Group>} opened
 */
function passLends({ lent, label }, lend, opened) {
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
    relay: other.relay ?? null,
    name: other.name,
    kind: other.kind,
    group: other.group ?? null,
    person: other.person,
    asks: other.asks,
    version: other.version ?? null,
  }));
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
  const had = !raw && (await store.holder(label));
  const seed = had ? null : crypto.getRandomValues(new Uint8Array(32));
  // With no holder had, a seed was made.
  const holder = had || core.actor(/** @type {Uint8Array} */ (seed));
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

/** @type {Io<Effect, PageMessage>} */
export const io = {
  hand: (message) => store.unseal(message.group).then((key) => handing(message, key)),
};
