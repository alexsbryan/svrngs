// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's words: its question, what apps use, who was removed from the
// group, who is visiting, who is on now and what is in it (v1ui-door), in
// one wording for the two pages that ask a person, the home page on a
// computer (home.rs serves this file) and the group's page on a phone
// (scripts/build-web.sh gives it this file). Its words are DESIGN §1's, a
// lend named as the bare command names it (svrngs cli.rs `lends`). Words
// only: the parts in ui/ draw them (ui/kit.js), and each page keeps how it
// asks a lend's choices and posts a yes or a stop (REVIEW_FINDINGS #110,
// #113).

// A lend as a person reads it: "Ana’s taste", and "Sam’s tonights-picks
// (movie-night)" when its name and kind differ; an app the group carries
// by its kind alone, "Money", its lend name holding the group's id
// (lend.rs `carried`) and its copies everyone's (v1ui-polish (b)), a `-`
// in it said as a space: a kind is one word (declared.rs
// `valid_header_name`), and "House-rules" reads "House rules" (canon-house).
export const spoken = (kind) => kind.replaceAll('-', ' ');
export const named = (l) => (l.group ? spoken(l.kind) : `${l.called ?? l.person}’s ${l.name === l.kind ? l.name : `${l.name} (${l.kind})`}`);

// What opening lend `l` in group `g` agrees to, said beside its open on
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
const does = (kind, shelf) => shelf.find((a) => a.name === kind)?.words ?? 'its own part of the group’s record.';
export const agreed = (l, shelf, g) => (l.group ? `${((w) => w[0].toUpperCase() + w.slice(1))(does(l.kind, shelf))} Opening it lets it read and write that, as you.` : `It writes what you say there into ${g.name ?? 'your group'}’s record, as you.${l.reads?.length ? ` It also opens your friends’ ${l.reads.map(spoken).join(' and ')} as you.` : ''}`);

// Until when a guest visits (guests-pages; GUESTS §5 "The pages"), as
// every member's page shows them (ui/visitors.js): a day and an hour.
export const until = (v) => new Date(v.until * 1000).toLocaleString([], { weekday: 'long', hour: 'numeric', minute: '2-digit' });

// Whose a lend is on group `g`'s index: its lender's, or the group's for an
// app the group carries, whichever computer's copy is listed (`once`).
export const owner = (l, g) => (l.group ? (g.name ?? 'your group') : (l.called ?? l.person));

// One copy of each app a group carries, the first listed, and every other
// lend as it is: each computer holding the app lends its copy
// (COMMUNITY_APPS §7, found 4; v1ui-shelf (a)).
export const once = (lent) => lent.filter((l, i) => !l.group || lent.findIndex((o) => o.group === l.group && o.kind === l.kind) === i);

// svrngs's question: may the app lent as `app` use the lend `lend`?
export const question = (app, lend) => `Let ${named(app)} use ${named(lend)}?`;

// A removal, a leave or an ask to remove, as everyone in the group is
// shown it (v1-keeper-office (b); groups-leave): "cy asked to remove dee on
// 12 Oct 2026, 14:03", "ana left on …", the time in the browser's own words.
export const removal = (r, rows = [], all = [...rows, { person: r.by, person_key: r.by_key, joined: r.by_joined }, { person: r.of, person_key: r.of_key, joined: r.of_joined }]) =>
  `${r.said === 'left' ? `${called(all, r.of_key) ?? r.of} left` : `${called(all, r.by_key) ?? r.by} ${r.said === 'removed' ? 'removed' : 'asked to remove'} ${called(all, r.of_key) ?? r.of}`} on ${new Date(r.at * 1000).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' })}`;

// The line said when what apps use of `lend` cannot be asked of its
// lender's computer.
export const unasked = (lend) => `${lend.person}’s computer cannot say now what apps use ${named(lend)}.`;

// The line said when an ask names an app or a lend not lent, or not
// asking, on the page asked: the home page all your groups', the group's
// page its group's (onair-economize (d), REVIEW_FINDINGS #113).
export const unlent = 'That app, or what it asks to use, is not lent here now.';

// A day, from seconds, in this computer's or phone's time, or in UTC when
// `utc`: its number and its month's first three letters, "3 Oct", as the
// command says it (chrono's `%-d %b`, cli.rs `namesakes_said`), never the
// browser's locale, which differs from it (#157).
const day = (t, utc, d = new Date(t * 1000)) => `${utc ? d.getUTCDate() : d.getDate()} ${'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ')[utc ? d.getUTCMonth() : d.getMonth()]}`;

