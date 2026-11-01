// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting now, both pages' (page-members (a); question.js
// `visiting`, :43, which it moves; GUESTS §5 "The pages"), from the record,
// as every member's page shows them: each guest, by host, until a day and
// an hour, the host by the name the door gives them (`called`).
import { html } from './view.js?v=__VERSION__';
import { called, until } from '../question.js?v=__VERSION__';

export const Visiting = ({ vs, rows = [] }) => html`<ul class="visiting">${[...vs].sort((a, b) => a.host.localeCompare(b.host)).map((v) => html`<li>${`${v.guest}, ${called(rows, v.host_key) ?? v.host}’s guest, until ${until(v)}`}</li>`)}</ul>`;
