// SPDX-License-Identifier: AGPL-3.0-or-later
// What is open to this WiFi (page-home-rooms (a); home.js `wifi` and the
// foot, :69-80, :202-203, :377-381; DESIGN §3, "The room"): a lend opened
// to this WiFi, or closed, for any device on it, a TV among them, said as
// /v1/room says it (room.rs `Served::said`). ROOM, the page's one read of
// /v1/room, made as the page first draws either of its lists, feeds both:
// the foot of each group's section (`ul.wifi`, its `view`), each lend with
// its address for a program, and "Open to this WiFi" under "What acts as
// you" (`part`, for acts.js, `ul#room`). Each read is counted, so an older
// answer is refused by name, and is asked again on each of the groups'
// rounds (HOME's `again` and `visible`), once a round however many groups
// draw a foot. What is open that cannot be read is said once, in `part`,
// never read as none; a press in a line then writes it over.
import { html, useEffect, useMachine, useStore } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Keep } from '../ui/parts.js?v=__VERSION__';
import { called, named, once } from '../question.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

const answer = (n) => (room) => ({ type: 'answered', n, room, why: `${room.length} open` });
const ROOM = { states: { room: ['reading', 'read', 'unread'] }, io: { read: ({ n }) => get('/v1/room').then(answer(n)) },
  step: (s, m, [asked, n] = ask(s, 'room'), old = m.n && late(s, 'room', m.n)) => (old ? no(s, old) : ({ read: () => (m.round <= s.round ? no(s, `round ${m.round} is read already`) : go(s, { room: 'reading' }, { asked, round: m.round }, { run: 'read', n, failed: 'unread' })),
    answered: () => go(s, { room: 'read' }, { room: m.room }), unread: () => go(s, { room: 'unread' }, { why: m.why }) })[m.type]?.() ?? no(s, `the room takes no ${m.type}`)) };
let one;
const room = () => (one ??= machine('ROOM', ROOM, { at: { room: 'reading' }, round: 0 }));
const on = (s, l) => (s.at.room === 'unread' ? undefined : s.room?.find((r) => r.key === l.key && r.name === l.name));

// One lend's line, as v1's span: its own answer drawn in place once its
// button is pressed (`now`), and each refusal appended.
const LINE = (l) => ({ states: { line: ['idle', 'turning'] }, io: { turn: ({ off }) => post('/v1/room', { [off ? 'off' : 'on']: { key: l.key, name: l.name } }).then((now) => ({ type: 'turned', now, off, why: `${now.length} open` })),
  copy: () => Promise.resolve().then(() => navigator.clipboard.writeText(l.origin)).then(() => ({ type: 'copied', words: ` Copied: ${l.origin}. Paste it where a program on this computer, such as a chat app, asks for an address.` }), () => ({ type: 'copied', words: ` Copy it from here: ${l.origin}`, why: 'the clipboard refused' })) },
  step: (s, m) => ({ turn: () => (s.at.line === 'turning' ? no(s, 'a turn is under way, and is answered first') : go(s, { line: 'turning' }, {}, { run: 'turn', off: m.off, failed: 'refused' })), turned: () => go(s, { line: 'idle' }, { now: m.now, was: m.off }),
    refused: () => go(s, { line: 'idle' }, { said: `${s.said} ${m.why}` }), copy: () => go(s, {}, {}, { run: 'copy', failed: 'copied' }), copied: () => go(s, {}, { copied: s.copied + (m.words ?? ` Copy it from here: ${l.origin}`) }) })[m.type]?.() ?? no(s, `a line takes no ${m.type}`) });
function Turn({ l, s, send }) {
  const read = useStore(room()), r = on(s.now ? { at: { room: 'read' }, room: s.now } : read, l);
  return html`<span>${` ${r?.said ?? `${s.was ? 'Closed to this WiFi: nobody on it can open it through this computer now. ' : ''}Open it to this WiFi, and anyone on it can open it as you while svrngs runs.`} `}<${Act} words=${r ? 'Close it to this WiFi' : 'Open it to this WiFi'} doing=${{ state: s.at.line === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', off: Boolean(r) }} send=${send} />${s.said}</span>`;
}
function Line({ l, words, foot }) {
  const [s, send] = useMachine(() => machine('LINE', LINE(l), { at: { line: 'idle' }, said: '', copied: '' }));
  return html`<li>${words}<${Turn} l=${l} s=${s} send=${send} />${foot && html`${' '}<${Act} words="Copy its address for a program" on=${{ type: 'copy' }} send=${send} />${' '}<${Keep} at=${l.origin} />${s.copied}`}</li>`;
}
// Each lend of the group's that is a page, at the section's foot (h:202).
function Foot({ s }) {
  useEffect(() => room().dispatch({ type: 'read', round: s.asked?.lends ?? 1 }), [s.asked?.lends]);
  const people = s.people?.people ?? [], whose = (l) => ({ ...l, called: called(people, people.find((r) => r.key === l.key)?.person_key) });
  return s.lends && html`<ul class="wifi">${once(s.lends.lent).filter((l) => !l.raw).map((l) => ((w) => html`<${Line} key=${`${l.key}:${l.name}`} l=${w} words=${`${named(w)}:`} foot />`)(whose(l)))}</ul>`;
}
export const view = { wifi: Foot };

// "Open to this WiFi", for acts.js: what ROOM read, each with its line, or
// why it cannot be read; none when nothing is open. A hook: acts.js calls
// it on each of its draws, so it is drawn again with each read.
export const rooms = (s) => (s.at.room === 'unread' || s.room?.length ? [html`<p>${s.at.room === 'unread' ? s.why : 'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.'}</p>`, html`<ul id="room">${s.at.room !== 'unread' && s.room.map((r) => html`<${Line} key=${`${r.key}:${r.name}`} l=${r} words=${`${r.person ? `${r.person}’s ` : ''}${r.name}:`} />`)}</ul>`] : []);
export const part = () => rooms(useStore(room()));
