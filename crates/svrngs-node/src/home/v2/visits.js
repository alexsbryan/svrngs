// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting a group, in its section on the home page (page-members
// (b); home.js:155-166; GUESTS §5 "The pages"), from the record as every
// member's page shows them, once GROUP has read its people: who you have
// over first (GUESTS's "Your guests", in words that are not an app's noun),
// each with Stop, which says what it does before it does it; then
// everyone's, by host, with until when (ui/visitors.js). Each Stop is
// VISITS, a machine of the visit's own: /visits stops it, and its answer's
// time is after every act its computer holds, so may be a second on
// (svrngs-core command.rs `sign_lines`); the page is drawn again then,
// sooner reading the guest as still here. A refusal is said after the one
// before, as v1's `li.append` does.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Visiting } from '../ui/visitors.js?v=__VERSION__';
import { until } from '../question.js?v=__VERSION__';
import { post } from './door.js?v=__VERSION__';

const VISITS = {
  states: { visit: ['on', 'stopping', 'stopped', 'refused'] },
  step: (s, m) => ({
    stop: () => go(s, { visit: 'stopping' }, {}, { run: 'stop', g: m.g, id: m.id, failed: 'refused' }),
    stopped: () => go(s, { visit: 'stopped' }, {}, { run: 'wait', ts: m.ts, failed: 'refused' }),
    refused: () => go(s, { visit: 'refused' }, { said: [...s.said, ` ${m.why}`] }),
  })[m.type]?.() ?? no(s, `a visit takes no ${m.type}`),
  io: { stop: ({ g, id }) => post('/visits', [g, id]).then((a) => ({ type: 'stopped', ts: a.ts_unix, why: `its Stop is at ${a.ts_unix}` })),
    wait: ({ ts }) => new Promise((r) => setTimeout(r, ts * 1000 - Date.now())).then(() => new Promise(() => location.reload())) },
};
function Stop({ g, v, name }) {
  const [s, send] = useMachine(() => machine('VISITS', VISITS, { at: { visit: 'on' }, said: [] }));
  return html`<li>${`${v.guest}, until ${until(v)}. Stop ends the visit now: what you lend stops opening to ${v.guest}, and everyone in ${name} sees it ended. `}<button onClick=${() => send({ type: 'stop', g: g.id, id: v.id })}>${`Stop ${v.guest}’s visit`}</button>${s.said}</li>`;
}

const listed = (g, rows, name, mine = (g.visiting ?? []).filter((v) => v.yours)) =>
  html`${!g.visiting && html`<p>This computer cannot say who is visiting ${name} now.</p>`}${mine.length > 0 && html`<h3>Who you have over</h3><ul class="yours">${mine.map((v) => html`<${Stop} key=${v.id} g=${g} v=${v} name=${name} />`)}</ul>`}${g.visiting?.length > 0 && html`<h3>Visiting now</h3><${Visiting} vs=${g.visiting} rows=${rows} />`}`;
export const view = { visits: ({ g, s }) => s.people && listed(g, s.people.people, g.name ?? 'this group') };
