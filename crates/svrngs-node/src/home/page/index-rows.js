// SPDX-License-Identifier: AGPL-3.0-or-later
// The rows of the group's index once GROUP has read its lends: each lend
// (lend-row.js), whose by the key its computer joined with (question.js
// `apart`, never written into the read); then each computer whose lends
// are not listed, said by why, so what a computer lends is never stale and
// never nobody's.
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { called, label, lentBy, once } from '../question.js?v=__VERSION__';
import { LendRow } from './lend-row.js?v=__VERSION__';
/** @import { Lends, PlaceProps } from './group.js?v=__VERSION__' */

const UNLISTED = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has not heard of yours yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  older: 'runs an older svrngs, so what they lend is not listed',
};

/**
 * The lends read, and what each row is drawn with.
 * @typedef {Omit<PlaceProps, 'joined'> & { lends: Lends }} IndexRowsProps
 */

/** @param {IndexRowsProps} props */
export function IndexRows({ lends, g: group, s: state, send, home, me }) {
  const people = state.people?.people ?? [];
  const lent = lends.lent.map((lend) => lentBy(lend, people));
  const each = once(lent).map(
    (lend) => html`
      <${LendRow}
        key=${label(lend)}
        l=${lend}
        shelf=${lends.shelf}
        g=${group}
        s=${state}
        send=${send}
        home=${home}
        me=${me}
      />
    `,
  );
  const unlisted = Object.entries(UNLISTED).flatMap(([how, why]) =>
    // Object.entries names each key a string; each is UNLISTED's, a list of
    // the lends' read.
    lends[/** @type {keyof typeof UNLISTED} */ (how)].map(
      (person) => `${called(people, person.person_key) ?? person.person}'s computer ${why}.`,
    ),
  );
  const none = each.length || unlisted.length ? [] : ['Nobody lends anything here yet.'];
  const said = [...unlisted, ...none].map((words) => html`<${Row} cells=${[words]} span=${4} />`);
  return [...each, ...said];
}
