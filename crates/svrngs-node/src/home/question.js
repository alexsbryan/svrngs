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

// svrngs's question: may the app lent as `app` use the lend `lend`?
export const question = (app, lend) => `Let ${named(app)} use ${named(lend)}?`;

// A removal or an ask to remove, as everyone in the group is shown it
// (v1-keeper-office (b)): "cy asked to remove dee on 12 Oct 2026, 14:03",
// the time in the browser's own words.
export const removal = (r) =>
  `${r.by} ${r.said === 'removed' ? 'removed' : 'asked to remove'} ${r.of} on ${new Date(r.at * 1000).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' })}`;

// The line said when what apps use of `lend` cannot be asked.
export const unasked = (lend) => `What apps use of ${named(lend)} cannot be asked now.`;

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

// A day in the browser's own words, from seconds.
const day = (t) => new Date(t * 1000).toLocaleDateString(undefined, { day: 'numeric', month: 'short' });

// Who made a group and since when this computer or phone has held it, so
// two of one name are told apart (#138): "sam’s, since Oct 14".
export const whose = (g) => `${g.maker ? `${g.maker}’s` : 'yours'}${g.since ? `, since ${day(g.since)}` : ''}`;

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

// A row of the index; `tag` th for its head.
export const row = (cells, tag = 'td') => el('tr', '', ...cells.map((c) => el(tag, '', c)));
