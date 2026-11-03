// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// The pages' one way to draw (page-scaffold (d)): Preact and htm, vendored
// (web/vendor/README.md), imported here and nowhere else, each at the one
// URL its version is stamped in (web/tests/specifiers.test.mjs), since two
// URLs load Preact twice and its hooks then fail. Served at /ui/ beside
// /vendor/ on each page.
import { h, render } from '../vendor/preact.js?v=__VERSION__';
import { useEffect, useLayoutEffect, useState } from '../vendor/hooks.js?v=__VERSION__';
import htm from '../vendor/htm.js?v=__VERSION__';

export const html = htm.bind(h);
export { render, useEffect };

// A machine's state (ui/machine.js), the view drawn again on each change.
// `useSyncExternalStore` is preact/compat's alone, so it is `useState` with
// the subscription in `useLayoutEffect`, read again as it subscribes, so a
// change between the draw and the subscription is not missed.
export function useStore(store) {
  const [state, set] = useState(store.get);
  useLayoutEffect(() => {
    set(store.get());
    return store.subscribe(() => set(store.get()));
  }, [store]);
  return state;
}

// A machine of a part's own, made once as the part is mounted: its state
// and its `dispatch`.
export function useMachine(make) {
  const [m] = useState(make);
  return [useStore(m), m.dispatch];
}
