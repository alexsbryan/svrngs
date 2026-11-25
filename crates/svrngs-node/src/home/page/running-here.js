// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here" on the home page: the programs svrngs runs, each
// with Start or Stop (`Runs`), and, once the person is in a group, the
// look's button, after what the look says while it looks, when it found
// nothing or when it could not look (running-look.js RUNNING), and each
// program it found, one line of its own (`Found`). A program is set to run
// with svrngs from its lend's row (keep-running.js), or `svrngs run`.
import { html, useEffect, useMachine, useRef, useState } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LEND } from './lend-found.js?v=__VERSION__';
import { GroupChoice } from './group-choice.js?v=__VERSION__';
import { answered, needs, RUNNING } from './running-look.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
/** @import { Paths } from './lend-found.js?v=__VERSION__' */
/** @import { Shown, State } from './running-look.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

// Said before the look's button: nothing before a look, as the button says
// what it does, nor after one that found something, whose lines say it.
/** @type {Record<State['at']['running'], (s: State) => string>} */
const SAID = {
  unlooked: () => '',
  looking: () => 'Looking for programs running on this computer…',
  unread: (state) => `You looked, and what runs on this computer cannot be read: ${state.why}`,
  // `state.found` is the look's, which set 'looked' with it.
  looked: (state) =>
    /** @type {Shown[]} */ (state.found).length
      ? ''
      : 'Nothing else on this computer is running at an address. Start a program, then look again.',
};

// A program found running here, one line of "Already running here": what
// it is, the kind svrngs knows or else what the look read answering, and
// its address. Folded under that line until the person opens it: fields
// for the name and the program of a kind svrngs does not know by its port,
// filled with the program answering where the table knows it by that name
// (running-look.js `program`), what a kind it knows needs first, and who
// can open it, one box a group of HOME's, each by its stable id, so it is
// lent to the groups ticked and never to everyone by default
// (SCOPE_PARALLEL.md, "Shared API shape"); one button lends it
// (lend-found.js LEND): pressed, it reviews it, the button then Yes, lend
// it beside who can use it and when, and the paths under Details. Lent, the
// row is one line that says to whom, which takes the focus until the lend's
// own row takes it (lent-row.js `Lent`); refused, it says why.
/** @type {[field: 'name' | 'kind', words: string][]} */
const FIELDS = [
  ['name', 'Its name: '],
  ['kind', 'Which program: '],
];

/**
 * A program found: it, HOME's groups to lend it to, and HOME's
 * `dispatch`, told once it is lent.
 * @typedef {{ r: Shown, groups: Choice[], home: Home }} FoundProps
 */

// The names of the groups ticked, "House and Trip", for the lend's button.
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
// A program found's line: the kind svrngs knows, else the program the look
// read answering on its port, by name, and its address.
/** @param {Shown} shown */
const line = ({ kind, answer, port }) => {
  const what = kind ?? answer?.program ?? (answer && answered(answer));
  return what ? `${what}, at localhost:${port}` : `Something at localhost:${port}`;
};

// What svrngs keeps private of it and, once reviewed, the paths apps may
// send to it by its terms and its groups' own rows besides, by whom, folded
// under Details; none, no fold.
/** @param {Paths} paths */
const Details = ({ keeps, sends, rows = [], by = '' }) =>
  keeps.length + sends.length + rows.length > 0 &&
  html`<details>
    <summary>Details</summary>
    ${sends.length > 0 && html`<p>Apps can send requests to ${sends.join(' and ')}.</p>`}
    ${keeps.length > 0 && html`<p>Kept private: ${keeps.join(', ')}.</p>`}
    ${rows.length > 0 && html`<p>Your group’s terms${by}: ${rows.join('; ')}.</p>`}
  </details>`;

