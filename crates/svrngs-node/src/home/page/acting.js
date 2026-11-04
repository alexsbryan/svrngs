// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the home page (acts-read.js ACTS), drawn by
// ui/acts.js `Acts`: group by group, the apps it carries and their new
// versions (ui/changes.js), and who decides for you in it, of its people
// by this computer's key; then what opens friends' things as you, each
// with its Stop, and what apps use of each asking lend (ui/uses.js); what
// you open knows; and WiFi and Play. Each group's GROUP read is told to
// HOME by its section (group-told.js `GroupTold`), and the section is drawn once
// every group has been told and /reaching read, and kept after. The
// machine is the section's own (ui/view.js `useMachine`).
import { html, useEffect, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Acts } from '../ui/acts.js?v=__VERSION__';
import { Changes } from '../ui/changes.js?v=__VERSION__';
import { Uses } from '../ui/uses.js?v=__VERSION__';
import { called, carrying, deciders, label, once, reading } from '../question.js?v=__VERSION__';
import { ACTS } from './acts-read.js?v=__VERSION__';
import { Reach } from './reach.js?v=__VERSION__';
import * as wifi from './wifi.js?v=__VERSION__';
import * as play from './play.js?v=__VERSION__';
/** @import { Group, Lends } from './group.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import { Asked } from '../ui/uses.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Acted, Choice, Message, State, Yes } from './acts-read.js?v=__VERSION__' */
/** @import { Told } from './acts.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

const NOTES = {
  opens: 'Each Stop holds from the app’s next request.',
  play: 'Any program on this computer reaches these as you while Play is on.',
};

/**
 * What a group's Changes draws of each yes kept for it, by its app: the
 * yes under way said busy, the rest as answered.
 * @param {string} id
 * @param {State} state
 */
const doingIn = (id, state) => {
  const its = Object.entries(state.acted).filter(([key]) => key.startsWith(`${id} `));
  return Object.fromEntries(
    its.map(([key, act]) => {
      const busy = state.acting === key && state.at.doing === 'busy';
      return [key.slice(id.length + 1), busy ? { state: 'busy' } : act];
    }),
  );
};

/**
 * A group's new versions, and each one a yes has done, which the record no
 * longer lists, so it stays said; none when the record lists none.
 * @param {string} id
 * @param {Lends} lends
 * @param {Record<string, Acted>} acted
 */
const changedIn = (id, lends, acted) => {
  const listed = lends.changed;
  if (!listed) return listed;
  /** @type {Change[]} */
  const done = [];
  for (const act of Object.values(acted)) {
    const change = act.c;
    if (!change || change.group !== id || act.state !== 'done') continue;
    if (!listed.some((each) => each.app === change.app)) done.push(change);
  }
  return [...listed, ...done];
};

