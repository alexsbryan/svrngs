// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's words: its question, what apps use, who was removed from the
// group, who is visiting, who is on now and what is in it, in
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
 * @property {ReadLend[]} [reading] what its open names, on a computer (surface.rs `opening`)
 * @property {number | null} [version] the lending computer's svrngs
 * @property {number} [limit] how many may have it open at once
 * @property {number} [using] how many have it open now
 * @property {string[] | null} [groups] the stable ids of the groups that may
 *   open it, in the owner's own `/lent` answer: `[]` paused, `null` legacy
 *   and needing selection (SCOPE_PARALLEL.md, owner `GET /lent`)
 * @property {string} [called] its lender, as `called` tells namesakes apart
 * @property {string | null} [check] what `svrngs check` finds in a carried
 *   app's folder here, in words for its card (surface.rs `lends`)
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
// (lend.rs `carried`) and its copies everyone's, a `-` in it said as a
// space: a kind is one word (declared.rs `valid_header_name`), and
// "House-rules" reads "House rules".
/** @param {string} kind */
export const spoken = (kind) => kind.replaceAll('-', ' ');
// A lend's label, the first name of its origin here: what /to, /choices
// and /reaching name it by. A split answers one part at least, so its
// first is there.
/**
 * @param {{ origin: string }} lend
 * @returns {string}
 */
export const label = (lend) => /** @type {string} */ (new URL(lend.origin).hostname.split('.')[0]);
/** @param {Pick<Lend, 'name' | 'kind' | 'person' | 'group' | 'called'>} lend */
export const named = (lend) => {
  if (lend.group) return spoken(lend.kind);
  const name = lend.name === lend.kind ? lend.name : `${lend.name} (${lend.kind})`;
  return `${lend.called ?? lend.person}’s ${name}`;
};

// What opening `lend` in `group` agrees to, said beside its open on
// both pages. The open names the app, and pressing it is the yes
// (svrn-docs WHAT_CAN_ACT_AS_YOU §9); said on the button,
// these words read as a grant asked of the person, Ana's 5 of 5 times
// (demo/xenia/findings/first-set/after-join-reads-as-permissions.md; the
// operator, 2026-10-26). An app the group carries reads and writes what the
// shelf, `shelf`, says it is, else its own part of the group's record; an
// app on a port may write its own part as you, by naming it
// (capability-signed's F1), which svrngs cannot know it does until it does,
// so the words say what opening it allows, never that it writes (one agent
// run, 2026-11-09: the page said a guestbook that wrote nothing "writes what
// you say there into Friends’s record"); a lend declared to read kinds (`--reads`, or a
// carried app's reads.txt) opens your friends' lends of them as you. Made
// from what the lend declares, never its kind of app. A carried app's says
// what it is for first, then what opening it lets it do: led by "It reads and writes", Ana read it as
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
export const agreed = (lend, shelf, group) => {
  const reads = lend.reads ?? [];
  // The open's own line names them (`readLine`) where it was told what it
  // reads; this computer's own app, which needs no yes, says it by kind.
  const also =
    reads.length > 0 && !lend.reading
      ? ` It also opens your friends’ ${reads.map(spoken).join(' and ')} as you, and they can see what it asks.`
      : '';
  if (lend.group) {
    const words = does(lend.kind, shelf);
    // tsc reads `words[0]` as possibly missing: `does` answers the shelf's
    // sentence for the app, or its own.
    const first = /** @type {string} */ (words[0]).toUpperCase();
    const may = reads.length > 0 ? 'read that, and never write it' : 'read and write that';
    // What its page asks that apps can't load, before anyone opens it
    // (EXTENSION_POINT §6.3): else a blank page says nothing.
    const told = lend.check ? ` ${lend.check}` : '';
    return `${first}${words.slice(1)} Opening it lets it ${may}, as you.${also}${told}`;
  }
  return `Opening it lets it write its own part of ${group.name ?? 'your group'}’s record, as you.${also}`;
};

// Until when a guest visits (svrn-docs GUESTS §5 "The pages"), as
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
// (svrn-docs COMMUNITY_APPS §7, found 4).
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
// shown it: "cy asked to remove dee on
// 12 Oct 2026, 14:03", "ana left on …", the time in the browser's own words.
// Each is told apart among the people rows and the two the removal names.
/**
 * @param {Removal} gone
 * @param {Named[]} [rows]
 */
