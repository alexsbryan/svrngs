// SPDX-License-Identifier: AGPL-3.0-or-later
// Direct connections on this WiFi (webrtc-network; DESIGN §3): the lender
// turns them on or off for the network this computer is on now, from the
// home page, as /v1/direct says it (webrtc/direct.rs). Read once as the
// part is mounted; each press answers with the setting as it stands then,
// so a refused press says why in its line.
import { html, useEffect, useState } from '../ui/view.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/**
 * What /v1/direct answers (webrtc/direct.rs `Setting`): whether direct
 * connections are on for this WiFi, and that said.
 * @typedef {{ on: boolean, said: string }} Setting
 */

/** What the button says its press does. @param {boolean} on */
export const words = (on) => (on ? 'Turn direct connections off' : 'Turn direct connections on');

/** @returns {Drawn} */
function Line() {
  const [setting, set] = useState(/** @type {Setting | null} */ (null));
  useEffect(() => {
    get('/v1/direct').then(set, () => {});
  }, []);
  if (!setting) return null;
  /** @param {Error} why */
  const refusal = (why) => set({ ...setting, said: why.message });
  const turn = () => post('/v1/direct', { on: !setting.on }).then(set, refusal);
  return html`<p>${`${setting.said} `} <button onClick=${turn}>${words(setting.on)}</button></p>`;
}

// "Direct connections", for acting.js: one line, drawn once it is read, so
// the setting is never invisible.
export const part = () => [html`<${Line} />`];
