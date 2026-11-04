// SPDX-License-Identifier: AGPL-3.0-or-later
// The group's page for a person working on it, served on this computer as
// the browser tests serve it (page.mjs `serve`, `proxied`, `certified`): the
// built page at `https://group.svrnme.test:<port>/`, each app at a name
// under it, reached through the tests' proxy, which ends TLS with their
// certificate. An app saves only there: on `localhost` and its names no
// engine gives the group's signer, framed under an app, the page's cookie,
// and over TLS on `page.localhost` WebKit doesn't either (DESIGN §3;
// demo/runs/security/2026-11-02-webkit-partition/holder-design/
// run-localhost.txt, run-localhost-tls.txt). So it prints a line that
// opens Chrome on a profile of its own, told to reach the page through the
// proxy and to take that one certificate, by its key; and how a group
// points at the page. Recorded working: demo/runs/dev-page/2026-11-02/.
//
//   node web/tests/dev.mjs [site dir]     (else scripts/build-web.sh's target/web/site)
import { X509Certificate, createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { certified, pageAt, proxied, serve } from './page.mjs';

const site = process.argv[2] ?? new URL('../../target/web/site/', import.meta.url).pathname;
if (!existsSync(`${site}/index.html`)) {
  console.error(`There's no built page at ${site}. Build it first: scripts/build-web.sh`);
  process.exit(3);
}
const server = await serve(site);
const page = pageAt(server);
const port = server.address().port;
// Chrome takes a certificate it doesn't trust when told its key's hash.
const spki = createHash('sha256')
  .update(new X509Certificate(certified().cert).publicKey.export({ type: 'spki', format: 'der' }))
  .digest('base64');
const flags = `--user-data-dir="$(mktemp -d)" --no-first-run --proxy-server=${await proxied()} --ignore-certificate-errors-spki-list=${spki} ${page}`;
const chrome =
  process.platform === 'darwin'
    ? `open -na "Google Chrome" --args ${flags}`
    : `google-chrome ${flags}`;
console.log(`Your group's page is at ${page}, for this computer only.

Open it in a Chrome of its own: paste this in another Terminal window.

  ${chrome}

That Chrome reaches the page through this command, and takes its certificate, which nothing else trusts. It keeps nothing you already use.

To join a group there, open any link to it in that Chrome, with what comes before the # changed to ${page} (for a svrngs://join/ link, svrngs://join/ changed to ${page}#).

Or point a group at this page, so svrngs prints its links here. Stop svrngs, then run one of these, which runs it again:

  svrngs new <name> --origin http://localhost:${port}/
  svrngs link <group> --origin http://localhost:${port}/

svrngs checks the page at http://localhost, since nothing but that Chrome reaches the https address, so its links start http://localhost:${port}/. Change that to ${page} too: at http://localhost, apps don't open. A new link stops the group's old one letting anyone in. --origin https://group.svrnme.sh/ points it back.

Safari can't save in an app here. It keeps the page's cookie from the app on every local address (demo/runs/security/2026-11-02-webkit-partition/).

After a change, run scripts/build-web.sh again and reload. Ctrl-C stops this.`);
