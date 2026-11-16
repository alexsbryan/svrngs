// SPDX-License-Identifier: AGPL-3.0-or-later
// The rows of the group's index once GROUP has read its lends: each lend
// (lend-row.js), whose by the key its computer joined with (index.js gives
// each its lender by question.js `lentBy`, never written into the read);
// then each computer whose lends are not listed, said by why, so what a
// computer lends is never stale and never nobody's.
import { html } from '../ui/view.js?v=__VERSION__';
import { Row } from '../ui/parts.js?v=__VERSION__';
import { called, label } from '../question.js?v=__VERSION__';
import { LendRow } from './lend-row.js?v=__VERSION__';
/** @import { Lends, Lent, PlaceProps } from './group.js?v=__VERSION__' */

const UNLISTED = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has not heard of yours yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  older: 'runs an older svrngs, so what they lend is not listed',
};

/**
 * The lends read, each listed once with its lender (`lent`), what is said
 * once above the table (`above`), and what each row is drawn with.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me'> & { lends: Lends, lent: Lent[], above: string[] }} IndexRowsProps
 */

/** @param {IndexRowsProps} props */
export function IndexRows({ lends, lent, above, g: group, s: state, me }) {
  const people = state.people?.people ?? [];
  const each = lent.map(
    (lend) => html`
      <${LendRow}
        key=${label(lend)}
        l=${lend}
        shelf=${lends.shelf}
        above=${above}
        g=${group}
        s=${state}
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
  const said = [...unlisted, ...none].map((words) => html`<${Row} cells=${[words]} span=${2} />`);
  return [...each, ...said];
}
