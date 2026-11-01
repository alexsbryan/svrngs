// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's people on the home page (page-members (a); home.js:121-132,
// :154), each in its place in the group's section (group.js PLACES), once
// GROUP has read them: the door (ui/door.js), "asking…" while the group's
// lends are being read, judged by their answer, "not known now" when they
// could not be read; where the group's folder is on this disk, each opened
// in a window of its own by its button, FOLDER, a machine of the button's
// own, each open counted, so the last one's answer alone is said, once,
// where v1's `p.append` said each after the one before (home.js:130);
// and who was removed or asked to be, when anyone was. `decides` is who
// decides for you in it, which "What acts as you" says (page-acts).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Door } from '../ui/door.js?v=__VERSION__';
import { deciders, removal } from '../question.js?v=__VERSION__';

// `body` posted as JSON, its answer read, or its refusal's words (home.js:28-37).
export const post = (path, body) => fetch(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }).then(async (r) => ((a) => (r.ok ? a : Promise.reject(new Error(a.error || `answered ${r.status}`))))(await r.json().catch(() => ({}))));
const its = (g) => (g.name ? `${g.name}’s folder` : 'The group’s folder');
const answered = (s, m, words) => (late(s, 'open', m.n) ? no(s, late(s, 'open', m.n)) : go(s, { folder: m.type }, { said: words }));
const FOLDER = {
  states: { folder: ['idle', 'opening', 'opened', 'unopened'] },
  step: (s, m) => ({
    open: ([asked, n] = ask(s, 'open')) => go(s, { folder: 'opening' }, { asked }, { run: 'open', n, group: m.group, dir: m.dir, failed: 'unopened' }),
    opened: () => answered(s, m, ' Opened, in a window of its own.'),
    unopened: () => answered(s, m, ` It could not be opened: ${m.why}.`),
  })[m.type]?.() ?? no(s, `a folder takes no ${m.type}`),
  io: { open: ({ n, group, dir }) => post('/folder', { group, dir }).then(() => ({ type: 'opened', n, why: dir })) },
};
function Folder({ g, dir }) {
  const [s, send] = useMachine(() => machine('FOLDER', FOLDER, { at: { folder: 'idle' }, said: '' }));
  return html`<p class="folder">${its(g)} is on this computer at ${dir}</p><p><button onClick=${() => send({ type: 'open', group: g.id, dir })}>Open this folder</button>${s.said}</p>`;
}

export const decides = (g, rows, me) => `${g.name ?? 'Your group'}: ${deciders(rows, me, 'computer')}.`;
export const view = {
  people: ({ s }) => s.people && html`<${Door} rows=${s.people.people} lends=${{ read: s.lends, unread: null }[s.at.lends]} />`,
  folder: ({ g, s }) => s.people && (g.folders === null ? html`<p>This computer cannot say where ${its(g)} is.</p>` : (g.folders ?? []).map((dir) => html`<${Folder} key=${dir} g=${g} dir=${dir} />`)),
  removals: ({ g, s }) => s.people && g.removals.length > 0 && html`<ul class="removals">${g.removals.map((r) => html`<li>${removal(r, s.people.people)}</li>`)}</ul>`,
};
