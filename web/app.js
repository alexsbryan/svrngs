// SPDX-License-Identifier: AGPL-3.0-or-later
// The phone's page, its edge (page-scaffold (e)): served as /app.js
// (scripts/build-web.sh), where the browser tests find ./wasm/ and
// ./store.js from the module's address.
// What no state can hold stays here: the browser read once into PAGE's
// first state, the worker's registration, an address changed in place, and
// the wasm, whose init is the first effect, run after this module has
// ended, never awaited at its top: WebKit wedged the tab for good when a
// person left while the module awaited the wasm's fetch (u4-7,
// web/app.js@19b63b2b:659-662). Everything else is PAGE's (web/page/page.js), drawn
// into web/index.html's #page. Where "v1" is named in this page's files, it
// is the page before it, at 19b63b2b.
import init from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
import { trace } from './ui/trace.js?v=__VERSION__';
import { html, machine, render } from './ui/kit.js?v=__VERSION__';
import { Page, compose } from './page/page.js?v=__VERSION__';
import * as acts from './page/acts.js?v=__VERSION__';
import * as among from './page/among.js?v=__VERSION__';
import * as ask from './page/ask.js?v=__VERSION__';
import * as computer from './page/computer.js?v=__VERSION__';
import * as door from './page/door.js?v=__VERSION__';
import * as enter from './page/enter.js?v=__VERSION__';
import * as index from './page/index.js?v=__VERSION__';
import * as join from './page/join.js?v=__VERSION__';
import * as shelf from './page/shelf.js?v=__VERSION__';
import * as uses from './page/uses.js?v=__VERSION__';
import * as visit from './page/visit.js?v=__VERSION__';
/** @import { Flow } from './page/page.js?v=__VERSION__' */

// The wasm by its one URL, from the cache the worker reads it from too
// (store.js `runtime`, u4-9); then the address's fragment, to open.
/** @type {Flow} */
const edge = {
  io: {
    init: () =>
      init({ module_or_path: store.runtime() }).then(() => ({
        type: 'start',
        fragment: location.hash.slice(1),
      })),
  },
};
const parts = compose({
  edge,
  enter,
  computer,
  join,
  ask,
  index,
  door,
  shelf,
  visit,
  among,
  acts,
  uses,
});
const page = machine(
  'PAGE',
  parts,
  parts.first({
    fine: matchMedia('(pointer: fine)').matches,
    // iPhone Safari's own: false outside a Home Screen web app, absent in
    // every other browser (web/page/join.js `NameForm`), so no lib declares it.
    standalone: /** @type {Navigator & { standalone?: boolean }} */ (navigator).standalone,
  }),
  [{ run: 'init', failed: 'failed' }],
);
// web/index.html holds #page.
render(
  html`<${Page} parts=${parts} store=${page} />`,
  /** @type {HTMLElement} */ (document.getElementById('page')),
);

// An address typed or tapped on this page is a new fragment on the same
// page, which the browser opens without loading it (web/app.js@19b63b2b:626).
// Back on screen, the group is read again (page-open (c); web/page/enter.js `shown`).
document.addEventListener(
  'visibilitychange',
  () => document.visibilityState === 'visible' && page.dispatch({ type: 'shown' }),
);
window.addEventListener(
  'hashchange',
  () => (trace('PAGE: the address changed here, so the page opens it'), location.reload()),
);
if ('serviceWorker' in navigator)
  window.addEventListener('load', () =>
    navigator.serviceWorker
      .register('./sw.js', { type: 'module' })
      .catch((e) => trace(`PAGE: the service worker did not register: ${e}`)),
  );
