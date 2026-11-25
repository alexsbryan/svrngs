// SPDX-License-Identifier: AGPL-3.0-or-later
// LEND, the machine of a program found running here (running-here.js `Found`): a
// kind svrngs knows is lent as it is, any other only once the person names
// it and says what it is. It is made to an explicit set of groups, never
// to everyone by default (SCOPE_PARALLEL.md, "Shared API shape"): with no
// group chosen the lend is refused, so joining a group later never widens
// it. The first press reviews it and lends nothing: who can use it and
// when, a line when its terms (svrngs's, and its groups' programs/<word>.txt)
// let apps send to it, and for the row's Details the paths of each and its
// groups' own rows besides, by whom, so each row kept is shown. The next,
// Yes, lend it, posts back the terms text and the review's token, so the lend
// keeps what was drawn, not the file as it is then (home.rs `lend_one`;
// CONTRACTS §6.8). A tick or a word typed lets them go, to be read again. A
// second press while one lends is refused. Once lent, HOME reads the groups
// again, so the index lists it and "What you lend" with it. Each refusal is
// said beside the button, after the ones before it, and kept there while it
// lends again; a review says its own lines in their place.
import { assertNever, go, no } from '../ui/machine.js?v=__VERSION__';
import { groupsAgain, post } from './routes.js?v=__VERSION__';
import { chosen } from './group-choice.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Shown } from './running-look.js?v=__VERSION__' */

/**
 * A program found's state: where its lend is, its port, its name and what
 * it is as typed, the stable ids of the groups chosen to open it, the
 * refusals said beside its button, and the lend made.
 * @typedef {object} State
 * @property {{ lend: 'unlent' | 'lending' | 'lent' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} port
 * @property {string} name
 * @property {string} kind
 * @property {string} said
 * @property {string[]} groups
 * @property {number | null} [pid] the process the look showed on the port
 * @property {number | null} [start] its start, so one that changed since is refused
 * @property {string} [terms] the text of its groups' own terms, drawn
 * @property {{ token: string } & Paths} [review] its token, and what its Details say
 * @property {Shown} [l]
 */

/**
 * What a review's Details say: the paths svrngs keeps back and apps may send
 * to by its terms, and its terms' other rows, with who wrote them (`by`).
 * @typedef {{ keeps: string[], sends: string[], rows?: string[], by?: string }} Paths
 */

/**
 * The messages a program found takes: its name or what it is typed; a
 * group ticked or unticked, by its stable id; Lend pressed; it lent; HOME
 * told; or the lend refused, with why.
 * @typedef {{ type: 'typed', field: 'name' | 'kind', value: string }
 *   | { type: 'choose', id: string, on: boolean }
 *   | { type: 'lend' }
 *   | { type: 'lent', l: Shown, why: string }
 *   | { type: 'drawn', drawn: { terms: string, said: string, review: State['review'] } }
 *   | { type: 'told', why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * The effects its step answers: the port lent, by the name and kind
 * typed, `null` where none is, to the groups chosen, by their stable ids;
 * and HOME told to read the groups again.
 * @typedef {{ run: 'lend', port: number, name: string | null, kind: string | null, groups: string[], terms: string | null, pid: number | null, start: number | null, token?: string, failed: 'refused' }
 *   | { run: 'again', failed: 'told' }} Effect
 */

// A lend's review, let go when a word is typed or a tick changes.
const UNSAID = { said: '', terms: undefined, review: undefined };
// Said under a review whose terms let apps send to it.
const SENDS = 'Apps your friends open can also send it requests, each after that friend says yes.';

/**
 * @param {Home} home
 * @returns {Machine<State, Message, Effect>}
 */
export const LEND = (home) => ({
  states: { lend: ['unlent', 'lending', 'lent'] },
  io: {
    // The effect less its own fields is what the route reads.
    lend: ({ run, failed, ...body }) =>
      post(body.token ? '/running/confirm' : '/running', body).then((lend) =>
        lend.terms === undefined
          ? { type: 'lent', l: lend, why: lend.name }
          : {
              type: 'drawn',
              drawn: {
                terms: lend.terms,
                said: [lend.said, lend.review.promise, lend.details.sends.length > 0 && SENDS]
                  .filter(Boolean)
                  .join(' '),
                review: { token: lend.token, ...lend.details },
              },
            },
      ),
    again: groupsAgain(home),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'typed':
        return go(state, {}, { [message.field]: message.value, ...UNSAID });
      case 'choose':
        return go(state, {}, { groups: chosen(state.groups, message), ...UNSAID });
      case 'lend':
        if (state.at.lend === 'lending')
          return no(state, 'a lend is under way, and is answered first');
        if (state.groups.length === 0)
          return go(state, {}, { said: 'Choose which groups can open this first.' });
        return go(
          state,
          { lend: 'lending' },
          {},
          {
            run: 'lend',
            port: state.port,
            name: state.name.trim() || null,
            kind: state.kind.trim() || null,
            groups: state.groups,
            terms: state.terms ?? null,
            pid: state.pid ?? null,
            start: state.start ?? null,
            ...(state.review && { token: state.review.token }),
            failed: 'refused',
          },
        );
      case 'lent':
        return go(state, { lend: 'lent' }, { l: message.l }, { run: 'again', failed: 'told' });
      case 'drawn':
        return go(state, { lend: 'unlent' }, message.drawn);
      case 'told':
        return go(state, {});
      case 'refused':
        return go(state, { lend: 'unlent' }, { said: `${state.said} ${message.why}` });
      default:
        return assertNever(state, message, 'a program found');
    }
  },
});
