// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page's edge, served at /home.js (home.rs `served`): it imports
// each feature's file (home-reading.js FILES), starts HOME with the groups'
// first read, draws the page (home-page.js `Home`) into
// home/index.html's #home, keeps the address to the page drawn, and
// reads the groups again when the page is shown again.
import { trace } from './ui/trace.js?v=__VERSION__';
import { html, render } from './ui/view.js?v=__VERSION__';
import { machine } from './ui/machine.js?v=__VERSION__';
import { FILES, HOME, first } from './page/home-reading.js?v=__VERSION__';
import { Home } from './page/home-page.js?v=__VERSION__';
import { GroupsLeft } from './page/groups-left.js?v=__VERSION__';
import { address } from './page/grouped.js?v=__VERSION__';
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
    const { grouped } = store.get();
    document.body.classList.toggle('layout-b', grouped);
    trace(`HOME: layout ${grouped ? 'b' : 'a'}`);
    // The address is the page drawn, less what was done here, which is
    // said once: a reload, or the page opened again, says it no more.
    // Replaced, never pushed: a write here is no place the person went,
    // and the page draws nothing anew on Back (it has no `popstate`), so
    // Back goes where they were before, not to the page the write was
    // made on, such as the group they left.
    let shown = store.get().q;
    const kept = () => {
      const to = grouped ? address(shown.view, shown.group) : `${location.pathname}?ui=a`;
      trace(`HOME: ${store.get().at.said} said, once, so the address is ${to}`);
      history.replaceState(null, '', to);
    };
    if (store.get().at.said !== 'none') kept();
    // A write here moves the page (HOME's `again`, with its `q`), and the
    // address with it.
    store.subscribe(() => {
      if (store.get().q === shown) return;
      shown = store.get().q;
      kept();
    });
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
