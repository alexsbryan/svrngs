// SPDX-License-Identifier: AGPL-3.0-or-later
// One group's section on the home page: its heading, in the group's
// colour, whose it is, how this computer meets it, and below, once this
// computer can read who is in it, the section as read (group-read.js),
// whose machine is GROUP (group-reading.js). web/tests/fallback.test.mjs
// draws `Group` with no DOM, so it holds no hook. The types here are the
// group's as /v1/groups, /v1/rail/people and /v1/lends answer, which each
// feature's place is drawn with.
import { html } from '../ui/view.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';
import { GroupRead } from './group-read.js?v=__VERSION__';
/** @import { Store } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Lend, Named, Removal, PeopleRow, Shelved } from '../question.js?v=__VERSION__' */
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Effect, Message, Round, State } from './group-reading.js?v=__VERSION__' */
/** @import { Flows } from './offered.js?v=__VERSION__' */
/** @import { PlaceName } from './group-read.js?v=__VERSION__' */

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
 * @property {[string | null, number][] | null} waiting a fetch that found no holder, by writer's person
 * @property {Visit[] | null} visiting
 * @property {'fallback' | null} fallback why it is met the backup way (svrngs-core relay.rs `Tier`), `null` at its own
 * @property {true} [vacates] whether they keep its order of removals and leaving without handing it on leaves nobody keeping it (a group made before sources-v1 refuses that leave instead)
 * @property {{ to_you: boolean, person: string | null }} [offered] a hand-on of its order of removals waiting on its Accept, to this computer or from it, and who is at its other end
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

// Why this computer reaches a group the backup way, by surface.rs's
// `fallback` (relay fallbacks; svrngs-core relay.rs `Tier`).
/** @type {Record<NonNullable<Group['fallback']>, (name: string) => string>} */
const BACKUP = {
  fallback: (name) =>
    `This computer is reachable through the public way for now, so a phone can open ${name} too. It goes back to the place ${name} usually meets once the public way stops answering.`,
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
 * @property {Partial<Record<PlaceName, (props: PlaceProps) => Drawn>>} places
 * @property {Flows<State, Effect>} flows
 * @property {Home} home
 * @property {string} me
 * @property {string} [joined]
 * @property {boolean} [grouped]
 * @property {boolean} [hidden]
 */

// The colour a group is drawn in on every page, one of look.css's twelve
// `data-colour`s, derived from its id.
/** @param {string} id */
export const colour = (id) => {
  let hash = 0;
  for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) % 12;
  return hash;
};
// Below a group's heading: who is in it could not be read, this person
// was removed from it, so nothing is read, or the section as read.
/**
 * @param {Group} group
 * @param {string} name
 * @param {Omit<GroupProps, 'g'>} props
 */
function below(group, name, props) {
  if (group.removed === null) return html`<p>This computer could not read who is in ${name}.</p>`;
  if (group.removed)
    return html`<p>You are no longer in ${name}. What you already have stays on this computer; nothing new reaches it.</p>`;
  return html`<${GroupRead} g=${group} ...${props} />`;
}

// The section, in its group's colour (look.css `data-colour`), headed by
// its name and whose it is; when this computer meets the group at its
// fallback (surface.rs `Held::fallback`), said after the group's controls
// under Connection details, closed at first: how it reaches the group is
// not what a person came to do (docs/APP_ACCESS_AND_STOP.md A3). A group
// that cannot be read is said under the heading, where it stops them.
/** @param {GroupProps} props */
export function Group({ g: group, ...props }) {
  const name = group.name ?? 'this group';
  const heading = group.name ?? 'A group whose name this computer cannot read';
  const made = group.made && html`<small> (${whose(group.made)})</small>`;
  return html`
    <section id=${`group-${group.id}`} data-colour=${colour(group.id)} hidden=${props.hidden}>
      <h2>${heading}${made}</h2>
      ${below(group, name, props)}
      ${group.fallback && html`<details class="said"><summary>Connection details</summary><p>${BACKUP[group.fallback](name)}</p></details>`}
    </section>
  `;
}
