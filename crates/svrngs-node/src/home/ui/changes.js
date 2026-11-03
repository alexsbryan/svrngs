// SPDX-License-Identifier: AGPL-3.0-or-later
// Each app on `group`'s table whose files someone changed since the
// version that runs as you, both pages' (page-acts (a); question.js
// `changes`, :212-229, which it moves; canon-pin (c); svrn-docs
// WHAT_RUNS_AS_YOU §3), never in the way: one line, "2 apps changed. See
// what’s new", opening to who and when for each, with "Open the new
// version", which sends `{type: 'yes', app}`. Until the person says yes,
// the version they said yes to, or the one brought, runs as them. Who
// changed it is told by key, of the people rows `rows`, this device's being
// `me`: an older computer's answer, with none, by name. `doing[app]` is the
// page's answer to its yes, as ui/parts.js `Act` draws it: once `done`, its
// line says so in its place, as v1 replaces the line; a refusal is said
// beside the button, after any before it, as v1 appends each. The fold's
// open is the page's, `fold changes`. Lines, for "Reads and writes for
// you" (ui/acts.js).
import { html } from './view.js?v=__VERSION__';
import { Act, Fold } from './parts.js?v=__VERSION__';
import { called, spoken } from '../question.js?v=__VERSION__';
/** @import { Drawn } from './view.js?v=__VERSION__' */
/** @import { Doing } from './parts.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */

/**
 * A change to an app the group carries: the group's id, the app's kind,
 * who changed its files, by name and first key, and when, in seconds.
 * @typedef {{ group: string, app: string, by: string, by_key?: string, at: number }} Change
 */

// A day as question.js `day` says it in this computer's or phone's time,
// "3 Oct" (its :95, not exported there, deleted with it at the switchover).
const day = (/** @type {number} */ seconds, date = new Date(seconds * 1000)) =>
  `${date.getDate()} ${'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ')[date.getMonth()]}`;
/**
 * The lines' props: the group, its name and this device's person in it as
 * each page holds them; its changes, `null` when they could not be read;
 * its people rows and this device's key; the page's answer to each yes, by
 * app; whether the fold is open; and where a yes and the fold are sent.
 * @typedef {object} ChangesProps
 * @property {{ id: string, name: string | null, person: string | null }} g
 * @property {Change[] | null} [changed]
 * @property {PeopleRow[]} rows
 * @property {string} [me]
 * @property {Record<string, Doing>} [doing]
 * @property {boolean} [open]
 * @property {(message: { type: 'yes', app: string } | { type: 'fold', fold: 'changes', open: boolean }) => void} send
 */

/**
 * @param {ChangesProps} props
 * @returns {Drawn[]}
 */
export function Changes({ g: group, changed, rows, me, doing = {}, open, send }) {
  if (changed === null)
    return [
      html`<p>This computer can’t say whether anyone changed an app in ${group.name ?? 'your group'}.</p>`,
    ];
  const you = (/** @type {Change} */ change) =>
    change.by_key
      ? change.by_key === rows.find((row) => row.key === me)?.person_key
      : change.by === group.person;
  const each = (changed ?? [])
    .filter((change) => change.group === group.id)
    .map((char) =>
      ((act) =>
        act.state === 'done'
          ? html`<p>${spoken(char.app)} opens at its new version for you now.</p>`
          : html`<p>${`${spoken(char.app)}: ${you(char) ? 'you' : (called(rows, char.by_key) ?? char.by)} changed its files on ${day(char.at)}. `}<${Act} words="Open the new version" doing=${act} on=${{ type: 'yes', app: char.app }} send=${send} /></p>`)(
        doing[char.app] ?? {},
      ),
    );
  return each.length
    ? [
        html`<${Fold} summary=${`${each.length} ${each.length === 1 ? 'app' : 'apps'} changed. See what’s new`} fold="changes" open=${open} send=${send}><p>The version you have keeps running until you open the new one.</p>${each}<//>`,
      ]
    : [];
}
