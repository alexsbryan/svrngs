// SPDX-License-Identifier: AGPL-3.0-or-later
// svrngs's question, what apps use and who was removed from the group, in
// one wording for the two pages
// that ask a person: the home page on a computer (home.js, served by
// home.rs) and the group's page on a phone (web/app.js, which
// scripts/build-web.sh gives this file). Its words are DESIGN §1's, a lend
// named as the bare command names it (svrngs cli.rs `lends`); each page
// keeps only how it posts a yes or a stop (REVIEW_FINDINGS #110).

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

// What apps use of `lend`, from its lender's `choices`: a line each, made
// by `line(words)`, the app named by `appOf(label)` when it is listed, each
// yes with Stop, which calls `stop(app, line)`.
export function uses(lend, choices, appOf, line, stop) {
  for (const [app, c] of Object.entries(choices)) {
    const by = appOf(app);
    const item = line(`${by ? named(by) : 'An app'} ${c.said === 'yes' ? 'uses' : 'was stopped using'} ${named(lend)}. `);
    if (c.said !== 'yes') continue;
    const button = Object.assign(document.createElement('button'), { textContent: 'Stop' });
    button.onclick = () => stop(app, item);
    item.append(button);
  }
}
