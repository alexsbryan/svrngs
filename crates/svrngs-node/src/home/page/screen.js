// SPDX-License-Identifier: AGPL-3.0-or-later
// A group on a screen (page-members (e); home.js `screen` and its link,
// :203 and :294-329, which it moves), for a TV, a spare laptop or a
// projector: its link at the foot of the group's section (`show`), opened
// in a tab of its own at `/?screen=<its id>`, where HOME draws this alone
// (home.js `Home`). Its door alone, big, each person's lends in their
// window (ui/door.js), read by SCREEN, a machine of the screen's own: each
// read asked by an edge timer a second after the last one began, `tick`, or
// as it lands if later (demo/runs/twentyfive/2026-10-17/README.md: read back
// to back, a 25-computer group's founder spent 180% of a core). People and
// lends are read together and drawn as one, so the door is judged before it
// takes the last one's place, and no window reads "asking…" again. Beside
// it, who joined and what was lent since the screen opened (`seen`, by key:
// a person's, or a lend's origin), at this computer's time, which the edge
// reads, newest first, the newest marked new there and in its window
// (`newest`): kept by this page alone, so opened again it starts over. The
// first read's were here before it opened, so no line says them. The page's
// class and title are set at the edge (`look`), not by a view.
import {
  ask,
  assertNever,
  called,
  counted,
  Door,
  go,
  html,
  late,
  machine,
  no,
  useMachine,
  whose,
} from '../ui/kit.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
/** @import { Group, Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * Who or what arrived, by its key, a person's or a lend's origin, and how
 * a line says it arrived.
 * @typedef {[key: string, words: string]} Seen
 */

/**
 * SCREEN's state: where its read is, the group's id, the people and lends
 * last read, every key seen since the screen opened, what arrived since,
 * newest first, each at its time, the newest's key, and why the last read
 * failed.
 * @typedef {object} State
 * @property {{ screen: 'reading' | 'read' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {string | undefined} id
 * @property {string[]} seen
 * @property {[time: string, words: string][]} since
 * @property {PeopleRow[]} [rows]
 * @property {Lends} [lends]
 * @property {string} [newest]
 * @property {string} [why]
 */

/**
 * The messages SCREEN takes: the page's look set; the timer's tick; read
 * `n` of the people and lends, begun at `began`, at this computer's
 * `time`; or why not.
 * @typedef {{ type: 'looked', why: string }
 *   | { type: 'tick' }
 *   | { type: 'read', n: number, rows: PeopleRow[], lends: Lends, began: number, time: string, why: string }
 *   | { type: 'unread', n: number, began: number, why: string }} Message
 */

/**
 * The effects its step answers: the page's class and title set to
 * `title`; read `n` of the group `id`; and a tick at `at`, in
 * milliseconds.
 * @typedef {{ run: 'look', title: string }
 *   | { run: 'read', n: number, id: string, failed: 'unread' }
 *   | { run: 'tick', at: number, failed: 'unread' }} Effect
 */

/**
 * @param {number} n
 * @param {string} one
 * @param {string} more
 */
const many = (n, one, more) => `${n} ${n === 1 ? one : more}`;
// Who is in and what is lent, each once by its key, with how a line says it arrived.
/**
 * @param {PeopleRow[]} rows
 * @param {Lent[]} lent
 */
const keyed = (rows, lent) =>
  [
    ...rows.map(
      /** @returns {Seen} */ (r) => [r.person_key, `${called(rows, r.person_key)} joined`],
    ),
    ...lent
      .filter((l) => !l.group)
      .map(
        /** @returns {Seen} */ (l) => [
          l.origin,
          `${called(rows, rows.find((r) => r.key === l.key)?.person_key) ?? l.person} lent ${l.name}`,
        ],
      ),
  ].filter(([k], i, all) => all.findIndex(([j]) => j === k) === i);
