# WebKit partitions a framed signer's storage, 2026-11-02

Why the security change's seventh part was redesigned (svrn-docs
`rings/current/HOLDER_APPROVAL.md`, agreed by the operator 2026-11-02).
Measured with Playwright 1.59.1 (WebKit 26.4, Chromium 147.0.7727.15) and
Chrome 151, on node 20.20.2, Darwin 27.0.0. No DNS: every name is routed by
`proxy.js`, a CONNECT proxy, to `server.js`'s HTTPS server.

- `diligence/`: does a group-origin frame under an app subdomain see the group
  page's IndexedDB? `node partition.js webkit chromium` → `run1.txt`;
  `node partition.js chromium-part chrome-part` → `run2.txt` (Chromium with
  its storage partitioning on, as shipped); `saa.js` → `run3.txt` (the
  Storage Access API). **WebKit: no, on every layout, the production shape
  `app1.group.svrnme.sh` → `group.svrnme.sh` included; Chromium: yes for
  same-site.** `vercel_routes.mjs` → `run4.txt`: the Vercel CLI's own routing
  serves the later `/sign.html` CSP, one header (inferred for the edge).
- `holder-design/`: the options A to D. `node measure.js` →
  `run-webkit.txt`, `run-chromium.txt`; `run-webkit-cookie-header.txt`;
  `localhost.js`, `localhost-tls.js` → the test layouts. **A `__Host-`
  cookie set by the group page is read by its frame under the app, is
  invisible to the app, and can't be planted by the app, in all three.**

The private keys are not kept. To run again, make the two self-signed
certificates beside the scripts:

    openssl req -x509 -newkey rsa:2048 -nodes -days 30 -subj /CN=svrnme \
      -keyout key.pem -out cert.pem -addext "subjectAltName=DNS:*.svrnme.sh,DNS:*.group.svrnme.sh,DNS:*.other.sh,DNS:*.svrnme.co.uk,DNS:*.group.svrnme.co.uk,DNS:*.svrnme.test,DNS:*.group.svrnme.test,DNS:*.other.test"
    openssl req -x509 -newkey rsa:2048 -nodes -days 30 -subj /CN=localhost \
      -keyout lkey.pem -out lcert.pem -addext "subjectAltName=DNS:localhost,DNS:*.localhost,DNS:page.localhost,DNS:*.page.localhost"

The scripts load Playwright from `web/node_modules`. Not measured: Safari on
a real iPhone, which HOLDER_APPROVAL §6 keeps owed.
