// SPDX-License-Identifier: AGPL-3.0-or-later
// Each app on group `g`'s table whose files someone changed since the
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

// A day as question.js `day` says it in this computer's or phone's time,
// "3 Oct" (its :95, not exported there, deleted with it at the switchover).
const day = (t, d = new Date(t * 1000)) => `${d.getDate()} ${'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ')[d.getMonth()]}`;
export function Changes({ g, changed, rows, me, doing = {}, open, send }) {
  if (changed === null) return [html`<p>This computer can’t say whether anyone changed an app in ${g.name ?? 'your group'}.</p>`];
  const you = (c) => (c.by_key ? c.by_key === rows.find((r) => r.key === me)?.person_key : c.by === g.person);
  const each = (changed ?? []).filter((c) => c.group === g.id).map((c) => ((d) => (d.state === 'done' ? html`<p>${spoken(c.app)} opens at its new version for you now.</p>`
    : html`<p>${`${spoken(c.app)}: ${you(c) ? 'you' : (called(rows, c.by_key) ?? c.by)} changed its files on ${day(c.at)}. `}<${Act} words="Open the new version" doing=${d} on=${{ type: 'yes', app: c.app }} send=${send} /></p>`))(doing[c.app] ?? {}));
  return each.length ? [html`<${Fold} summary=${`${each.length} ${each.length === 1 ? 'app' : 'apps'} changed. See what’s new`} fold="changes" open=${open} send=${send}><p>The version you have keeps running until you open the new one.</p>${each}<//>`] : [];
}
