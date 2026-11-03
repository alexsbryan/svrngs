// SPDX-License-Identifier: AGPL-3.0-or-later
// One group's section on the home page, and GROUP, its machine, one per
// group (page-open (d); home.js `section` and `show`, :100-114 and :116-239).
// GROUP reads the group's people and what they lend, each a region of its
// own, counted, so an older answer is refused by name; both are asked at
// once, and the people, answered first, are drawn without waiting for the
// lends (start-home: a group shown before its slowest computer). It reads
// again on HOME's `again` and `visible`, each a new `round` from HOME.
// What each read draws is each feature's, in its place (PLACES), from
// home/page/<file>.js: a file that draws nothing draws nothing here.
import {
  ask,
  go,
  html,
  late,
  machine,
  no,
  useEffect,
  useMachine,
  whose,
} from '../ui/kit.js?v=__VERSION__';
/** @import { Core, Effect as AnyEffect, Machine, Step, Store } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lend, Named, Removal, PeopleRow, Shelved } from '../question.js?v=__VERSION__' */
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */

/**
 * A group as /v1/groups answers it (surface.rs `Held`), the group `g` each
 * feature's place is drawn for. Each `null` is what this computer cannot
 * read of it, which a place says, never reads as none.
 * @typedef {object} Group
 * @property {string} id
 * @property {string | null} name
 * @property {string | null} person this computer's person in it, `null` when it stands in it as nobody
 * @property {boolean | null} removed whether its keeper removed this computer
 * @property {Removal[] | null} removals
 * @property {boolean | null} left whether this computer's person left it
 * @property {boolean | null} keeps whether they keep its order of removals
 * @property {{ name: string, maker: string | null, since: number | null } | null} made who made it, `null` for this computer's key, and since when it is held here (group.rs `Namesake`)
 * @property {string[] | null} folders
 * @property {Visit[] | null} visiting
 * @property {'unanswered' | 'unowned' | null} fallback why it is met the backup way (svrngs-core relay.rs `Tier`), `null` at its own
 */

/**
 * Someone visiting now, as /v1/groups lists them (rail.rs `visiting`): the
 * visit's id, which its host's Stop names, and whether this computer's
 * person hosts them.
 * @typedef {Visitor & { id: string, yours: boolean }} Visit
 */

/**
 * The group's people, as /v1/rail/people answers (rail.rs `standing_json`).
 * @typedef {{ people: PeopleRow[] }} People
 */

/**
 * A lend as /v1/lends lists it: at its origin here, and a port's by how its
 * bytes travel (lend.rs `Raw`).
 * @typedef {Lend & { origin: string, raw?: 'udp' | 'tcp' | null }} Lent
 */

/**
 * What the group's people lend, as /v1/lends answers (surface.rs `lends`):
 * each lend, the computers that did not say, by how (node.rs `Unlisted`),
 * the group's shelf, and the apps whose files changed, `null` when that
 * cannot be read.
 * @typedef {object} Lends
 * @property {Lent[]} lent
 * @property {Named[]} unanswered
 * @property {Named[]} unheard
 * @property {Named[]} refused
 * @property {Named[]} older
 * @property {Shelved[]} shelf
 * @property {Change[] | null} changed
 */

/** @typedef {'people' | 'lends'} Kind */

/** @typedef {'reading' | 'read' | 'unread'} Reads */

/**
 * GROUP's state: each read's region, the group's id, and what each read
 * answered last, or why it could not.
 * @typedef {object} State
 * @property {{ people: Reads, lends: Reads }} at
 * @property {Record<string, number>} [asked]
 * @property {string} id
 * @property {People} [people]
 * @property {Lends} [lends]
 * @property {string} [why]
 */

/**
 * A round of reading, by HOME's message that began it (home.js `reading`).
 * @typedef {{ type: 'read' | 'again' | 'visible' }} Round
 */

/**
 * A read answered: the people, or what they lend.
 * @typedef {{ type: 'answered', kind: 'people', n: number, body: People, why: string }
 *   | { type: 'answered', kind: 'lends', n: number, body: Lends, why: string }} Answered
 */

/**
 * The messages GROUP takes: a round, each read answered, or unread with why.
 * @typedef {Round | Answered | { type: 'unread', kind: Kind, n: number, why: string }} Message
 */

/**
 * The effect its step answers: a read of `kind` for the group `id`.
 * @typedef {{ run: 'get', kind: Kind, n: number, id: string, failed: 'unread' }} Effect
 */

