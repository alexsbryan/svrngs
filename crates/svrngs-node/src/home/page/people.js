// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's people in its section, once GROUP has read them: the door
// (ui/door.js), each person's lends judged by GROUP's read of them. While
// that read is under way the door says "asking…", so it is given no
// lends; when they could not be read, `null`, which it says as not known
// now.
import { html, useEffect, useMachine, useState } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Door } from '../ui/door.js?v=__VERSION__';
import { LEAVE } from './leaving.js?v=__VERSION__';
import { computers, joinedSaid, label, personOf, removal } from '../question.js?v=__VERSION__';
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

/**
 * A person of the door, their name and how they stand as the door says
 * them, and the group, its read and HOME's `dispatch`.
 * @typedef {Pick<PlaceProps, 'g' | 's' | 'me' | 'home'> & { p: PeopleRow, name: string, drawn: string, words: string, gone: (words: string) => void }} PersonProps
 */

// A person in the door, pressed: everyone tries to press a person's box. The
// name is a button, named with how they stand ("Kim, on now"), that opens the
// browser's own popover, as "Address" does, saying only what this page holds
// of them: how they stand and when they joined; what they share here, each
// a link to its way in under its caption, never a second way in, so what
// opening it agrees to stays beside it; or that what they lend is not
// listed. Of yourself, that it is you. And removing them, in two presses,
// since a press does not undo it: a fold that says what it does, and in it
// the button, which asks the node what `svrngs remove` asks (removal.rs
// `RemovalAsk`), the person by name and by when they joined, so never one
// of two namesakes by guess. From whoever keeps the removals it removes;
// from anyone else the node only asks the keeper, and the words say so.
// Each answer is LEAVE's (leaving.js), said in a line a screen reader is
// told of. Removed, their cell goes with the groups' next read, theirs
// alone (ui/door.js keys a cell by its person, so the next person never
// takes a popover left open), and the focus goes to People first.
/** @param {PersonProps} props */
function Person({ p: person, name, drawn, words, g: group, s: state, me, home, gone }) {
  const place = group.name ?? 'this group';
  const [asking, send] = useMachine(() =>
    machine('LEAVE', LEAVE(home), { at: { leave: 'idle' }, said: [], to: 0, name: place }),
  );
  const whose = (/** @type {string} */ key) => personOf(state.people?.people ?? [], key);
  const id = `person-${group.id}-${person.person_key}`;
  // Removed: the focus goes to the fold the door is in, People, before
  // their cell goes with the groups' next read, where it would fall to the
  // top of the page.
  useEffect(() => {
    const fold = document.querySelector(`#${id}`)?.closest('details')?.firstElementChild;
    if (asking.on && asking.at.leave === 'idle') {
      gone(
        `${name} is no longer in ${place}. What they already have stays with them; nothing new reaches them.`,
      );
      /** @type {HTMLElement | null | undefined} */ (fold)?.focus();
    }
  }, [asking.at.leave]);
  const said = [name, words].filter(Boolean).join(', ');
  const shares = (state.lends?.lent ?? [])
    .filter((lend) => !lend.group && whose(lend.key) === person.person_key)
    .map((lend) => html`<li><a href=${`#at-${group.id}-${label(lend)}`}>${lend.name}</a></li>`);
  const act = group.keeps ? `Remove ${name}` : `Ask to remove ${name}`;
  const does = group.keeps
    ? `This puts ${name} out of ${place} on every computer and phone of theirs. What they already have stays with them; nothing new reaches them. This page cannot undo it.`
    : `This asks whoever decides who is removed from ${place}. ${name} stays in it until they do.`;
  const body = { group: group.id, person: person.person, joined: person.joined };
  // While it asks, a second press asks nothing; the button is not disabled,
  // which would drop the focus from it to the top of the page.
  const busy = asking.at.leave === 'asking';
  const ask = () => busy || send({ type: 'ask', on: 'remove', path: '/v1/removals', body });
  const theirs = html`
    <p>${said}, ${joinedSaid(person.joined)}.</p>
    ${drawn === 'off' ? html`<p>${computers([name])} did not answer, so what they lend is not listed.</p>` : html`<p>${shares.length > 0 ? 'Shared here:' : 'Nothing shared here.'}</p><ul>${shares}</ul>`}
    <details><summary>${act} from ${place}</summary><p>${does}</p><button type="button" aria-disabled=${busy} onClick=${ask}>${act}</button></details>
    <p role="status">${asking.said.map(([, answer]) => answer)}</p>
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
export function People({ g: group, s: state, me, home }) {
  // A removal made here, said by the door in the node's words for it
  // (cli.rs `Outcome::Out`): the popover that asked it goes with the cell.
  const [gone, setGone] = useState('');
  if (!state.people) return null;
  /** @type {NonNullable<import('../ui/door.js?v=__VERSION__').DoorProps['about']>} */
  const about = (person, name, drawn, words) =>
    html`<${Person} p=${person} name=${name ?? person.person} drawn=${drawn} words=${words} g=${group} s=${state} me=${me} home=${home} gone=${setGone} />`;
  return html`<${Door} rows=${state.people.people} lends=${judged(state)} about=${about} /><p role="status">${gone}</p>`;
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
  // Told to a screen reader as a line arrives: a removal made on this page
  // is said here, its person's popover gone with their cell.
  return html`<ul class="removals" aria-live="polite">${said}</ul>`;
}
