// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's question, what apps use, who was removed from the group, who is
// visiting, who is on now and what is in it (the door and the index's rows, v1ui-door), in
// one wording for the two pages
// that ask a person: the home page on a computer (home.js, served by
// home.rs) and the group's page on a phone (web/app.js, which
// scripts/build-web.sh gives this file). Its words are DESIGN §1's, a lend
// named as the bare command names it (svrngs cli.rs `lends`); each page
// keeps only how it asks a lend's choices and posts a yes or a stop
// (REVIEW_FINDINGS #110, #113).

// A lend as a person reads it: "Ana’s taste", and "Sam’s tonights-picks
// (movie-night)" when its name and kind differ; an app the group carries
// by its kind alone, "Money", its lend name holding the group's id
// (lend.rs `carried`) and its copies everyone's (v1ui-polish (b)), a `-`
// in it said as a space: a kind is one word (declared.rs
// `valid_header_name`), and "House-rules" reads "House rules" (canon-house).
export const spoken = (kind) => kind.replaceAll('-', ' ');
export const named = (l) => (l.group ? spoken(l.kind) : `${l.called ?? l.person}’s ${l.name === l.kind ? l.name : `${l.name} (${l.kind})`}`);

// What opening lend `l` in group `g` agrees to, said on its open on both
// pages, pressing which is the yes (capability-open; svrn-docs
// WHAT_CAN_ACT_AS_YOU §9): an app the group carries reads and writes what
// the shelf, `shelf`, says it is, else its own part of the group's record;
// an app on a port writes what you say there as you, by naming it
// (capability-signed, F1); a lend declared to read kinds (`--reads`) opens
// your friends' lends of them as you. Made from what the lend declares,
// never its kind of app.
const does = (kind, shelf) => shelf.find((a) => a.name === kind)?.words ?? 'its own part of the group’s record.';
const writes = (g) => `. It writes what you say there into ${g.name ?? 'your group'}’s record, as you`;
export const agreed = (l, shelf, g) => `Open ${named(l)}${l.group ? `. It reads and writes ${does(l.kind, shelf)}` : `${writes(g)}.${l.reads?.length ? ` It also opens your friends’ ${l.reads.map(spoken).join(' and ')} as you.` : ''}`}`;

// Who is visiting now (guests-pages; GUESTS §5 "The pages"), from the
// record, as every member's page shows them: each guest, by host, until a
// day and an hour.
export const until = (v) => new Date(v.until * 1000).toLocaleString([], { weekday: 'long', hour: 'numeric', minute: '2-digit' });
export const visiting = (vs, rows = []) => Object.assign(el('ul', '', ...[...vs].sort((a, b) => a.host.localeCompare(b.host)).map((v) => el('li', `${v.guest}, ${called(rows, v.host_key) ?? v.host}’s guest, until ${until(v)}`))), { className: 'visiting' });

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

// What apps use of each lend of `lent` that asks, from its lender's
// choices, `choicesOf(l)`, each page asking by its own transport: a line
// each, made by `line(words)`, the app named when `lent` lists it by
// `labelOf`, each yes with Stop, which calls `stop(l, app, line)`; a lend
// whose choices cannot be asked said so, with why.
export async function uses(lent, labelOf, choicesOf, line, stop) {
  for (const l of lent.filter((o) => o.asks)) {
    let choices;
    try {
      choices = await choicesOf(l);
    } catch (e) {
      line(`${unasked(l)} ${e.message ?? ''}`.trim());
      continue;
    }
    for (const [app, c] of Object.entries(choices)) {
      const by = lent.find((o) => labelOf(o) === app);
      const item = line(`${by ? named(by) : 'An app'} ${c.said === 'yes' ? 'uses' : 'was stopped using'} ${named(l)}. `);
      if (c.said !== 'yes') continue;
      const button = Object.assign(document.createElement('button'), { textContent: 'Stop' });
      button.onclick = () => stop(l, app, item);
      item.append(button);
    }
  }
}