/**
 * A message to a feature, `flow` its file's name in home/page/, which that
 * file's `take` takes (home.js).
 * @typedef {{ type: string, flow: string }} Taken
 */

/**
 * Each feature's module, by its file's name, as home.js imports them: a
 * file that takes a message exports `take`, a step of the machine `S`
 * whose effects are `E`. A method, so a `take` of one message's type
 * stands for any.
 * @template S
 * @template {AnyEffect} [E=AnyEffect]
 * @typedef {Record<string, { take?(s: S, m: Taken): Step<S, E> }>} Flows
 */

// A route's answer is the JSON its server writes (`any`), typed where it is
// read, by the reader that knows its route.
// `body` posted as JSON to `path`, its answer, or its words when it refuses
// (home.js@19b63b2b:28-37).
/**
 * @param {string} path
 * @param {unknown} body
 * @returns {Promise<any>}
 */
export const post = (path, body) =>
  fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  }).then(async (response) =>
    ((answer) =>
      response.ok
        ? answer
        : Promise.reject(new Error(answer.error || `answered ${response.status}`)))(
      await response.json().catch(() => ({})),
    ),
  );
// A route's answer, or its words when it refuses, never read as empty
// (home.js@19b63b2b:25-30); refused as the page's session lapsed (home.rs:198,
// session.rs `refused`), said as how to open it again.
/**
 * @param {string} path
 * @returns {Promise<any>}
 */
export const get = (path) =>
  fetch(path).then(async (response) =>
    response.ok
      ? response.json()
      : Promise.reject(
          new Error(
            response.status === 401
              ? 'this page’s session has ended. Open svrngs from your Applications, or on Linux your applications menu, and it opens here again'
              : (await response.json().catch(() => ({}))).error || `answered ${response.status}`,
          ),
        ),
  );
// A message to a feature (`message.flow`, its file's name) is taken by its
// `take`, or refused by name.
/**
 * @template S
 * @template {AnyEffect} E
 * @param {Flows<S, E>} flows
 * @param {S} state
 * @param {Taken} message
 * @returns {Step<S, E>}
 */
export const offered = (flows, state, message, flow = flows[message.flow]) =>
  flow?.take ? flow.take(state, message) : no(state, `nothing here takes ${message.type}`);

// Each place of a group's section, in v1's order (home.js@19b63b2b:116-232): the
// index, its people, its folder, the links made here, leaving, removals,
// who is visiting, the shelf, what is open to this WiFi, and its link to a screen (`show`);
// then what it read, told to HOME for "What acts as you" (home/page/acts.js `Told`), drawing nothing.
const PLACES = /** @type {const} */ ([
  'index',
  'people',
  'folder',
  'links',
  'leave',
  'removals',
  'visits',
  'shelf',
  'wifi',
  'show',
  'told',
]);
const ROUTES = { people: '/v1/rail/people', lends: '/v1/lends' };
// Both read, each counted (ui/machine.js `ask`), the round's why traced.
/**
 * @param {State} state
 * @param {Round} message
 * @returns {Step<State, Effect>}
 */
const reading = (
  state,
  message,
  [peopleAsked, people] = ask(state, 'people'),
  [asked, lends] = ask({ asked: peopleAsked }, 'lends'),
) =>
  go(
    state,
    { people: 'reading', lends: 'reading' },
    { asked },
    .../** @type {[Kind, number][]} */ ([
      ['people', people],
      ['lends', lends],
    ]).map(
      ([kind, serial]) =>
        /** @type {Effect} */ ({ run: 'get', kind, n: serial, id: state.id, failed: 'unread' }),
    ),
  );
// A message no case takes is offered to the feature it names, as HOME's
// are: none sent to GROUP names one, so `message` there is `never`, and a message
// type GROUP declares and no case takes fails `types` at `offered`.
/**
 * @param {Flows<State, Effect>} flows
 * @returns {Machine<State, Message, Effect>}
 */