/** @type {Machine<State, Message, Effect>} */
const SCREEN = {
  states: { screen: ['reading', 'read', 'unread'] },
  step: (s, m) => {
    switch (m.type) {
      case 'looked':
        return go(s, {});
      case 'tick': {
        const [asked, n] = ask(s, 'screen');
        // A tick follows a read, run only for a group found (`Screen`).
        return go(
          s,
          {},
          { asked },
          { run: 'read', n, id: /** @type {string} */ (s.id), failed: 'unread' },
        );
      }
      case 'read': {
        const all = keyed(m.rows, m.lends.lent);
        const now = s.at.screen === 'reading' ? [] : all.filter(([k]) => !s.seen.includes(k));
        return late(s, 'screen', m.n)
          ? no(s, late(s, 'screen', m.n))
          : go(
              s,
              { screen: 'read' },
              {
                rows: m.rows,
                lends: m.lends,
                seen: [...new Set([...s.seen, ...all.map(([k]) => k)])],
                since: [
                  ...now
                    .map(/** @returns {[time: string, words: string]} */ ([, w]) => [m.time, w])
                    .reverse(),
                  ...s.since,
                ],
                newest: now.at(-1)?.[0] ?? s.newest,
              },
              { run: 'tick', at: m.began + 1000, failed: 'unread' },
            );
      }
      case 'unread':
        return late(s, 'screen', m.n)
          ? no(s, late(s, 'screen', m.n))
          : go(
              s,
              { screen: 'unread' },
              { why: m.why },
              { run: 'tick', at: m.began + 1000, failed: 'unread' },
            );
      default:
        return assertNever(s, m, 'a screen');
    }
  },
  io: {
    look: ({ title }) => (
      ([document.body.className, document.title] = ['screen', title]),
      { type: 'looked', why: title }
    ),
    read: ({ n, id }, began = Date.now(), ns = encodeURIComponent(id)) =>
      Promise.all([get(`/v1/rail/people?namespace=${ns}`), get(`/v1/lends?namespace=${ns}`)]).then(
        ([{ people: rows }, lends]) => ({
          type: 'read',
          n,
          rows,
          lends,
          began,
          time: new Date().toLocaleTimeString([], { timeStyle: 'short' }),
          why: `screen ${n}`,
        }),
        (e) => ({ type: 'unread', n, began, why: e.message }),
      ),
    tick: ({ at }) =>
      new Promise((r) => setTimeout(r, at - Date.now())).then(() => ({ type: 'tick' })),
  },
};

/** @param {HOME.PlaceProps} props */
function Screen({ s: { groups, q } }) {
  // HOME draws the screen once the groups are read (home.js `Home`).
  const g = /** @type {Group[]} */ (groups).find((x) => x.id === q.screen),
    here = g && !g.left && g.removed === false;
  const [s] = useMachine(() =>
    machine(
      'SCREEN',
      SCREEN,
      { at: { screen: 'reading' }, asked: { screen: 1 }, id: g?.id, seen: [], since: [] },
      [
        { run: 'look', title: `${g?.name ?? 'A group'} on a screen` },
        .../** @satisfies {Effect[]} */ (
          here ? [{ run: 'read', n: 1, id: g.id, failed: 'unread' }] : []
        ),
      ],
    ),
  );
  if (!here)
    return html`<p>This computer cannot show that group on a screen: you are not in it here.</p>`;
  // `s.lends` is read with `s.rows`, by one read ('read' sets both), so it
  // is there whenever they are, and so is each of `counted`'s numbers.
  const [n, on, things] = s.rows
    ? counted(s.rows, /** @type {Lends} */ (s.lends).lent, s.lends)
    : [];
  const count =
    s.at.screen === 'unread'
      ? `This computer cannot read ${g.name ?? 'this group'} now: ${s.why}`
      : s.rows
        ? `${many(/** @type {number} */ (n), 'person', 'people')} · ${on} on now · ${many(/** @type {number} */ (things), 'thing', 'things')} lent`
        : '';
  return html`<section data-colour=${[...g.id].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 12, 0)}><h2>${g.name ?? 'A group whose name this computer cannot read'}${g.made && html`<small> (${whose(g.made)})</small>`}</h2><p>${count}</p><div>${s.rows && html`<${Door} rows=${s.rows} lent=${/** @type {Lends} */ (s.lends).lent} lends=${s.lends} newest=${s.newest} />`}</div><aside><h2>Since this screen opened</h2><p>Kept on this screen only: it starts over when the screen is opened again.</p><ol>${s.since.map(([t, w], i) => html`<li><time>${t}</time> ${w} ${i === 0 && html`<mark>new</mark>`}</li>`)}</ol><p>Nothing new yet. Who joins, and what they lend, shows here as it happens.</p></aside><footer><p>Lit: answered just now. Dashed: did not answer. Dotted: in from a phone.</p><p>Each thing lent opens in a tab as ${g.person}, from this computer.</p></footer></section>`;
}

/** @type {{ screen: (props: HOME.PlaceProps) => Drawn, show: (props: PlaceProps) => Drawn }} */
export const view = {
  screen: Screen,
  show: ({ g, s }) =>
    s.lends &&
    html`<p><a href=${`/?screen=${encodeURIComponent(g.id)}`} target="_blank">Show ${g.name ?? 'this group'} on a screen</a></p>`,
};