// An element: `tag`, its words and then `kids`.
export const el = (tag, text, ...kids) => {
  const e = document.createElement(tag);
  e.append(text ?? '', ...kids);
  return e;
};

// A day, from seconds, in this computer's or phone's time: its number and
// its month's first three letters, "3 Oct", as the command says it
// (chrono's `%-d %b`, cli.rs `namesakes_said`), never the browser's
// locale, which differs from it (#157).
const day = (t, d = new Date(t * 1000)) => `${d.getDate()} ${'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ')[d.getMonth()]}`;

// The person whose first key is `key`, as the people rows `rows`
// (/v1/rail/people) name them (capability-keys; svrn-docs
// WHAT_CAN_ACT_AS_YOU §9): their name, and where someone else in `rows`
// shares it, when they joined, `when` saying it, "ada (joined 3 Oct)", as
// the command tells them apart (cli.rs `joined_said`); undefined for a key
// no row holds. Who did something is told by key, never by name.
export const called = (rows, key, when = (j) => (j === 'started' ? 'started the group' : `joined ${day(j.at)}`)) => {
  const p = rows.find((r) => r.person_key === key);
  return p && (rows.some((r) => r.person === p.person && r.person_key !== key) ? `${p.person} (${when(p.joined)})` : p.person);
};

// Each of `lent` given `called`, its lender as `called` says them by the
// key of the computer lending it, so `named` and `owner` tell two lenders
// of one name apart.
export const apart = (lent, rows) => lent.forEach((l) => (l.called = called(rows, rows.find((r) => r.key === l.key)?.person_key)));

// Who made a group and since when this computer or phone has held it, so
// two of one name are told apart (#138): "sam’s, since 14 Oct".
export const whose = (g) => `${g.maker ? `${g.maker}’s` : 'yours'}${g.since ? `, since ${day(g.since)}` : ''}`;

// The groups a new one shares its name with (/group, the node's one rule;
// #138), each said as `whose`, in the command's words (cli.rs
// `namesakes_said`): `verb` says it before the group is made or after.
export const namesakes = (n, name, verb) => (n.length ? `You’re already in ${n.map((g) => `${g.name} (${whose(g)})`).join(' and ')}. ${verb} ${n.length > 1 ? 'another' : 'a second'} group called ${name}.` : '');

// Each person once, of the rows /v1/rail/people gives a key each; two of
// one name told apart by when each joined (bash-same-name), in the form
// `svrngs remove --joined` takes (home.rs `WHEN`): this computer's time as
// the `sv-SE` locale writes it, then its offset as ±hhmm, so a fall-back
// night's repeated hour reads back to one instant (#126).
// Each with what holds their keys, `device` (v1ui-door (c)). In the order
// they joined, whoever started the group first, so a newcomer's window comes
// last and no window moves as people arrive (the screen, home.js `screen`;
// the Draft 2 canvas's frames 8-10): one order for every door. `started`
// is a string, whose `.at` is a method, so it is asked by name.
function people(rows) {
  const since = (p) => (p.joined === 'started' ? 0 : p.joined.at);
  const each = [...new Map(rows.map((r) => [r.person_key, r])).values()].sort((a, b) => since(a) - since(b));
  const at = (d, m = -d.getTimezoneOffset()) => `${d.toLocaleString('sv-SE')} ${m < 0 ? '-' : '+'}${String(Math.trunc(Math.abs(m) / 60) * 100 + (Math.abs(m) % 60)).padStart(4, '0')}`;
  const when = (j) => (j === 'started' ? 'started the group' : `joined ${at(new Date(j.at * 1000))}`);
  return each.map((p) => [p, called(rows, p.person_key, when), rows.filter((r) => r.person_key === p.person_key).map((r) => r.device)]);
}

