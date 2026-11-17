// The browser half of the conformance canary's test (u5-2;
// crates/svrngs-node/tests/conformance.rs; apps/web-conformance). In
// Playwright's headless `engine` (page.mjs) it opens each lend in
// `args.lends` — Sam's and Bo's canary — the way `args.mode` says, runs each
// of the canary's checks in its page, and prints what it saw as one JSON
// line, with each request the page sent under a lend's address and whether
// the lend's worker answered it, what of those the page's own host was
// asked, and what WebKit's processes held meanwhile (u5-12):
// - `computer`: Alex's computer's browser opens each lend's origin on
//   Alex's loopback (`args.origins`), as the bare command prints it.
// - `wildcard`: a phone, holding `args.key` in a group joined by
//   `args.token`, opens the group's page on `localhost`, whose every name
//   answers, and each lend from its list (DESIGN §7).
// In each lent page it also looks for anything that unseals the person's
// key, and on a computer posts to the record API (`args.api`); and every
// byte the browser receives is scanned for `args.secrets`.
//
//   node web/tests/conformance.mjs '{"engine": ..., "mode": ..., "site": ...,
//     "lends": [{"who": "sam", "button": "sam’s app", "label": ..., "origin": ...}, ...],
//     "key": ..., "token": ..., "api": ..., "append": ...,
//     "secrets": [...], "routes": [...]}'
//
// `routes`, when given, runs only those checks in each lend, in order, and
// none after: u5-12's runs of the upload after the seeks, which list a lend
// once for each try, and the playback step's seeks alone (conformance.rs
// `seeks_back_to_back`).
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { execSync, spawn } from 'node:child_process';
import http from 'node:http';
import { hold, launch, pageAt, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const phone = args.mode !== 'computer';
const server = phone ? await serve(args.site) : null;
const pageOrigin = server && pageAt(server);

// In a lent page: the groups a `svrngs` store at this origin holds, which
// a lent page would unseal the person's key from (DESIGN §8, risk 5).
async function unsealable() {
  const shim = document.querySelector('script[src*="shim.js"]');
  const version = shim ? new URL(shim.src).searchParams.get('v') : null;
  const names = (await indexedDB.databases?.())?.map((d) => d.name) ?? [];
  if (!version) return { groups: 0, names };
  const store = await import(`/store.js?v=${version}`);
  return { groups: (await store.groups()).length, names };
}

// Every route but `seeks`, the one that plays, which the playback step asks
// alone (crates/svrngs-node/tests/conformance.rs `seeks_back_to_back`).
const ROUTES = [
  'root',
  'isolated',
  'host',
  'range',
  'upload',
  'websocket',
  'signin',
  'guarded',
  'worker',
];
const routes = args.routes ?? ROUTES;
// The clock and this host's load (the u5 queue's rule for a run that times
// seeks).
const load = () => execSync('echo $(date -u +%FT%TZ) $(uptime)').toString().trim();

const { context, close } = await launch(args.engine);
// What `eval` does in a worker served as svrngs serves its own (the session
// worker on a computer, lend.js on a phone): the policy that came with the
// page's worker's script, sent with a worker at an origin of this driver's
// own, which answers `probe` with what `eval` did there. The policy is
// carried, not the script rewritten: a worker the browser stopped runs
// again from the bytes its server sent, in Chromium and WebKit alike.
const realms = http.createServer((req, res) => {
  const [, n, file] = /^\/(\d+)\/(.*)$/.exec(new URL(req.url, 'http://x').pathname) ?? [];
  if (file === '')
    return res
      .writeHead(200, { 'content-type': 'text/html' })
      .end('<!doctype html><title>realm</title><script src=page.js></script>');
  if (file === 'page.js')
    return res.writeHead(200, { 'content-type': 'text/javascript' }).end(`(async () => {
  await navigator.serviceWorker.register('sw.js');
  await navigator.serviceWorker.ready;
  if (!navigator.serviceWorker.controller) await new Promise((r) => navigator.serviceWorker.addEventListener('controllerchange', r, { once: true }));
  document.title = await (await fetch('probe')).text();
})().catch((e) => (document.title = String(e)));`);
  if (file === 'sw.js')
    return res.writeHead(200, {
      'content-type': 'text/javascript',
      ...(policies[n] ? { 'content-security-policy': policies[n] } : {}),
    }).end(`self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (e) => e.waitUntil(self.clients.claim()));
self.addEventListener('fetch', (e) => {
  if (!e.request.url.endsWith('/probe')) return;
  let said = 'eval ran';
  try { (0, eval)('1'); } catch (error) { said = error.name; }
  e.respondWith(new Response(said));
});`);
  res.writeHead(404).end();
});
const policies = [];
await new Promise((done) => realms.listen(0, '127.0.0.1', done));
// `page`'s worker's policy, as its server sends it with the worker's
// script (the node, or the page's host here), and what `eval` does under it.
async function realm(page) {
  const script = new URL(await page.evaluate(() => navigator.serviceWorker.controller.scriptURL));
  const policy = await new Promise((done, failed) =>
    http
      .get(
        {
          host: '127.0.0.1',
          port: script.port,
          path: script.pathname + script.search,
          headers: { host: script.host },
        },
        (res) => (res.resume(), done(res.headers['content-security-policy'] ?? null)),
      )
      .on('error', failed),
  );
  const n = policies.push(policy) - 1;
  const probe = await context.newPage();
  await probe.goto(`http://127.0.0.1:${realms.address().port}/${n}/`);
  await probe.waitForFunction(() => document.title !== 'realm', null, { timeout: 30_000 });
  const said = await probe.title();
  await probe.close();
  return { ok: said === 'EvalError', saw: `${said} under ${policy}` };
}
const logs = [];
const leaked = [];
// Each request under the lend being opened, `<time> <method> <url> <status>
// <who answered>`: the lend's worker, or not.
let sent = [];
const answered = (r, by) => sent.push(`${new Date().toISOString()} ${r.method()} ${r.url()} ${by}`);
context.on('response', (r) =>
  answered(
    r.request(),
    `${r.status()} ${r.fromServiceWorker() ? 'by the worker' : 'not by the worker'}`,
  ),
);
context.on('requestfailed', (r) => answered(r, `failed: ${r.failure()?.errorText}`));
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
// What each of this browser's content processes holds, a second apart, as
// `<time> <pid>:<MB> …`, while a lend's checks run (u5-12): a worker's
// process that grows, then is gone. WebKit's only, on macOS. Every WebKit
// browser's content processes are launchd's, so this one's are told apart
// by its own log line as each starts; its own process is the child of
// Playwright's `pw_run.sh`, this process's child.
let memory = [];
const ours = new Set();
const ps = () =>
  execSync('ps -axo pid=,ppid=,rss=,command=')
    .toString()
    .trim()
    .split('\n')
    .map((l) => l.trim().split(/\s+/));
const shell = ps().find(
  ([, ppid, , cmd, run]) => ppid === String(process.pid) && `${cmd} ${run}`.includes('pw_run.sh'),
)?.[0];
const ui =
  args.engine === 'webkit' &&
  ps().find(([, ppid, , cmd]) => ppid === shell && cmd.endsWith('/Playwright'))?.[0];
const started = ui
  ? spawn('/usr/bin/log', ['stream', '--style', 'compact', '--predicate', `processID == ${ui}`])
  : null;
started?.stdout.on('data', (d) => {
  for (const [, pid] of String(d).matchAll(/\[PID=(\d+)\] WebProcessProxy::didFinishLaunching/g))
    ours.add(pid);
});
if (started) await new Promise((r) => setTimeout(r, 2000));
const sampler = setInterval(() => {
  const held = ps().filter(([pid]) => ours.has(pid));
  if (held.length)
    memory.push(
      `${new Date().toISOString()} ${held.map(([pid, , rss]) => `${pid}:${Math.round(rss / 1024)}`).join(' ')}`,
    );
}, 1000);
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
    await page.evaluate(hold, { hex: args.key, token: args.token, person: 'alex' });
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
    const seen = { at: page.url(), load: [load()] };
    const hostAt = server?.asked.length;
    [sent, memory] = [[], []];
    for (const route of routes) {
      seen[route] = await run(route);
    }
    seen.load.push(load());
    if (!args.routes) seen.realm = await realm(page);
    seen.requests = sent.filter((l) => l.includes(lend.label));
    seen.memory = memory;
    // What the page's host was asked meanwhile that is not the page's own
    // files: a lend's request no worker answered. Not `host`, the name of
    // the canary's Host check, whose verdict it would take the place of.
    if (phone) seen.asked = server.asked.slice(hostAt).filter((l) => !/^\S+ GET \S+ 200$/.test(l));
    // The lend's worker's trace, each range in it (u5-11): a phone's only.
    if (phone)
      seen.trace = await page
        .evaluate(() => fetch('svrngs-trace').then((r) => r.text()))
        .catch((e) => String(e));
    if (args.routes) {
      out[lend.who] = seen;
      continue;
    }
    seen.stored = await run('stored', lend.who);
    await page.reload();
    await ready();
    seen.cookie = await run('cookie', lend.who);
    seen.storage = await run('storage', lend.who);
    if (phone)
      seen.unsealable = await page.evaluate(unsealable).catch((e) => ({ error: String(e) }));
    if (args.api) {
      seen.recordPost = await page.evaluate(
        ([api, body]) =>
          fetch(`${api}/v1/rail/append`, {
            method: 'POST',
            headers: { 'content-type': 'text/plain' },
            body,
          })
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
  clearInterval(sampler);
  started?.kill();
  await close();
  server?.close();
  realms.close();
}
