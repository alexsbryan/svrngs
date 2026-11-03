// SPDX-License-Identifier: AGPL-3.0-or-later
// What apps use, on the phone (page-acts (b); web/app.js `uses` and
// `choose`, :286-296, :327-344): `details#uses` over `ul#used`, folded
// (ui/parts.js `Fold`, `fold uses`), drawn under "Opens friends’ things as
// you" (web/page/acts.js) once a lend asks, else hidden in its place, as
// web/index.html@19b63b2b:97 has it. Opened, it asks each lender that asks for its
// choices, each read counted, so an older answer is refused by name, and
// `asking` draws #used empty, so every line is an answer drawn after it
// was opened (web/tests/tabs.mjs). Each yes has Stop, the person's stop
// posted to its lender's door, and then it asks again; a refusal is said
// after the button. Lines by ui/uses.js `Uses`, as `li`. The dial is the
// one endpoint's (web/page/enter.js `dialled`), at the edge.
import { ask, Fold, go, html, late, no, Uses } from '../ui/kit.js?v=__VERSION__';
import { dialled } from './enter.js?v=__VERSION__';

export const owns = ['using', 'choices', 'unstopped'];
export const states = { uses: ['shut', 'asking', 'asked'] };
const asking = (s, patch = {}, [asked, n] = ask(s, 'uses')) =>
  go(
    s,
    { uses: 'asking' },
    { ...patch, asked, choices: {} },
    ...s.lent
      .filter((l) => l.asks)
      .map((l) => ({ run: 'choices', group: s.group, l, n, failed: 'chosen' })),
  );
export const takes = {
  'fold uses': (s, m) =>
    m.open ? asking(s, { using: true }) : go(s, { uses: 'shut' }, { using: false }),
  chosen: (
    s,
    m,
    choices = { ...s.choices, [m.lend]: m.choices ? { choices: m.choices } : { why: m.why } },
  ) =>
    late(s, 'uses', m.n)
      ? no(s, late(s, 'uses', m.n))
      : go(
          s,
          { uses: s.lent.every((l) => !l.asks || choices[l.label]) ? 'asked' : 'asking' },
          { choices },
        ),
  stop: (s, { lend, app }, l = s.lent?.find((o) => o.label === lend)) =>
    !l
      ? no(s, `nothing ${s.group?.name ?? 'here'} lends is ${lend}`)
      : go(s, {}, {}, { run: 'unuse', group: s.group, l, app, failed: 'unstopped' }),
  unused: (s) => asking(s),
  unstopped: (s, m) =>
    go(
      s,
      {},
      { unstopped: { ...s.unstopped, [m.of]: `${s.unstopped?.[m.of] ?? ''}${m.said ?? m.why}` } },
    ),
};

// The Fold, for web/page/acts.js's "Opens friends’ things as you".
export const Used = ({ s, send }) =>
  html`<${Fold} id="uses" summary="What apps use" fold="uses" open=${s.using} send=${send}><ul id="used">${Uses({ lent: s.lent, labelOf: (l) => l.label, choices: s.choices ?? {}, tag: 'li', said: s.unstopped, send })}</ul><//>`;
export const view = {
  uses: ({ s }) =>
    !s.lent?.some((l) => l.asks) &&
    html`<details id="uses" hidden><summary>What apps use</summary><ul id="used"></ul></details>`,
};

// The edge: a lender's door dialled as the person, its words when it does
// not answer or refuses (app.js@19b63b2b:288-296), read whole before the dial closes, as
// a body after its head is cut off by the close (bash-film-load).
const trace = (l) => globalThis.__trace?.(l),
  saying = (said) => Object.assign(new Error(said), { said });
const door = (group, l, method, body, away = '') =>
  dialled(group, l)
    .catch((e) =>
      Promise.reject(
        (trace(`PAGE: the dial failed: ${e}`),
        saying(`${l.person}’s computer did not answer.${away}`)),
      ),
    )
    .then(async (d) => {
      const bytes = body && new TextEncoder().encode(JSON.stringify(body)),
        head = body
          ? [
              ['content-type', 'application/json'],
              ['content-length', String(bytes.length)],
            ]
          : [['accept', 'application/json']];
      const r = await d
        .fetch(method, `/choices/${l.label}`, head, body ? new Blob([bytes]).stream() : null)
        .then(async (res) => [res.ok, res.status, await res.text()])
        .finally(() => d.close());
      return r[0]
        ? JSON.parse(r[2] || 'null')
        : Promise.reject(
            (trace(`PAGE: /choices/${l.label} answered ${r[1]}: ${r[2]}`),
            saying(body ? `${l.person}’s computer did not keep your answer.` : `answered ${r[1]}`)),
          );
    });
// The person's answer `said` for the app labelled `app` on `l`, for the ask too.
export const choose = (group, l, app, said) =>
  door(group, l, 'POST', { app, said }, ' Try again later.');
export const io = {
  choices: ({ group, l, n }) =>
    door(group, l, 'GET').then(
      (a) => ({
        type: 'chosen',
        n,
        lend: l.label,
        choices: a.choices,
        why: `${l.name}: ${Object.keys(a.choices).length} app(s)`,
      }),
      (e) => ({ type: 'chosen', n, lend: l.label, why: e.message }),
    ),
  unuse: ({ group, l, app }) =>
    choose(group, l, app, 'stop').then(
      () => ({ type: 'unused', why: `${app} stopped using ${l.name}` }),
      (e) => ({ type: 'unstopped', of: `${l.label} ${app}`, said: e.message, why: e.message }),
    ),
};