// The door (V1_UI_BUILD §3.1): each person once, a cell in a grid, unjudged
// until the group's computers have answered or not, within one round
// (/v1/lends, `judge`): then lit, "on now", while their computer answers;
// drawn off when it did not, never lit and never left out; and one in only
// from a phone, which nobody asks, marked so with no presence claimed.
// Presence is only "on now", never when last seen. Given `lent`, as the
// screen gives it (home.js `screen`), each person's own lends in their
// window, a link each by its name, opened in a tab as this computer's
// person; an app the group carries is the group's, in nobody's window.
export function door(rows, lent = []) {
  const theirs = (p) => lent.filter((l) => !l.group && rows.find((r) => r.key === l.key)?.person_key === p.person_key).flatMap((l) => [' ', Object.assign(el('a', l.name), { href: l.origin, target: '_blank' })]);
  const cells = people(rows).map(([p, name, devices]) => [p, devices, el('li', '', el('strong', name), ...theirs(p), ' ', el('small', 'asking…'))]);
  const judge = (lends) => cells.forEach(([p, devices, li]) => {
    const phone = devices.includes('browser') ? (devices.includes('computer') ? ', a phone too' : 'in from a phone') : '';
    const silent = lends?.unanswered.some((u) => (u.person_key ?? u) === (u.person_key ? p.person_key : p.person));
    const [drawn, words] = !devices.includes('computer') ? ['phone', ''] : !lends ? ['', 'not known now'] : silent ? ['off', 'did not answer'] : ['on', 'on now'];
    [li.className, li.lastChild.textContent] = [drawn, words + phone];
  });
  return [Object.assign(el('ul', '', ...cells.map((c) => c[2])), { className: 'door' }), judge];
}

// "Bring things to the table" (V1_UI_BUILD §3.4; v1ui-shelf (c)): the
// shelf, svrngs's own apps as `lends.shelf` lists them, each not on group
// `g`'s table yet ticked, and one button bringing the ticked ones, by
// `bring(names)`, which answers each `{app, already}`; then what happened
// said. One already on the table is said so. A group whose name cannot be
// read is "your group".
export function table(g, lends, bring) {
  const name = g.name ?? 'your group';
  const on = (a) => lends.lent.some((l) => l.group === g.id && l.kind === a.name);
  const ticks = lends.shelf.filter((a) => !on(a)).map((a) => [a.name, Object.assign(el('input'), { type: 'checkbox', checked: true })]);
  const said = el('p', ticks.length ? `These come with svrngs. Bring one, and everyone in ${name} opens it on their own computer or phone, while any computer in ${name} is on.` : '');
  const s = el('section', '', el('h3', 'Bring things to the table'), ...lends.shelf.filter(on).map((a) => el('p', `${spoken(a.name)} is on the table.`)), said);
  for (const [name, tick] of ticks) s.append(el('p', '', el('label', '', tick, ` ${spoken(name)}: ${lends.shelf.find((a) => a.name === name).words}`)));
  const done = (r) => (r.already ? `${spoken(r.app)} is on the table already.` : `You brought ${spoken(r.app)} to ${name}: everyone in it can open it in a few seconds.`);
  const go = () => bring(ticks.filter(([, t]) => t.checked).map(([n]) => n)).then((b) => said.replaceChildren(b.map(done).join(' ') || 'Tick something to bring first.'), (e) => said.replaceChildren(`Nothing was brought: ${e.message}`));
  if (ticks.length) s.append(el('p', '', Object.assign(el('button', `Bring the ticked ones to ${name}`), { onclick: go })));
  return s;
}

