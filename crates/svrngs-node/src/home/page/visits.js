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
/** @import { Row } from '../question.js?v=__VERSION__' */

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
  step: (s, m) => {
    switch (m.type) {
      case 'stop':
        return go(
          s,
          { visit: 'stopping' },
          {},
          { run: 'stop', g: m.g, id: m.id, failed: 'refused' },
        );
      case 'stopped':
        return go(s, { visit: 'stopped' }, {}, { run: 'wait', ts: m.ts, failed: 'refused' });
      case 'waited':
        return go(s, { visit: 'waited' });
      case 'refused':
        return go(s, { visit: 'refused' }, { said: [...s.said, ` ${m.why}`] });
      default:
        return assertNever(s, m, 'a visit');
    }
  },
  io: {
    stop: ({ g, id }) =>
      post('/visits', [g, id]).then((a) => ({
        type: 'stopped',
        ts: a.ts_unix,
        why: `its Stop is at ${a.ts_unix}`,
      })),
    wait: ({ ts }) =>
      new Promise((r) => setTimeout(r, ts * 1000 - Date.now())).then(
        () => (home({ type: 'again' }), { type: 'waited', why: 'the groups are read again' }),
      ),
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
function Stop({ g, v, name, home }) {
  const [s, send] = useMachine(() =>
    machine('VISITS', VISITS(home), { at: { visit: 'on' }, said: [] }),
  );
  return html`<li>${`${v.guest}, until ${until(v)}. Stop ends the visit now: what you lend stops opening to ${v.guest}, and everyone in ${name} sees it ended. `}<button onClick=${() => send({ type: 'stop', g: g.id, id: v.id })}>${`Stop ${v.guest}’s visit`}</button>${s.said}</li>`;
}

/**
 * @param {Group} g
 * @param {Row[]} rows
 * @param {Home} home
 * @param {string} name
 * @returns {Drawn}
 */
const listed = (g, rows, home, name, mine = (g.visiting ?? []).filter((v) => v.yours)) =>
  html`${!g.visiting && html`<p>This computer cannot say who is visiting ${name} now.</p>`}${mine.length > 0 && html`<h3>Who you have over</h3><ul class="yours">${mine.map((v) => html`<${Stop} key=${v.id} g=${g} v=${v} name=${name} home=${home} />`)}</ul>`}${(g.visiting?.length ?? 0) > 0 && html`<h3>Visiting now</h3><${Visiting} vs=${g.visiting} rows=${rows} />`}`;
/** @type {{ visits: (props: PlaceProps) => Drawn }} */
export const view = {
  visits: ({ g, s, home }) => s.people && listed(g, s.people.people, home, g.name ?? 'this group'),
};
