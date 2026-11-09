// SPDX-License-Identifier: AGPL-3.0-or-later
// A form posting what the person says of an app reaching friends' lends as
// them, to /reaching (home.rs): its yes, which goes on to the app in a tab
// of its own, so it opens in a new tab; or its Stop, which comes back to
// this page.
import { html, useState } from '../ui/view.js?v=__VERSION__';
import { named, readLine } from '../question.js?v=__VERSION__';
import { Fold, Tick } from '../ui/parts.js?v=__VERSION__';
/** @import { Drawn, Targeted } from '../ui/view.js?v=__VERSION__' */
/** @import { ReadLend } from '../question.js?v=__VERSION__' */

/**
 * The form's props: the app's label, the button's words, and what it posts:
 * the lends its open names, ticked and unticked.
 * @typedef {{ app: string, words: Drawn, said?: 'yes' | 'stop', reads?: string[], left?: string[] }} ReachProps
 */

/** @param {ReachProps} props */
export function Reach({ app, words, said = 'yes', reads = [], left = [] }) {
  const target = said === 'yes' ? '_blank' : undefined;
  return html`
    <form method="post" action="/reaching" target=${target}>
      <input type="hidden" name="app" value=${app} />
      <input type="hidden" name="said" value=${said} />
      <input type="hidden" name="reads" value=${reads.join(' ')} />
      <input type="hidden" name="left" value=${left.join(' ')} />
      <button>${words}</button>
    </form>
  `;
}

/**
 * An app reading friends' lends, opened from its button: `reading`, what
 * /v1/lends says its open names (surface.rs `opening`), and the kinds.
 * @typedef {{ app: string, words: Drawn, reading: ReadLend[], kinds: string[] }} OpeningProps
 */

// Open and its line, and under the line each lend it names, tickable: the
// line and what the button posts are always the ticks' (§4's "each one
// untickable"), one fold for a handful of lenders and for many alike.
/** @param {OpeningProps} props */
export function Opening({ app, words, reading, kinds }) {
  const [off, setOff] = useState(() =>
    reading.filter((one) => !one.ticked).map((one) => one.label),
  );
  const on = reading.filter((one) => !off.includes(one.label));
  /** @param {string} label */
  const toggle = (label) => (/** @type {Targeted<HTMLInputElement>} */ ticked) =>
    setOff(ticked.currentTarget.checked ? off.filter((other) => other !== label) : [...off, label]);
  const line = readLine(on, reading.length, kinds);
  const ticks = reading.map(
    (one) =>
      html`<${Tick}
        key=${one.label}
        on=${!off.includes(one.label)}
        change=${toggle(one.label)}
      >${` ${named(one)}`}<//>`,
  );
  return html`
    <${Reach} app=${app} words=${words} reads=${on.map((one) => one.label)} left=${off} />
    ${
      reading.length === 0
        ? html`<p>${line}</p>`
        : html`<${Fold} summary=${line} fold="reading" send=${() => {}}>${ticks}<//>`
    }
  `;
}
