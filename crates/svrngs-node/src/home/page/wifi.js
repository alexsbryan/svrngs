// SPDX-License-Identifier: AGPL-3.0-or-later
// What is open to this WiFi (DESIGN §3, "The room"): a lend opened to this
// WiFi, or closed, for any device on it, a TV among them, said as /v1/room
// says it (room.rs `Served::said`). ROOM is the page's one read of
// /v1/room, asked as this file loads, beside HOME's read of the groups, so
// a lend's cell has it to draw. It feeds two lists: each lend's cell in
// the index (`Address`, `ul.wifi`), and "Open to this WiFi" under "What
// acts as you" (`part`, for acting.js, `ul#room`). Each read is counted, so an older
// answer is refused by name, and is asked again on each of the groups'
// rounds, once a round however many cells ask. What is open that
// cannot be read is said once, in `part`, never read as none; a press in a
// line then writes it over.
import { html, useEffect, useStore } from '../ui/view.js?v=__VERSION__';
import { day, named } from '../question.js?v=__VERSION__';
import { go, machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Keep } from '../ui/parts.js?v=__VERSION__';
import { keyOf, same, turns } from './lend-key.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
import { Turn } from './play.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lent } from './group.js?v=__VERSION__' */

/**
 * A lend by its lender's computer's key and its name, as ROOM is asked to
 * turn it, and its lender's name when it is said.
 * @typedef {{ key: string, name: string, person?: string | null }} Which
 */

/**
 * Where a lend open to this WiFi is served now, or why not (room.rs
 * `Served`, serde's snake_case): at an address, or closed for now.
 * @typedef {{ at: string } | 'elsewhere' | 'unread' | 'no_address' | 'taken'} Served
 */

/**
 * A lend open to this WiFi, as /v1/room answers it (room.rs `InRoom`): its
 * lender, `null` for a key no group held here names now, where it is
 * served, and that said.
 * @typedef {object} InRoom
 * @property {string | null} person
 * @property {string} key
 * @property {string} name
 * @property {Served} served
 * @property {string} said
 */

/**
 * A turn answered: the lend it turned, and whether off.
 * @typedef {{ l: Which, off: boolean }} Turned
 */

/**
 * ROOM's state: where its read or turn is, the round last read, each
 * lend's refusals, by `keyOf`, what a press here closed, what is open, and,
 * while a turn is under way, where it was and the lend it turns.
 * @typedef {object} State
 * @property {{ room: 'reading' | 'read' | 'unread' | 'turning' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} round
 * @property {Which[]} closed
 * @property {Record<string, string>} said
 * @property {InRoom[]} [room]
 * @property {State['at']['room']} [was]
 * @property {Which} [l]
 * @property {string} [why]
 */

/**
 * What is open, read or turned `n`, and the turn it answers.
 * @typedef {{ type: 'answered', n: number, room: InRoom[], turned?: Turned, why: string }} Answered
 */

/**
 * The messages ROOM takes: a round's read; a line's press; what is open;
 * or a read or turn that failed, with why. A read and a turn carry no
 * read's number: `old` reads `message.n` of every message.
 * @typedef {{ type: 'read', round: number, n?: undefined }
 *   | { type: 'turn', l: Which, off: boolean, n?: undefined }
 *   | Answered
 *   | { type: 'unread', n: number, why: string }
 *   | { type: 'refused', n: number, why: string }} Message
 */

/**
 * The effects its step answers: read `n` of what is open, and turn `n`,
 * the lend `l` opened or closed to this WiFi.
 * @typedef {{ run: 'read', n: number, failed: 'unread' }
 *   | { run: 'turn', n: number, l: Which, off: boolean, failed: 'refused' }} Effect
 */

const answer =
  /**
   * @param {number} serial
   * @param {Turned} [turned]
   */
  (serial, turned) =>
    /**
     * @param {InRoom[]} room
     * @returns {Answered}
     */
    (room) => ({
      type: 'answered',
      n: serial,
      room,
      turned,
      why: `${room.length} open`,
    });
// A turn is answered by what is open then, written over what was read, and
// what it closed is kept, said closed in both lists until the page is left.
/** @type {Machine<State, Message, Effect>} */
export const ROOM = {
  states: { room: ['reading', 'read', 'unread', 'turning'] },
  io: {
    read: ({ n: serial }) => get('/v1/room').then(answer(serial)),
    turn: ({ n: serial, l: lend, off }) =>
      post('/v1/room', { [off ? 'off' : 'on']: { key: lend.key, name: lend.name } }).then(
        answer(serial, { l: lend, off }),
      ),
  },
  step: turns('room', {
    busy: 'what is open',
    taker: 'the room',
    refused: (_, why) => ` ${why}`,
    // A read turns nothing, and keeps what was closed.
    answered: (state, message) => {
      const { room: open } = message;
      const turned = /** @type {Turned | undefined} */ (message.turned);
      if (!turned) return go(state, { room: 'read' }, { room: open });
      const others = state.closed.filter((/** @type {Which} */ closed) => !same(closed)(turned.l));
      const closed = turned.off ? [...others, turned.l] : others;
      return go(state, { room: 'read' }, { room: open, closed });
    },
  }),
};
export const room = machine(
  'ROOM',
  ROOM,
  { at: { room: 'reading' }, asked: { room: 1 }, round: 1, closed: [], said: {} },
  [{ run: 'read', n: 1, failed: 'unread' }],
);
/**
 * @param {State} state
 * @param {Which} lend
 */
const on = (state, lend) => (state.at.room === 'unread' ? undefined : state.room?.find(same(lend)));

