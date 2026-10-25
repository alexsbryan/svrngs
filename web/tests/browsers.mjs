// SPDX-License-Identifier: AGPL-3.0-or-later
// One browser of each engine for a run of browser tests
// (crates/svrngs-node/tests/browser/harness.rs `browsers`): each test's
// driver connects to it and opens a context of its own (page.mjs `launch`),
// so a run starts two browsers where it started one a test. Prints their
// addresses as one line, {"chromium": "<ws>", "webkit": "<ws>"}, and closes
// both when its stdin ends, as the test binary that started it exits.
//
//   node web/tests/browsers.mjs

import { chromium, webkit } from 'playwright';

const servers = { chromium: await chromium.launchServer(), webkit: await webkit.launchServer() };
console.log(JSON.stringify({ chromium: servers.chromium.wsEndpoint(), webkit: servers.webkit.wsEndpoint() }));
process.stdin.on('end', () => Promise.all(Object.values(servers).map((s) => s.close())).then(() => process.exit(0)));
process.stdin.resume();
