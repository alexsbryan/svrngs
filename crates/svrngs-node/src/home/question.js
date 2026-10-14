// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's question, what apps use and who was removed from the group, in
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
