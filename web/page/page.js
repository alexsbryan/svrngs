// SPDX-License-Identifier: AGPL-3.0-or-later
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
/** @import { Effect, Machine, Runners, Step, Store } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as acts from './acts.js?v=__VERSION__' */
/** @import * as among from './among.js?v=__VERSION__' */
/** @import * as ask from './ask.js?v=__VERSION__' */
/** @import * as computer from './computer.js?v=__VERSION__' */
/** @import * as door from './door.js?v=__VERSION__' */
/** @import * as enter from './enter.js?v=__VERSION__' */
/** @import * as index from './index.js?v=__VERSION__' */
/** @import * as join from './join.js?v=__VERSION__' */
/** @import * as uses from './uses.js?v=__VERSION__' */
/** @import * as visit from './visit.js?v=__VERSION__' */

// PAGE's types, each flow's composed, as `compose` composes them at run
// time: its regions, each a closed set, from each flow's `states`; its
// fields, each from the flow that owns it; its messages, each flow's.

/**
 * Each region and its closed set, as each flow's `states` names it.
 * @typedef {typeof frame.states & typeof enter.states & typeof computer.states
 *   & typeof join.states & typeof ask.states & typeof index.states
 *   & typeof visit.states & typeof among.states & typeof acts.states
 *   & typeof uses.states} Regions
 */

/**
 * The page's named state in each region.
 * @typedef {{ [Region in keyof Regions]: Regions[Region][number] }} At
 */

/**
 * The frame's own fields: the browser, read once at the edge (app.js), and
 * the words of a failed effect.
 * @typedef {{ fine: boolean, standalone?: boolean, said?: string }} FrameFields
 */

/**
 * PAGE's state: each region's named state, the reads asked (ui/machine.js
 * `ask`), and every flow's fields. A field is typed as the steps that read
 * it take it: present where a step reads it only once its region says it
 * was set, so `group` once a tab holds one; optional where a step asks.
 * @typedef {{ at: At, asked?: Record<string, number> } & FrameFields & enter.Fields
 *   & computer.Fields & join.Fields & ask.Fields & index.Fields & door.Fields
 *   & visit.Fields & among.Fields & acts.Fields & uses.Fields} State
 */

/**
 * The frame's message: an effect failed, in words when it has any.
 * @typedef {{ type: 'failed', said?: string, n?: number }} Failed
 */

/**
 * Every message PAGE takes, each flow's, and why it was sent, for the trace.
 * @typedef {(Failed | enter.Message | computer.Message | join.Message | ask.Message
 *   | index.Message | door.Message | visit.Message | among.Message | acts.Message
 *   | uses.Message | { type: 'start', fragment: string }) & { why?: string }} Message
 */

/**
 * A message sent to another flow, which the frame's io dispatches.
 * @typedef {{ run: 'send', m: Message, failed?: string }} Send
 */

/**
 * The rest of a message of type `T`, as a flow's `send(type, rest)` takes it:
 * required when the message has a field that is.
 * @template {Message['type']} T
 * @typedef {Omit<Extract<Message, { type: T }>, 'type'>} Rest
 */

/**
 * A flow's `send(type, rest)`: the effect sending the message of type `type`,
 * its `rest`, to the flow that takes it.
 * @typedef {<T extends Message['type']>(type: T, ...rest: {} extends Rest<T> ? [rest?: Rest<T>] : [rest: Rest<T>]) => Send} Sender
 */

/**
 * The heading's words, and its tone.
 * @typedef {[words: string, tone?: 'good' | 'bad']} Said
 */

/**
 * What a slot or a part answers when it has nothing to say.
 * @typedef {false | '' | 0 | null | undefined} Nothing
 */

/**
 * The props every place's part is drawn with.
 * @typedef {object} Props
 * @property {State} s
 * @property {(message: Message) => void} send
 * @property {[string, Drawn[]][]} acts what acts as you, by part (`PARTS`)
 */

/**
 * A flow as `compose` reads it: each part it leaves out empty.
 * @typedef {object} Flow
 * @property {readonly string[]} [owns]
 * @property {Record<string, readonly string[]>} [states]
 * @property {Record<string, (s: State, m: any) => Step<State>>} [takes]
 * @property {Runners<Message>} [io]
 * @property {Record<string, (props: Props) => Drawn>} [view]
 * @property {Record<string, (s: State) => Said | Nothing>} [heading]
 * @property {Record<string, (s: State) => string[] | Nothing>} [response]
 * @property {(s: State) => Record<string, Drawn[]>} [acts]
 */

/**
 * PAGE's parts, as `compose` answers them: its machine, its first state,
 * and what its root draws.
 * @typedef {Machine<State, Message> & {
 *   first: (fields: FrameFields) => State,
 *   heading: (s: State) => Said,
 *   response: (s: State) => string,
 *   acts: (s: State) => [string, Drawn[]][],
 *   places: ((props: Props) => Drawn)[],
 * }} Parts
 */

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
  states: /** @type {const} */ ({ page: ['opening', 'failed'] }),
  io: {
    /** @param {Send} send */
    send: ({ m: message }) => message,
  },
  takes: {
    /**
     * @param {State} state
     * @param {Failed} message
     */
    failed: (state, message) => go(state, { page: 'failed' }, { said: message.said ?? COULD_NOT }),
  },
  heading: {
    /**
     * @param {State} state
     * @returns {Said | Nothing}
     */
    failed: (state) => state.at.page === 'failed' && [/** @type {string} */ (state.said), 'bad'],
  },
};

