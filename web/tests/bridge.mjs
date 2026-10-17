// The browser half of arch-peer (a)'s test (crates/svrngs-node/tests/browser):
// a page on another origin than any of svrngs's asks every port `args.ports`
// names on 127.0.0.1, as a program holding no key would (lend_origin.rs
// `no_loopback_port_on_this_computer_reaches_a_friends_door`):
// `GET /lend/pad/probe-<port>`, read if the browser lets it be read, and
// `POST /choices/book` saying yes for `args.app`, sent as a form posts, so no
// preflight asks first. With `args.public`, Chromium is told the page is on
// the internet (`--ip-address-space-overrides`), as a page a person browses
// to is, and given the person's leave to reach the local network, which it
// asks before a page from the internet reaches loopback (Chromium 147
// refuses one unasked: "Permission was denied for this request to access the
// `loopback` address space"); WebKit has no such flag.
// Whether a request reached the door is the Rust half's to read, at the far
// end; this half says what the page was let see.
//
//   node web/tests/bridge.mjs '{"engine": ..., "ports": [...], "app": ...,
//     "public": true|false}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import http from 'node:http';
import { launch } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = http.createServer((_, res) =>
  res.writeHead(200, { 'content-type': 'text/html' }).end('<!doctype html><title>elsewhere</title>'),
);
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const at = server.address().port;
const flags = args.public ? [`--ip-address-space-overrides=127.0.0.1:${at}=public`] : [];
const { context, close } = await launch(args.engine, { args: flags });
if (args.public) await context.grantPermissions(['local-network-access']);
const logs = [];
context.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
try {
  const page = await context.newPage();
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  await page.goto(`http://127.0.0.1:${at}/`);
  const asked = await page.evaluate(
    async ({ ports, app }) => {
      const tried = async (f) => {
        try {
          return await f();
        } catch (e) {
          return { error: String(e) };
        }
      };
      const out = [];
      for (const port of ports) {
        const base = `http://127.0.0.1:${port}`;
        const lend = await tried(async () => {
          const r = await fetch(`${base}/lend/pad/probe-${port}`);
          return { status: r.status, read: (await r.text()).slice(0, 80) };
        });
        const lendBlind = await tried(async () => {
          const r = await fetch(`${base}/lend/pad/probe-${port}`, { mode: 'no-cors' });
          return { type: r.type };
        });
        const yes = await tried(async () => {
          const r = await fetch(`${base}/choices/book`, {
            method: 'POST',
            mode: 'no-cors',
            headers: { 'content-type': 'text/plain' },
            body: JSON.stringify({ app, said: 'yes' }),
          });
          return { type: r.type };
        });
        out.push({ port, lend, lendBlind, yes });
      }
      return out;
    },
    { ports: args.ports, app: args.app },
  );
  console.log(JSON.stringify({ page: `http://127.0.0.1:${at}/`, asked, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
