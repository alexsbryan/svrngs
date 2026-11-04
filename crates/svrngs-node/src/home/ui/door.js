// SPDX-License-Identifier: AGPL-3.0-or-later
// The door, both pages' (svrn-docs V1_UI_BUILD §3.1): each person once, a cell in
// a grid, in the order they joined, whoever started the group first, so no
// window moves as people arrive; two of one name told apart by when each
// joined, in the form `svrngs remove --joined` takes (home.rs `WHEN`),
// this computer's time then its offset as ±hhmm (#126). Unjudged, "asking…",
// while `lends` is not yet answered (undefined); "not known now" when it
// could not be read (null); then lit, "on now", while their computer
// answers, drawn off when it did not, and one in only from a phone marked
// so, with no presence claimed. Given `lent`, as the screen gives it, each
// person's own lends in their window, a link each, opened in a tab, the
// screen's `newest` marked new; an app the group carries is the group's,
// in nobody's window. `counted` is what the screen counts of it
// (group-screen.js): its people, those on now, and the things in their
// windows.
import { html } from './view.js?v=__VERSION__';
import { called } from '../question.js?v=__VERSION__';
/** @import { Joined, Named, PeopleRow } from '../question.js?v=__VERSION__' */

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
// A time as `svrngs remove --joined` takes it: this computer's, then its
// offset from UTC as ±hhmm.
/** @param {Date} date */
const at = (date) => {
  const minutes = -date.getTimezoneOffset();
  const sign = minutes < 0 ? '-' : '+';
  const hhmm = Math.trunc(Math.abs(minutes) / 60) * 100 + (Math.abs(minutes) % 60);
  return `${date.toLocaleString('sv-SE')} ${sign}${String(hhmm).padStart(4, '0')}`;
};
const when = (/** @type {Joined} */ joined) =>
  joined === 'started' ? 'started the group' : `joined ${at(new Date(joined.at * 1000))}`;
// Each person once, with their name as the door says it and what holds their keys.
/**
 * @param {PeopleRow[]} rows
 * @returns {Person[]}
 */
const people = (rows) =>
  [...new Map(rows.map((row) => [row.person_key, row])).values()]
    .sort((left, right) => since(left) - since(right))
    .map((person) => [
      person,
      called(rows, person.person_key, when),
      rows.filter((row) => row.person_key === person.person_key).map((row) => row.device),
    ]);

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
    .filter(
      (lend) =>
        !lend.group && rows.find((row) => row.key === lend.key)?.person_key === person.person_key,
    )
    .map((lend) => {
      const fresh = lend.origin === newest && html`${' '}<mark>new</mark>`;
      return html`${' '}<a href=${lend.origin} target="_blank">${lend.name}</a>${fresh}`;
    });

// A person's cell: their name, their own lends, then how they are judged.
/**
 * @param {DoorProps & { lent: DoorLent[] }} door
 * @param {Person} person
 */
const cell = ({ rows, lent, lends, newest }, [person, name, devices]) => {
  const [drawn, words] = judged(person, devices, lends);
  const lending = theirs(rows, lent, person, newest);
  return html`
    <li class=${drawn || undefined}>
      <strong>${name}</strong>
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
 */

/** @param {DoorProps} props */
export function Door({ rows, lent = [], lends, newest }) {
  const cells = people(rows).map((person) => cell({ rows, lent, lends, newest }, person));
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
