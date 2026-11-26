// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here" on the home page, once the person is in a group:
// its button, after what the look says while it looks, when it found
// nothing or when it could not look (running-look.js RUNNING), and each
// program it found, one line of its own (`Found`).
import { html, useEffect, useMachine, useRef, useState } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LEND } from './lend-found.js?v=__VERSION__';
import { GroupChoice } from './group-choice.js?v=__VERSION__';
import { answered, needs, RUNNING } from './running-look.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
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
// its address. Folded under that line until the person opens it: Run it
// with svrngs, fields for the name and the program of a kind svrngs does
// not know, what a kind it knows needs first, and who can open it, one
// box a group of HOME's, each by its stable id, so it is lent to the
// groups ticked and never to everyone by default (SCOPE_PARALLEL.md,
// "Shared API shape"); one button lends it (lend-found.js LEND), and the
// row then says what happened, or why not.
/** @type {[field: 'name' | 'kind', words: string][]} */
const FIELDS = [
  ['name', 'Its name: '],
  ['kind', 'Which program: '],
];

/**
 * A program found: it, HOME's groups to lend it to, HOME's `dispatch`,
 * told once it is lent, and its Run it with svrngs link, if any, drawn
 * apart from the fields under it.
 * @typedef {{ r: Shown, groups: Choice[], home: Home, run: Drawn }} FoundProps
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
// A program found's line: the kind svrngs knows, else what the look read
// answering on its port, and its address.
/** @param {Shown} shown @param {string} at */
const line = ({ kind, answer }, at) => {
  const what = kind ?? (answer ? answered(answer) : null);
  return what ? `${what}, at ${at}` : `Something at ${at}`;
};

/** @param {FoundProps} props */
function Found({ r: shown, groups, home, run }) {
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
  const fields =
    !shown.kind &&
    FIELDS.map(
      ([name, words]) =>
        html`<${Field} name=${name} words=${words} value=${state[name]} attrs=${{ disabled: state.at.lend === 'lending' }} send=${send} />`,
    );
  const busy = state.at.lend === 'lending' ? 'busy' : 'idle';
  const names = chosen(groups, state.groups);
  const kind = shown.kind ?? 'it';
  const words = names.length ? `Lend ${kind} to ${names.join(' and ')}` : `Lend ${kind}`;
  return html`
    <li>
      <details>
        <summary>${line(shown, at)}</summary>
        ${run}
        ${fields}
        <p>${needs(shown)} Who can open ${kind}?</p>
        <${GroupChoice}
          groups=${groups}
          ticked=${state.groups}
          waiting=${busy === 'busy'}
          send=${send}
        />
        <${Act}
          words=${state.review?.button ?? words}
          doing=${{ state: busy }}
          on=${{ type: 'lend' }}
          send=${send}
        />
        ${state.said}
      </details>
    </li>
  `;
}

/** @typedef {{ name: string, text: string, port?: number, action?: 'start' | 'stop', button?: string }} RunRow */
/** @typedef {{ run: string, starting: string, fields: [string, string][], rows: RunRow[] }} RunView */

/** @param {{ shares: ((run: (port: number) => Drawn) => Drawn) | null }} props */
function RunForm({ shares }) {
  const [view, set] = useState(/** @type {RunView | null} */ (null));
  const [said, say] = useState('');
  const [busy, wait] = useState(false);
  const asked = useRef(0);
  const refresh = () => get('/run').then(set);
  /** @param {Error} error */
  const refused = (error) => say(error.message);
  useEffect(() => void refresh().catch(refused), []);
  /** @param {string} path @param {unknown} body */
  const act = (path, body) => {
    wait(true);
    say(path === '/run' ? (view?.starting ?? '') : '');
    return post(path, body)
      .then((answer) => say(answer ?? ''), refused)
      .then(() => refresh().catch(refused))
      .finally(() => wait(false));
  };
  /** @param {import('../ui/view.js?v=__VERSION__').Targeted<HTMLFormElement>} event */
  const submit = (event) => {
    event.preventDefault();
    const { name, port, folder, command } = Object.fromEntries(new FormData(event.currentTarget));
    void act('/run', { name, port: Number(port), folder, command });
  };
  /** @param {number} port */
  const suggest = async (port) => {
    const form = /** @type {HTMLFormElement | null} */ (document.querySelector('#run-form form'));
    if (!form || busy) return;
    form.reset();
    form.closest('details')?.setAttribute('open', '');
    // Selecting a row or typing in the form gives the next ticket to that act.
    const ticket = ++asked.current;
    const got = await get(`/run?prefill=${port}`).catch(() => null);
    if (ticket !== asked.current || got?.kind !== 'prefill' || got.port !== port) return;
    for (const input of form.querySelectorAll('input')) input.value = got[input.name] ?? '';
  };
  if (shares === null && (view?.rows.length ?? 0) === 0) return null;
  return html`<section id="running"><h2>Already running here</h2><div id="run-form">
    ${
      view !== null &&
      html`<details><summary>${view.run}</summary>
      <form onSubmit=${submit} onInput=${() => asked.current++}><fieldset disabled=${busy}>
        ${view.fields.map(([key, label]) => html`<label>${label} <input name=${key} required /></label>`)}
        <button>${view.run}</button>
      </fieldset></form>
    </details>`
    }
    <p role="status">${said}</p>
    ${view?.rows.map(
      ({ name, port, text, action, button }) => html`<p key=${name}>
      ${name} ${port} · ${text}
      ${action !== undefined && html`<button disabled=${busy} onClick=${() => act('/run/action', [name, action])}>${button}</button>`}
    </p>`,
    )}
  </div>${shares?.((port) => view && !view.rows.some((row) => row.port === port) && html`<p><a href="#run-form" onClick=${() => suggest(port)}>${view.run}</a></p>`)}</section>`;
}

/** @param {HOME.PlaceProps} props */
function Running({ s, home }) {
  const groups = /** @type {Choice[]} */ (s.groups);
  const [state, send] = useMachine(() =>
    machine('RUNNING', RUNNING, { at: { running: 'unlooked' } }),
  );
  // Each row's key carries its look's number, so a look again draws each
  // program it finds as a new row, with nothing typed or said in it.
  /** @param {(port: number) => Drawn} run */
  const found = (run) =>
    state.at.running === 'looked' &&
    /** @type {Shown[]} */ (state.found).map(
      (row) =>
        html`<${Found}
          key=${`${state.asked?.running}:${row.port}`}
          r=${row}
          groups=${groups}
          home=${home}
          run=${run(row.port)}
        />`,
    );
  return html`<${RunForm} shares=${
    groups.length > 0
      ? (/** @type {Parameters<typeof found>[0]} */ run) => html`
      <p>
        ${SAID[state.at.running](state)}
        <${Act}
          words="Look for programs running on this computer"
          doing=${{ state: state.at.running === 'looking' ? 'busy' : 'idle' }}
          on=${{ type: 'look' }}
          send=${send}
        />
      </p>
      <ul>${found(run)}</ul>
   `
      : null
  } />`;
}

/** @type {{ running: (props: HOME.PlaceProps) => Drawn }} */
export const view = { running: Running };
