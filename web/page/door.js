// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's door, its removals and who is visiting (page-members (a);
// web/app.js@19b63b2b:459-481), each drawn once the group is read (web/page/enter.js),
// in its place as web/index.html@19b63b2b:83, :85-86 has it: #door, the group
// titled and its door (ui/door.js), judged by the answer the computer
// reached gave, so whose computer did not answer is drawn off; #removals,
// each removal and ask, or a line when they cannot be read (`null`); and
// #visitors, who is visiting now (ui/visitors.js), with a line for you when
// a guest is yours, whose Stop is on the home page that made their link.
// #door holds enter's choice of tab too (enter.js `chosen`). As the first
// visit listed ends, the group is read again (enter.js `reread`), so it
// drops off the page as it ends, where v1 listed it until the next open
// (page-members (b)); one end waited for at a time, the earliest.
import {
  Door,
  go,
  html,
  no,
  removal,
  useEffect,
  Visiting,
  whose,
} from '../ui/kit.js?v=__VERSION__';
import { chosen } from './enter.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Message as PageMessage, Props, State } from './page.js?v=__VERSION__' */

/**
 * The fields this flow owns (`owns`): the end of a visit waited for, in
 * seconds.
 * @typedef {{ ending?: number }} Fields
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'ends', until: number }} Message
 */

/**
 * The effect its step answers: the end of a visit, waited for.
 * @typedef {{ run: 'end', until: number, failed: string }} Effect
 */

/** @param {Group} group */
const titled = (group) => `${group.name}${group.maker ? ` (${whose(group)})` : ''}`;
// Yours by your first key (keys-names), from a computer at version 2 by name (app.js@19b63b2b:466-468).
// Asked only of someone visiting (`visitors`).
/**
 * @param {State} state
 * @param {string} [you]
 */
const yours = (state, you = state.people.find((row) => row.key === state.me)?.person_key) =>
  /** @type {Visitor[]} */ (state.visiting).some((visit) =>
    visit.host_key ? visit.host_key === you : visit.host === state.group.person,
  );
/** @param {State} state */
const visitors = (state) =>
  state.visiting
    ? state.visiting.length > 0 &&
      html`<h2>Visiting now</h2><${Visiting} vs=${state.visiting} rows=${state.people} />${yours(state) && html`<p>You stop your guest’s visit on the home page of the computer you made their link on.</p>`}`
    : html`<p>Who is visiting ${state.group.name} cannot be read just now.</p>`;

export const owns = ['ending'];
/** @type {Takes<State, Message, Effect>} */
export const takes = {
  ends: (state, { until }) =>
    until === state.ending
      ? no(state, `the end at ${until} is waited for already`)
      : go(state, {}, { ending: until }, { run: 'end', until, failed: 'failed' }),
};
/** @type {Io<Effect, PageMessage>} */
export const io = {
  end: ({ until }) =>
    new Promise((resolve) => setTimeout(resolve, until * 1000 - Date.now())).then(() => ({
      type: 'reread',
      why: `a visit ended at ${until}`,
    })),
};
// Drawn while anyone is visiting, it draws nothing, and asks for the earliest end.
/**
 * @typedef {object} EndsProps
 * @property {Visitor[]} vs who is visiting
 * @property {Props['send']} send
 */
/** @param {EndsProps} props */
function Ends({ vs, send }) {
  useEffect(() => send({ type: 'ends', until: Math.min(...vs.map((visit) => visit.until)) }), [vs]);
  return null;
}

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  door: ({ s: state, send }) =>
    html`<div id="door">${chosen(state, send)}${state.lent && html`<p>${titled(state.group)}</p><${Door} rows=${state.people} lends=${{ unanswered: state.unanswered }} />`}</div>`,
  removals: ({ s: state }) =>
    html`<ul id="removals">${state.lent && (state.removals ?? [null]).map((gone) => html`<li>${gone ? removal(gone, state.people) : `Who was removed from ${state.group.name} cannot be read just now.`}</li>`)}</ul>`,
  // No one visiting, or who cannot be read: `undefined > 0` is false.
  visitors: ({ s: state, send }) =>
    html`<section id="visitors">${state.lent && visitors(state)}${/** @type {number} */ (state.visiting?.length) > 0 && html`<${Ends} vs=${state.visiting} send=${send} />`}</section>`,
};
