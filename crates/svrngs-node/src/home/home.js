// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page's edge, served at /home.js (home.rs `served`): it imports
// each feature's file (home-reading.js FILES), starts HOME with the groups'
// first read, draws the page (home-page.js `Home`) into
// home/index.html's #home, forgets a notice the address holds, and reads
// the groups again when the page is shown again.
import { trace } from './ui/trace.js?v=__VERSION__';
import { html, render } from './ui/view.js?v=__VERSION__';
import { machine } from './ui/machine.js?v=__VERSION__';
import { FILES, HOME, first } from './page/home-reading.js?v=__VERSION__';
import { Home } from './page/home-page.js?v=__VERSION__';
import { GroupsLeft } from './page/groups-left.js?v=__VERSION__';
/** @import { Effect, Flows } from './page/home-reading.js?v=__VERSION__' */

// Never awaited at the module's top, as the phone's web/app.js is not: a
// page left while its module still awaited, then opened again, spun
// WebKit's module loader at most of a core and never loaded (its
// cpu_resource reports, JSModuleLoader::evaluate from a microtask).
Promise.all(FILES.map((file) => import(`./page/${file}.js?v=__VERSION__`)))
  .then((mods) => {
    // Each module imported by its name in FILES, so each name is there.
    const flows = /** @type {Flows} */ (
      Object.fromEntries(FILES.map((file, i) => [file, mods[i]]))
    );
    const places = Object.assign({ left: GroupsLeft }, ...mods.map((flow) => flow.view));
    const store = machine(
      'HOME',
      HOME(flows),
      first(location),
      /** @type {Effect[]} */ ([{ run: 'groups', n: 1, round: 'read', failed: 'unread' }]),
    );
    // What was done here is said once: the address forgets it, so a
    // reload, or the page opened again, says it no more.
    const said = store.get().at.said;
    if (said !== 'none') {
      trace(`HOME: ${said} said, once`);
      history.replaceState(null, '', location.pathname);
    }
    // home/index.html holds #home, as it does where its files fail below.
    render(
      html`<${Home} store=${store} flows=${flows} places=${places} />`,
      /** @type {HTMLElement} */ (document.getElementById('home')),
    );
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible') store.dispatch({ type: 'visible' });
    });
  })
  .catch((error) => {
    trace(`HOME: its files did not load: ${error}`);
    render(
      html`<p>This computer cannot read your groups: ${error.message}</p>`,
      /** @type {HTMLElement} */ (document.getElementById('home')),
    );
  });
