// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// The index of what the group lends, and opening one (page-index (a)):
// web/app.js's list (:530-541), a line each, and `open` (:227-284). Each
// line is its open, named, then what opening it agrees to (`agreed`),
// " Full now." when it is, and on a fine pointer the address to keep;
// drawn from state's `lent`, so each read draws it again. `open` takes a
// lend's label and, from `#open=` (page-join), the path to go to, never a
// closure's lend. The pass, the holder and the hand-off are the edge's.
// A guest's lends are drawn here too (web/page/visit.js `listed`).
import {
  agreed,
  alias,
  aliased,
  go,
  html,
  Keep,
  named,
  no,
  once,
  opens,
  Row,
} from '../ui/kit.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { handoffCookie, sealOpen } from '../authority.js?v=__VERSION__';
import { listed } from './visit.js?v=__VERSION__';

/**
 * The fields this flow owns (`owns`): the label of the lend opening, and
 * why it did not open.
 * @typedef {{ opening?: string, unopened?: string }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'open', label: string, at?: string }
 *   | { type: 'handed' }
 *   | { type: 'unopened', said?: string }} Message
 */

// How long a pass opens a lend, in seconds (app.js@19b63b2b:75): its origin asks for
// another after, which the page signs without a word to the person.
export const PASS_LIFE = 12 * 60 * 60;

export const owns = ['opening', 'unopened'];
export const states = /** @type {const} */ ({ open: ['shut', 'opening', 'handed', 'refused'] });
export const takes = {
  open: (s, { label, at }, l = s.lent?.find((o) => o.label === label)) =>
    !l
      ? no(s, `nothing ${s.group?.name ?? 'here'} lends is ${label}`)
      : s.at.open === 'opening'
        ? no(s, `${s.opening} is opening already`)
        : go(
            s,
            { open: 'opening' },
            { opening: label, unopened: '' },
            {
              run: 'hand',
              group: s.group,
              lent: s.lent,
              unanswered: s.unanswered ?? [],
              label,
              at,
              failed: 'unopened',
            },
          ),
  handed: (s) =>
    s.at.open === 'opening' ? go(s, { open: 'handed' }) : no(s, 'nothing is opening'),
  unopened: (s, m, l = s.lent.find((o) => o.label === s.opening)) =>
    s.at.open === 'opening'
      ? go(s, { open: 'refused' }, { unopened: m.said ?? `${named(l)} did not open: ${m.why}` })
      : no(s, 'nothing is opening'),
};
// "Opening …" until the page has gone, or why it did not (app.js@19b63b2b:238).
export const response = {
  enter: (s, l = s.lent?.find((o) => o.label === s.opening)) =>
    s.at.open === 'refused' ? [s.unopened] : l && s.at.open !== 'shut' && [`Opening ${named(l)}…`],
};

// The page's own address, which only the edge reads; and the address to
// keep for `l` there (svrn-docs rings/current/IN_THE_BROWSER.md §4): its
// alias, or, where two share one, its label, which never collides; the
// alias as the page reads it, a space as `+` (app.js@19b63b2b:530).
const here = () => `${location.origin}${location.pathname}`;
const kept = (s, l, a = alias(l, s.lent, s.people)) =>
  `${here()}#open=${aliased(a, s.lent, s.people).length === 1 ? encodeURIComponent(a).replaceAll('%20', '+').replaceAll('%40', '@') : l.label}`;
export const view = {
  lent: ({ s, send }) =>
    html`<table class="index"><tbody id="lent">${listed(s, send)}${once(s.lent ?? []).map((l) => html`<${Row} key=${l.label} cells=${[html`<span><button onClick=${() => send({ type: 'open', label: l.label })}>${named(l)}</button>${` ${agreed(l, s.shelf, s.group)}${opens(l)[1] === 'full' ? ' Full now.' : ''}`}${s.fine && html`${' '}<${Keep} at=${kept(s, l)} />`}</span>`]} />`)}</tbody></table>`,
};

// The edge (app.js@19b63b2b:234-284): the yes the open is, kept with the group when the
// lend reads others' and none was kept for a `#open=`; a pass to the lend's
// origin for it and the lends of each kind it reads, signed by `key`, which
// never enters state; and the hand-off in the fragment, which no origin is
// sent. The holder is the lend origin's one key (capability-signed (H)),
// its bytes sent only when it holds none or a lend is on an older svrngs.
const trace = (l) => globalThis.__trace?.(l),
  wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');
export async function handing(
  { group, lent, unanswered, label, at },
  key,
  until = Math.floor(Date.now() / 1000) + PASS_LIFE,
) {
  const w = await wasm(),
    l = lent.find((o) => o.label === label);
  if (!('serviceWorker' in navigator))
    return { type: 'unopened', said: `This browser cannot open ${named(l)}.` };
  const yes = l.reads?.length && (at === undefined || !group.reads?.[label]);
  const g = yes
    ? await store.opened({
        ...group,
        reads: { ...group.reads, [label]: { said: 'yes', at: Math.floor(Date.now() / 1000) } },
      })
    : group;
  const stopped = g.reads?.[label]?.said === 'stop';
  if (stopped) trace(`PAGE: ${l.name}'s reads were stopped here, so its pass names none`);
  const lends = [
    l,
    ...(stopped ? [] : lent.filter((o) => o !== l && (l.reads ?? []).includes(o.kind))),
  ].map((o) => ({
    label: o.label,
    key: o.key,
    relay: o.relay ?? null,
    name: o.name,
    kind: o.kind,
    group: o.group ?? null,
    person: o.person,
    asks: o.asks,
    version: o.version ?? null,
  }));
  const raw = lends.some((o) => !w.links(o.version)),
    had = !raw && (await store.holder(label));
  const seed = had ? null : crypto.getRandomValues(new Uint8Array(32)),
    holder = had || w.actor(seed);
  if (raw)
    trace(
      'PAGE: a lend of this open is at a computer on an older svrngs, so its holder goes as bytes',
    );
  if (!had && !raw) await store.holder(label, holder);
  const start = `${store.lendOrigin(location.protocol, label, location.host)}/open.html?v=__VERSION__${at ? `&back=${encodeURIComponent(at)}` : ''}`;
  // This open's approver (HOLDER_APPROVAL §3.D): named in both passes, its
  // seed sealed into the open and its sealing key left in a cookie for the
  // group's signer alone, which approves the app's entries with it.
  const approver = crypto.getRandomValues(new Uint8Array(32)),
    approves = w.actor(approver);
  const handoff = {
    label,
    lends,
    pass: w.pass(
      key,
      holder,
      label,
      lends.slice(1).map((o) => o.label),
      until,
      approves,
    ),
    // An app on a port writes its kind's part of the record as you by naming the act (DESIGN §3, "Lent apps"); a guest's open, naming no group, writes none.
    writes:
      l.group || !group.id
        ? null
        : w.pass(key, holder, w.carried_label(l.key, l.kind, group.id), [], until, approves),
    group: group.id,
    holder,
    seed: seed && store.hex(seed),
    raw,
    relay: w.relay_of(group.token) ?? null,
    page: here(),
    unanswered: unanswered.map((u) => u.person ?? u),
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
export const io = { hand: (m) => store.unseal(m.group).then((key) => handing(m, key)) };