const GROUP = (flows) => ({
  states: { people: ['reading', 'read', 'unread'], lends: ['reading', 'read', 'unread'] },
  step: (state, message) => {
    switch (message.type) {
      case 'read':
      case 'again':
      case 'visible':
        return reading(state, message);
      case 'answered':
        return late(state, message.kind, message.n)
          ? no(state, late(state, message.kind, message.n))
          : go(
              state,
              /** @type {Partial<State['at']>} */ ({ [message.kind]: 'read' }),
              /** @type {Partial<State>} */ ({ [message.kind]: message.body }),
            );
      case 'unread':
        return late(state, message.kind, message.n)
          ? no(state, late(state, message.kind, message.n))
          : go(
              state,
              /** @type {Partial<State['at']>} */ ({ [message.kind]: 'unread' }),
              { why: message.why },
            );
      default:
        return offered(flows, state, message);
    }
  },
  io: {
    get: ({ kind, n: serial, id }) =>
      get(`${ROUTES[kind]}?namespace=${encodeURIComponent(id)}`).then(
        (body) => ({ type: 'answered', kind, n: serial, body, why: `${kind} ${serial}` }),
        (error) => ({ type: 'unread', kind, n: serial, why: error.message }),
      ),
  },
});

// Why this computer reaches a group the backup way, by surface.rs's
// `fallback` (relay fallbacks; svrngs-core relay.rs `Tier`).
/** @type {Record<NonNullable<Group['fallback']>, (name: string) => string>} */
const BACKUP = {
  unanswered: (name) =>
    `This computer can’t reach the place ${name} usually meets, so it’s reaching ${name} a backup way for now. There’s nothing to do: it switches back by itself once that place answers.`,
  unowned: (name) =>
    `This computer is also in a group with no meeting place of its own, so it reaches every group, ${name} too, the backup way. The others in ${name} still reach it.`,
};

/**
 * What each place of a group's section is drawn with: the group, GROUP's
 * state and its `dispatch`, HOME's `dispatch`, this computer's key, and
 * the group just joined here, by name (`?joined=`).
 * @typedef {object} PlaceProps
 * @property {Group} g
 * @property {State} s
 * @property {Store<State, Message>['dispatch']} send
 * @property {Home} home
 * @property {string} me
 * @property {string} [joined]
 */

/**
 * HOME's `dispatch`, which each place may tell: a message by its type,
 * with what it carries.
 * @typedef {(message: { type: string, [field: string]: unknown }) => void} Home
 */

/**
 * A group's section's props: the group, HOME's round of reading, each
 * place's view by its name (PLACES), and the rest each place is drawn with.
 * @typedef {object} GroupProps
 * @property {Group} g
 * @property {Round} round
 * @property {Partial<Record<(typeof PLACES)[number], (props: PlaceProps) => Drawn>>} places
 * @property {Flows<State, Effect>} flows
 * @property {Home} home
 * @property {string} me
 * @property {string} [joined]
 */

// The section, in its group's colour, derived from its id (look.css
// `data-colour`), headed by its name and whose it is (#138); when this
// computer meets the group at its fallback, said under it (surface.rs
// `Held::fallback`; relay fallbacks); who is in it unread, or the person
// removed, said and nothing read (home.js@19b63b2b:113-114).
// Below a group's heading: who is in it could not be read, this person
// was removed from it, or its places, once it is read.
/**
 * @param {Group} group
 * @param {string} name
 * @param {Omit<GroupProps, 'g'>} props
 */
const below = (group, name, props) => {
  if (group.removed === null) return html`<p>This computer could not read who is in ${name}.</p>`;
  if (group.removed)
    return html`<p>You are no longer in ${name}. What you already have stays on this computer; nothing new reaches it.</p>`;
  return html`<${Read} g=${group} ...${props} />`;
};
/** @param {GroupProps} props */
export const Group = ({ g: group, ...props }) =>
  ((name) =>
    html`<section data-colour=${[...group.id].reduce((hash, char) => (hash * 31 + char.charCodeAt(0)) % 12, 0)}><h2>${group.name ?? 'A group whose name this computer cannot read'}${group.made && html`<small> (${whose(group.made)})</small>`}</h2>${group.fallback && html`<p class="said">${BACKUP[group.fallback](name)}</p>`}${below(group, name, props)}</section>`)(
    group.name ?? 'this group',
  );
/** @param {GroupProps} props */
function Read({ g: group, round, places, flows, home, me, joined }) {
  const [state, send] = useMachine(() =>
    machine('GROUP', GROUP(flows), { at: { people: 'reading', lends: 'reading' }, id: group.id }),
  );
  useEffect(() => send({ type: round.type }), [round]);
  return html`${PLACES.map((place) => places[place] && html`<${places[place]} g=${group} s=${state} send=${send} home=${home} me=${me} joined=${joined} />`)}${(state.at.people === 'unread' || state.at.lends === 'unread') && html`<p>This computer cannot read what ${group.name ?? 'this group'} lends: ${state.why}</p>`}`;
}
