// SPDX-License-Identifier: AGPL-3.0-or-later
// The door, both pages' (page-members (a); question.js `people` and `door`,
// :151-188, which it moves; V1_UI_BUILD §3.1): each person once, a cell in
// a grid, in the order they joined, whoever started the group first, so no
// window moves as people arrive; two of one name told apart by when each
// joined, in the form `svrngs remove --joined` takes (home.rs `WHEN`),
// this computer's time then its offset as ±hhmm (#126). Unjudged, "asking…",
// while `lends` is not yet answered (undefined); "not known now" when it
// could not be read (null); then lit, "on now", while their computer
// answers, drawn off when it did not, and one in only from a phone marked
// so, with no presence claimed. Given `lent`, as the screen gives it, each
// person's own lends in their window, a link each, opened in a tab, the
// screen's `newest` marked new (home.js@19b63b2b:322); an app the group carries is
// the group's, in nobody's window. `counted` is what the screen counts of
// it (home.js@19b63b2b:324): its people, those on now, and the things in their windows.
import { html } from './view.js?v=__VERSION__';
import { called } from '../question.js?v=__VERSION__';

const since = (p) => (p.joined === 'started' ? 0 : p.joined.at);
const at = (d, m = -d.getTimezoneOffset()) =>
  `${d.toLocaleString('sv-SE')} ${m < 0 ? '-' : '+'}${String(Math.trunc(Math.abs(m) / 60) * 100 + (Math.abs(m) % 60)).padStart(4, '0')}`;
const when = (j) => (j === 'started' ? 'started the group' : `joined ${at(new Date(j.at * 1000))}`);
// Each person once, with their name as the door says it and what holds their keys.
const people = (rows) =>
  [...new Map(rows.map((r) => [r.person_key, r])).values()]
    .sort((a, b) => since(a) - since(b))
    .map((p) => [
      p,
      called(rows, p.person_key, when),
      rows.filter((r) => r.person_key === p.person_key).map((r) => r.device),
    ]);

// Its class and its words, judged by `lends` (/v1/lends' `unanswered`).
function judged(p, devices, lends) {
  if (lends === undefined) return ['', 'asking…'];
  const phone = devices.includes('browser')
    ? devices.includes('computer')
      ? ', a phone too'
      : 'in from a phone'
    : '';
  const silent = lends?.unanswered.some(
    (u) => (u.person_key ?? u) === (u.person_key ? p.person_key : p.person),
  );
  const [drawn, words] = !devices.includes('computer')
    ? ['phone', '']
    : !lends
      ? ['', 'not known now']
      : silent
        ? ['off', 'did not answer']
        : ['on', 'on now'];
  return [drawn, words + phone];
}

// A person's window: their name, their own lends, and how they are judged.
const theirs = (rows, lent, p, newest) =>
  lent
    .filter((l) => !l.group && rows.find((r) => r.key === l.key)?.person_key === p.person_key)
    .map(
      (l) =>
        html`${' '}<a href=${l.origin} target="_blank">${l.name}</a>${l.origin === newest && html`${' '}<mark>new</mark>`}`,
    );
const cell =
  (rows, lent, lends, newest) =>
  ([p, name, devices], [drawn, words] = judged(p, devices, lends)) =>
    html`<li class=${drawn || undefined}><strong>${name}</strong>${theirs(rows, lent, p, newest)}${' '}<small>${words}</small></li>`;
export const Door = ({ rows, lent = [], lends, newest }) =>
  html`<ul class="door">${people(rows).map((p) => cell(rows, lent, lends, newest)(p))}</ul>`;
export const counted = (rows, lent, lends, ps = people(rows)) => [
  ps.length,
  ps.filter(([p, , d]) => judged(p, d, lends)[0] === 'on').length,
  ps.flatMap(([p]) => theirs(rows, lent, p)).length,
];
