# The catalogue dry run — what the five did against the census

u5-catalogue-dry (ralph/next/u5/STATE.md). The census (`CENSUS.md`,
c7342dc) was committed before any app ran; the data after it, in order:
the first pass (db7e32c, `data/first-pass/`), the wildcard at 3c32f86
(81f8bb2, 35aa7fc, `data/earlier/`), and the run judged here, svrngs at
935727c: the wildcard (bbc9142) and the floor (003e0d5), `data/*.json`,
each act's load at its start and end in `data/steps.log`. Five apps in
podman on this host, each lent by Sam with `svrngs share <port> --kind
<name>`; Bo on a phone through the page by the link, in Chromium and in
WebKit as an iPhone; each act checked from the lender's side through the
app's own API, not by what Bo's page says (`act.mjs`). Rerun with
`run.sh` (its header names the steps).

## The outcomes

| App | Act | Wildcard: predicted | Chromium | WebKit | Floor: predicted | Chromium | WebKit |
|---|---|---|---|---|---|---|---|
| Home Assistant | a state the lender changes shows live | works | did: the lamp off, then on, 65 ms | did: 148 ms | does not (root) | did not: `/frontend_latest/core.*.js` 404 | did not: the same |
| Immich | takes a 100 MB video, plays it, seeks | works | did: 99,135,178 bytes kept; played in 1,784 ms; seeks 1,062, 906, 2,589 ms | did: 99,135,176 bytes; played in 2,272 ms; seeks 1,046, 3,855, **15,007** ms | does not (root) | did not: `/_app/immutable/…` 404 | did not: the same |
| Nextcloud | takes a file from the browser | works | did: 5 MiB read back whole | did: the same | does not (root) | did not: `/core/css/server.css` 404, its form never ready | did not: the same |
| Actual | opens a budget | works (password typed once) | did: a budget opened, held by the lender's server | did: the same | does not (root) | did not: `/static/js/index.*.js` 404 | did not: the same |
| Paperless-ngx | takes in a PDF | works | did: consumed as document 5 | did: as document 6 | does not (root) | did not: its sign-in refused, "CSRF verification failed", its `/static/base.css` 404 | did not: the same |

Twenty outcomes, each equal to its prediction. Those with the wildcard
came only after six routes the canary gained, from what the first passes
missed (Actual's first-pass stop was the act's own step, 03cb481):

| What the run found | Route the canary gained, watched failing | Fixed in |
|---|---|---|
| Nextcloud's shim refused under its `script-src 'nonce-…' 'strict-dynamic'` (db7e32c) | its page under a nonce script policy (2ad303d) | dbd6b00 |
| Home Assistant "Loading..." for ever: `ws.OPEN` undefined on a phone's socket (db7e32c) | the socket's state read against its own constants (2ad303d) | 40b0490 |
| Immich back at its sign-in: its cookie unread until a reload (db7e32c) | the sign-in's cookie read unreloaded (2ad303d) | c950d87 |
| Paperless's sign-in 403, "Origin checking failed" (db7e32c) | a post taken only from its own address (2ad303d) | 20868ee (the door) |
| Paperless's upload 400, "No file was submitted." (03cb481) | the upload told its length (57710d3) | 3c32f86 |
| Nextcloud's sign-in in WebKit refused, three runs of three: WebKit's worker sees no Origin (81f8bb2, 35aa7fc) | a post with no Origin refused too (3f96333) | 935727c |

Each fix's commit names the canary passing after it; at 935727c every
route passes on a computer and with the wildcard, and on the floor all
but the root, local storage and a post from its own address, each
"needs the wildcard" (`cargo test -p svrngs-node --test conformance`).

**One prediction's reason was wrong, its outcome right.** The census
predicted Paperless would work with the wildcard *because* no `Origin`
would reach it from a phone ("Unverified until the run"). One did
(Chromium's, db7e32c), and the census's falsifier names that case: "then
`Origin` reaches it, and the origin route is the canary's to gain". It
gained it, and Paperless works for a different reason: the door tells it
a post from the lend's own address came from its own (20868ee), and the
worker names that address where the browser does not (935727c). The same
fix moves the census's *computer* column for Paperless, which said
**does not** because the `Origin` there, `http://<label>.localhost:<port>`,
names the viewer's port: the canary's "post, Origin checked" passes on a
computer since 20868ee. Paperless on a computer was not run here.

**The floor.** Each app opened at `/l/<label>/` and stopped at its
sign-in, every one for the root: its pages ask `/`-absolute paths, which
the floor's lends host answers 404 (`data/*-floor-*.json`, `failed`).
Paperless's sign-in form, made on its server, rendered and posted, and
was refused by its CSRF check: the floor's one origin names no lend
(the canary's "post, Origin checked", "needs the wildcard"). No app did
its act on the floor, which the census's falsifier would have flagged.

## The seeks, and the load

Immich's seeks are timed (Demo A's bar: playing again within 5 s).
At 935727c: Chromium 1,062, 906 and 2,589 ms; WebKit 1,046, 3,855 and
15,007 ms. **One miss of six**, WebKit's seek to 85 s, at a load of
5.5 to 7.7 on 12 cores (`data/steps.log`, 05:01:47 to 05:04:05), so not a
loaded host by this queue's rule. At 3c32f86 (`data/earlier/`): Chromium
2,039, 633 and 1,995 ms; WebKit 690, 1,788 and **7,408** ms, load 1.4 to
3.3. Both misses are WebKit's last seek, to 85 s, in a file uploaded
moments before: twelve
seeks, two missed, both kept. This is not Jellyfin's path and not Demo
A's rate (u5-9 judges that), but it is the carriage's, and u5-9 reads it.

The floor's acts ran at loads of 2.7 to 51.4 (a build in another tree,
05:09 to 05:14); they time no seek, and each miss is a 404 or a 403,
which load does not make.

## What §1 can say

From the census and this run (DESIGN §1, rewritten in the next commit):

- **Works** with the wildcard, on a computer by the census: Jellyfin
  (measured, u5-4), Immich, Nextcloud, Home Assistant (measured here),
  Paperless-ngx (measured with the wildcard; a computer by the canary
  since 20868ee), Audiobookshelf, Mealie, Grocy (census only).
- **Works after a setting the lender types once:** Actual, its server's
  password, typed once by each person (measured); Navidrome,
  `ND_EXTAUTH_USERHEADER=X-Svrngs-Person` and `ND_EXTAUTH_TRUSTEDSOURCES`;
  Calibre-Web, "Allow Reverse Proxy Authentication" with the header named
  `X-Svrngs-Person` (census only).
- **Not yet:** Firefly III and BookStack write an address built from
  `APP_URL` into their pages, which no setting typed once can make a
  computer's per-viewer port; Vikunja and Forgejo uncertain on a computer
  (census). Each works after that setting with the wildcard, by the census;
  §1 leaves them out until a run says more.
- **The floor carries none of them:** every one serves its pages from its
  root. On a phone each needs a page host that gives every app an address
  of its own (ours does).
- **The five acts on a real phone** ride in u5-9's package (Demo A's
  session); this run's phones are WebKit and Chromium on this host.
