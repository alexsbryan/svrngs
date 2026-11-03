// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's people on the home page (page-members (a); home.js@19b63b2b:121-132,
// :154), each in its place in the group's section (group.js PLACES), once
// GROUP has read them: the door (ui/door.js), "asking…" while the group's
// lends are being read, judged by their answer, "not known now" when they
// could not be read; where the group's folder is on this disk, each opened
// in a window of its own by its button, FOLDER, a machine of the button's
// own, each open counted, so the last one's answer alone is said, once,
// where v1's `p.append` said each after the one before (home.js@19b63b2b:130);
// and who was removed or asked to be, when anyone was. `decides` is who
// decides for you in it, which "What acts as you" says (page-acts).
import {
  ask,
  assertNever,
  deciders,
  Door,
  go,
  html,
  late,
  machine,
  no,
  removal,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { post } from './group.js?v=__VERSION__';
/** @import { Group, Lends, People, PlaceProps, Reads } from './group.js?v=__VERSION__' */
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Removal, Row } from '../question.js?v=__VERSION__' */

/**
 * A folder's button's state: where its open is, the opens asked, and the
 * last one's answer, said.
 * @typedef {object} State
 * @property {{ folder: 'idle' | 'opening' | 'opened' | 'unopened' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} said
 */

/**
 * The messages a folder's button takes: pressed, for the group's folder
 * `dir`; then open `n` opened, or not, with why.
 * @typedef {{ type: 'open', group: string, dir: string }
 *   | { type: 'opened', n: number, why: string }
 *   | { type: 'unopened', n: number, why: string }} Message
 */

/**
 * The effect its step answers: the folder opened, open `n`.
 * @typedef {{ run: 'open', n: number, group: string, dir: string, failed: 'unopened' }} Effect
 */

/** @param {{ name: string | null }} g */
const its = (g) => (g.name ? `${g.name}’s folder` : 'The group’s folder');
/**
 * @param {State} s
 * @param {Extract<Message, { n: number }>} m
 * @param {string} words
 * @returns {Step<State, Effect>}
 */
const answered = (s, m, words) =>
  late(s, 'open', m.n) ? no(s, late(s, 'open', m.n)) : go(s, { folder: m.type }, { said: words });
/** @type {Machine<State, Message, Effect>} */
const FOLDER = {
  states: { folder: ['idle', 'opening', 'opened', 'unopened'] },
  step: (s, m) => {
    switch (m.type) {
      case 'open': {
        const [asked, n] = ask(s, 'open');
        return go(
          s,
          { folder: 'opening' },
          { asked },
          { run: 'open', n, group: m.group, dir: m.dir, failed: 'unopened' },
        );
      }
      case 'opened':
        return answered(s, m, ' Opened, in a window of its own.');
      case 'unopened':
        return answered(s, m, ` It could not be opened: ${m.why}.`);
      default:
        return assertNever(s, m, 'a folder');
    }
  },
  io: {
    open: ({ n, group, dir }) =>
      post('/folder', { group, dir }).then(() => ({ type: 'opened', n, why: dir })),
  },
};
/**
 * A folder of the group's, with its button.
 * @typedef {{ g: Group, dir: string }} FolderProps
 */

/** @param {FolderProps} props */
function Folder({ g, dir }) {
  const [s, send] = useMachine(() =>
    machine('FOLDER', FOLDER, { at: { folder: 'idle' }, said: '' }),
  );
  return html`<p class="folder">${its(g)} is on this computer at ${dir}</p><p><button onClick=${() => send({ type: 'open', group: g.id, dir })}>Open this folder</button>${s.said}</p>`;
}

/**
 * @param {{ name: string | null }} g
 * @param {Row[]} rows
 * @param {string} me
 */
export const decides = (g, rows, me) =>
  `${g.name ?? 'Your group'}: ${deciders(rows, me, 'computer')}.`;
// The lends as the door judges them: read, unread (`null`), or, while they
// are read, none yet, "asking…".
/** @type {Record<'people' | 'folder' | 'removals', (props: PlaceProps) => Drawn>} */
export const view = {
  people: ({ s }) =>
    s.people &&
    html`<${Door} rows=${s.people.people} lends=${/** @type {Partial<Record<Reads, Lends | null>>} */ ({ read: s.lends, unread: null })[s.at.lends]} />`,
  folder: ({ g, s }) =>
    s.people &&
    (g.folders === null
      ? html`<p>This computer cannot say where ${its(g)} is.</p>`
      : (g.folders ?? []).map((dir) => html`<${Folder} key=${dir} g=${g} dir=${dir} />`)),
  removals: ({ g, s }) =>
    s.people &&
    // `null` only beside `removed`'s, and a group whose `removed` is
    // `null` draws no place (group.js `Group`). `s.people` is read, past
    // `s.people &&`, which tsc does not carry into the `map`'s callback.
    /** @type {Removal[]} */ (g.removals).length > 0 &&
    html`<ul class="removals">${/** @type {Removal[]} */ (g.removals).map((r) => html`<li>${removal(r, /** @type {People} */ (s.people).people)}</li>`)}</ul>`,
};
