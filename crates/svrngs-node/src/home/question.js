// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's words: its question, what apps use, who was removed from the
// group, who is visiting, who is on now and what is in it (v1ui-door), in
// one wording for the two pages that ask a person, the home page on a
// computer (home.rs serves this file) and the group's page on a phone
// (scripts/build-web.sh gives it this file). Its words are DESIGN §1's, a
// lend named as the bare command names it (svrngs cli.rs `lends`). Words
// only: the parts in ui/ draw them, and each page keeps how it
// asks a lend's choices and posts a yes or a stop (REVIEW_FINDINGS #110,
// #113).

/**
 * A lend, as a computer's node lists it to either page (/people/lends,
 * /v1/rail/lends): whose computer lends it, its name and kind, and what it
 * declares. `group` is the group's id, for an app the group carries
 * (lend.rs `carried`); `called` is `apart`'s.
 * @typedef {object} Lend
 * @property {string} key the lending computer's key
 * @property {string | null} [relay]
 * @property {string} name
 * @property {string} kind
 * @property {string} person the lender's name
 * @property {string | null} [group]
 * @property {string} [asks] the lender's words, asking for a yes
 * @property {string[]} [reads] the kinds it opens as you (`--reads`)
 * @property {number | null} [version] the lending computer's svrngs
 * @property {number} [limit] how many may have it open at once
 * @property {number} [using] how many have it open now
 * @property {string} [called] its lender, as `called` tells namesakes apart
 */

/**
 * When someone joined: the group's maker started it, everyone else joined
 * at `at`, in seconds.
 * @typedef {'started' | { at: number }} Joined
 */

/**
 * A person, as `called` tells them apart: their name, their first key, and
 * when they joined.
 * @typedef {{ person: string, person_key: string, joined: Joined }} Named
 */

/**
 * A people row (/v1/rail/people): one key of a person's, a device's, `key`,
 * and the person it is a key of.
 * @typedef {Named & { key: string, device?: 'computer' | 'browser' }} PeopleRow
 */

/**
 * A removal, a leave or an ask to remove, by whom and of whom, each by
 * name, key and when they joined, and when, in seconds.
 * @typedef {object} Removal
 * @property {string} said 'left', 'removed', or else an ask to remove
 * @property {string} by
 * @property {string} by_key
 * @property {Joined} by_joined
 * @property {string} of
 * @property {string} of_key
 * @property {Joined} of_joined
 * @property {number} at
 */

/**
 * An app on the group's shelf, by its kind, and the shelf's words for what
 * it is.
 * @typedef {{ name: string, words?: string }} Shelved
 */

// A lend as a person reads it: "Ana’s taste", and "Sam’s tonights-picks
// (movie-night)" when its name and kind differ; an app the group carries
// by its kind alone, "Money", its lend name holding the group's id
// (lend.rs `carried`) and its copies everyone's (v1ui-polish (b)), a `-`
// in it said as a space: a kind is one word (declared.rs
// `valid_header_name`), and "House-rules" reads "House rules" (canon-house).
/** @param {string} kind */
export const spoken = (kind) => kind.replaceAll('-', ' ');
/** @param {Lend} lend */
export const named = (lend) =>
  lend.group
    ? spoken(lend.kind)
    : `${lend.called ?? lend.person}’s ${lend.name === lend.kind ? lend.name : `${lend.name} (${lend.kind})`}`;

// What opening `lend` in `group` agrees to, said beside its open on
// both pages. The open names the app, and pressing it is the yes
// (capability-open; svrn-docs WHAT_CAN_ACT_AS_YOU §9); said on the button,
// these words read as a grant asked of the person, Ana's 5 of 5 times
// (demo/xenia/findings/first-set/after-join-reads-as-permissions.md; the
// operator, 2026-10-26). An app the group carries reads and writes what the
// shelf, `shelf`, says it is, else its own part of the group's record; an
// app on a port writes what you say there as you, by naming it
// (capability-signed, F1); a lend declared to read kinds (`--reads`) opens
// your friends' lends of them as you. Made from what the lend declares,
// never its kind of app. A carried app's says what it is for first, then
// what opening it lets it do: led by "It reads and writes", Ana read it as
// done on her behalf without her yes ("Money can read and write who owes
// whom on my behalf, and I never agreed to that", the fifth set, `together`
// run 1; demo/xenia/findings/said-too-much-the-group-s-page.md).
/**
 * @param {string} kind
 * @param {Shelved[]} shelf
 */
