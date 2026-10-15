// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's question, what apps use, who was removed from the group, who is
// on now and what is in it (the door and the index's rows, v1ui-door), in
// one wording for the two pages
// that ask a person: the home page on a computer (home.js, served by
// home.rs) and the group's page on a phone (web/app.js, which
// scripts/build-web.sh gives this file). Its words are DESIGN §1's, a lend
// named as the bare command names it (svrngs cli.rs `lends`); each page
// keeps only how it asks a lend's choices and posts a yes or a stop
// (REVIEW_FINDINGS #110, #113).

// A lend as a person reads it: "Ana’s taste", and "Sam’s tonights-picks
// (movie-night)" when its name and kind differ.
export const named = (l) => `${l.person}’s ${l.name === l.kind ? l.name : `${l.name} (${l.kind})`}`;

// One copy of each app a group carries, the first listed, and every other
// lend as it is: each computer holding the app lends its copy
// (COMMUNITY_APPS §7, found 4; v1ui-shelf (a)).
export const once = (lent) => lent.filter((l, i) => !l.group || lent.findIndex((o) => o.group === l.group && o.kind === l.kind) === i);

// svrngs's question: may the app lent as `app` use the lend `lend`?
export const question = (app, lend) => `Let ${named(app)} use ${named(lend)}?`;

// A removal or an ask to remove, as everyone in the group is shown it
// (v1-keeper-office (b)): "cy asked to remove dee on 12 Oct 2026, 14:03",
// the time in the browser's own words.
export const removal = (r) =>
  `${r.by} ${r.said === 'removed' ? 'removed' : 'asked to remove'} ${r.of} on ${new Date(r.at * 1000).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' })}`;

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
// Each with what holds their keys, `device` (v1ui-door (c)).
function people(rows) {
  const each = [...new Map(rows.map((r) => [r.person_key, r])).values()];
  const at = (d, m = -d.getTimezoneOffset()) => `${d.toLocaleString('sv-SE')} ${m < 0 ? '-' : '+'}${String(Math.trunc(Math.abs(m) / 60) * 100 + (Math.abs(m) % 60)).padStart(4, '0')}`;
  const when = (j) => (j === 'started' ? 'started the group' : `joined ${at(new Date(j.at * 1000))}`);
  return each.map((p) => [p, each.filter((q) => q.person === p.person).length > 1 ? `${p.person} (${when(p.joined)})` : p.person, rows.filter((r) => r.person_key === p.person_key).map((r) => r.device)]);
}

// The door (V1_UI_BUILD §3.1): each person once, a cell in a grid, unjudged
// until the group's computers have answered or not, within one round
// (/v1/lends, `judge`): then lit, "on now", while their computer answers;
// drawn off when it did not, never lit and never left out; and one in only
// from a phone, which nobody asks, marked so with no presence claimed.
// Presence is only "on now", never when last seen.
export function door(rows) {
  const cells = people(rows).map(([p, name, devices]) => [p, devices, el('li', '', el('strong', name), ' ', el('small', 'asking…'))]);
  const judge = (lends) => cells.forEach(([p, devices, li]) => {
    const phone = devices.includes('browser') ? (devices.includes('computer') ? ', a phone too' : 'in from a phone') : '';
    const [drawn, words] = !devices.includes('computer') ? ['phone', ''] : !lends ? ['', 'not known now'] : lends.unanswered.includes(p.person) ? ['off', 'did not answer'] : ['on', 'on now'];
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
  const s = el('section', '', el('h3', 'Bring things to the table'), ...lends.shelf.filter(on).map((a) => el('p', `${a.name} is on the table.`)), said);
  for (const [name, tick] of ticks) s.append(el('p', '', el('label', '', tick, ` ${name}: ${lends.shelf.find((a) => a.name === name).words}`)));
  const done = (r) => (r.already ? `${r.app} is on the table already.` : `You brought ${r.app} to ${name}: everyone in it can open it in a few seconds.`);
  const go = () => bring(ticks.filter(([, t]) => t.checked).map(([n]) => n)).then((b) => said.replaceChildren(b.map(done).join(' ') || 'Tick something to bring first.'), (e) => said.replaceChildren(`Nothing was brought: ${e.message}`));
  if (ticks.length) s.append(el('p', '', Object.assign(el('button', `Bring the ticked ones to ${name}`), { onclick: go })));
  return s;
}

// How lend `l` opens, `where`, and whether it is full now: the index's row
// as both pages judge it, so a change to either lands on both (#158). One
// whose computer does not say whether it signs you in, a computer before
// v1ui-door, is said so, never read as no (#159): `older`, whose computer
// that is, its lender's unless the page knows it is another's.
export const opens = (l, where, older = `${l.person}’s computer`) => [`${where}${l.signs_in ? ', signing you in once' : l.signs_in === false ? '' : `. ${older} runs an older svrngs, so this may ask you to sign in`}`, l.limit && l.using >= l.limit ? 'full' : 'on'];

// A row of the index; `tag` th for its head.
export const row = (cells, tag = 'td') => el('tr', '', ...cells.map((c) => el(tag, '', c)));
