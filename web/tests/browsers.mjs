// SPDX-License-Identifier: AGPL-3.0-or-later
// One browser of each engine for a run of browser tests
// (crates/svrngs-node/tests/browser/harness.rs `browsers`): each test's
// driver connects to it and opens a context of its own (page.mjs `launch`),
// so a run starts two browsers where it started one a test, each as
// page.mjs `starting` has it start. Prints their
// addresses as one line, {"chromium": "<ws>", "webkit": "<ws>", "proxy":
// "<http>"}, the proxy every context reaches the page's names through
// (page.mjs `proxied`), and closes
// both when its stdin ends, as the test binary that started it exits.
//
//   node web/tests/browsers.mjs

import { chromium, webkit } from 'playwright';
import { IGNORING, proxied, starting } from './page.mjs';

const types = { chromium, webkit };
const options = { chromium: { ...starting(chromium), args: IGNORING }, webkit: starting(webkit) };
let ending = false;
const servers = {};
// A browser that exits before stdin ends crashed (a Chromium CHECK, a
// SIGSEGV: ~/Library/Logs/DiagnosticReports/chrome-headless-shell-
// 2026-11-23-134118.ips, -2026-11-25-145645.ips), and every test after it
// then failed to connect, 15 of 24 scenes. It starts again at the address
// it had, the one each driver is given: the tests driving it as it
// crashed fail, and the tests after find it there.
async function serve(name, at) {
  const server = await types[name].launchServer({ ...options[name], ...at });
  const url = new URL(server.wsEndpoint());
  if (at) console.error(`browsers.mjs: the run's ${name} is back at ${url}`);
  server.on('close', (code, signal) => {
    if (ending) return;
    console.error(
      `browsers.mjs: the run's shared ${name} crashed (exit ${code}, signal ${signal}), failing any test driving it; starting it again at ${url} for the tests after`,
    );
    servers[name] = serve(name, { port: Number(url.port), wsPath: url.pathname }).catch((e) =>
      console.error(
        `browsers.mjs: the run's ${name} did not start again, so no test after can drive it: ${e}`,
      ),
    );
  });
  return server;
}
for (const name of Object.keys(types)) servers[name] = await serve(name);
console.log(
  JSON.stringify({
    chromium: servers.chromium.wsEndpoint(),
    webkit: servers.webkit.wsEndpoint(),
    proxy: await proxied(),
  }),
);
process.stdin.on('end', () => {
  ending = true;
  const closing = Object.values(servers).map(async (s) => (await s)?.close());
  Promise.all(closing).then(() => process.exit(0));
});
process.stdin.resume();
