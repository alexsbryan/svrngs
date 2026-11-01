// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the home page (page-acts (c); home.js:205-207,
// :114-124, :278-286 and `uses`, :118-124): ui/acts.js `Acts`, its parts
// derived where v1 pushed each line in by `add`. Each group's GROUP read is
// told to HOME by its place in the group's section (`Told`; group.js
// PLACES), so the one section draws, group by group, the apps it carries
// and their new versions (ui/changes.js), and who decides for you in it, of
// its people by this computer's key; then what opens friends' things as
// you, the yeses /reaching gives, each Stop the form v1 posts
// (acts.mjs:64), and what apps use of each asking lend, from /choices
// (ui/uses.js, as `p`); knows; and WiFi and Play, page-home-rooms' parts.
// Drawn once every group has been told and /reaching read, as v1 drew the
// page's foot only then (h:326-328). Its machine, ACTS, is the section's
// own (ui/view.js `useMachine`), each read counted, read again on each of
// HOME's rounds; a yes to a new version is posted to /runs, kept in
// `acted` by what it acts on, so it stays said once read again; a Stop on
// what apps use reloads the page, as v1 does (h:122).
import { html, useEffect, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Acts } from '../ui/acts.js?v=__VERSION__';
import { Changes } from '../ui/changes.js?v=__VERSION__';
import { Uses } from '../ui/uses.js?v=__VERSION__';
import { called, carrying, deciders, once, reading } from '../question.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { Reach, label } from './index.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';
import * as wifi from './wifi.js?v=__VERSION__';
import * as play from './play.js?v=__VERSION__';

// HOME's: what each group's GROUP last read, by its id.
export const take = (s, m) => (m.read ? go(s, {}, { actsRead: true }) : go(s, {}, { told: { ...s.told, [m.id]: { people: m.people, lends: m.lends } } }));
// The page's foot held until the section has read (home.js `Home`).
export const holds = (s) => s.groups.length > 0 && !s.actsRead;
const Told = ({ g, s, home }) => (useEffect(() => void (s.at.people !== 'reading' && s.at.lends !== 'reading' && home({ type: 'told', flow: 'acts', id: g.id, people: s.people?.people, lends: s.lends, why: `${g.name}: people ${s.at.people}, lends ${s.at.lends}` })), [s.people, s.lends, s.at.people, s.at.lends]), null);

const ACTS = { states: { reaching: ['reading', 'read', 'unread'], doing: ['idle', 'busy', 'done', 'refused'] },
  io: { read: ({ n, asking }) => Promise.all([get('/reaching').catch((e) => ({ why: e.message })), ...asking.map((l) => get(`/choices?lend=${l}`).then(({ choices }) => [l, { choices }], (e) => [l, { why: e.message }]))]).then(([yeses, ...choices]) => ({ type: 'reached', n, yeses, choices: Object.fromEntries(choices), why: `${choices.length} asking` })),
    yes: ({ g, app }) => post('/runs', [g, app]).then(() => ({ type: 'acted', why: `${app} at its new version` })), stop: ({ lend, app }) => post('/choices', { lend, app, said: 'stop' }).then(() => (location.reload(), { type: 'told', why: 'the page loads again' })) },
  step: (s, m) => ({
    read: () => ((asked, n) => go(s, {}, { asked }, { run: 'read', n, asking: m.asking, failed: 'unreached' }))(...ask(s, 'reaching')),
    reached: () => (late(s, 'reaching', m.n) ? no(s, late(s, 'reaching', m.n)) : go(s, { reaching: 'read' }, { yeses: m.yeses, choices: m.choices })),
    unreached: () => go(s, { reaching: 'unread' }, { yeses: { why: m.why }, choices: {} }),
    fold: () => go(s, {}, { folds: { ...s.folds, [m.fold]: m.open } }),
    yes: () => (s.at.doing === 'busy' ? no(s, `${s.acting} is under way, and is answered first`) : go(s, { doing: 'busy' }, { acting: `${m.g} ${m.app}`, acted: { ...s.acted, [`${m.g} ${m.app}`]: { ...s.acted[`${m.g} ${m.app}`], c: m.c } } }, { run: 'yes', g: m.g, app: m.app, failed: 'unacted' })),
    acted: () => go(s, { doing: 'done' }, { acted: { ...s.acted, [s.acting]: { ...s.acted[s.acting], state: 'done' } } }, { run: 'again', failed: 'told' }),
    unacted: () => go(s, { doing: 'refused' }, { acted: { ...s.acted, [s.acting]: { ...s.acted[s.acting], state: 'refused', said: `${s.acted[s.acting].said ?? ''} ${m.why}` } } }),
    stop: () => go(s, {}, {}, { run: 'stop', lend: m.lend, app: m.app, failed: 'unstopped' }),
    unstopped: () => go(s, {}, { said: { ...s.said, [`${m.lend} ${m.app}`]: m.why } }), told: () => go(s, {}),
  })[m.type]?.() ?? no(s, `what acts as you takes no ${m.type}`) };
