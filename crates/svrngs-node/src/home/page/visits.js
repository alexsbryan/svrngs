// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting a group, in its section on the home page (page-members
// (b); home.js@19b63b2b:155-166; GUESTS §5 "The pages"), from the record as every
// member's page shows them, once GROUP has read its people: who you have
// over first (GUESTS's "Your guests", in words that are not an app's noun),
// each with Stop, which says what it does before it does it; then
// everyone's, by host, with until when (ui/visitors.js). Each Stop is
// VISITS, a machine of the visit's own: /visits stops it, and its answer's
// time is after every act its computer holds, so may be a second on
// (svrngs-core command.rs `sign_lines`); the groups are read again then
// (HOME's `again`), sooner reading the guest as still here, where v1
// reloaded the page (page-members (b)). A refusal is said after the one
// before, as v1's `li.append` does.
import {
  assertNever,
  go,
  html,
  machine,
  until,
  useMachine,
  Visiting,
} from '../ui/kit.js?v=__VERSION__';
import { post } from './group.js?v=__VERSION__';
/** @import { Group, Home, PlaceProps, Visit } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */

/**
 * A visit's state: where its Stop is, and each refusal said, in order.
 * @typedef {object} State
 * @property {{ visit: 'on' | 'stopping' | 'stopped' | 'waited' | 'refused' }} at
 * @property {Record<string, number>} [asked]
 * @property {string[]} said
 */

/**
 * The messages a visit takes: Stop pressed, for the group `g`'s visit
 * `id`; its Stop recorded at `ts`, in seconds; that time waited for; or
 * refused, with why.
 * @typedef {{ type: 'stop', g: string, id: string }
 *   | { type: 'stopped', ts: number, why: string }
 *   | { type: 'waited', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effects its step answers: the Stop posted, and its time waited for.
 * @typedef {{ run: 'stop', g: string, id: string, failed: 'refused' }
 *   | { run: 'wait', ts: number, failed: 'refused' }} Effect
 */

/**
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
const VISITS = (home) => ({
  states: { visit: ['on', 'stopping', 'stopped', 'waited', 'refused'] },
  step: (state, message) => {
    switch (message.type) {
      case 'stop':
        return go(
          state,
          { visit: 'stopping' },
          {},
          { run: 'stop', g: message.g, id: message.id, failed: 'refused' },
        );
      case 'stopped':
        return go(
          state,
          { visit: 'stopped' },
          {},
          { run: 'wait', ts: message.ts, failed: 'refused' },
        );
      case 'waited':
        return go(state, { visit: 'waited' });
      case 'refused':
        return go(state, { visit: 'refused' }, { said: [...state.said, ` ${message.why}`] });
      default:
        return assertNever(state, message, 'a visit');
    }
  },
  io: {
    stop: ({ g: group, id }) =>
      post('/visits', [group, id]).then((answer) => ({
        type: 'stopped',
        ts: answer.ts_unix,
        why: `its Stop is at ${answer.ts_unix}`,
      })),
    wait: ({ ts }) =>
      new Promise((resolve) => setTimeout(resolve, ts * 1000 - Date.now())).then(() => {
        home({ type: 'again' });
        return { type: 'waited', why: 'the groups are read again' };
      }),
  },
});
/**
 * A visit you host, with its Stop: the group, the visit, the group's name
 * as said, and HOME's `dispatch`, told to read again once it ends.
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
  return html`<li>${`${visit.guest}, until ${until(visit)}. Stop ends the visit now: what you lend stops opening to ${visit.guest}, and everyone in ${name} sees it ended. `}<button onClick=${() => send({ type: 'stop', g: group.id, id: visit.id })}>${`Stop ${visit.guest}’s visit`}</button>${state.said}</li>`;
}

/**
 * @param {Group} group
 * @param {PeopleRow[]} rows
 * @param {Home} home
 * @param {string} name
 * @returns {Drawn}
 */
const listed = (group, rows, home, name) => {
  const mine = (group.visiting ?? []).filter((visit) => visit.yours);
  return html`${!group.visiting && html`<p>This computer cannot say who is visiting ${name} now.</p>`}${mine.length > 0 && html`<h3>Who you have over</h3><ul class="yours">${mine.map((visit) => html`<${Stop} key=${visit.id} g=${group} v=${visit} name=${name} home=${home} />`)}</ul>`}${(group.visiting?.length ?? 0) > 0 && html`<h3>Visiting now</h3><${Visiting} vs=${group.visiting} rows=${rows} />`}`;
};
/** @type {{ visits: (props: PlaceProps) => Drawn }} */
export const view = {
  visits: ({ g: group, s: state, home }) =>
    state.people && listed(group, state.people.people, home, group.name ?? 'this group'),
};