// PAGE's parts from `flows` ({name: module}), each part a flow leaves out
// empty. As it composes, a
// message type, region, field, io or place two flows claim is refused, and
// a slot ORDER does not read; a slot is shared, its flows' lines in turn.
/**
 * @param {Record<string, Flow>} flows
 * @returns {Parts}
 */
export function compose(flows) {
  const all = Object.entries(/** @type {Record<string, Flow>} */ ({ frame, ...flows })).map(
    ([name, flow]) =>
      /** @type {Required<Flow> & { name: string }} */ ({
        name,
        owns: [],
        states: {},
        io: {},
        view: {},
        heading: {},
        response: {},
        acts: () => ({}),
        takes: {},
        ...flow,
      }),
  );
  /** @type {Record<string, (typeof all)[number]>} */
  const claimed = {},
    keys = Object.keys,
    owner = (/** @type {string} */ kind, /** @type {string} */ name) =>
      claimed[`${kind} ${name}`]?.name;
  for (const flow of all) {
    for (const [kind, names] of /** @type {[string, readonly string[]][]} */ ([
      ['message type', keys(flow.takes)],
      ['region', keys(flow.states)],
      ['io', keys(flow.io)],
      ['place', keys(flow.view)],
      ['field', flow.owns],
    ])) {
      for (const name of names)
        if ((claimed[`${kind} ${name}`] ??= flow) !== flow)
          throw new Error(`${kind} ${name} is claimed by ${owner(kind, name)} and by ${flow.name}`);
    }
    const slot = [...keys(flow.heading), ...keys(flow.response)].find(
      (name) => !ORDER.includes(name),
    );
    if (slot)
      throw new Error(
        `${flow.name}'s heading or response names ${slot}, which is not one of ${ORDER.join(', ')}`,
      );
  }
  // One take per message, its flow's: a part's own message by its name
  // first (ui/parts.js: Fold's `fold <fold>`, Field's `typed <field>`), as
  // two flows each draw a Fold. An answer that moves a field or region its
  // flow does not own is refused, naming it. `asked` is the core's counter
  // of reads (ui/machine.js `ask`), every flow's.
  /**
   * @param {State} state
   * @param {Message & { fold?: string, field?: string }} message
   * @returns {Step<State>}
   */
  const step = (state, message) => {
    const type = /** @type {string} */ (
      [`${message.type} ${message.fold ?? message.field}`, message.type].find(
        (candidate) => claimed[`message type ${candidate}`],
      )
    );
    const flow = claimed[`message type ${type}`],
      result = flow?.takes[type]?.(state, message);
    if (!flow || !result || result.refused)
      return result ?? no(state, `nothing here takes ${message.type}`);
    const out = [
      .../** @type {(keyof State)[]} */ (keys({ ...state, ...result.state }))
        .filter((name) => name !== 'at' && name !== 'asked' && result.state[name] !== state[name])
        .map((name) => /** @type {[string, string]} */ (['field', name])),
      .../** @type {(keyof At)[]} */ (keys({ ...state.at, ...result.state.at }))
        .filter((name) => result.state.at[name] !== state.at[name])
        .map((name) => /** @type {[string, string]} */ (['region', name])),
    ].find(([kind, name]) => owner(kind, name) !== flow.name);
    return out
      ? no(state, `${flow.name} moved ${out.join(' ')}, which is ${owner(...out) ?? 'nobody'}'s`)
      : result;
  };
  const states = Object.assign({}, ...all.map((flow) => flow.states)),
    each = (/** @type {'heading' | 'response'} */ pick, /** @type {State} */ state) =>
      ORDER.flatMap((slot) => all.map((flow) => flow[pick][slot]?.(state)).filter(Boolean));
  return {
    states,
    step,
    io: Object.assign({}, ...all.map((flow) => flow.io)),
    // The first state: each region at the first of its set, and `fields`.
    // Its other fields are absent until their flows set them, as `State` says.
    first: (fields) =>
      /** @type {State} */ ({
        at: Object.fromEntries(Object.entries(states).map(([region, [at]]) => [region, at])),
        ...fields,
      }),
    heading: (state) => /** @type {Said | undefined} */ (each('heading', state)[0]) ?? ['Opening…'],
    response: (state) => each('response', state).flat().join('\n'),
    acts: (state) =>
      PARTS.map(
        (place) =>
          /** @type {[string, Drawn[]]} */ ([
            place,
            all.flatMap((flow) => flow.acts(state)[place] ?? []),
          ]),
      ).filter(([, drawn]) => drawn.length),
    places: /** @type {Parts['places']} */ (
      PLACES.map((place) => claimed[`place ${place}`]?.view[place]).filter(Boolean)
    ),
  };
}

// The page, web/index.html's body in its order, inside #page: the heading,
// each place its flow's part, and #response. The tab is titled by the
// group the page is in (app.js@19b63b2b:405, :422).
/**
 * @typedef {object} PageProps
 * @property {Parts} parts
 * @property {Store<State, Message>} store
 */
/** @param {PageProps} props */
export function Page({ parts, store }) {
  const state = useStore(store),
    [words, tone] = parts.heading(state),
    acts = parts.acts(state);
  useEffect(() => void (state.group && (document.title = state.group.name)), [state.group?.name]);
  return html`<${Heading} words=${words} tone=${tone} />${parts.places.map((Place) => html`<${Place} s=${state} send=${store.dispatch} acts=${acts} />`)}<pre id="response">${parts.response(state)}</pre>`;
}
