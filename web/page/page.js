// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// The phone's one machine, PAGE, composed from its flows, and `Page`, its
// one root (page-scaffold (e); ralph/next/page/CHARTER.md, settled 4).
// Each flow module (web/page/<flow>.js) exports `owns` (its fields),
// `states` (its regions), `takes` (message type -> step), `io`, `view`
// (place -> part), and `heading` and `response` (slot -> words of state)
// and `acts` (state -> {part: vnodes}). Flows talk only by messages: a take
// answers `{run: 'send', m}`, which the frame's io dispatches. Kept apart
// from app.js, the edge, so node holds it to its laws with no browser
// (web/tests/pages/frame.test.mjs).
import { go, Heading, html, no, useEffect, useStore } from '../ui/kit.js?v=__VERSION__';

// The heading is the first a slot gives, in this order, and the response
// every slot's lines in it, where v1's eleven writers of #status and seven
// of #response each wrote over the last (web/app.js).
export const ORDER = ['failed', 'visit', 'ask', 'among', 'elsewhere', 'join', 'enter'];
// "What acts as you", its parts in question.js's PARTS order (:237).
const PARTS = ['reads', 'opens', 'knows', 'wifi', 'play', 'decides'];
// web/index.html's body, :63-97, each place one flow's part.
const PLACES = [
  'computer',
  'join',
  'ask',
  'lent',
  'door',
  'shelf',
  'removals',
  'visitors',
  'acts',
  'how',
  'uses',
];
const COULD_NOT = 'This browser could not open the page.';

// The frame's own flow: `fine` and `standalone`, read once at the edge,
// and a failed effect, its words in `said` when it has any (app.js@19b63b2b:88).
const frame = {
  owns: ['fine', 'standalone', 'said'],
  states: { page: ['opening', 'failed'] },
  io: { send: ({ m }) => m },
  takes: { failed: (s, m) => go(s, { page: 'failed' }, { said: m.said ?? COULD_NOT }) },
  heading: { failed: (s) => s.at.page === 'failed' && [s.said, 'bad'] },
};

// PAGE's parts from `flows` ({name: module}), each part a flow leaves out
// empty. As it composes, a
// message type, region, field, io or place two flows claim is refused, and
// a slot ORDER does not read; a slot is shared, its flows' lines in turn.
export function compose(flows) {
  const all = Object.entries({ frame, ...flows }).map(([name, f]) => ({
    name,
    owns: [],
    states: {},
    io: {},
    view: {},
    heading: {},
    response: {},
    acts: () => ({}),
    takes: {},
    ...f,
  }));
  const claimed = {},
    keys = Object.keys,
    owner = (kind, n) => claimed[`${kind} ${n}`]?.name;
  for (const f of all) {
    for (const [kind, names] of [
      ['message type', keys(f.takes)],
      ['region', keys(f.states)],
      ['io', keys(f.io)],
      ['place', keys(f.view)],
      ['field', f.owns],
    ]) {
      for (const n of names)
        if ((claimed[`${kind} ${n}`] ??= f) !== f)
          throw new Error(`${kind} ${n} is claimed by ${owner(kind, n)} and by ${f.name}`);
    }
    const slot = [...keys(f.heading), ...keys(f.response)].find((n) => !ORDER.includes(n));
    if (slot)
      throw new Error(
        `${f.name}'s heading or response names ${slot}, which is not one of ${ORDER.join(', ')}`,
      );
  }
  // One take per message, its flow's: a part's own message by its name
  // first (ui/parts.js: Fold's `fold <fold>`, Field's `typed <field>`), as
  // two flows each draw a Fold. An answer that moves a field or region its
  // flow does not own is refused, naming it. `asked` is the core's counter
  // of reads (ui/machine.js `ask`), every flow's.
  const step = (s, m) => {
    const type = [`${m.type} ${m.fold ?? m.field}`, m.type].find(
      (t) => claimed[`message type ${t}`],
    );
    const f = claimed[`message type ${type}`],
      r = f?.takes[type](s, m);
    if (!f || r.refused) return r ?? no(s, `nothing here takes ${m.type}`);
    const out = [
      ...keys({ ...s, ...r.state })
        .filter((k) => k !== 'at' && k !== 'asked' && r.state[k] !== s[k])
        .map((k) => ['field', k]),
      ...keys({ ...s.at, ...r.state.at })
        .filter((k) => r.state.at[k] !== s.at[k])
        .map((k) => ['region', k]),
    ].find(([kind, k]) => owner(kind, k) !== f.name);
    return out
      ? no(s, `${f.name} moved ${out.join(' ')}, which is ${owner(...out) ?? 'nobody'}'s`)
      : r;
  };
  const states = Object.assign({}, ...all.map((f) => f.states)),
    each = (pick, s) => ORDER.flatMap((slot) => all.map((f) => f[pick][slot]?.(s)).filter(Boolean));
  return {
    states,
    step,
    io: Object.assign({}, ...all.map((f) => f.io)),
    // The first state: each region at the first of its set, and `fields`.
    first: (fields) => ({
      at: Object.fromEntries(Object.entries(states).map(([r, [at]]) => [r, at])),
      ...fields,
    }),
    heading: (s) => each('heading', s)[0] ?? ['Opening…'],
    response: (s) => each('response', s).flat().join('\n'),
    acts: (s) =>
      PARTS.map((p) => [p, all.flatMap((f) => f.acts(s)[p] ?? [])]).filter(([, n]) => n.length),
    places: PLACES.map((p) => claimed[`place ${p}`]?.view[p]).filter(Boolean),
  };
}

// The page, web/index.html's body in its order, inside #page: the heading,
// each place its flow's part, and #response. The tab is titled by the
// group the page is in (app.js@19b63b2b:405, :422).
export function Page({ parts, store }) {
  const s = useStore(store),
    [words, tone] = parts.heading(s),
    acts = parts.acts(s);
  useEffect(() => void (s.group && (document.title = s.group.name)), [s.group?.name]);
  return html`<${Heading} words=${words} tone=${tone} />${parts.places.map((P) => html`<${P} s=${s} send=${store.dispatch} acts=${acts} />`)}<pre id="response">${parts.response(s)}</pre>`;
}
