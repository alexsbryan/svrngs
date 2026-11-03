// SPDX-License-Identifier: AGPL-3.0-or-later
// One browser of each engine for a run of browser tests
// (crates/svrngs-node/tests/browser/harness.rs `browsers`): each test's
// driver connects to it and opens a context of its own (page.mjs `launch`),
// so a run starts two browsers where it started one a test. Prints their
// addresses as one line, {"chromium": "<ws>", "webkit": "<ws>", "proxy":
// "<http>"}, the proxy every context reaches the page's names through
// (page.mjs `proxied`), and closes
// both when its stdin ends, as the test binary that started it exits.
//
//   node web/tests/browsers.mjs

import { chromium, webkit } from 'playwright';
import { IGNORING, proxied } from './page.mjs';

const servers = {
  chromium: await chromium.launchServer({ args: IGNORING }),
  webkit: await webkit.launchServer(),
};
console.log(
  JSON.stringify({
    chromium: servers.chromium.wsEndpoint(),
    webkit: servers.webkit.wsEndpoint(),
    proxy: await proxied(),
  }),
);
process.stdin.on('end', () =>
  Promise.all(Object.values(servers).map((s) => s.close())).then(() => process.exit(0)),
);
process.stdin.resume();
