// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// "Make a group" (page-join (d); home.js@19b63b2b:415-422): a group made here as
// `svrngs new` makes one, by the svrngs running here (start-group), from
// its name and the person's. Its machine, MAKE, is the place's own
// (ui/view.js `useMachine`). A name a group here has already is said below
// as it is typed (#138), each name asked of `GET /group` counted, so an
// older answer is refused by name (`late`), where v1 compared the typed
// value (home.js@19b63b2b:420). Pressed, a second press is refused while it makes; made,
// HOME says so, and again whose namesakes there are (`take`), and reads the
// groups again, where v1 opened the page again (home.js@19b63b2b:364; page-join (d)'s
// second fix), and the form is empty again; a refusal is said beside the
// button once, the last in place of the one before, where v1 appended each
// (home.js@19b63b2b:421; page-join (d)'s first fix).
import {
  Act,
  ask,
  Field,
  go,
  html,
  late,
  machine,
  namesakes,
  no,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';

/**
 * HOME's message this file takes: a group made, by its name, the groups
 * of its name said, and why its folder could not be made, `null` when it
 * was.
 * @typedef {{ type: 'made', flow: 'make', q: { made: string, also: string, unplaced: string | null } }} Taken
 */

const FIRST = { at: { make: 'typing' }, name: '', person: '', before: '', said: '' };
export const MAKE = (home) => ({
  states: { make: ['typing', 'checking', 'checked', 'making', 'refused'] },
  io: {
    check: ({ name, n }) =>
      get(`/group?${new URLSearchParams({ name })}`)
        .then(
          (b) => namesakes(b, name, 'This makes'),
          (e) => (name ? e.message : ''),
        )
        .then((words) => ({ type: 'checked', n, words })),
    make: ({ name, person }) =>
      post('/group', [name, person]).then(
        ([made, unplaced, beside]) => (
          home({
            type: 'made',
            flow: 'make',
            q: { made, also: namesakes(beside, made, 'This is'), unplaced },
          }),
          home({ type: 'again' }),
          { type: 'made', why: 'said, the groups read again' }
        ),
      ),
  },
  step: (s, m) =>
    ({
      typed: () =>
        m.field === 'person'
          ? go(s, {}, { person: m.value })
          : ((asked, n) =>
              go(
                s,
                { make: 'checking' },
                { name: m.value, asked },
                { run: 'check', name: m.value, n, failed: 'checked' },
              ))(...ask(s, 'name')),
      checked: () =>
        late(s, 'name', m.n)
          ? no(s, late(s, 'name', m.n))
          : go(
              s,
              { make: s.at.make === 'checking' ? 'checked' : s.at.make },
              { before: m.words ?? m.why },
            ),
      make: () =>
        s.at.make === 'making'
          ? no(s, 'a group is being made, and is answered first')
          : go(
              s,
              { make: 'making' },
              {},
              { run: 'make', name: s.name, person: s.person, failed: 'refused' },
            ),
      made: () => go(s, FIRST.at, FIRST),
      refused: () => go(s, { make: 'refused' }, { said: ` ${m.why}` }),
    })[m.type]?.() ?? no(s, `making a group takes no ${m.type}`),
});
export const take = (s, m) => go(s, { said: 'made' }, { q: m.q });

const Make = ({ home }) =>
  ((s, send) =>
    html`<section id="make"><h2>Make a group</h2><p>${[
      ['name', 'A name for the group: '],
      ['person', 'Your name in it: '],
    ].map(
      ([n, w]) => html`<${Field} name=${n} words=${w} value=${s[n]} send=${send} />`,
    )}<${Act} words="Make a group" doing=${{ state: s.at.make === 'making' ? 'busy' : 'idle', said: s.said }} on=${{ type: 'make' }} send=${send} /></p><p>${s.before}</p></section>`)(
    ...useMachine(() => machine('MAKE', MAKE(home), FIRST)),
  );
export const view = { make: ({ home }) => html`<${Make} home=${home} />` };
