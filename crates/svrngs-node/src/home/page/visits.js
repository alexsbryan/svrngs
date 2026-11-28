// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting a group, in its section on the home page (GUESTS §5 "The
// pages"), from the record as every member's page shows them, once GROUP
// has read its people: who you have over first, in words that are not an
// app's noun, each with its Stop (`Stop`); then everyone's, by host,
// with until when (ui/visitors.js).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { assertNever, go, machine } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
import { Visiting } from '../ui/visitors.js?v=__VERSION__';
import { until } from '../question.js?v=__VERSION__';
import { VISITS } from './visit.js?v=__VERSION__';
/** @import { Group, Home, PlaceProps, Visit } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */

/**
 * The keeper's say of how many each computer can have visiting at once:
 * where a change is, and its refusal, said.
 * @typedef {object} SlotsState
 * @property {{ slots: 'set' | 'changing' | 'refused' }} at
 * @property {string} said
 */

/**
 * The messages it takes: a change pressed, to `n` for the group `g`; the
 * change taken; or refused, with why.
 * @typedef {{ type: 'change', g: string, n: number }
 *   | { type: 'changed', why: string }
 *   | { type: 'refused', why: string }} SlotsMessage
 */

/**
 * The effect its step answers: the change posted, then the groups read again.
 * @typedef {{ run: 'change', g: string, n: number, failed: 'refused' }} SlotsEffect
 */

/**
 * SLOTS, the machine of that say on the home page: a press posts /slots,
 * home.rs `slots`, and once it is taken HOME reads the groups again.
 * @param {Home} home
 * @returns {Machine<SlotsState, SlotsMessage, SlotsEffect>}
 */
export const SLOTS = (home) => ({
  states: { slots: ['set', 'changing', 'refused'] },
  step: (state, message) => {
    switch (message.type) {
      case 'change': {
        const { g, n } = message;
        return go(
          state,
          { slots: 'changing' },
          { said: '' },
          { run: 'change', g, n, failed: 'refused' },
        );
      }
      case 'changed':
        return go(state, { slots: 'set' });
      case 'refused':
        return go(state, { slots: 'refused' }, { said: ` ${message.why}` });
      default:
        return assertNever(state, message, 'how many each computer can have visiting');
    }
  },
  io: {
    change: ({ g, n }) =>
      post('/slots', [g, n]).then(() => {
        home({ type: 'again' });
        return { type: 'changed', why: `${n} at once, the groups read again` };
      }),
  },
});

/**
 * The guest limit, how many each computer can host at a time, said beside
 * "Have a guest" in its words: the keeper changes it by one, anyone else
 * reads it.
 * @param {{ g: Group, n: number, home: Home }} props
 */
function Slots({ g: group, n, home }) {
  const [state, send] = useMachine(() =>
    machine('SLOTS', SLOTS(home), /** @type {SlotsState} */ ({ at: { slots: 'set' }, said: '' })),
  );
  const change = (/** @type {number} */ to) => () => send({ type: 'change', g: group.id, n: to });
  const fewer = group.keeps && n > 0 && html`<button onClick=${change(n - 1)}>One fewer</button>`;
  const more = group.keeps && html`<button onClick=${change(n + 1)}>One more</button>`;
  return html`<p class="slots">Guest limit: ${n} at a time on each computer in the group. ${fewer} ${more}${state.said}</p>`;
}

// A visit you host, with its Stop (visit.js VISITS). The line says what
// Stop does before the person presses it.
/**
 * A visit you host: the group, the visit, the group's name as said, and
 * HOME's `dispatch`, told to read again once it ends.
 * @typedef {object} StopProps
 * @property {Group} g
 * @property {Visit} v
 * @property {string} name
 * @property {Home} home
 */

/** @param {StopProps} props */
function Stop({ g: group, v: visit, name, home }) {
  const [state, send] = useMachine(() =>
    machine('VISITS', VISITS(home), { at: { visit: 'on' }, said: [] }),
  );
  const guest = visit.guest;
  const words = `${guest}, until ${until(visit)}. Stop ends the visit now: what you lend stops opening to ${guest}, and everyone in ${name} sees it ended. `;
  const stop = () => send({ type: 'stop', g: group.id, id: visit.id });
  return html`
    <li>
      ${words}
      <button onClick=${stop}>${`Stop ${guest}’s visit`}</button>
      ${state.said}
    </li>
  `;
}

/** @param {PlaceProps} props */
function Visits({ g: group, s: state, home }) {
  if (!state.people) return null;
  const rows = state.people.people;
  const name = group.name ?? 'this group';
  const visiting = group.visiting;
  const unknown = !visiting && html`<p>This computer cannot say who is visiting ${name} now.</p>`;
  const mine = (visiting ?? []).filter((visit) => visit.yours);
  const stops = mine.map(
    (visit) => html`<${Stop} key=${visit.id} g=${group} v=${visit} name=${name} home=${home} />`,
  );
  const hosted =
    mine.length > 0 &&
    html`
      <h3>Who you have over</h3>
      <ul class="yours">${stops}</ul>
    `;
  // Their own guests are said above, with Stop, so not again here.
  const others = (visiting ?? []).filter((visit) => !visit.yours);
  const now =
    others.length > 0 &&
    html`
      <h3>Visiting now</h3>
      <${Visiting} vs=${others} rows=${rows} />
    `;
  const each = group.slots;
  const slots = each != null && html`<${Slots} g=${group} n=${each} home=${home} />`;
  return html`${slots}${unknown}${hosted}${now}`;
}

/** @type {{ visits: (props: PlaceProps) => Drawn }} */
export const view = { visits: Visits };