const NOTES = { opens: 'Each Stop holds from the app’s next request.', play: 'Any program on this computer reaches these as you while Play is on.' };

function Acting({ s: h, home }) {
  const [s, send] = useMachine(() => machine('ACTS', { ...ACTS, io: { ...ACTS.io, again: () => (home({ type: 'again' }), { type: 'told', why: 'the groups read again' }) } }, { at: { reaching: 'reading', doing: 'idle' }, acted: {}, said: {} }));
  const groups = h.groups.filter((g) => !g.left && g.removed === false), told = (g) => h.told?.[g.id], rows = (g) => told(g)?.people ?? [];
  const lent = groups.flatMap((g) => (told(g)?.lends?.lent ?? []).map((l) => ({ ...l, called: called(rows(g), rows(g).find((r) => r.key === l.key)?.person_key) })));
  const asking = lent.filter((l) => l.asks).map(label), ready = groups.every(told);
  useEffect(() => void (ready && send({ type: 'read', asking })), [ready, h.round, asking.join()]);
  useEffect(() => void (ready && s.at.reaching !== 'reading' && !h.actsRead && home({ type: 'told', flow: 'acts', read: true, why: 'what acts as you is read' })), [ready, s.at.reaching]);
  if (!ready || s.at.reaching === 'reading') return null;
  const yes = (g) => (m) => (m.type === 'yes' ? send({ ...m, g: g.id, c: told(g).lends.changed?.find((c) => c.group === g.id && c.app === m.app) }) : send(m));
  const doing = (g) => Object.fromEntries(Object.entries(s.acted).filter(([k]) => k.startsWith(`${g.id} `)).map(([k, a]) => [k.slice(g.id.length + 1), s.acting === k && s.at.doing === 'busy' ? { state: 'busy' } : a]));
  const changed = (g, L = told(g).lends) => L.changed && [...L.changed, ...Object.values(s.acted).filter((a) => a.c?.group === g.id && a.state === 'done' && !L.changed.some((c) => c.app === a.c.app)).map((a) => a.c)];
  const parts = {
    reads: groups.filter((g) => told(g).lends).flatMap((g) => [...once(told(g).lends.lent).filter((o) => o.group).map((l) => html`<p>${carrying(l.kind, told(g).lends.shelf)}</p>`), ...Changes({ g, changed: changed(g), rows: rows(g), me: h.key, doing: doing(g), open: s.folds?.changes, send: yes(g) })]),
    opens: [...(s.yeses.why !== undefined ? [html`<p>This computer cannot say which apps open your friends’ things as you: ${s.yeses.why}</p>`] : Object.entries(s.yeses).map(([app, c]) => html`<p>${reading(lent.find((o) => label(o) === app && o.reads?.length), c)}${c.said === 'yes' && html`<${Reach} app=${app} words="Stop" said="stop" />`}</p>`)),
      ...Uses({ lent, labelOf: label, choices: s.choices, tag: 'p', said: s.said, send })],
    knows: [html`<p>What you open knows your name.</p>`], wifi: wifi.part({ s: h, home }), play: play.part({ s: h, home }),
    decides: groups.filter((g) => told(g).people).map((g) => html`<p>${`${g.name ?? 'Your group'}: ${deciders(rows(g), h.key, 'computer')}.`}</p>`),
  };
  return html`<${Acts} parts=${parts} notes=${NOTES} open=${s.folds?.acts} send=${send} />`;
}
export const view = { acts: ({ s, home }) => s.groups.length > 0 && html`<${Acting} s=${s} home=${home} />`, told: Told };
