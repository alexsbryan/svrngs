// SPDX-License-Identifier: AGPL-3.0-or-later
// The phone's v2 page, its edge (page-scaffold (e)): served as /app.js in
// the v2 build (web/v2/build.sh), where web/app.js is in v1's, as the
// browser tests find ./wasm/ and ./store.js from the module's address.
// What no state can hold stays here: the browser read once into PAGE's
// first state, the worker's registration, an address changed in place, and
// the wasm, whose init is the first effect, run after this module has
// ended, never awaited at its top: WebKit wedged the tab for good when a
// person left while the module awaited the wasm's fetch (u4-7,
// web/app.js:659-662). Everything else is PAGE's (web/v2/page.js), drawn
// into #page (web/v2/body.html).
import init from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
import { trace } from './ui/trace.js?v=__VERSION__';
import { html, render } from './ui/view.js?v=__VERSION__';
import { machine } from './ui/machine.js?v=__VERSION__';
import { Page, compose } from './v2/page.js?v=__VERSION__';
import * as acts from './v2/acts.js?v=__VERSION__';
import * as among from './v2/among.js?v=__VERSION__';
import * as ask from './v2/ask.js?v=__VERSION__';
import * as computer from './v2/computer.js?v=__VERSION__';
import * as door from './v2/door.js?v=__VERSION__';
import * as enter from './v2/enter.js?v=__VERSION__';
import * as index from './v2/index.js?v=__VERSION__';
import * as join from './v2/join.js?v=__VERSION__';
import * as shelf from './v2/shelf.js?v=__VERSION__';
import * as uses from './v2/uses.js?v=__VERSION__';
import * as visit from './v2/visit.js?v=__VERSION__';

// The wasm by its one URL, from the cache the worker reads it from too
// (store.js `runtime`, u4-9); then the address's fragment, to open.
const edge = { io: { init: () => init({ module_or_path: store.runtime() }).then(() => ({ type: 'start', fragment: location.hash.slice(1) })) } };
const parts = compose({ edge, enter, computer, join, ask, index, door, shelf, visit, among, acts, uses });
const page = machine('PAGE', parts, parts.first({ fine: matchMedia('(pointer: fine)').matches, standalone: navigator.standalone }), [{ run: 'init', failed: 'failed' }]);
render(html`<${Page} parts=${parts} store=${page} />`, document.getElementById('page'));

// An address typed or tapped on this page is a new fragment on the same
// page, which the browser opens without loading it (web/app.js:626).
// Back on screen, the group is read again (page-open (c); web/v2/enter.js `shown`).
document.addEventListener('visibilitychange', () => document.visibilityState === 'visible' && page.dispatch({ type: 'shown' }));
window.addEventListener('hashchange', () => (trace('PAGE: the address changed here, so the page opens it'), location.reload()));
if ('serviceWorker' in navigator) window.addEventListener('load', () => navigator.serviceWorker.register('./sw.js', { type: 'module' }).catch((e) => trace(`PAGE: the service worker did not register: ${e}`)));
