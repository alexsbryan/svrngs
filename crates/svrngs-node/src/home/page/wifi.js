// SPDX-License-Identifier: AGPL-3.0-or-later
// What is open to this WiFi (page-home-rooms (a); home.js `wifi` and the
// foot, :69-80, :202-203, :377-381; DESIGN §3, "The room"): a lend opened
// to this WiFi, or closed, for any device on it, a TV among them, said as
// /v1/room says it (room.rs `Served::said`). ROOM, the page's one read of
// /v1/room, asked as this file loads, beside HOME's read of the groups, as
// v1 reads it before it draws a group (home.js@19b63b2b:377), feeds both of its lists:
// the foot of each group's section (`ul.wifi`, its `view`), each lend with
// its address for a program, and "Open to this WiFi" under "What acts as
// you" (`part`, for acts.js, `ul#room`). Each read is counted, so an older
// answer is refused by name, and is asked again on each of the groups'
// rounds (HOME's `again` and `visible`), once a round however many groups
// draw a foot. What is open that cannot be read is said once, in `part`,
// never read as none; a press in a line then writes it over.
import { Act, ask, called, go, html, Keep, late, machine, named, no, once, useEffect, useMachine, useStore } from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';

const answer = (n, turned) => (room) => ({ type: 'answered', n, room, turned, why: `${room.length} open` }), same = (a) => (b) => a.key === b.key && a.name === b.name, k = (l) => `${l.key}:${l.name}`;
// A turn is answered by what is open then, written over what was read, and
// what it closed is kept, said closed in both lists until the page is left.
export const ROOM = { states: { room: ['reading', 'read', 'unread', 'turning'] },
  io: { read: ({ n }) => get('/v1/room').then(answer(n)), turn: ({ n, l, off }) => post('/v1/room', { [off ? 'off' : 'on']: { key: l.key, name: l.name } }).then(answer(n, { l, off })) },
  step: (s, m, [asked, n] = ask(s, 'room'), old = m.n && late(s, 'room', m.n)) => (old ? no(s, old) : s.at.room === 'turning' && ['read', 'turn'].includes(m.type) ? no(s, 'a turn is under way, and its answer is what is open') : ({ read: () => (m.round <= s.round ? no(s, `round ${m.round} is read already`) : go(s, { room: 'reading' }, { asked, round: m.round }, { run: 'read', n, failed: 'unread' })),
    turn: () => go(s, { room: 'turning' }, { asked, was: s.at.room, l: m.l }, { run: 'turn', n, l: m.l, off: m.off, failed: 'refused' }), refused: () => go(s, { room: s.was }, { said: { ...s.said, [k(s.l)]: ` ${m.why}` } }),
    answered: () => go(s, { room: 'read' }, { room: m.room, closed: m.turned ? [...s.closed.filter((c) => !same(c)(m.turned.l)), ...(m.turned.off ? [m.turned.l] : [])] : s.closed }), unread: () => go(s, { room: 'unread' }, { why: m.why }) })[m.type]?.() ?? no(s, `the room takes no ${m.type}`)) };
const room = machine('ROOM', ROOM, { at: { room: 'reading' }, asked: { room: 1 }, round: 1, closed: [], said: {} }, [{ run: 'read', n: 1, failed: 'unread' }]);
const on = (s, l) => (s.at.room === 'unread' ? undefined : s.room?.find(same(l)));

// A copy of the lend's address for a program, said beside it once, the
// last press's words in place of the one before, where v1 appends each
// (home.js@19b63b2b:202).
const COPY = (at) => ({ states: { copy: ['uncopied', 'copied'] }, io: { copy: () => Promise.resolve().then(() => navigator.clipboard.writeText(at)).then(() => ({ type: 'copied', words: ` Copied: ${at}. Paste it where a program on this computer, such as a chat app, asks for an address.` }), () => ({ type: 'copied', why: 'the clipboard refused' })) },
  step: (s, m) => ({ copy: () => go(s, {}, {}, { run: 'copy', failed: 'copied' }), copied: () => go(s, { copy: 'copied' }, { said: m.words ?? ` Copy it from here: ${at}` }) })[m.type]?.() ?? no(s, `a copy takes no ${m.type}`) });
const Copy = ({ at }) => (([s, send]) => html`${' '}<${Act} words="Copy its address for a program" on=${{ type: 'copy' }} send=${send} />${' '}<${Keep} at=${at} />${s.said}`)(useMachine(() => machine('COPY', COPY(at), { at: { copy: 'uncopied' }, said: '' })));
// One lend's line, in either list, drawn from ROOM alone: what is open
// said, or closed, if a press here closed it, and why a turn was refused.
const Line = ({ l, words, foot }) => ((s, r = on(s, l)) => html`<li>${words}<span>${` ${r?.said ?? `${s.closed.some(same(l)) ? 'Closed to this WiFi: nobody on it can open it through this computer now. ' : ''}Open it to this WiFi, and anyone on it can open it as you while svrngs runs.`} `}<${Act} words=${r ? 'Close it to this WiFi' : 'Open it to this WiFi'} doing=${{ state: s.at.room === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', l: r ?? { key: l.key, name: l.name }, off: Boolean(r) }} send=${room.dispatch} />${s.said[k(l)]}</span>${foot && html`<${Copy} at=${l.origin} />`}</li>`)(useStore(room));
// Each lend of the group's that is a page, at the section's foot (home.js@19b63b2b:202),
// once the room has answered, as v1 draws none before (home.js@19b63b2b:377).
function Foot({ s }) {
  useEffect(() => room.dispatch({ type: 'read', round: s.asked?.lends ?? 1 }), [s.asked?.lends]);
  const r = useStore(room), people = s.people?.people ?? [], whose = (l) => ({ ...l, called: called(people, people.find((r) => r.key === l.key)?.person_key) });
  return s.lends && (r.room || r.at.room === 'unread') && html`<ul class="wifi">${once(s.lends.lent).filter((l) => !l.raw).map((l) => ((w) => html`<${Line} key=${k(l)} l=${w} words=${`${named(w)}:`} foot />`)(whose(l)))}</ul>`;
}
export const view = { wifi: Foot };

// "Open to this WiFi", for acts.js: what ROOM read, each with its line, or
// why it cannot be read; none when nothing is open. A hook: acts.js calls
// it on each of its draws, so it is drawn again with each read.
export const rooms = (s, all = [...(s.at.room === 'unread' ? [] : s.room ?? []), ...s.closed.filter((c) => !on(s, c))]) => (s.at.room === 'unread' || all.length ? [html`<p>${s.at.room === 'unread' ? s.why : 'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.'}</p>`, html`<ul id="room">${all.map((r) => html`<${Line} key=${k(r)} l=${r} words=${`${r.person ? `${r.person}’s ` : ''}${r.name}:`} />`)}</ul>`] : []);
export const part = () => rooms(useStore(room));
