# NEEDS_HUMAN — Jellyfin's SSO plugin cannot complete through the door's own origin

**Question.** The stock Jellyfin SSO plugin 4.0.0.4's browser half builds
the app's addresses from the `Host` the door sets (`localhost:<lend port>`),
while the browser is on the lend's own origin (`<label>.localhost:<api
port>`); the callback page waits for `localStorage` the cross-origin iframe
can never write, and bars 1–3 fail. Meeting it needs one of the four forks
at the end, and none is the row's or the design's to take. Which?

The row is `oidc-jellyfin` (`ralph/next/oidc/STATE.md`), its run at
`demo/runs/oidc/2026-10-21-jellyfin/` (README, PREDICTIONS, data).

## What was built and run

- Jellyfin **10.11.11**, arm64 tarball, sha256
  `27c0bcbeac7a698cf0fd034e347f124f69083bc4463816099b52bebead614a47`;
  SSO plugin **4.0.0.4**, sha256
  `c09f16ba31059a434ddd7f811e4f9608d4b4c4514cc80a5bf1ca33bee61e1107`.
- Kim lent Jellyfin with `svrngs share 18096 --name jellyfin --kind
  jellyfin --sign-in-with-svrngs`; the plugin was configured with the
  issuer, client id, secret, `DoNotValidateEndpoints` and `DisableHttps`,
  and its login button put in Jellyfin's Branding "Login disclaimer".
- Alex (Chromium, another computer), Bo (WebKit as an iPhone, a device
  link), Cy (a member Kim removed). `data/verdicts.json`:

```
alex     failed   no sign-in: page.waitForFunction: Timeout 60000ms exceeded.; the page says: Logging in...; the account is None, not Alex-*; the film did not play: None
bo       failed   no sign-in: page.waitForFunction: Timeout 60000ms exceeded.; the page says: Logging in... ; the account is None, not Bo-*; the film did not play: None
apart    failed   Alex None (None by token), Bo None (None by token); distinct users False, distinct tokens False
removed  passed   refused in the door's words: 'You are no longer in this group, so nothing of it is open to you.'; no Cy- user; code minted: False
planted  passed   the code named 'Bo-c4b05374b122cfa9' ('Bo'), not Alex; a second redemption answered 400 invalid_grant
```

- The trust floor is unchanged; `scripts/lean_check.py` passed
  (17,374/17,375 and 3,152/3,153). `scripts/check.sh` is never-ran: this
  session stopped here for the operator.

## The evidence of the cause

The browser is on the lend's own origin; the plugin's callback body names
the app's `localhost:<lend port>` everywhere. `data/raw.json`,
`people.alex.callback`:

```
url         http://<label>.localhost:60811/sso/OID/redirect/svrngs?code=…&state=…
iframe      http://localhost:18096/web/index.html
absolute    http://localhost:18096/sso/OID/Link/svrngs/
            http://localhost:18096/web/index.html
            http://localhost:18096/sso/OID/Auth/svrngs
stored      ["_deviceId2"]
credentials null
```

The code's own lines:

- `SSO-Auth/WebResponse.cs:464` — the iframe `src` is
  `punycodeBaseUrl + "/web/index.html"`, an absolute address.
- `SSO-Auth/WebResponse.cs:466-470` — the page loops until **its own**
  `localStorage` holds `jellyfin_credentials`; the iframe writes the other
  origin's.
- `SSO-Auth/WebResponse.cs:506` — `window.location.replace(punycodeBaseUrl
  + "/web/index.html")`, absolute again.
- `SSO-Auth/Api/SSOController.cs:1283-1313` — `GetRequestBase` takes
  `Host = Request.Host.Host`; `SchemeOverride` and `PortOverride` change
  the scheme and port only, never the host.
- `crates/svrngs-node/src/door.rs:1498` — the door sets `Host` to
  `own_address(port)` (`:744`), `localhost:<lend port>`, for every request
  it carries to the app.
- The browser's origin is the lend's own (`crates/svrngs-node/src/origin.rs`,
  its module doc): `<label>.localhost:<api port>`. The door rewrites a
  `Location` into a path (`crates/svrngs-node/src/door.rs:774`, `as_path`)
  and rewrites no body (SIGN_IN_WITH_SVRNGS §5).

The server half is not the problem: the plugin's token exchange reached
the provider (the first run's only refusal was the run's own secret parse,
the sentence's final period; fixed in `run.sh`), and the planted bar's
redemption at `data/raw.json` `bars.planted.claims` is the right person:

```
sub c4b05374b122cfa91d1e8877b0debd74bc96037f1df25790d5c8da07f2bbe41d
preferred_username Bo-c4b05374b122cfa9   name Bo   groups ["member"]
iss http://127.0.0.1:61815/oidc/jellyfin   aud 0-H7kAdosoIBuPd0vlQMRcu7Wi17Qbs-
```

On this host the "phone" is WebKit on the lender's computer, so the
absolute `localhost:18096` would still answer there; what stops it is the
two origins, not the phone. On a real phone the absolute address would be
the phone's own machine as well.

## What was tried

- The settings §5a names: `DoNotValidateEndpoints=true` and
  `DisableHttps=true`, and the lender's http rule. Both applied; the plugin's discovery, the
  authorize ask and the server-side token exchange all work.
- The lender's SSO button in the Branding login disclaimer, as the
  plugin's README says. The button came and was pressed.
- `SchemeOverride`/`PortOverride` cannot set the host (source above), so
  they cannot make the callback's origin the lend's.
- Bo's phone as a device link (`svrngs link House --device`), so no name
  is typed on the phone either; it reaches the same callback and hangs.

## The four forks

1. **Pass the browser's Host to the app for this lend** (or every lend).
   The plugin's callback, iframe and final redirect would then be on the
   lend's own origin, and the flow completes. But the door's per-lend
   origin is u5-2's isolation: one lend's cookies and storage cannot be
   read by another (`crates/svrngs-node/src/origin.rs:1-20`). This is a
   design and trust change, not this row's.
2. **Rewrite the app's body addresses.** SIGN_IN_WITH_SVRNGS §5 says the
   door rewrites no address in a body; doing it here is written in
   Jellyfin's words (fence 6).
3. **An adapter in svrngs for Jellyfin's callback.** Fence 6 and the
   charter's "an app-specific tweak" leave this to the operator; it is
   what the trial exists to avoid.
4. **A different pinned plugin whose callback uses relative addresses, or
   Jellyfin dropped from the trial.** `oidc-forgejo` depends on this row;
   dropping Jellyfin moves the trial's first app to Forgejo, which the
   operator's 2026-10-21 ordering put second.

## Reproduce

```
W=/tmp/oidc-run bash demo/runs/oidc/2026-10-21-jellyfin/run.sh
python3 demo/runs/oidc/2026-10-21-jellyfin/judge.py demo/runs/oidc/2026-10-21-jellyfin/data
```

The run needs this host's arm64 Jellyfin tarball signed ad-hoc (the
`run.sh` does it; the tarball's apphost is unsigned) and Playwright's
Chromium and WebKit (`web/node_modules`). Playwright is installed here.

## What would falsify the cause

A plugin setting or a release that makes the callback page's addresses
relative (or the host configurable); or a door change that passes the
browser's Host for the lend. Either would let bars 1–3 pass; this run's
`data/raw.json` is the evidence against both at HEAD.
