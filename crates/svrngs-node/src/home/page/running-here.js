// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here" on the home page, once the person is in a group:
// what the look says (running-look.js RUNNING), its button, and each
// program it found, a row of its own (`Found`).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LEND } from './lend-found.js?v=__VERSION__';
import { GroupChoice } from './group-choice.js?v=__VERSION__';
import { answered, needs, RUNNING } from './running-look.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
/** @import { Shown, State } from './running-look.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/** @type {Record<State['at']['running'], (s: State) => string>} */
const SAID = {
  unlooked: () =>
    'This page lists the programs running on this computer only when you ask: looking takes it a few seconds of work.',
  looking: () => 'Looking for programs running on this computer…',
  unread: (state) => `You looked, and what runs on this computer cannot be read: ${state.why}`,
  // `state.found` is the look's, which set 'looked' with it.
  looked: (state) =>
    /** @type {Shown[]} */ (state.found).length
      ? 'You looked just now: the programs running on this computer, each at its address. Lend one to the groups you choose, and the people in them open it in their browser, as themselves, while this computer is on. You can change who can open it, or stop lending it, below at any time.'
      : 'You looked just now: nothing else on this computer is running at an address. Start a program, then look again.',
};

// A program found running here, a row of "Already running here": a kind
// svrngs knows named with what it needs first, any other with fields for
// its name and what it is; who can open it, one box a group of HOME's,
// each by its stable id, so it is lent to the groups ticked and never to
// everyone by default (SCOPE_PARALLEL.md, "Shared API shape"); one button
// lends it (lend-found.js LEND), and the row then says what happened, or
// why not.
/** @type {[field: 'name' | 'kind', words: string][]} */
const FIELDS = [
  ['name', 'Its name: '],
  ['kind', 'Which program: '],
];

/**
 * A program found: it, the kinds the table knows by name alone, HOME's
 * groups to lend it to, and HOME's `dispatch`, told once it is lent.
 * @typedef {{ r: Shown, named: string[], groups: Choice[], home: Home }} FoundProps
 */

// The names of the groups ticked, "House, Trip", for the lend's button.
/**
 * @param {Choice[]} groups
 * @param {string[]} ticked
 */
const chosen = (groups, ticked) =>
  groups.filter((group) => ticked.includes(group.id)).map((group) => group.name ?? 'this group');

// The process the look showed on a port and its start, which the lend
// sends, so a program that changed since is refused (SHARE_BINDING §5.1).
/** @param {Shown} shown */
const shownProcess = ({ answer }) => ({ pid: answer?.pid ?? null, start: answer?.start ?? null });
// What the look read answering on a port, said.
/** @param {Shown} shown */
const runs = ({ answer }) => (answer ? ` It is ${answered(answer)}.` : '');

/** @param {FoundProps} props */
function Found({ r: shown, named, groups, home }) {
  const [state, send] = useMachine(() =>
    machine('LEND', LEND(home), {
      at: { lend: 'unlent' },
      port: shown.port,
      name: '',
      kind: '',
      said: '',
      groups: [],
      ...shownProcess(shown),
    }),
  );
  const at = `http://localhost:${shown.port}`;
  if (state.at.lend === 'lent') {
    // `state.l` is the lend 'lent' made, which set 'lent' with it.
    const lent = /** @type {Shown} */ (state.l);
    return html`<li>${`You lent ${lent.name}, at ${at}: the groups you chose can open it now. ${needs(lent)} `}</li>`;
  }
  const known = named.length ? `; if it is ${named.join(' or ')}, say that` : '';
  const said = shown.kind
    ? `${shown.kind}, at ${at}.${runs(shown)} ${needs(shown)} `
    : `Something at ${at}.${runs(shown)} To lend it, give it a name and say which program it is, each one word${known}. `;
  const fields =
    !shown.kind &&
    FIELDS.map(
      ([name, words]) =>
        html`<${Field} name=${name} words=${words} value=${state[name]} send=${send} />`,
    );
  const busy = state.at.lend === 'lending' ? 'busy' : 'idle';
  const names = chosen(groups, state.groups);
  const who = shown.kind ? `Who can open ${shown.kind}?` : 'Who can open it?';
  const words = names.length
    ? `Lend ${shown.kind ?? 'it'} to ${names.join(' and ')}`
    : `Lend ${shown.kind ?? 'it'}`;
  return html`
    <li>
      ${said}
      ${fields}
      <p>${who}</p>
      <${GroupChoice}
        groups=${groups}
        ticked=${state.groups}
        waiting=${busy === 'busy'}
        send=${send}
      />
      <${Act}
        words=${words}
        doing=${{ state: busy }}
        on=${{ type: 'lend' }}
        send=${send}
      />
      ${state.said}
    </li>
  `;
}

/**
 * "Already running here": HOME's groups to lend one to, and its `dispatch`,
 * which each lend tells.
 * @typedef {{ groups: Choice[], home: Home }} RunningProps
 */

/** @param {RunningProps} props */
function Running({ groups, home }) {
  const [state, send] = useMachine(() =>
    machine('RUNNING', RUNNING, { at: { running: 'unlooked' } }),
  );
  // Each row's key carries its look's number, so a look again draws each
  // program it finds as a new row, with nothing typed or said in it.
  const look = state.asked?.running;
  const found =
    state.at.running === 'looked' &&
    /** @type {Shown[]} */ (state.found).map(
      (row) =>
        html`<${Found}
          key=${`${look}:${row.port}`}
          r=${row}
          named=${state.named}
          groups=${groups}
          home=${home}
        />`,
    );
  const busy = state.at.running === 'looking' ? 'busy' : 'idle';
  return html`
    <section id="running">
      <h2>Already running here</h2>
      <p>${SAID[state.at.running](state)}</p>
      <p>
        <${Act}
          words="Look for programs running on this computer"
          doing=${{ state: busy }}
          on=${{ type: 'look' }}
          send=${send}
        />
      </p>
      <ul>${found}</ul>
    </section>
  `;
}

// Drawn once the person is in a group (home-page.js `MEMBERS`).
/** @param {HOME.PlaceProps} props */
function RunningPlace({ s: state, home }) {
  return html`<${Running} groups=${/** @type {Choice[]} */ (state.groups)} home=${home} />`;
}

/** @type {{ running: (props: HOME.PlaceProps) => Drawn }} */
export const view = { running: RunningPlace };