// When someone joined, as both pages tell two of one name apart: its day
// in UTC, one for everyone, as a lend's alias carries it from one person's
// browser to another's (`alias`), where two browsers' own days near
// midnight differ, one at UTC+14 and one at UTC-11 always. So the name the
// page shows beside a lend and the alias it gives to keep say one day.
const joinedSaid = (j) => (j === 'started' ? 'started the group' : `joined ${day(j.at, true)}`);

// The person whose first key is `key`, as the people rows `rows`
// (/v1/rail/people) name them (capability-keys; svrn-docs
// WHAT_CAN_ACT_AS_YOU §9): their name, and where someone else in `rows`
// shares it, when they joined, `when` saying it, "ada (joined 3 Oct)" (its
// day in UTC, `joinedSaid`), as the command tells them apart (cli.rs
// `joined_said`); undefined for a key
// no row holds. Who did something is told by key, never by name.
export const called = (rows, key, when = joinedSaid) => {
  const p = rows.find((r) => r.person_key === key);
  return p && (rows.some((r) => r.person === p.person && r.person_key !== key) ? `${p.person} (${when(p.joined)})` : p.person);
};

// Each of `lent` given `called`, its lender as `called` says them by the
// key of the computer lending it, so `named` and `owner` tell two lenders
// of one name apart.
export const apart = (lent, rows) => lent.forEach((l) => (l.called = called(rows, rows.find((r) => r.key === l.key)?.person_key)));

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
export const alias = (l, lent, rows, at = (o) => `${o.name}@${o.called ?? o.person}`, computer = rows.find((r) => r.key === l.key)) => l.group ? l.kind : lent.some((o) => !o.group && o.key !== l.key && at(o) === at(l)) && computer ? `${at(l)} (computer ${joinedSaid(computer.joined)})` : at(l);

// The lends of `lent` the alias `a` names: one, else none or more than
// one, which the page says, never a nearest match.
export const aliased = (a, lent, rows) => once(lent).filter((l) => alias(l, lent, rows) === a);

// Who made a group and since when this computer or phone has held it, so
// two of one name are told apart (#138): "sam’s, since 14 Oct".
export const whose = (g) => `${g.maker ? `${g.maker}’s` : 'yours'}${g.since ? `, since ${day(g.since)}` : ''}`;

// The groups a new one shares its name with (/group, the node's one rule;
// #138), each said as `whose`, in the command's words (cli.rs
// `namesakes_said`): `verb` says it before the group is made or after.
export const namesakes = (n, name, verb) => (n.length ? `You’re already in ${n.map((g) => `${g.name} (${whose(g)})`).join(' and ')}. ${verb} ${n.length > 1 ? 'another' : 'a second'} group called ${name}.` : '');

// An app the group carries, as "What acts as you" says it: what it reads
// and writes, the shelf's words, and no Stop, as it runs only when opened.
export const carrying = (kind, shelf) => `${spoken(kind)} reads and writes ${does(kind, shelf).replace(/\.$/, '')}, when you open it.`;

// What the person said, `c`, of lend `l` opening their friends' lends of
// the kinds it reads as them (capability-open), since or on its day; `l`
// undefined when it is not lent now.
export const reading = (l, c, [who, what] = l ? [named(l), `your friends’ ${l.reads.map(spoken).join(' and ')}`] : ['An app not lent now', 'your friends’ things']) => c.said === 'yes' ? `${who} opens ${what} as you. Since ${day(c.at)}. ` : `${who} stopped opening ${what} as you on ${day(c.at)}. Opening it again says yes again.`;

// Who can decide for you (§4): the devices holding a key of the person
// whose device key is `me`, of the people rows, this one, a `here`, first
// and nobody else's, a namesake's included, by key, never by name.
export const deciders = (rows, me, here, first = rows.find((r) => r.key === me)?.person_key, own = (d) => rows.filter((r) => r.person_key === first && r.key !== me && r.device === d).length, said = (n, d, one) => (n ? [`your ${here === d ? 'other ' : ''}${n > 1 ? `${n} ${one}s` : one}`] : [])) => [`This ${here === 'computer' ? 'computer' : 'phone'}`, ...said(own('computer'), 'computer', 'computer'), ...said(own('browser'), 'browser', 'phone')].join(', ').replace(/, ([^,]*)$/, ', and $1');

// How lend `l` opens, `where`, and whether it is full now: the index's row
// as both pages judge it, so a change to either lands on both (#158).
export const opens = (l, where) => [where, l.limit && l.using >= l.limit ? 'full' : 'on'];

