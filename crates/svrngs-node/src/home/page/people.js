// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's people in its section, once GROUP has read them: the door
// (ui/door.js), each person's lends judged by GROUP's read of them. While
// that read is under way the door says "asking…", so it is given no
// lends; when they could not be read, `null`, which it says as not known
// now.
import { html } from '../ui/view.js?v=__VERSION__';
import { Copy } from '../ui/copy.js?v=__VERSION__';
import { Door, joinedAt } from '../ui/door.js?v=__VERSION__';
import { computers, joinedSaid, label, removal } from '../question.js?v=__VERSION__';
/** @import { Lends, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { PeopleRow, Removal } from '../question.js?v=__VERSION__' */

/**
 * The lends as the door is given them, by GROUP's read of them.
 * @param {PlaceProps['s']} state
 * @returns {Lends | null | undefined}
 */
const judged = (state) => {
  if (state.at.lends === 'read') return state.lends;
  if (state.at.lends === 'unread') return null;
  return undefined;
};

// A name or a group as a shell takes it whatever it holds: in single
// quotes, one inside it closed, escaped and opened again.
/** @param {string} words */
const quoted = (words) => `'${words.replaceAll("'", "'\\''")}'`;

/**
 * A person of the door, their name and how they stand as the door says
 * them, and the group and its read.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me'> & { p: PeopleRow, name: string, drawn: string, words: string }} PersonProps
 */

// A person in the door, pressed: everyone tries to press a person's box. The
// name is a button, named with how they stand ("Kim, on now"), that opens the
// browser's own popover, as "Address" does, saying only what this page holds
// of them: how they stand and when they joined; what they share here, each
// a link to its way in under its caption, never a second way in, so what
// opening it agrees to stays beside it; or that what they lend is not
// listed. Of yourself, that it is you. Whoever keeps the group's order of
// removals is given the line that removes them, whole, to paste in a
// terminal: the page removes nobody (cli.rs `Verb::Remove`; from anyone
// else the same line only asks the keeper, so it is not offered them).
/** @param {PersonProps} props */
function Person({ p: person, name, drawn, words, g: group, s: state, me }) {
  const whose = (/** @type {string} */ key) =>
    state.people?.people.find((row) => row.key === key)?.person_key;
  const id = `person-${group.id}-${person.person_key}`;
  const said = [name, words].filter(Boolean).join(', ');
  const shares = (state.lends?.lent ?? [])
    .filter((lend) => !lend.group && whose(lend.key) === person.person_key)
    .map((lend) => html`<li><a href=${`#at-${group.id}-${label(lend)}`}>${lend.name}</a></li>`);
  // Of two people of one name the line says which, by when they joined, as
  // the door tells them apart: without it the command changes nothing.
  const which = name === person.person ? '' : ` --joined ${quoted(joinedAt(person.joined))}`;
  const line = `svrngs remove ${quoted(person.person)} --group ${quoted(group.name ?? '')}${which}`;
  const theirs = html`
    <p>${said}, ${joinedSaid(person.joined)}.</p>
    ${drawn === 'off' ? html`<p>${computers([name])} did not answer, so what they lend is not listed.</p>` : html`<p>${shares.length > 0 ? 'Shared here:' : 'Nothing shared here.'}</p><ul>${shares}</ul>`}
    ${group.keeps === true && group.name !== null && html`<p>To remove ${name} from ${group.name}, paste this line in a terminal on this computer. What they already have stays with them; nothing new reaches them.</p><p><code>${line}</code> <${Copy} text=${line} words=${{ copy: 'Copy the line', copied: 'Copied.' }} /></p>`}
  `;
  return html`
    <button type="button" popovertarget=${id} aria-label=${said}><strong>${name}</strong></button>
    <div popover="auto" id=${id} class="person" role="dialog" aria-label=${said} tabindex="-1" autofocus>
      ${whose(me) === person.person_key ? html`<p>This is you.</p>` : theirs}
      <button type="button" popovertarget=${id} popovertargetaction="hide" aria-label=${`Close: ${said}`}>Close</button>
    </div>
  `;
}

/** @param {PlaceProps} props */
export function People({ g: group, s: state, me }) {
  if (!state.people) return null;
  /** @type {NonNullable<import('../ui/door.js?v=__VERSION__').DoorProps['about']>} */
  const about = (person, name, drawn, words) =>
    html`<${Person} p=${person} name=${name ?? person.person} drawn=${drawn} words=${words} g=${group} s=${state} me=${me} />`;
  return html`<${Door} rows=${state.people.people} lends=${judged(state)} about=${about} />`;
}

// Who was removed from a group, or asked to be, under its door once its
// people are read, each said by question.js `removal`; nothing when no one
// was.
/** @param {PlaceProps} props */
export function Removals({ g: group, s: state }) {
  if (!state.people) return null;
  // `removals` is `null` only beside a `removed` of `null`, and a group
  // whose `removed` is `null` draws no place (group.js `below`).
  const removals = /** @type {Removal[]} */ (group.removals);
  if (removals.length === 0) return null;
  const people = state.people.people;
  const said = removals.map((gone) => html`<li>${removal(gone, people)}</li>`);
  return html`<ul class="removals">${said}</ul>`;
}
