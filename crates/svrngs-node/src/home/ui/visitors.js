// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting now, both pages' (svrn-docs GUESTS §5 "The pages"), from the record,
// as every member's page shows them: each guest, by host, until a day and
// an hour, the host by the name the door gives them (`called`).
import { html } from './view.js?v=__VERSION__';
import { called, until } from '../question.js?v=__VERSION__';
/** @import { Named } from '../question.js?v=__VERSION__' */

/**
 * A guest visiting now: their name, whose guest, by name and first key,
 * and until when, in seconds.
 * @typedef {{ guest: string, host: string, host_key?: string, until: number }} Visitor
 */

/**
 * Who is visiting, and the group's people, by whom each host is named.
 * @typedef {object} VisitingProps
 * @property {Visitor[]} vs
 * @property {Named[]} [rows]
 */

/** @param {VisitingProps} props */
export function Visiting({ vs, rows = [] }) {
  const byHost = [...vs].sort((left, right) => left.host.localeCompare(right.host));
  const lines = byHost.map((visit) => {
    const host = called(rows, visit.host_key) ?? visit.host;
    const line = `${visit.guest}, ${host}’s guest, until ${until(visit)}`;
    return html`<li>${line}</li>`;
  });
  return html`<ul class="visiting">${lines}</ul>`;
}