const does = (kind, shelf) =>
  shelf.find((app) => app.name === kind)?.words ?? 'its own part of the group’s record.';
/**
 * @param {Lend} lend
 * @param {Shelved[]} shelf
 * @param {{ name?: string | null }} group
 */
export const agreed = (lend, shelf, group) =>
  lend.group
    ? `${((words) => /** @type {string} */ (words[0]).toUpperCase() + words.slice(1))(does(lend.kind, shelf))} Opening it lets it read and write that, as you.`
    : `It writes what you say there into ${group.name ?? 'your group'}’s record, as you.${lend.reads?.length ? ` It also opens your friends’ ${lend.reads.map(spoken).join(' and ')} as you.` : ''}`;

// Until when a guest visits (guests-pages; GUESTS §5 "The pages"), as
// every member's page shows them (ui/visitors.js): a day and an hour.
/** @param {{ until: number }} visit */
export const until = (visit) =>
  new Date(visit.until * 1000).toLocaleString([], {
    weekday: 'long',
    hour: 'numeric',
    minute: '2-digit',
  });

// Whose a lend is on `group`'s index: its lender's, or the group's for an
// app the group carries, whichever computer's copy is listed (`once`).
/**
 * @param {Lend} lend
 * @param {{ name?: string | null }} group
 */
export const owner = (lend, group) =>
  lend.group ? (group.name ?? 'your group') : (lend.called ?? lend.person);

// One copy of each app a group carries, the first listed, and every other
// lend as it is: each computer holding the app lends its copy
// (COMMUNITY_APPS §7, found 4; v1ui-shelf (a)).
/**
 * @template {Lend} L
 * @param {L[]} lent
 */
export const once = (lent) =>
  lent.filter(
    (lend, i) =>
      !lend.group ||
      lent.findIndex((other) => other.group === lend.group && other.kind === lend.kind) === i,
  );

// svrngs's question: may the app lent as `app` use the lend `lend`?
/**
 * @param {Lend} app
 * @param {Lend} lend
 */
export const question = (app, lend) => `Let ${named(app)} use ${named(lend)}?`;

// A removal, a leave or an ask to remove, as everyone in the group is
// shown it (v1-keeper-office (b); groups-leave): "cy asked to remove dee on
// 12 Oct 2026, 14:03", "ana left on …", the time in the browser's own words.
/**
 * @param {Removal} gone
 * @param {Named[]} [rows]
 * @param {Named[]} [all]
 */
export const removal = (
  gone,
  rows = [],
  all = [
    ...rows,
    { person: gone.by, person_key: gone.by_key, joined: gone.by_joined },
    { person: gone.of, person_key: gone.of_key, joined: gone.of_joined },
  ],
) =>
  `${gone.said === 'left' ? `${called(all, gone.of_key) ?? gone.of} left` : `${called(all, gone.by_key) ?? gone.by} ${gone.said === 'removed' ? 'removed' : 'asked to remove'} ${called(all, gone.of_key) ?? gone.of}`} on ${new Date(gone.at * 1000).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' })}`;

// The line said when what apps use of `lend` cannot be asked of its
// lender's computer.
/** @param {Lend} lend */
export const unasked = (lend) =>
  `${lend.person}’s computer cannot say now what apps use ${named(lend)}.`;

