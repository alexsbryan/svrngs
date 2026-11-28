// SPDX-License-Identifier: AGPL-3.0-or-later
// The door, both pages' (svrn-docs V1_UI_BUILD §3.1): each person once, a cell in
// a grid, in the order they joined, whoever started the group first, so no
// window moves as people arrive; two of one name told apart by which of
// them joined first, "Ana (1st)" and "Ana (2nd)", true however close they
// joined (#126), and short, so a cell stays one line; a person's popover
// says the day (page/people.js). Unjudged, "asking…",
// while `lends` is not yet answered (undefined); "not known now" when it
// could not be read (null); then lit, "on now", while their computer
// answers, drawn off when it did not, and one in only from a phone marked
// so, with no presence claimed; given `me`, as the home page gives it, its
// own person said "you" (question.js `called`). Given `lent`, as the screen gives it, each
// person's own lends in their window, a link each, opened in a tab, the
// screen's `newest` marked new; an app the group carries is the group's,
// in nobody's window. `counted` is what the screen counts of it
// (group-screen.js): its people, those on now, and the things in their
// windows.
import { html } from './view.js?v=__VERSION__';
import { called, personOf } from '../question.js?v=__VERSION__';
/** @import { Named, PeopleRow } from '../question.js?v=__VERSION__' */
/** @import { Drawn } from './view.js?v=__VERSION__' */

/**
 * A lend as the door draws it: its group, when an app the group carries;
 * its lender's key; where it opens; and its name.
 * @typedef {{ group?: string | null, key: string, origin: string, name: string }} DoorLent
 */

/**
 * What the door judges by: whose computers did not answer, each by name and
 * first key (home/page/group.js `Lends`, web/page/enter.js `Whom`), or by
 * name alone, as a computer at version 2 named them (version.rs), which the
 * door still draws (web/tests/pages/members.test.mjs, "the door as
 * question.js `door` draws it").
 * @typedef {{ unanswered: ({ person: string, person_key: string } | string)[] }} Unanswered
 */

/**
 * A person once: their row, their name as the door says it, and the
 * devices that hold their keys.
 * @typedef {[p: PeopleRow, name: string | undefined, devices: PeopleRow['device'][]]} Person
 */

const since = (/** @type {Named} */ person) => (person.joined === 'started' ? 0 : person.joined.at);
// The nth, as one says it: 1st, 2nd, 3rd, 4th, 11th, 21st.
/** @type {Record<string, string>} */
const SUFFIX = { one: 'st', two: 'nd', few: 'rd', other: 'th' };
const ORDINAL = new Intl.PluralRules('en-US', { type: 'ordinal' });
const nth = (/** @type {number} */ count) => `${count}${SUFFIX[ORDINAL.select(count)]}`;
// Each person once, with their name as the door says it and what holds their keys.
/**
 * @param {PeopleRow[]} rows
 * @param {string} [me]
 * @returns {Person[]}
 */
const people = (rows, me) => {
  const you = personOf(rows, me);
  const everyone = [...new Map(rows.map((row) => [row.person_key, row])).values()].sort(
    (left, right) => since(left) - since(right),
  );
  return everyone.map((person) => {
    const namesakes = everyone.filter(
      (other) => other.person === person.person && other.person_key !== you,
    );
    const when = () => nth(namesakes.indexOf(person) + 1);
    return [
      person,
      called(rows, person.person_key, when, you),
      rows.filter((row) => row.person_key === person.person_key).map((row) => row.device),
    ];
  });
};

// What a person's devices add to their words: a phone beside a computer,
// or a phone alone.
/** @param {PeopleRow['device'][]} devices */
const phoneSaid = (devices) => {
  if (!devices.includes('browser')) return '';
  return devices.includes('computer') ? ', a phone too' : 'in from a phone';
};

