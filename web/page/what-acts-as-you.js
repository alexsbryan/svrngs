// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you" on the phone, once the tab holds a group: ui/acts.js
// `Acts` in #acts, each part derived from state. The apps the group
// carries and their new versions (ui/changes.js); what opens friends'
// things as you (opens-as-you.js), then what apps use (what-apps-use.js);
// what you open knowing your name; and who decides for you, of the people
// by this phone's key, `me`.
//
// Lines a carry left unsent are carried again on each read that holds
// them: refused, a carry asks no read again (acts.js `answered`), so it is
// tried once a read.
import { html, useEffect } from '../ui/view.js?v=__VERSION__';
import { Acts } from '../ui/acts.js?v=__VERSION__';
import { Changes } from '../ui/changes.js?v=__VERSION__';
import { carrying, deciders, once } from '../question.js?v=__VERSION__';
import { PASS_LIFE } from './hand.js?v=__VERSION__';
import { OpensAsYou } from './opens-as-you.js?v=__VERSION__';
import { WhatAppsUse } from './what-apps-use.js?v=__VERSION__';
/** @import { Said } from '../store.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Lent } from './enter.js?v=__VERSION__' */
/** @import { Acted } from './acts.js?v=__VERSION__' */
/** @import { Props, State } from './page.js?v=__VERSION__' */

const NOTES = {
  opens: `A Stop here holds from the next pass this phone signs for the app, within ${PASS_LIFE / 3600} hours.`,
};

/**
 * The answer of the act keyed `key`: busy while it is under way, else as
 * kept.
 * @param {State} state
 * @param {string} key
 * @returns {Acted}
 */
const doing = (state, key) => {
  if (state.acting === key && state.at.doing === 'busy') return { state: 'busy' };
  return state.acted?.[key] ?? {};
};

/**
 * Each change a yes took, done, that the group, read again, lists no
 * more, so "opens at its new version for you now" stays said.
 * @param {State} state
 * @returns {Change[]}
 */
const doneChanges = (state) => {
  const acts = Object.values(state.acted ?? {});
  const taken = acts.flatMap((act) => (act.c && act.state === 'done' ? [act.c] : []));
  return taken.filter(
    (change) => !state.changed?.some((one) => one.app === change.app && one.group === change.group),
  );
};

/**
 * The parts `Acts` draws, by part.
 * @param {State} state once the group and its lends are read
 * @param {Props['send']} send
 */
const partsOf = (state, send) => {
  const lent = /** @type {Lent[]} */ (state.lent);
  const carried = once(lent)
    .filter((lend) => lend.group)
    .map((lend) => html`<p>${carrying(lend, state.shelf)}</p>`);
  const yeses = Object.keys(state.acted ?? {}).filter((key) => key.startsWith('yes '));
  const changes = Changes({
    g: state.group,
    changed: state.changed && [...state.changed, ...doneChanges(state)],
    rows: state.people,
    me: state.me,
    doing: Object.fromEntries(yeses.map((key) => [key.slice(4), doing(state, key)])),
    open: state.folds?.changes,
    send,
  });
  const reading = lent.filter(
    (lend) => lend.reads && lend.reads.length > 0 && state.group.reads?.[lend.label] !== undefined,
  );
  const opens = reading.map((lend) => {
    // Filtered to the lends that read, of which the group keeps a yes or a stop.
    const reads = /** @type {Lent & { reads: string[] }} */ (lend);
    const said = /** @type {Said} */ (state.group.reads?.[lend.label]);
    const act = doing(state, `stop ${lend.label}`);
    return html`<${OpensAsYou} lend=${reads} said=${said} act=${act} send=${send} />`;
  });
  const asks = lent.some((lend) => lend.asks);
  const used = asks ? [html`<${WhatAppsUse} s=${state} send=${send} />`] : [];
  return {
    reads: [...carried, ...changes],
    opens: [...opens, ...used],
    knows: [html`<p>What you open knows your name.</p>`],
    decides: [html`<p>${deciders(state.people, state.me, 'browser')}.</p>`],
  };
};

/**
 * Drawn once the tab holds a group (acts.js `view`).
 * @typedef {object} WhatActsAsYouProps
 * @property {State} s
 * @property {Props['send']} send
 */

/** @param {WhatActsAsYouProps} props */
export function WhatActsAsYou({ s: state, send }) {
  useEffect(() => {
    const unsent = state.group.unsent?.length ?? 0;
    if (state.at.list === 'read' && unsent > 0) {
      send({ type: 'carry', why: `${unsent} line(s) unsent` });
    }
  }, [state.lent]);
  if (!state.lent) return html`<section id="acts"></section>`;
  const parts = partsOf(state, send);
  return html`<${Acts} parts=${parts} notes=${NOTES} open=${state.folds?.acts} send=${send} />`;
}