/** @param {HOME.PlaceProps} props */
export function Acting({ s: homeState, home }) {
  const [state, send] = useMachine(() =>
    machine('ACTS', ACTS(home), {
      at: { reaching: 'reading', doing: 'idle' },
      acted: {},
      said: {},
      folds: homeState.q.review ? { acts: true, changes: true } : {},
    }),
  );
  // HOME draws the section once the groups are read (home-page.js `Home`).
  const groups = /** @type {Group[]} */ (homeState.groups).filter(
    (group) => !group.left && group.removed === false,
  );
  /** @param {Group} group */
  const told = (group) => homeState.told?.[group.id];
  /** @param {Group} group */
  const rows = (group) => told(group)?.people ?? [];
  const lent = groups.flatMap((group) => {
    const people = rows(group);
    return (told(group)?.lends?.lent ?? []).map((lend) => {
      const lender = people.find((row) => row.key === lend.key)?.person_key;
      return { ...lend, called: called(people, lender) };
    });
  });
  const asking = lent.filter((lend) => lend.asks).map(label);
  const ready = groups.every(told);
  useEffect(() => {
    if (ready) send({ type: 'read', asking });
  }, [ready, homeState.round, asking.join()]);
  useEffect(() => {
    if (!ready || state.at.reaching === 'reading' || homeState.actsRead) return;
    home({ type: 'told', flow: 'acts', read: true, why: 'what acts as you is read' });
  }, [ready, state.at.reaching]);
  // Drawn once read; a group new in a round (one made or joined) is said
  // read until it is told, the rest kept, so the section never drops out
  // and comes back after each write.
  if (state.at.reaching === 'reading') return null;
  // Each group `shown` is told; the casts of `told(group)` below say so,
  // and that a part drawn for its lends or its people has them, as it
  // filtered by them.
  const shown = groups.filter(told);
  /** @param {Group} group */
  const lends = (group) => /** @type {{ lends: Lends }} */ (told(group)).lends;
  /**
   * Changes' messages, a yes given its group and its change.
   * @param {Group} group
   */
  const yes = (group) => /** @param {Omit<Yes, 'g' | 'c'> | Message} message */ (message) => {
    if (message.type !== 'yes') return send(message);
    const change = lends(group).changed?.find(
      (each) => each.group === group.id && each.app === message.app,
    );
    return send({ ...message, g: group.id, c: change });
  };
  /** @param {Group} group */
  const readsIn = (group) => {
    const shared = once(lends(group).lent).filter((lend) => lend.group);
    const carried = shared.map((lend) => html`<p>${carrying(lend.kind, lends(group).shelf)}</p>`);
    const changes = Changes({
      g: group,
      changed: changedIn(group.id, lends(group), state.acted),
      rows: rows(group),
      me: homeState.key,
      doing: doingIn(group.id, state),
      open: state.folds?.changes,
      send: yes(group),
    });
    const heading = homeState.q.ui === 'b' && html`<h4>${group.name ?? 'Your group'}</h4>`;
    return [heading, ...carried, ...changes];
  };
  const reads = shown.filter((group) => /** @type {Told} */ (told(group)).lends).flatMap(readsIn);
  // The yeses are read, or why not, once ACTS is past 'reading'.
  const unread = /** @type {{ why?: string }} */ (state.yeses).why;
  const yeses = /** @type {Record<string, Choice>} */ (state.yeses);
  const reached =
    unread !== undefined
      ? [
          html`<p>This computer cannot say which apps open your friends’ things as you: ${unread}</p>`,
        ]
      : Object.entries(yeses).map(([app, choice]) => {
          const lend = /** @type {(Lend & { reads: string[] }) | undefined} */ (
            lent.find((one) => label(one) === app && one.reads && one.reads.length > 0)
          );
          const stop =
            choice.said === 'yes' && html`<${Reach} app=${app} words="Stop" said="stop" />`;
          return html`<p>${reading(lend, choice)}${stop}</p>`;
        });
  // Read past `reading`, the choices are set: the read's, or none.
  const uses = Uses({
    lent,
    labelOf: label,
    choices: /** @type {Record<string, Asked>} */ (state.choices),
    tag: 'p',
    said: state.said,
    send,
  });
  const decides = shown
    .filter((group) => /** @type {Told} */ (told(group)).people)
    .map((group) => {
      const who = deciders(rows(group), /** @type {string} */ (homeState.key), 'computer');
      return html`<p>${`${group.name ?? 'Your group'}: ${who}.`}</p>`;
    });
  // WiFi's and Play's parts read their stores as hooks of this section's
  // (wifi.js and play.js `part`), so they are called here, in this order,
  // on each draw past `reading`.
  const parts = {
    reads,
    opens: [...reached, ...uses],
    knows: [html`<p>What you open knows your name.</p>`],
    wifi: wifi.part(),
    play: play.part(),
    decides,
  };
  const rereading = !ready && html`<p class="said">Reading your groups…</p>`;
  return html`
    <${Acts} parts=${parts} notes=${NOTES} open=${state.folds?.acts} send=${send} />
    ${rereading}
  `;
}
