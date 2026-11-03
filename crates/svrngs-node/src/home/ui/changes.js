// SPDX-License-Identifier: AGPL-3.0-or-later
// Each app on `group`'s table whose files someone changed since the
// version that runs as you, both pages' (svrn-docs WHAT_RUNS_AS_YOU §3),
// never in the way: one line, "2 apps changed. See what’s new", opening to
// who and when for each, with "Open the new version", which sends `{type:
// 'yes', app}`. Until the person says yes, the version they said yes to, or
// the one brought, runs as them. Who changed it is told by key, of the
// people rows `rows`, this device's being `me`: an older computer's answer,
// with none, by name. `doing[app]` is the page's answer to its yes, as
// ui/parts.js `Act` draws it: once `done`, its line says so in its place; a
// refusal is said beside the button, after any before it. The fold's open
// is the page's, `fold changes`. Lines, for "Reads and writes for you"
// (ui/acts.js).
import { html } from './view.js?v=__VERSION__';
import { Act, Fold } from './parts.js?v=__VERSION__';
import { called, day, spoken } from '../question.js?v=__VERSION__';
/** @import { Drawn } from './view.js?v=__VERSION__' */
/** @import { Doing } from './parts.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */

/**
 * A change to an app the group carries: the group's id, the app's kind,
 * who changed its files, by name and first key, and when, in seconds.
 * @typedef {{ group: string, app: string, by: string, by_key?: string, at: number }} Change
 */

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
    .map((change) => {
      const act = doing[change.app] ?? {};
      const app = spoken(change.app);
      if (act.state === 'done') return html`<p>${app} opens at its new version for you now.</p>`;
      const by = you(change) ? 'you' : (called(rows, change.by_key) ?? change.by);
      const line = `${app}: ${by} changed its files on ${day(change.at)}. `;
      return html`
        <p>
          ${line}
          <${Act} words="Open the new version" doing=${act} on=${{ type: 'yes', app: change.app }} send=${send} />
        </p>
      `;
    });
  if (each.length === 0) return [];
  const summary = `${each.length} ${each.length === 1 ? 'app' : 'apps'} changed. See what’s new`;
  return [
    html`
      <${Fold} summary=${summary} fold="changes" open=${open} send=${send}>
        <p>The version you have keeps running until you open the new one.</p>
        ${each}
      <//>
    `,
  ];
}
