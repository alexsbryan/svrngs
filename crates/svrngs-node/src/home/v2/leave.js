// SPDX-License-Identifier: AGPL-3.0-or-later
// Leaving a group, in its section on the home page (page-members (d);
// home.js:40-42, :145-153; groups-leave), what stays and what stops said
// before; whoever keeps the order of removals chooses, beside it, who of
// the group's other people keeps it next. Each press is LEAVE, a machine
// of the section's own, which asks the node (removal.rs) and says each
// answer after the one before, beside the button pressed, as v1's
// `p.append` does: the node's outcome (removal.rs `Outcome`, a 409 one
// too), or a refusal in its own words, its session lapsed (a 401's page,
// home.rs:198) said as group.js `get` says it, where v1 read every
// answer as JSON (home.js:42) and said a parse error. A leave goes on to
// the page saying so: HOME takes `left` here (`take`), which says it and
// reads the groups again, never a reload (v1's `location.assign`).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { called } from '../question.js?v=__VERSION__';

const SAYS = { handed: (s) => `${s.person} keeps the order of removals in ${s.name} now, so you can leave it.`, decides: (s) => `You keep the order of removals in ${s.name}: choose who keeps it after you, then leave.`, not_in: (s) => `You are no longer in ${s.name}.` };
const LEAVE = (home) => ({
  states: { leave: ['idle', 'asking', 'handed', 'decides', 'refused'] },
  step: (s, m, said = (w) => ({ said: [...s.said, [s.on, ` ${w}`]] })) => ({
    to: () => go(s, {}, { to: m.i }),
    ask: () => go(s, { leave: 'asking' }, { on: m.on, person: m.body.person }, { run: 'ask', path: m.path, body: m.body, failed: 'refused' }),
    answered: () => (m.o === 'left' ? go(s, { leave: 'idle' }, {}, { run: 'send', m: { type: 'left', flow: 'leave', name: s.name } }) : go(s, { leave: { handed: 'handed', decides: 'decides' }[m.o] ?? 'refused' }, said(SAYS[m.o]?.(s) ?? `${s.name} cannot be changed on this computer now.`))),
    refused: () => go(s, { leave: 'refused' }, said(`${s.name} cannot be changed on this computer now: ${m.why}`)),
    sent: () => go(s, {}),
  })[m.type]?.() ?? no(s, `leaving takes no ${m.type}`),
  io: { ask: ({ path, body }) => fetch(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }).then(async (r) => ((a) => (a.outcome ? { type: 'answered', o: a.outcome, why: a.outcome } : Promise.reject(new Error(r.status === 401 ? 'this page’s session has ended. Open svrngs from your Applications, or on Linux your applications menu, and it opens here again' : a.error || `answered ${r.status}`))))(await r.json().catch(() => ({})))),
    send: ({ m }) => (home(m), { type: 'sent', why: m.type }) },
});
function Leave({ g, rows, me, home, name = g.name ?? 'this group' }) {
  const [s, send] = useMachine(() => machine('LEAVE', LEAVE(home), { at: { leave: 'idle' }, said: [], to: 0, name })), own = rows.find((r) => r.key === me)?.person_key;
  const others = [...new Map(rows.filter((r) => r.device === 'computer' && r.person_key !== own).map((r) => [r.person_key, r])).values()];
  const press = (on, path, body) => () => send({ type: 'ask', on, path, body }), says = (on) => s.said.filter((w) => w[0] === on).map((w) => w[1]);
  return html`${g.keeps && others.length > 0 && html`<p>You keep the order of removals in ${name}. Choose who keeps it after you, then leave: <select onChange=${(e) => send({ type: 'to', i: e.target.value })}>${others.map((r, i) => html`<option value=${i}>${called(rows, r.person_key)}</option>`)}</select><button onClick=${press('hand', '/v1/handovers', { group: g.id, ...others[s.to] })}>Hand it on</button>${says('hand')}</p>`}<p class="leave">Leaving ${name} stops it reaching you, and stops what you lend reaching its people. What you have of it stays on this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet. <button onClick=${press('leave', '/v1/leaves', { group: g.id })}>Leave ${name}</button>${says('leave')}</p>`;
}

// HOME's `left`: the leave said (home.js SAYS `left`), the groups read again.
export const take = (s, m, [asked, n] = ask(s, 'groups')) => (m.type === 'left' ? go(s, { groups: 'reading', said: 'left' }, { asked, q: { ...s.q, left: m.name } }, { run: 'groups', n, round: 'again', failed: 'unread' }) : no(s, `leaving takes no ${m.type}`));
export const view = { leave: ({ g, s, home, me }) => s.people && html`<${Leave} g=${g} rows=${s.people.people} me=${me} home=${home} />` };
