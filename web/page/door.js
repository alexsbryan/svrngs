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
import { Door, go, html, no, removal, useEffect, Visiting, whose } from '../ui/kit.js?v=__VERSION__';
import { chosen } from './enter.js?v=__VERSION__';

const titled = (g) => `${g.name}${g.maker ? ` (${whose(g)})` : ''}`;
// Yours by your first key (keys-names), from a computer at version 2 by name (app.js@19b63b2b:466-468).
const yours = (s, you = s.people.find((r) => r.key === s.me)?.person_key) => s.visiting.some((v) => (v.host_key ? v.host_key === you : v.host === s.group.person));
const visitors = (s) => (s.visiting ? s.visiting.length > 0 && html`<h2>Visiting now</h2><${Visiting} vs=${s.visiting} rows=${s.people} />${yours(s) && html`<p>You stop your guest’s visit on the home page of the computer you made their link on.</p>`}` : html`<p>Who is visiting ${s.group.name} cannot be read just now.</p>`);

export const owns = ['ending'];
export const takes = { ends: (s, { until }) => (until === s.ending ? no(s, `the end at ${until} is waited for already`) : go(s, {}, { ending: until }, { run: 'end', until, failed: 'failed' })) };
export const io = { end: ({ until }) => new Promise((r) => setTimeout(r, until * 1000 - Date.now())).then(() => ({ type: 'reread', why: `a visit ended at ${until}` })) };
// Drawn while anyone is visiting, it draws nothing, and asks for the earliest end.
function Ends({ vs, send }) {
  useEffect(() => send({ type: 'ends', until: Math.min(...vs.map((v) => v.until)) }), [vs]);
  return null;
}

export const view = {
  door: ({ s, send }) => html`<div id="door">${chosen(s, send)}${s.lent && html`<p>${titled(s.group)}</p><${Door} rows=${s.people} lends=${{ unanswered: s.unanswered }} />`}</div>`,
  removals: ({ s }) => html`<ul id="removals">${s.lent && (s.removals ?? [null]).map((r) => html`<li>${r ? removal(r, s.people) : `Who was removed from ${s.group.name} cannot be read just now.`}</li>`)}</ul>`,
  visitors: ({ s, send }) => html`<section id="visitors">${s.lent && visitors(s)}${s.visiting?.length > 0 && html`<${Ends} vs=${s.visiting} send=${send} />`}</section>`,
};