// Each app on group `g`'s table whose files someone changed since the
// version that runs as you (`lends.changed`; canon-pin (c); svrn-docs
// WHAT_RUNS_AS_YOU §3), never in the way: one line, "2 apps changed. See
// what’s new", opening to who and when for each, with a button saying yes
// to it by `yes(app)`. Until you do, the version you said yes to, or the
// one brought, runs as you, on your computers and phones alike. Who
// changed it is told by key, of the people rows `rows`, this device's
// being `me`: an older computer's answer, with none, by name.
export function changes(g, lends, yes, rows, me) {
  if (lends.changed === null) return [el('p', `This computer can’t say whether anyone changed an app in ${g.name ?? 'your group'}.`)];
  const you = (c) => (c.by_key ? c.by_key === rows.find((r) => r.key === me)?.person_key : c.by === g.person);
  const each = (lends.changed ?? []).filter((c) => c.group === g.id).map((c) => {
    const p = el('p', `${spoken(c.app)}: ${you(c) ? 'you' : (called(rows, c.by_key) ?? c.by)} changed its files on ${day(c.at)}. `);
    p.append(Object.assign(el('button', 'Open the new version'), { onclick: () => yes(c.app).then(() => p.replaceChildren(`${spoken(c.app)} opens at its new version for you now.`), (e) => p.append(` ${e.message}`)) }));
    return p;
  });
  return each.length ? [el('details', '', el('summary', `${each.length} ${each.length === 1 ? 'app' : 'apps'} changed. See what’s new`), el('p', 'The version you have keeps running until you open the new one.'), ...each)] : [];
}

// "What acts as you" (capability-what-acts; svrn-docs WHAT_CAN_ACT_AS_YOU
// §9): one section on both pages, a part for each thing an app or a
// device can do as you, in this order, each headed only once `add(part,
// ...lines)` gives it a line, then `notes[part]`, a page's own words.
const PARTS = { reads: 'Reads and writes for you', opens: 'Opens friends’ things as you', knows: 'Knows who you are', wifi: 'Open to this WiFi', play: 'Open to programs on this computer', decides: 'Can decide for you' };
export function acts(notes) {
  const parts = Object.fromEntries(Object.keys(PARTS).map((k) => [k, el('div')]));
  const add = (k, ...lines) => (parts[k].children.length || parts[k].append(el('h3', PARTS[k]), ...(notes[k] ? [el('p', notes[k])] : [])), parts[k].append(...lines));
  return [Object.assign(el('section', '', el('h2', 'What acts as you'), ...Object.values(parts)), { id: 'acts' }), add];
}

// An app the group carries, as "What acts as you" says it: what it reads
// and writes, the shelf's words, and no Stop, as it runs only when opened.
export const carrying = (kind, shelf) => `${spoken(kind)} reads and writes ${does(kind, shelf).replace(/\.$/, '')}, when you open it.`;

// What the person said, `c`, of lend `l` opening their friends' lends of
// the kinds it reads as them (capability-open), since or on its day; `l`
// undefined when it is not lent now.
export const reading = (l, c) => {
  const [who, what] = l ? [named(l), `your friends’ ${l.reads.map(spoken).join(' and ')}`] : ['An app not lent now', 'your friends’ things'];
  return c.said === 'yes' ? `${who} opens ${what} as you. Since ${day(c.at)}. ` : `${who} stopped opening ${what} as you on ${day(c.at)}. Opening it again says yes again.`;
};

// Who can decide for you (§4): the devices holding a key of the person
// whose device key is `me`, of the people rows, this one, a `here`, first
// and nobody else's, a namesake's included, by key, never by name.
export function deciders(rows, me, here) {
  const first = rows.find((r) => r.key === me)?.person_key;
  const own = (d) => rows.filter((r) => r.person_key === first && r.key !== me && r.device === d).length;
  const said = (n, d, one) => (n ? [`your ${here === d ? 'other ' : ''}${n > 1 ? `${n} ${one}s` : one}`] : []);
  const all = [`This ${here === 'computer' ? 'computer' : 'phone'}`, ...said(own('computer'), 'computer', 'computer'), ...said(own('browser'), 'browser', 'phone')];
  return all.join(', ').replace(/, ([^,]*)$/, ', and $1');
}

// How lend `l` opens, `where`, and whether it is full now: the index's row
// as both pages judge it, so a change to either lands on both (#158).
export const opens = (l, where) => [where, l.limit && l.using >= l.limit ? 'full' : 'on'];

// A row of the index; `tag` th for its head.
export const row = (cells, tag = 'td') => el('tr', '', ...cells.map((c) => el(tag, '', c)));
