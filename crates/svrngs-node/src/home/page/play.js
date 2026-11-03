// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
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
import {
  Act,
  ask,
  go,
  html,
  late,
  machine,
  no,
  useEffect,
  useStore,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';

const answer = (n) => (plays) => ({ type: 'answered', n, plays, why: `${plays.length} on` }),
  same = (a) => (b) => a.key === b.key && a.name === b.name,
  k = (l) => `${l.key}:${l.name}`;
// Where a program here connects to what Play opened, copied, or why not.
const copy = (r) =>
  r?.open.is === 'at'
    ? navigator.clipboard.writeText(`127.0.0.1:${r.port}`).then(
        () => ' Copied.',
        () => ' Copy it from here.',
      )
    : '';
// A turn, a cell's Play or a Stop, is PLAY's: answered by what has Play on
// then, written over what was read, with what it copied kept for its lend
// (`copied`), and each refusal appended to its lend's (`said`), so every
// list draws it.
export const PLAY = {
  states: { play: ['reading', 'read', 'unread', 'turning'] },
  io: {
    read: ({ n }) => get('/v1/play').then(answer(n)),
    turn: ({ n, l, off }) =>
      post('/v1/play', { [off ? 'off' : 'on']: l }).then(async (plays) => ({
        ...answer(n)(plays),
        copied: off ? '' : await copy(plays.find(same(l))),
      })),
  },
  step: (s, m, [asked, n] = ask(s, 'play'), old = m.n && late(s, 'play', m.n)) =>
    old
      ? no(s, old)
      : s.at.play === 'turning' && ['read', 'turn'].includes(m.type)
        ? no(s, 'a turn is under way, and its answer is what has Play on')
        : ({
            read: () =>
              m.round <= s.round
                ? no(s, `round ${m.round} is read already`)
                : go(
                    s,
                    { play: 'reading' },
                    { asked, round: m.round },
                    { run: 'read', n, failed: 'unread' },
                  ),
            turn: () =>
              go(
                s,
                { play: 'turning' },
                { asked, was: s.at.play, l: m.l },
                { run: 'turn', n, l: m.l, off: m.off, failed: 'refused' },
              ),
            refused: () =>
              go(
                s,
                { play: s.was },
                { said: { ...s.said, [k(s.l)]: `${s.said[k(s.l)] ?? ''} ${m.why}` } },
              ),
            answered: () =>
              go(
                s,
                { play: 'read' },
                {
                  plays: m.plays,
                  copied: m.copied === undefined ? s.copied : { ...s.copied, [k(s.l)]: m.copied },
                },
              ),
            unread: () => go(s, { play: 'unread' }, { why: m.why }),
          }[m.type]?.() ?? no(s, `Play takes no ${m.type}`)),
};
const play = machine(
  'PLAY',
  PLAY,
  { at: { play: 'reading' }, asked: { play: 1 }, round: 1, said: {}, copied: {} },
  [{ run: 'read', n: 1, failed: 'unread' }],
);
const on = (s, l) => (s.at.play === 'unread' ? undefined : s.plays?.find(same(l)));

// A port's Play or Stop, in either list, drawn from PLAY alone: what has
// Play on said, with what its press copied, and why a turn was refused.
const Turn = ({ l, words }) =>
  ((s, r = on(s, l)) =>
    html`<${words ? 'p' : 'span'}>${words ? words(l) : `${r ? `${r.said}${s.copied[k(l)] ?? ''}` : 'Play opens it on this computer, for a program here to connect to.'} `}<${Act} words=${r ? 'Stop' : 'Play'} doing=${{ state: s.at.play === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', l: { key: l.key, name: l.name }, off: Boolean(r) }} send=${play.dispatch} />${s.said[k(l)]}<//>`)(
    useStore(play),
  );
// The index's cell for a port another computer lends (home.js@19b63b2b:182), asking
// PLAY again on each round of its group's reads.
export const cell = ({ l, s }) => (
  useEffect(() => play.dispatch({ type: 'read', round: s.asked?.lends ?? 1 }), [s.asked?.lends]),
  html`<${Turn} l=${l} />`
);

// Each Play on, for acts.js, with its Stop, or why what has Play on cannot
// be read; none when nothing has. A hook: acts.js calls it on each of its
// draws, so it is drawn again with each read.
const stop = (r) =>
  `Open on this computer: ${r.name}${r.person ? ` (${r.person}’s)` : ''}, while Play is on. `;
export const plays = (s) =>
  s.at.play === 'unread'
    ? [html`<p>${s.why}</p>`]
    : (s.plays ?? []).map((r) => html`<${Turn} key=${k(r)} l=${r} words=${stop} />`);
export const part = () => plays(useStore(play));
