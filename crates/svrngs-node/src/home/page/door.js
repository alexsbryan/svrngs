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
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, assertNever, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Door } from '../ui/door.js?v=__VERSION__';
import { deciders, removal } from '../question.js?v=__VERSION__';
import { post } from './group.js?v=__VERSION__';
/** @import { Group, Lends, People, PlaceProps, Reads } from './group.js?v=__VERSION__' */
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Removal, PeopleRow } from '../question.js?v=__VERSION__' */

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

/** @param {{ name: string | null }} group */
const its = (group) => (group.name ? `${group.name}’s folder` : 'The group’s folder');
/**
 * @param {State} state
 * @param {Extract<Message, { n: number }>} message
 * @param {string} words
 * @returns {Step<State, Effect>}
 */
const answered = (state, message, words) =>
  late(state, 'open', message.n)
    ? no(state, late(state, 'open', message.n))
    : go(state, { folder: message.type }, { said: words });
/** @type {Machine<State, Message, Effect>} */
const FOLDER = {
  states: { folder: ['idle', 'opening', 'opened', 'unopened'] },
  step: (state, message) => {
    switch (message.type) {
      case 'open': {
        const [asked, serial] = ask(state, 'open');
        return go(
          state,
          { folder: 'opening' },
          { asked },
          { run: 'open', n: serial, group: message.group, dir: message.dir, failed: 'unopened' },
        );
      }
      case 'opened':
        return answered(state, message, ' Opened, in a window of its own.');
      case 'unopened':
        return answered(state, message, ` It could not be opened: ${message.why}.`);
      default:
        return assertNever(state, message, 'a folder');
    }
  },
  io: {
    open: ({ n: serial, group, dir }) =>
      post('/folder', { group, dir }).then(() => ({ type: 'opened', n: serial, why: dir })),
  },
};
/**
 * A folder of the group's, with its button.
 * @typedef {{ g: Group, dir: string }} FolderProps
 */

/** @param {FolderProps} props */
function Folder({ g: group, dir }) {
  const [state, send] = useMachine(() =>
    machine('FOLDER', FOLDER, { at: { folder: 'idle' }, said: '' }),
  );
  return html`<p class="folder">${its(group)} is on this computer at ${dir}</p><p><button onClick=${() => send({ type: 'open', group: group.id, dir })}>Open this folder</button>${state.said}</p>`;
}

/**
 * @param {{ name: string | null }} group
 * @param {PeopleRow[]} rows
 * @param {string} me
 */
export const decides = (group, rows, me) =>
  `${group.name ?? 'Your group'}: ${deciders(rows, me, 'computer')}.`;
// The lends as the door judges them: read, unread (`null`), or, while they
// are read, none yet, "asking…".
/** @type {Record<'people' | 'folder' | 'removals', (props: PlaceProps) => Drawn>} */
export const view = {
  people: ({ s: state }) =>
    state.people &&
    html`<${Door} rows=${state.people.people} lends=${/** @type {Partial<Record<Reads, Lends | null>>} */ ({ read: state.lends, unread: null })[state.at.lends]} />`,
  folder: ({ g: group, s: state }) =>
    state.people &&
    (group.folders === null
      ? html`<p>This computer cannot say where ${its(group)} is.</p>`
      : group.folders.map((dir) => html`<${Folder} key=${dir} g=${group} dir=${dir} />`)),
  removals: ({ g: group, s: state }) =>
    state.people &&
    // `null` only beside `removed`'s, and a group whose `removed` is
    // `null` draws no place (group.js `Group`). `state.people` is read, past
    // `state.people &&`, which tsc does not carry into the `map`'s callback.
    /** @type {Removal[]} */ (group.removals).length > 0 &&
    html`<ul class="removals">${/** @type {Removal[]} */ (group.removals).map((gone) => html`<li>${removal(gone, /** @type {People} */ (state.people).people)}</li>`)}</ul>`,
};