// The line said when an ask names an app or a lend not lent, or not
// asking, on the page asked: the home page all your groups', the group's
// page its group's (onair-economize (d), REVIEW_FINDINGS #113).
export const unlent = 'That app, or what it asks to use, is not lent here now.';

// A day, from seconds, in this computer's or phone's time, or in UTC when
// `utc`: its number and its month's first three letters, "3 Oct", as the
// command says it (chrono's `%-d %b`, cli.rs `namesakes_said`), never the
// browser's locale, which differs from it (#157).
/**
 * @param {number} seconds
 * @param {boolean} [utc]
 */
const day = (seconds, utc, date = new Date(seconds * 1000)) =>
  `${utc ? date.getUTCDate() : date.getDate()} ${'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ')[utc ? date.getUTCMonth() : date.getMonth()]}`;

// When someone joined, as both pages tell two of one name apart: its day
// in UTC, one for everyone, as a lend's alias carries it from one person's
// browser to another's (`alias`), where two browsers' own days near
// midnight differ, one at UTC+14 and one at UTC-11 always. So the name the
// page shows beside a lend and the alias it gives to keep say one day.
/** @param {Joined} joined */
const joinedSaid = (joined) =>
  joined === 'started' ? 'started the group' : `joined ${day(joined.at, true)}`;

// The person whose first key is `key`, as the people rows `rows`
// (/v1/rail/people) name them (capability-keys; svrn-docs
// WHAT_CAN_ACT_AS_YOU §9): their name, and where someone else in `rows`
// shares it, when they joined, `when` saying it, "ada (joined 3 Oct)" (its
// day in UTC, `joinedSaid`), as the command tells them apart (cli.rs
// `joined_said`); undefined for a key
// no row holds. Who did something is told by key, never by name.
/**
 * @param {Named[]} rows
 * @param {string | undefined} key
 * @param {(joined: Joined) => string} [when]
 */
export const called = (rows, key, when = joinedSaid) => {
  const person = rows.find((row) => row.person_key === key);
  return (
    person &&
    (rows.some((row) => row.person === person.person && row.person_key !== key)
      ? `${person.person} (${when(person.joined)})`
      : person.person)
  );
};

// Each of `lent` given `called`, its lender as `called` says them by the
// key of the computer lending it, so `named` and `owner` tell two lenders
// of one name apart.
/**
 * @param {Lend[]} lent
 * @param {PeopleRow[]} rows
 */
export const apart = (lent, rows) =>
  lent.forEach(
    (lend) => (lend.called = called(rows, rows.find((row) => row.key === lend.key)?.person_key)),
  );

// A lend's alias (svrn-docs rings/current/IN_THE_BROWSER.md, "Decided";
// DESIGN §3, "Lent apps", call 1), what the group's page opens by name at
// `#open=<alias>`: its name at whose it is, "films@Sam", whose as `apart`
// says it, so two lenders of one name are told apart by when each joined,
// "films@Sam (joined 24 Oct)", and two computers of one person lending one
// name by when each computer joined, "films@Sam (computer joined 25 Oct)";
// an app the group carries by its kind, "Money", as `once` lists one copy.
// Every page reads the same record, so an alias names the same lend on
// each, its day in UTC (`joinedSaid`) whatever the browser's time zone.
// `rows` the people rows.
/**
 * @param {Lend} lend
 * @param {Lend[]} lent
 * @param {PeopleRow[]} rows
 */
export const alias = (lend, lent, rows) => {
  const at = (/** @type {Lend} */ one) => `${one.name}@${one.called ?? one.person}`;
  const computer = rows.find((row) => row.key === lend.key);
  if (lend.group) return lend.kind;
  const shared = lent.some(
    (other) => !other.group && other.key !== lend.key && at(other) === at(lend),
  );
  if (shared && computer) return `${at(lend)} (computer ${joinedSaid(computer.joined)})`;
  return at(lend);
};

// The lends of `lent` the alias `wanted` names: one, else none or more than
// one, which the page says, never a nearest match.
/**
 * @template {Lend} L
 * @param {string} wanted
 * @param {L[]} lent
 * @param {PeopleRow[]} rows
 */
