// SPDX-License-Identifier: AGPL-3.0-or-later
// A form posting what the person says of an app reaching friends' lends as
// them, to /reaching (home.rs): its yes, which goes on to the app in a tab
// of its own, so it opens in a new tab; or its Stop, which comes back to
// this page.
import { html } from '../ui/view.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/**
 * The form's props: the app's label, the button's words, and what it posts.
 * @typedef {{ app: string, words: Drawn, said?: 'yes' | 'stop', reads?: string[] }} ReachProps
 */

/** @param {ReachProps} props */
export function Reach({ app, words, said = 'yes', reads = [] }) {
  const target = said === 'yes' ? '_blank' : undefined;
  return html`
    <form method="post" action="/reaching" target=${target}>
      <input type="hidden" name="app" value=${app} />
      <input type="hidden" name="said" value=${said} />
      <input type="hidden" name="reads" value=${reads.join(' ')} />
      <button>${words}</button>
    </form>
  `;
}
