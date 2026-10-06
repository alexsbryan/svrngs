// The browser half of the conformance canary's test (u5-2;
// crates/svrngs-node/tests/conformance.rs; apps/web-conformance). In
// Playwright's headless `engine` (page.mjs) it opens each lend in
// `args.lends` — Sam's and Bo's canary — the way `args.mode` says, runs each
// of the canary's checks in its page, and prints what it saw as one JSON
// line:
// - `computer`: Alex's computer's browser opens each lend's origin on
//   Alex's loopback (`args.origins`), as the bare command prints it.
// - `wildcard`, `floor`: a phone, holding `args.key` in a group joined by
//   `args.token`, opens the group's page — on `localhost`, whose every name
//   answers, or on 127.0.0.1 with the apps under `localhost` (`args.under`)
//   — and each lend from its list (DESIGN §7).
// In each lent page it also looks for anything that unseals the person's
// key, and on a computer posts to the record API (`args.api`); and every
// byte the browser receives is scanned for `args.secrets`.
//
//   node web/tests/conformance.mjs '{"engine": ..., "mode": ..., "site": ...,
//     "lends": [{"who": "sam", "button": "sam’s app", "label": ..., "origin": ...}, ...],
//     "key": ..., "token": ..., "under": ..., "api": ..., "append": ...,
//     "secrets": [...]}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { hold, launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const phone = args.mode !== 'computer';
const server = phone ? await serve(args.site) : null;
const port = server?.address().port;
const pageOrigin = args.mode === 'floor' ? `http://127.0.0.1:${port}/` : `http://localhost:${port}/`;
const under = args.mode === 'floor' ? `http://localhost:${port}/` : undefined;

// In a lent page: the groups a `svrngs` store at this origin holds, which
// a lent page would unseal the person's key from (DESIGN §11, risk 5).
async function unsealable() {
  const shim = document.querySelector('script[src*="shim.js"]');
  const version = shim ? new URL(shim.src).searchParams.get('v') : null;
  const names = (await indexedDB.databases?.())?.map((d) => d.name) ?? [];
  if (!version) return { groups: 0, names };
  const store = await import(`/store.js?v=${version}`);
  return { groups: (await store.groups()).length, names };
}

const { context, close } = await launch(args.engine);
const logs = [];
const leaked = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
context.on('response', async (r) => {
  try {
    const text = `${JSON.stringify(await r.allHeaders())}${await r.text()}`;
    for (const s of args.secrets) if (text.includes(s)) leaked.push(`${r.url()} holds ${s}`);
  } catch {}
});
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  if (phone) {
    await page.goto(pageOrigin);
    await page.evaluate(hold, { hex: args.key, token: args.token, lends: under, person: 'alex' });
  }
  const run = (name, ...a) => page.evaluate(([n, a]) => window.conformance[n](...a), [name, a]);
  const ready = () => page.waitForFunction(() => window.conformance, null, { timeout: 60_000 });
  const out = {};
  for (const lend of args.lends) {
    if (phone) {
      await page.goto('about:blank');
      await page.goto(`${pageOrigin}#${args.token}`);
      await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
      const button = page.locator('#lent button', { hasText: lend.button });
      // Wherever the page opens it: the test judges where (`at`).
      const opened = (u) => u.href.includes(lend.label) && !u.pathname.endsWith('open.html');
      await Promise.all([page.waitForURL(opened, { timeout: 90_000 }), button.click()]);
    } else {
      await page.goto(lend.origin);
    }
    await ready();
    const seen = { at: page.url() };
    for (const route of ['root', 'isolated', 'host', 'range', 'upload', 'websocket', 'signin', 'worker']) {
      seen[route] = await run(route);
    }
    seen.stored = await run('stored', lend.who);
    await page.reload();
    await ready();
    seen.cookie = await run('cookie', lend.who);
    seen.storage = await run('storage', lend.who);
    if (phone) seen.unsealable = await page.evaluate(unsealable).catch((e) => ({ error: String(e) }));
    if (args.api) {
      seen.recordPost = await page.evaluate(
        ([api, body]) =>
          fetch(`${api}/v1/rail/append`, { method: 'POST', headers: { 'content-type': 'text/plain' }, body })
            .then((r) => `answered ${r.status}`)
            .catch((e) => `not read: ${e}`),
        [args.api, args.append],
      );
    }
    out[lend.who] = seen;
  }
  console.log(JSON.stringify({ lends: out, leaked, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server?.close();
}
