// SPDX-License-Identifier: AGPL-3.0-or-later
// The home's v2 page, its one root (page-open (d); home.js's root, :347-429):
// served at /v2/home.js to a test build's node chosen v2 (home/v2/serve.rs)
// and drawn into home/v2/index.html's #home. HOME reads the person's
// groups from /v1/groups, and the address once: the ask (/ask), a group on
// a screen (?screen=), or the page, with what was just done here said
// (?made=, ?joined=, ?left=; home.js:363-367), said once. It draws in v1's order, each
// group's section (home/v2/group.js), then each feature's place, from
// home/v2/<file>.js; a message no file takes is refused by name
// (group.js `offered`). It
// takes one fixed message, `again`, after a write, and `visible`, on
// coming back on screen: each reads the groups again, then each group.
// The edge, below the page, reads the address and forgets a notice in it.
import { trace } from '../ui/trace.js?v=__VERSION__';
import { html, render, useStore } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';
import { Group, get, offered } from './group.js?v=__VERSION__';

// Each feature's file, and the page's places after the groups, in
// home.js:378-427's order.
const FILES = ['shelf', 'index', 'running', 'lending', 'door', 'visits', 'leave', 'screen', 'join', 'make', 'links', 'open', 'acts', 'ask', 'wifi', 'play'], PLACES = ['acts', 'left', 'open', 'join', 'make', 'running', 'lent'];
const reading = (s, m, [asked, n] = ask(s, 'groups')) => go(s, {}, { asked }, { run: 'groups', n, round: m.type, failed: 'unread' });
const HOME = (flows) => ({
  states: { groups: ['reading', 'read', 'unread'], said: ['none', 'made', 'joined', 'left'] },
  step: (s, m) => ({ again: reading, visible: reading,
    read: () => (late(s, 'groups', m.n) ? no(s, late(s, 'groups', m.n)) : go(s, { groups: 'read' }, { groups: m.groups, key: m.key, round: { type: m.round } })),
    unread: () => (late(s, 'groups', m.n) ? no(s, late(s, 'groups', m.n)) : go(s, { groups: 'unread' }, { why: m.why })) })[m.type]?.(s, m) ?? offered(flows, s, m),
  io: { groups: ({ n, round }) => get('/v1/groups').then(({ groups, key }) => ({ type: 'read', n, groups, key, round, why: `${groups.length} group(s)` }), (e) => ({ type: 'unread', n, why: e.message })) },
});
// The address, read once (home.js:358-359), and what it says was done here.
const first = ({ pathname, search, origin }, q = Object.fromEntries(new URLSearchParams(search))) =>
  ({ at: { groups: 'reading', said: ['made', 'joined', 'left'].find((k) => k in q) ?? 'none' }, asked: { groups: 1 }, ask: pathname === '/ask', q, origin });
const SAID = { none: () => '', made: (q, u) => `You made ${q.made}. ${q.also ? `${q.also} ` : ''}${u}Invite someone to it below.`, joined: (q, u) => `You’re in ${q.joined}${q.person ? `, as ${q.person}.` : ' already.'} ${u}`, left: (q) => `You left ${q.left}. Rejoining needs a new invitation, and this computer can’t rejoin yet.` };
// The groups you left, with what stays of each here (home.js:385-388).
const left = ({ s }) => ((gone) => gone.length > 0 && html`<section id="left"><h2>Groups you left</h2>${gone.map((g) => html`<p>${g.name ?? 'A group'}${g.made ? ` (${whose(g.made)})` : ''}: what you have of it stays on this computer${g.folders?.length ? `, its folder at ${g.folders.join(' and ')}` : ''}, and nothing new of it reaches you.</p>`)}</section>`)(s.groups.filter((g) => g.left));

// The page's foot, last, once no feature's file `holds` it while it reads, as
// v1 drew it after every read (home.js:326-328); and "What you lend" held
// with it, as v1 drew it after "What acts as you" and its WiFi (h:377-424),
// which a driver's `text` waits on (home.mjs:166; main.rs:3536 on v2).
export function Home({ store, flows, places }) {
  const s = useStore(store), q = s.q, home = store.dispatch, held = () => Object.values(flows).some((f) => f.holds?.(s)), P = (k) => places[k] && html`<${places[k]} s=${s} home=${home} />`;
  if (s.at.groups !== 'read') return html`<p class=${s.at.groups === 'reading' ? 'said' : undefined}>${s.at.groups === 'reading' ? 'Reading your groups…' : `This computer cannot read your groups: ${s.why}`}</p>`;
  if (s.ask || q.screen) return P(s.ask ? 'ask' : 'screen');
  const said = SAID[s.at.said](q, q.unplaced ? `Its folder could not be made: ${q.unplaced}. ` : '');
  return html`${said && html`<p>${said}</p>`}${!s.groups.length && html`<p class="none">You’re in no group yet. Join one below from the link someone sent you, or make your own.</p>`}${s.groups.filter((g) => !g.left).map((g) => html`<${Group} key=${g.id} g=${g} joined=${q.joined} me=${s.key} round=${s.round} places=${places} flows=${flows} home=${home} />`)}${PLACES.map((k) => (k !== 'lent' || !held()) && P(k))}${!held() && html`<p class="address">Your home page, the same address each time svrngs starts: ${s.origin}/</p>`}`;
}

// The edge: each feature's file, by its one URL, then HOME, its first read
// run as it starts, drawn; and the page shown again, read again. Never
// awaited at the module's top (home.js:347-353, WebKit's module loader).
Promise.all(FILES.map((f) => import(`./${f}.js?v=__VERSION__`))).then((mods) => {
  const flows = Object.fromEntries(FILES.map((f, i) => [f, mods[i]])), places = Object.assign({ left }, ...mods.map((f) => f.view));
  const store = machine('HOME', HOME(flows), first(location), [{ run: 'groups', n: 1, round: 'read', failed: 'unread' }]);
  // What was done here is said once (page-open (e); CHARTER 9): the address
  // forgets it, so a reload, or the page opened again, says it no more.
  if (store.get().at.said !== 'none') history.replaceState(null, '', (trace(`HOME: ${store.get().at.said} said, once`), location.pathname));
  render(html`<${Home} store=${store} flows=${flows} places=${places} />`, document.getElementById('home'));
  document.addEventListener('visibilitychange', () => document.visibilityState === 'visible' && store.dispatch({ type: 'visible' }));
}).catch((e) => (trace(`HOME: its files did not load: ${e}`), render(html`<p>This computer cannot read your groups: ${e.message}</p>`, document.getElementById('home'))));
