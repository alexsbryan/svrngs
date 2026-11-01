// SPDX-License-Identifier: AGPL-3.0-or-later
// Play (page-home-rooms (b); home.js `play` and its Stops, :82-94,
// :382-384; give-play, play.rs): a port a member lends, UDP or TCP, opened
// on this computer while Play is on, for a program here to connect to,
// said as /v1/play says it (`Open::said`), its address copied as it is
// turned on, the clipboard at the edge. PLAY, the page's one read of
// /v1/play, asked as this file loads and again once a round of the groups'
// reads, each read counted, so an older answer is refused by name, feeds
// both of its lists: the index's cell for each port (`cell`, for
// index.js), and each Play on with its Stop under "What acts as you"
// (`part`, for acts.js). What has Play on that cannot be read is said
// once, in `part`, never read as none; Play then writes it over.
import { html, useEffect, useMachine, useStore } from '../ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act } from '../ui/parts.js?v=__VERSION__';
import { get } from './group.js?v=__VERSION__';
import { post } from './running.js?v=__VERSION__';

const answer = (n) => (plays) => ({ type: 'answered', n, plays, why: `${plays.length} on` }), same = (a) => (b) => a.key === b.key && a.name === b.name;
export const PLAY = { states: { play: ['reading', 'read', 'unread'] }, io: { read: ({ n }) => get('/v1/play').then(answer(n)) },
  step: (s, m, [asked, n] = ask(s, 'play'), old = m.n && late(s, 'play', m.n)) => (old ? no(s, old) : ({ read: () => (m.round <= s.round ? no(s, `round ${m.round} is read already`) : go(s, { play: 'reading' }, { asked, round: m.round }, { run: 'read', n, failed: 'unread' })),
    answered: () => go(s, { play: 'read' }, { plays: m.plays }), unread: () => go(s, { play: 'unread' }, { why: m.why }) })[m.type]?.() ?? no(s, `Play takes no ${m.type}`)) };
const play = machine('PLAY', PLAY, { at: { play: 'reading' }, asked: { play: 1 }, round: 1 }, [{ run: 'read', n: 1, failed: 'unread' }]);
const on = (s, l) => (s.at.play === 'unread' ? undefined : s.plays?.find(same(l)));
// Where a program here connects to what Play opened, copied, or why not.
const copy = (r) => (r?.open.is === 'at' ? navigator.clipboard.writeText(`127.0.0.1:${r.port}`).then(() => ' Copied.', () => ' Copy it from here.') : '');

// A port's Play or Stop, as v1's span: its own answer drawn in place once
// pressed (`now`), with what was copied, and each refusal appended; the
// Stop under "What acts as you" tells it nothing, nor it the Stop.
const TURN = (l) => ({ states: { turn: ['idle', 'turning'] }, io: { turn: ({ off }) => post('/v1/play', { [off ? 'off' : 'on']: { key: l.key, name: l.name } }).then(async (now) => ({ type: 'turned', now, copied: off ? '' : await copy(now.find(same(l))), why: `${now.length} on` })) },
  step: (s, m) => ({ turn: () => (s.at.turn === 'turning' ? no(s, 'a turn is under way, and is answered first') : go(s, { turn: 'turning' }, {}, { run: 'turn', off: m.off, failed: 'refused' })), turned: () => go(s, { turn: 'idle' }, { now: m.now, copied: m.copied }),
    refused: () => go(s, { turn: 'idle' }, { said: `${s.said} ${m.why}` }) })[m.type]?.() ?? no(s, `Play takes no ${m.type}`) });
function Turn({ l, words }) {
  const [s, send] = useMachine(() => machine('TURN', TURN(l), { at: { turn: 'idle' }, said: '' })), read = useStore(play), r = on(s.now ? { at: { play: 'read' }, plays: s.now } : read, l);
  if (words && s.now && !r) return null;
  return html`<${words ? 'p' : 'span'}>${words ? words(l) : `${r ? `${r.said}${s.copied ?? ''}` : 'Play opens it on this computer, for a program here to connect to.'} `}<${Act} words=${r ? 'Stop' : 'Play'} doing=${{ state: s.at.turn === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', off: Boolean(r) }} send=${send} />${s.said}<//>`;
}
// The index's cell for a port another computer lends (h:182), asking
// PLAY again on each round of its group's reads.
export function cell({ l, s }) {
  useEffect(() => play.dispatch({ type: 'read', round: s.asked?.lends ?? 1 }), [s.asked?.lends]);
  return html`<${Turn} l=${l} />`;
}

// Each Play on, for acts.js, with its Stop, or why what has Play on cannot
// be read; none when nothing has. A hook: acts.js calls it on each of its
// draws, so it is drawn again with each read.
const stop = (r) => `Open on this computer: ${r.name}${r.person ? ` (${r.person}’s)` : ''}, while Play is on. `;
export const plays = (s) => (s.at.play === 'unread' ? [html`<p>${s.why}</p>`] : (s.plays ?? []).map((r) => html`<${Turn} key=${`${r.key}:${r.name}`} l=${r} words=${stop} />`));
export const part = () => plays(useStore(play));