export const aliased = (wanted, lent, rows) =>
  once(lent).filter((lend) => alias(lend, lent, rows) === wanted);

// Who made a group and since when this computer or phone has held it, so
// two of one name are told apart (#138): "sam’s, since 14 Oct".
/** @param {{ maker?: string | null, since?: number | null }} group */
export const whose = (group) =>
  `${group.maker ? `${group.maker}’s` : 'yours'}${group.since ? `, since ${day(group.since)}` : ''}`;

// The groups a new one shares its name with (/group, the node's one rule;
// #138), each said as `whose`, in the command's words (cli.rs
// `namesakes_said`): `verb` says it before the group is made or after.
/**
 * @param {{ name: string, maker?: string, since?: number }[]} same
 * @param {string} name
 * @param {string} verb
 */
export const namesakes = (same, name, verb) =>
  same.length
    ? `You’re already in ${same.map((group) => `${group.name} (${whose(group)})`).join(' and ')}. ${verb} ${same.length > 1 ? 'another' : 'a second'} group called ${name}.`
    : '';

// An app the group carries, as "What acts as you" says it: what it reads
// and writes, the shelf's words, and no Stop, as it runs only when opened.
/**
 * @param {string} kind
 * @param {Shelved[]} shelf
 */
export const carrying = (kind, shelf) =>
  `${spoken(kind)} reads and writes ${does(kind, shelf).replace(/\.$/, '')}, when you open it.`;

// What the person said, `said`, of `lend` opening their friends' lends of
// the kinds it reads as them (capability-open), since or on its day; `lend`
// undefined when it is not lent now.
/**
 * @param {(Lend & { reads: string[] }) | undefined} lend
 * @param {{ said: string, at: number }} said
 * @param {[string, string]} [whoWhat]
 */
export const reading = (
  lend,
  said,
  [who, what] = lend
    ? [named(lend), `your friends’ ${lend.reads.map(spoken).join(' and ')}`]
    : ['An app not lent now', 'your friends’ things'],
) =>
  said.said === 'yes'
    ? `${who} opens ${what} as you. Since ${day(said.at)}. `
    : `${who} stopped opening ${what} as you on ${day(said.at)}. Opening it again says yes again.`;

// Who can decide for you (§4): the devices holding a key of the person
// whose device key is `me`, of the people rows, this one, a `here`, first
// and nobody else's, a namesake's included, by key, never by name.
/**
 * @param {PeopleRow[]} rows
 * @param {string} me
 * @param {'computer' | 'browser'} here
 */
export const deciders = (rows, me, here) => {
  const first = rows.find((row) => row.key === me)?.person_key;
  /** @param {'computer' | 'browser'} device */
  const own = (device) =>
    rows.filter((row) => row.person_key === first && row.key !== me && row.device === device)
      .length;
  /**
   * @param {number} count
   * @param {'computer' | 'browser'} device
   * @param {string} one
   */
  const said = (count, device, one) =>
    count ? [`your ${here === device ? 'other ' : ''}${count > 1 ? `${count} ${one}s` : one}`] : [];
  return [
    `This ${here === 'computer' ? 'computer' : 'phone'}`,
    ...said(own('computer'), 'computer', 'computer'),
    ...said(own('browser'), 'browser', 'phone'),
  ]
    .join(', ')
    .replace(/, ([^,]*)$/, ', and $1');
};

// How `lend` opens, `where`, and whether it is full now: the index's row
// as both pages judge it, so a change to either lands on both (#158).
// A lend with no count of who is using it is not full: `undefined >=` is
// false.
/**
 * @param {Lend} lend
 * @param {string} [where]
 * @returns {[string | undefined, 'full' | 'on']}
 */
export const opens = (lend, where) => [
  where,
  lend.limit && /** @type {number} */ (lend.using) >= lend.limit ? 'full' : 'on',
];