/** @param {FoundProps} props */
function Found({ r: shown, groups, home }) {
  const [state, send] = useMachine(() =>
    machine('LEND', LEND(home), {
      at: { lend: 'unlent' },
      port: shown.port,
      name: shown.program ?? '',
      kind: shown.program ?? '',
      said: '',
      groups: [],
      ...shownProcess(shown),
    }),
  );
  const names = chosen(groups, state.groups).join(' and ');
  const done = useRef(/** @type {HTMLParagraphElement | null} */ (null));
  useEffect(() => done.current?.focus(), [state.at.lend]);
  if (state.at.lend === 'lent') {
    // `state.l` is the lend 'lent' made, which set 'lent' with it.
    const lent = /** @type {Shown} */ (state.l);
    return html`<li><p class="lit" tabindex="-1" data-lent=${lent.name} ref=${done}>Lent to ${names}.</p></li>`;
  }
  const fields =
    !shown.kind &&
    FIELDS.map(
      ([name, words]) =>
        html`<${Field} name=${name} words=${words} value=${state[name]} attrs=${{ disabled: state.at.lend === 'lending' }} send=${send} />`,
    );
  const busy = state.at.lend === 'lending' ? 'busy' : 'idle';
  const kind = shown.kind ?? 'it';
  const words = names ? `Lend ${kind} to ${names}` : `Lend ${kind}`;
  return html`
    <li>
      <details>
        <summary>${line(shown)}</summary>
        ${fields}
        <p>${needs(shown)} Who can open ${kind}?</p>
        <${GroupChoice}
          groups=${groups}
          ticked=${state.groups}
          waiting=${busy === 'busy'}
          send=${send}
        />
        <${Act}
          words=${state.review ? 'Yes, lend it' : words}
          doing=${{ state: busy }}
          on=${{ type: 'lend' }}
          send=${send}
        />
        <span role="status">${state.said}</span>
        <${Details} ...${state.review ?? { keeps: shown.keeps ?? [], sends: [] }} />
      </details>
    </li>
  `;
}

/** @typedef {{ name: string, text: string, port?: number, action?: 'start' | 'stop', button?: string }} RunRow */

// The programs svrngs runs (GET /run's rows), read as it is drawn and on
// each of HOME's rounds, so one started from a lend's row is listed; each
// with Start or Stop. Drawn with `shares`, the look, once in a group.
/** @param {{ shares: Drawn | null, round: HOME.State['round'] }} props */
function Runs({ shares, round }) {
  const [rows, set] = useState(/** @type {RunRow[]} */ ([]));
  const [said, say] = useState('');
  const [busy, wait] = useState(false);
  const refresh = () => get('/run').then((view) => set(view.rows));
  /** @param {Error} error */
  const refused = (error) => say(error.message);
  useEffect(() => void refresh().catch(refused), [round]);
  /** @param {RunRow['name']} name @param {RunRow['action']} action */
  const act = (name, action) => {
    wait(true);
    say('');
    return post('/run/action', [name, action])
      .catch(refused)
      .then(() => refresh().catch(refused))
      .finally(() => wait(false));
  };
  if (shares === null && rows.length === 0) return null;
  return html`<section id="running"><h2>Already running here</h2><div id="runs">
    <p role="status">${said}</p>
    ${rows.map(
      ({ name, port, text, action, button }) => html`<p key=${name}>
      ${name} ${port} · ${text}
      ${action !== undefined && html`<button disabled=${busy} onClick=${() => act(name, action)}>${button}</button>`}
    </p>`,
    )}
  </div>${shares}</section>`;
}

/** @param {HOME.PlaceProps} props */
function Running({ s, home }) {
  const groups = /** @type {Choice[]} */ (s.groups);
  const [state, send] = useMachine(() =>
    machine('RUNNING', RUNNING, { at: { running: 'unlooked' } }),
  );
  // Each row's key carries its look's number, so a look again draws each
  // program it finds as a new row, with nothing typed or said in it.
  const found =
    state.at.running === 'looked' &&
    /** @type {Shown[]} */ (state.found).map(
      (row) =>
        html`<${Found}
          key=${`${state.asked?.running}:${row.port}`}
          r=${row}
          groups=${groups}
          home=${home}
        />`,
    );
  const look = html`
    <p>
      ${SAID[state.at.running](state)}
      <${Act}
        words="Look for programs running on this computer"
        doing=${{ state: state.at.running === 'looking' ? 'busy' : 'idle' }}
        on=${{ type: 'look' }}
        send=${send}
      />
    </p>
    <ul>${found}</ul>
  `;
  return html`<${Runs} shares=${groups.length > 0 ? look : null} round=${s.round} />`;
}

/** @type {{ running: (props: HOME.PlaceProps) => Drawn }} */
export const view = { running: Running };
