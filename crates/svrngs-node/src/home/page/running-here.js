// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here": what the look says (running-look.js RUNNING),
// its button, and each program it found, a row of its own (found.js).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act } from '../ui/parts.js?v=__VERSION__';
import { Found } from './found.js?v=__VERSION__';
import { RUNNING } from './running-look.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Shown, State } from './running-look.js?v=__VERSION__' */

/** @type {Record<State['at']['running'], (s: State) => string>} */
const SAID = {
  unlooked: () =>
    'This page lists the programs running on this computer only when you ask: looking takes it a few seconds of work.',
  looking: () => 'Looking for programs running on this computer…',
  unread: (state) => `You looked, and what runs on this computer cannot be read: ${state.why}`,
  // `state.found` is the look's, which set 'looked' with it.
  looked: (state) =>
    /** @type {Shown[]} */ (state.found).length
      ? 'You looked just now: the programs running on this computer, each at its address. Lend one, and everyone in your groups opens it in their browser, as themselves, while this computer is on. You can stop lending it below at any time.'
      : 'You looked just now: nothing else on this computer is running at an address. Start a program, then look again.',
};

/**
 * "Already running here": HOME's `dispatch`, which each lend tells.
 * @typedef {{ home: Home }} RunningProps
 */

/** @param {RunningProps} props */
export function Running({ home }) {
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
        html`<${Found} key=${`${look}:${row.port}`} r=${row} named=${state.named} home=${home} />`,
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
