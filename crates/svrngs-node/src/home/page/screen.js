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
 * @param {number} count
 * @param {string} one
 * @param {string} more
 */
const many = (count, one, more) => `${count} ${count === 1 ? one : more}`;
// Who is in and what is lent, each once by its key, with how a line says it arrived.
/**
 * @param {PeopleRow[]} rows
 * @param {Lent[]} lent
 */
const keyed = (rows, lent) =>
  [
    ...rows.map(
      /** @returns {Seen} */ (row) => [row.person_key, `${called(rows, row.person_key)} joined`],
    ),
    ...lent
      .filter((lend) => !lend.group)
      .map(
        /** @returns {Seen} */ (lend) => [
          lend.origin,
          `${called(rows, rows.find((row) => row.key === lend.key)?.person_key) ?? lend.person} lent ${lend.name}`,
        ],
      ),
  ].filter(([key], i, all) => all.findIndex(([other]) => other === key) === i);
/** @type {Machine<State, Message, Effect>} */
const SCREEN = {
  states: { screen: ['reading', 'read', 'unread'] },
  step: (state, message) => {
    switch (message.type) {
      case 'looked':
        return go(state, {});
      case 'tick': {
        const [asked, serial] = ask(state, 'screen');
        // A tick follows a read, run only for a group found (`Screen`).
        return go(
          state,
          {},
          { asked },
          { run: 'read', n: serial, id: /** @type {string} */ (state.id), failed: 'unread' },
        );
      }
      case 'read': {
        const all = keyed(message.rows, message.lends.lent);
        const now =
          state.at.screen === 'reading' ? [] : all.filter(([key]) => !state.seen.includes(key));
        return late(state, 'screen', message.n)
          ? no(state, late(state, 'screen', message.n))
          : go(
              state,
              { screen: 'read' },
              {
                rows: message.rows,
                lends: message.lends,
                seen: [...new Set([...state.seen, ...all.map(([key]) => key)])],
                since: [
                  ...now
                    .map(
                      /** @returns {[time: string, words: string]} */ ([, words]) => [
                        message.time,
                        words,
                      ],
                    )
                    .reverse(),
                  ...state.since,
                ],
                newest: now.at(-1)?.[0] ?? state.newest,
              },
              { run: 'tick', at: message.began + 1000, failed: 'unread' },
            );
      }
      case 'unread':
        return late(state, 'screen', message.n)
          ? no(state, late(state, 'screen', message.n))
          : go(
              state,
              { screen: 'unread' },
              { why: message.why },
              { run: 'tick', at: message.began + 1000, failed: 'unread' },
            );
      default:
        return assertNever(state, message, 'a screen');
    }
  },
  io: {
    look: ({ title }) => (
      ([document.body.className, document.title] = ['screen', title]),
      { type: 'looked', why: title }
    ),
    read: ({ n: serial, id }, began = Date.now(), ns = encodeURIComponent(id)) =>
      Promise.all([get(`/v1/rail/people?namespace=${ns}`), get(`/v1/lends?namespace=${ns}`)]).then(
        ([{ people: rows }, lends]) => ({
          type: 'read',
          n: serial,
          rows,
          lends,
          began,
          time: new Date().toLocaleTimeString([], { timeStyle: 'short' }),
          why: `screen ${serial}`,
        }),
        (error) => ({ type: 'unread', n: serial, began, why: error.message }),
      ),
    tick: ({ at }) =>
      new Promise((resolve) => setTimeout(resolve, at - Date.now())).then(() => ({ type: 'tick' })),
  },
};

/** @param {HOME.PlaceProps} props */
function Screen({ s: { groups, q: query } }) {
  // HOME draws the screen once the groups are read (home.js `Home`).
  const group = /** @type {Group[]} */ (groups).find((one) => one.id === query.screen);
  const here = group && !group.left && group.removed === false;
  const [state] = useMachine(() =>
    machine(
      'SCREEN',
      SCREEN,
      { at: { screen: 'reading' }, asked: { screen: 1 }, id: group?.id, seen: [], since: [] },
      [
        { run: 'look', title: `${group?.name ?? 'A group'} on a screen` },
        .../** @satisfies {Effect[]} */ (
          here ? [{ run: 'read', n: 1, id: group.id, failed: 'unread' }] : []
        ),
      ],
    ),
  );
  if (!here)
    return html`<p>This computer cannot show that group on a screen: you are not in it here.</p>`;
  // `state.lends` is read with `state.rows`, by one read ('read' sets both), so it
  // is there whenever they are, and so is each of `counted`'s numbers.
  const [people, on, things] = state.rows
    ? counted(state.rows, /** @type {Lends} */ (state.lends).lent, state.lends)
    : [];
  const count =
    state.at.screen === 'unread'
      ? `This computer cannot read ${group.name ?? 'this group'} now: ${state.why}`
      : state.rows
        ? `${many(/** @type {number} */ (people), 'person', 'people')} · ${on} on now · ${many(/** @type {number} */ (things), 'thing', 'things')} lent`
        : '';
  return html`<section data-colour=${[...group.id].reduce((hash, char) => (hash * 31 + char.charCodeAt(0)) % 12, 0)}><h2>${group.name ?? 'A group whose name this computer cannot read'}${group.made && html`<small> (${whose(group.made)})</small>`}</h2><p>${count}</p><div>${state.rows && html`<${Door} rows=${state.rows} lent=${/** @type {Lends} */ (state.lends).lent} lends=${state.lends} newest=${state.newest} />`}</div><aside><h2>Since this screen opened</h2><p>Kept on this screen only: it starts over when the screen is opened again.</p><ol>${state.since.map(([tone, words], i) => html`<li><time>${tone}</time> ${words} ${i === 0 && html`<mark>new</mark>`}</li>`)}</ol><p>Nothing new yet. Who joins, and what they lend, shows here as it happens.</p></aside><footer><p>Lit: answered just now. Dashed: did not answer. Dotted: in from a phone.</p><p>Each thing lent opens in a tab as ${group.person}, from this computer.</p></footer></section>`;
}

/** @type {{ screen: (props: HOME.PlaceProps) => Drawn, show: (props: PlaceProps) => Drawn }} */
export const view = {
  screen: Screen,
  show: ({ g: group, s: state }) =>
    state.lends &&
    html`<p><a href=${`/?screen=${encodeURIComponent(group.id)}`} target="_blank">Show ${group.name ?? 'this group'} on a screen</a></p>`,
};