/**
 * One lend's line: the lend, and what is said of it before its press.
 * @typedef {{ l: Which, words: string }} LineProps
 */

const CLOSED = 'Closed to this WiFi: nobody on it can open it through this computer now. ';

// One lend's line, in either list, drawn from ROOM alone: what is open
// said, or closed, if a press here closed it, and why a turn was refused.
// What opening one does is said once above each list, never per line
// (`ANYONE`, `OPEN_IT`).
// It stays beside ROOM, whose store it reads, as `rooms` draws it, and
// web/tests/pages/rooms.test.mjs imports ROOM and `rooms` from this file.
/** @param {LineProps} props */
export function Line({ l: lend, words }) {
  const state = useStore(room);
  const open = on(state, lend);
  const said = open?.said ?? (state.closed.some(same(lend)) ? CLOSED : '');
  const doing = { state: state.at.room === 'turning' ? 'busy' : 'idle' };
  const turn = {
    type: 'turn',
    l: open ?? { key: lend.key, name: lend.name },
    off: Boolean(open),
  };
  const act = open ? 'Close it to this WiFi' : 'Open it to this WiFi';
  return html`
    <li>
      ${words}
      <span>
        ${`${said} `}
        <${Act}
          words=${act}
          aria=${{ 'aria-label': `${act}, ${words.slice(0, -1)}` }}
          doing=${doing}
          on=${turn}
          send=${room.dispatch}
        />
        ${state.said[keyOf(lend)]}
      </span>
    </li>
  `;
}

// "Address" in a lend's cell of the index (lend-name.js): a button that
// opens the browser's own popover, holding what is for other devices and
// programs and is no part of deciding to open the lend: first its name, so
// a wrong press shows at once, and when it was first lent, then a line for each place a person wants
// it, each named by where they are and none doing what another does. For
// this browser, its address to bookmark, which opens it again by the home
// page's session (origin.rs `homeward`). For a program on this computer,
// another computer's lend no folder carries has Play, unless its cell leads
// with Play already, for programs: the address lets in
// a browser's session or svrngs's own client and no other program while
// Play is off (play.rs `shown`), so Play is the one way in for a chat app,
// and it copies the address as it is turned on (play.js `copy`). A carried
// app is a page, and this computer's own lend is at its own port here, so
// neither has one. For a TV or phone on this WiFi, its line (`Line`),
// under what opening it to this WiFi does, never out of sight of its
// button. It is drawn once ROOM has answered, so no line says a lend is
// closed before ROOM has said what is open, and it asks ROOM again on each
// round of its group's reads, once a round however many cells ask.
const OPEN_IT =
  'For a TV or phone on this WiFi: open it to the WiFi, and anyone on it can open it as you while svrngs runs.';
const PROGRAM =
  'For a program on this computer, such as a chat app: press Play, then paste the address it copies.';

/**
 * The lend, its popover's id, one of its own in the page, the round of
 * its group's reads, and this computer's key, by which a lend is another's.
 * @typedef {{ l: Lent, id: string, round: number, me: string }} AddressProps
 */

/** @param {AddressProps} props */
export function Address({ l: lend, id, round, me }) {
  useEffect(() => room.dispatch({ type: 'read', round }), [round]);
  const roomState = useStore(room);
  if (roomState.room === undefined && roomState.at.room !== 'unread') return null;
  // `popover="auto"`: one open at a time, closed by Escape or a press
  // outside it, all the browser's doing.
  // Named with its lend, "Address of Kim’s navidrome", the popover too: it
  // takes the focus as it opens, so what it is is said, and Tab goes on to
  // its first button; the browser gives the focus back to "Address". Last
  // in it, "Close", the browser's own too (`popovertargetaction="hide"`):
  // Escape and a press outside close it, and nobody is told so.
  const name = `Address of ${named(lend)}`;
  return html`
    <button type="button" popovertarget=${id} aria-label=${name}>Address</button>
    <div popover="auto" id=${id} role="dialog" aria-label=${name} tabindex="-1" autofocus>
      <p><strong>${named(lend)}</strong>${lend.since === undefined ? '' : ` Lent on ${day(lend.since)}.`}</p>
      <p>For this browser: <${Keep} at=${lend.origin} /></p>
      ${lend.key !== me && !lend.group && !lend.for_programs && html`<p>${PROGRAM} <${Turn} l=${lend} round=${round} /></p>`}
      <p>${OPEN_IT}</p>
      <ul class="wifi"><${Line} l=${lend} words=${`${named(lend)}:`} /></ul>
      <button type="button" popovertarget=${id} popovertargetaction="hide" aria-label=${`Close: ${name}`}>Close</button>
    </div>
  `;
}

const ANYONE =
  'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.';

/** @param {Which} lend */
const lineWords = (lend) => `${lend.person ? `${lend.person}’s ` : ''}${lend.name}:`;

// "Open to this WiFi", for acting.js: what ROOM read and what a press here
// closed, each with its line, or why it cannot be read; none when nothing
// is open.
/**
 * @param {State} state
 * @returns {Drawn[]}
 */
export function rooms(state) {
  const unread = state.at.room === 'unread';
  const open = unread ? [] : (state.room ?? []);
  const all = [...open, ...state.closed.filter((closed) => !on(state, closed))];
  if (!unread && all.length === 0) return [];
  const lines = all.map(
    (lend) => html`<${Line} key=${keyOf(lend)} l=${lend} words=${lineWords(lend)} />`,
  );
  return [html`<p>${unread ? state.why : ANYONE}</p>`, html`<ul id="room">${lines}</ul>`];
}

// A hook: acting.js calls it on each of its draws, so the list is drawn
// again with each read.
export const part = () => rooms(useStore(room));