export const removal = (gone, rows = []) => {
  const all = [
    ...rows,
    { person: gone.by, person_key: gone.by_key, joined: gone.by_joined },
    { person: gone.of, person_key: gone.of_key, joined: gone.of_joined },
  ];
  const of = called(all, gone.of_key) ?? gone.of;
  const at = new Date(gone.at * 1000).toLocaleString([], {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
  if (gone.said === 'left') return `${of} left on ${at}`;
  const by = called(all, gone.by_key) ?? gone.by;
  const did = gone.said === 'removed' ? 'removed' : 'asked to remove';
  return `${by} ${did} ${of} on ${at}`;
};

// The line said when what apps use of `lend` cannot be asked of its
// lender's computer.
/** @param {Lend} lend */
export const unasked = (lend) =>
  `${lend.person}’s computer cannot say now what apps use ${named(lend)}.`;

// The line said when an ask names an app or a lend not lent, or not
// asking, on the page asked: the home page all your groups', the group's
// page its group's (REVIEW_FINDINGS #113).
export const unlent = 'That app, or what it asks to use, is not lent here now.';

const MONTHS = 'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ');

// A day, from seconds, in this computer's or phone's time, or in UTC when
// `utc`: its number and its month's first three letters, "3 Oct", as the
// command says it (chrono's `%-d %b`, cli.rs `namesakes_said`), never the
// browser's locale, which differs from it (#157).
/**
 * @param {number} seconds
 * @param {boolean} [utc]
 */
export const day = (seconds, utc) => {
  const date = new Date(seconds * 1000);
  const month = utc ? date.getUTCMonth() : date.getMonth();
  return `${utc ? date.getUTCDate() : date.getDate()} ${MONTHS[month]}`;
};

// When someone joined, as both pages tell two of one name apart: its day
// in UTC, one for everyone, as a lend's alias carries it from one person's
// browser to another's (`alias`), where two browsers' own days near
// midnight differ, one at UTC+14 and one at UTC-11 always. So the name the
// page shows beside a lend and the alias it gives to keep say one day.
/** @param {Joined} joined */
const joinedSaid = (joined) =>
  joined === 'started' ? 'started the group' : `joined ${day(joined.at, true)}`;

// The person whose first key is `key`, as the people rows `rows`
// (/v1/rail/people) name them (svrn-docs WHAT_CAN_ACT_AS_YOU §9): their name, and where someone else in `rows`
// shares it, when they joined, `when` saying it, "ada (joined 3 Oct)" (its
// day in UTC, `joinedSaid`), as the command tells them apart (cli.rs
// `joined_said`); undefined for a key no row holds. Who did something is
// told by key, never by name.
/**
 * @param {Named[]} rows
 * @param {string | undefined} key
 * @param {(joined: Joined) => string} [when]
 */
export const called = (rows, key, when = joinedSaid) => {
  const person = rows.find((row) => row.person_key === key);
  if (!person) return undefined;
  const namesake = rows.some((row) => row.person === person.person && row.person_key !== key);
  return namesake ? `${person.person} (${when(person.joined)})` : person.person;
};

// Each of `lent` given `called`, its lender as `called` says them by the
// key of the computer lending it, so `named` and `owner` tell two lenders
// of one name apart.
/**
 * @param {Lend[]} lent
 * @param {PeopleRow[]} rows
 */
export const apart = (lent, rows) => {
  for (const lend of lent) {
    const computer = rows.find((row) => row.key === lend.key);
    lend.called = called(rows, computer?.person_key);
  }
};

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
export const whose = (group) => {
  const maker = group.maker ? `${group.maker}’s` : 'yours';
  const since = group.since != null && group.since > 0 ? `, since ${day(group.since)}` : '';
  return `${maker}${since}`;
};

// A group's name, and whose it is when someone else made it, as the phone
// titles it.
/** @param {{ name: string, maker?: string | null, since?: number | null }} group */
export const titled = (group) => `${group.name}${group.maker ? ` (${whose(group)})` : ''}`;

// The computer a dial went through, or the link's when none is named.
/** @param {{ person: string } | undefined} computer */
export const whoseComputer = (computer) =>
  computer ? `${computer.person}’s computer` : 'The computer that sent your link';

// Whose computer did not answer, when the way to it did: all a page can know
// of it, so never that it sleeps, nor since when (DESIGN §3).
export const silent = (/** @type {string} */ whom) => `${whom} did not answer, so it may be off.`;

// Why a dial failed: what the runtime refused with (web/src/lib.rs `refuse!`)
// or the browser did, each try's when several failed, each cause in its own
// words; `quiet` when only the computer was silent. The one mapping both a
// page and a lend's worker say a dial's failure by (KEEP_GOING_AT_HOME 18).
/**
 * @param {any} error
 * @param {string} quiet
 */
export const unreached = (error, quiet) => {
  const why = String(error?.errors?.join(' ') ?? error);
  if (/NotAllowedError|SecurityError/.test(why))
    return 'This browser refused what svrngs needs to reach your group. Allow it in the browser’s settings and try again.';
  if (why.includes('names no relay this page'))
    return 'This link names no way in this page can use. Ask whoever sent it for a new one.';
  if (why.includes('no relay answered'))
    return 'Nothing on the way to your group answered, so no computer was asked. Check your connection and try again.';
  if (/LAYER (parse|key|wasm-bindgen)/.test(why))
    return 'This page could not start the connection, so no computer was asked. Reload it and try again.';
  return quiet;
};

// The groups a new one shares its name with (/group, the node's one rule;
// #138), each said as `whose`, in the command's words (cli.rs
// `namesakes_said`): `verb` says it before the group is made or after.
/**
 * @param {{ name: string, maker?: string, since?: number }[]} same
 * @param {string} name
 * @param {string} verb
 */
export const namesakes = (same, name, verb) => {
  if (!same.length) return '';
  const groups = same.map((group) => `${group.name} (${whose(group)})`).join(' and ');
  const which = same.length > 1 ? 'another' : 'a second';
  return `You’re already in ${groups}. ${verb} ${which} group called ${name}.`;
};

// An app the group carries, as "What acts as you" says it: what it reads
// and, unless it reads friends' lends, writes (CARRIED_APP_READS §5.5), the
// shelf's words, and no Stop, as it runs only when opened.
/**
 * @param {Pick<Lend, 'kind' | 'reads'>} lend
 * @param {Shelved[]} shelf
 */
export const carrying = ({ kind, reads = [] }, shelf) =>
  `${spoken(kind)} ${reads.length ? 'reads' : 'reads and writes'} ${does(kind, shelf).replace(/\.$/, '')}, when you open it.`;

// What the person said, `said`, of `lend` opening their friends' lends as
// them, since or on its day; `lend` undefined when it is not lent now. A
// yes names its lends, so it says what they chose, and a yes from before
// that named none reads as nothing until the next open.
/**
 * @param {(Lend & { reads: string[] }) | undefined} lend
 * @param {{ said: string, at: number, lends?: string[] }} said
 */
export const reading = (lend, said) => {
  const who = lend ? named(lend) : 'An app not lent now';
  const kinds = lend ? lend.reads.map(spoken).join(' and ') : 'things';
  if (said.said === 'yes' && said.lends?.length === 0)
    return `${who} opens nothing of your friends’ as you. Open it again to choose what it can. `;
  const chosen = said.lends ? `the ${kinds} you chose,` : `your friends’ ${kinds}`;
  if (said.said === 'yes') return `${who} opens ${chosen} as you. Since ${day(said.at)}. `;
  return `${who} stopped opening your friends’ ${kinds} as you on ${day(said.at)}. Opening it again says yes again.`;
};

// What the line beside a reading app's Open says: who it names, by person,
// or by how many past a handful (`HANDFUL`, within the 20-word budget a
// line was read to, demo/xenia findings); then what its last yes didn't
// name, or who can see it. Kinds, `spoken`, say what it reads: svrngs
// cannot say "films" for a program it does not know.
const HANDFUL = 4;
/** @param {string[]} words */
const list = (words) =>
  words.length > 1 ? `${words.slice(0, -1).join(', ')} and ${words.at(-1)}` : `${words[0]}`;
/** @param {string[]} people */
const owners = (people) => list(people.map((person) => `${person}’s`));
/**
 * A lend its open names, as /v1/lends answers it (surface.rs `opening`).
 * @typedef {{ label: string, person: string, name: string, kind: string, ticked?: boolean, new: boolean }} ReadLend
 */
/**
 * @param {ReadLend[]} on  the lends left ticked
 * @param {number} lent  how many it could read
 * @param {string[]} kinds  what it reads, the program's names
 */
export const readLine = (on, lent, kinds) => {
  const read = kinds.map(spoken).join(' and ');
  if (lent === 0) return `Reads your friends’ ${read}. Nobody lends one now.`;
  if (on.length === 0) return `Reads none of your friends’ ${read}.`;
  /** @param {ReadLend[]} lends */
  const persons = (lends) => [...new Set(lends.map((one) => one.person))];
  const byKind = [...new Set(on.map((one) => one.kind))].map(
    (kind) => `${owners(persons(on.filter((one) => one.kind === kind)))} ${spoken(kind)}`,
  );
  const said =
    persons(on).length > HANDFUL ? `${read} from ${persons(on).length} friends` : list(byKind);
  const fresh = persons(on.filter((one) => one.new));
  const after = fresh.length
    ? `${owners(fresh)} ${fresh.length > 1 ? 'are' : 'is'} new since you last opened it.`
    : 'They can see it’s you, and what it asks for.';
  return `Reads ${said}. ${after}`;
};

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

// Whether `lend` is full now: the index's row
// as both pages judge it, so a change to either lands on both (#158).
// A lend with no count of who is using it is not full: `undefined >=` is
// false. Nor is one with no limit (`null` on the wire; never 0, lend.rs
// `Offered::limit` is a `NonZeroU16`): none reaches `Infinity`.
/**
 * @param {Lend} lend
 * @returns {'full' | 'on'}
 */
export const opens = (lend) =>
  /** @type {number} */ (lend.using) >= (lend.limit ?? Infinity) ? 'full' : 'on';
