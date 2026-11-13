// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting a group, in its section on the home page (GUESTS §5 "The
// pages"), from the record as every member's page shows them, once GROUP
// has read its people: who you have over first, in words that are not an
// app's noun, each with its Stop (stop-visit.js); then everyone's, by host,
// with until when (ui/visitors.js).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { assertNever, go, machine } from '../ui/machine.js?v=__VERSION__';
import { post } from './routes.js?v=__VERSION__';
import { Visiting } from '../ui/visitors.js?v=__VERSION__';
import { Stop } from './stop-visit.js?v=__VERSION__';
/** @import { Group, Home, PlaceProps } from './group.js?v=__VERSION__' */
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
 * How many each computer can have visiting at once: the keeper changes it by
 * one, anyone else reads it.
 * @param {{ g: Group, n: number, home: Home }} props
 */
function Slots({ g: group, n, home }) {
  const [state, send] = useMachine(() =>
    machine('SLOTS', SLOTS(home), /** @type {SlotsState} */ ({ at: { slots: 'set' }, said: '' })),
  );
  const change = (/** @type {number} */ to) => () => send({ type: 'change', g: group.id, n: to });
  const fewer = group.keeps && n > 0 && html`<button onClick=${change(n - 1)}>One fewer</button>`;
  const more = group.keeps && html`<button onClick=${change(n + 1)}>One more</button>`;
  return html`<p class="slots">Each computer can have ${n} visiting at once ${fewer} ${more}${state.said}</p>`;
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
  const now =
    (visiting?.length ?? 0) > 0 &&
    html`
      <h3>Visiting now</h3>
      <${Visiting} vs=${visiting} rows=${rows} />
    `;
  const each = group.slots;
  const slots = each != null && html`<${Slots} g=${group} n=${each} home=${home} />`;
  return html`${slots}${unknown}${hosted}${now}`;
}

/** @type {{ visits: (props: PlaceProps) => Drawn }} */
export const view = { visits: Visits };