// A person's class and words by their devices: a phone alone is judged
// no further; a computer is not known with no `lends` read, off if it did
// not answer that read, and on if it did.
/**
 * @param {PeopleRow['device'][]} devices
 * @param {Unanswered | null} lends
 * @param {boolean | undefined} silent
 * @returns {[drawn: string, words: string]}
 */
const standing = (devices, lends, silent) => {
  if (!devices.includes('computer')) return ['phone', ''];
  if (!lends) return ['', 'not known now'];
  if (silent) return ['off', 'did not answer'];
  return ['on', 'on now'];
};

// Its class and its words, judged by `lends` (/v1/lends' `unanswered`).
/**
 * @param {PeopleRow} person
 * @param {PeopleRow['device'][]} devices
 * @param {Unanswered | null} [lends]
 * @returns {[drawn: string, words: string]}
 */
function judged(person, devices, lends) {
  if (lends === undefined) return ['', 'asking…'];
  const phone = phoneSaid(devices);
  const silent = lends?.unanswered.some((who) =>
    typeof who === 'string' ? who === person.person : who.person_key === person.person_key,
  );
  const [drawn, words] = standing(devices, lends, silent);
  return [drawn, words + phone];
}

// A person's own lends in their window, each a link, the newest marked new.
/**
 * @param {PeopleRow[]} rows
 * @param {DoorLent[]} lent
 * @param {PeopleRow} person
 * @param {string} [newest]
 */
const theirs = (rows, lent, person, newest) =>
  lent
    .filter((lend) => !lend.group && personOf(rows, lend.key) === person.person_key)
    .map((lend) => {
      const fresh = lend.origin === newest && html`${' '}<mark>new</mark>`;
      return html`${' '}<a href=${lend.origin} target="_blank">${lend.name}</a>${fresh}`;
    });

// A person's cell: their name, their own lends, then how they are judged.
// Keyed by the person: when someone goes, their cell goes, and the next
// person's is not drawn into it, taking what was open in it.
/**
 * @param {DoorProps & { lent: DoorLent[] }} door
 * @param {Person} person
 */
const cell = ({ rows, lent, lends, newest, about }, [person, name, devices]) => {
  const [drawn, words] = judged(person, devices, lends);
  const lending = theirs(rows, lent, person, newest);
  return html`
    <li key=${person.person_key} class=${drawn || undefined}>
      ${about ? about(person, name, drawn, words) : html`<strong>${name}</strong>`}
      ${lending}${' '}<small>${words}</small>
    </li>
  `;
};

/**
 * The door's props: the group's people rows; its lends, none when the
 * page gives none; what it judges by, `undefined` unread and `null`
 * unreadable; and the newest lend's origin.
 * @typedef {object} DoorProps
 * @property {PeopleRow[]} rows
 * @property {DoorLent[]} [lent]
 * @property {Unanswered | null} [lends]
 * @property {string} [newest]
 * @property {string} [me] this device's key, whose person is "you"
 * @property {(person: PeopleRow, name: string | undefined, drawn: string, words: string) => Drawn} [about]
 *   what stands for a person's name, where a page makes it a control (the
 *   home page's, page/people.js); not given, the name alone, as the phone's
 *   page and the screen draw it
 */

/** @param {DoorProps} props */
export function Door({ lent = [], ...props }) {
  const cells = people(props.rows, props.me).map((person) => cell({ ...props, lent }, person));
  return html`<ul class="door">${cells}</ul>`;
}

/**
 * @param {PeopleRow[]} rows
 * @param {DoorLent[]} lent
 * @param {Unanswered | null} [lends]
 * @returns {[people: number, on: number, things: number]}
 */
export const counted = (rows, lent, lends) => {
  const everyone = people(rows);
  const on = everyone.filter(([person, , devices]) => judged(person, devices, lends)[0] === 'on');
  const things = everyone.flatMap(([person]) => theirs(rows, lent, person));
  return [everyone.length, on.length, things.length];
};
