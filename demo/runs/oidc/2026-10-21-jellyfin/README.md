# Jellyfin through its SSO plugin — what the run gave, 2026-10-21

Row oidc-jellyfin (`ralph/next/oidc/STATE.md`). The setup and predictions
are `PREDICTIONS.md`, committed (a6403b64) before the driver and before any
data. The driver and its judges were committed next, then this data. Every
person is a program; one host and one run are not a rate (AGENTS.md 7).

## How it ran

```
W=/tmp/oidc-run bash demo/runs/oidc/2026-10-21-jellyfin/run.sh
```

- **Jellyfin 10.11.11**, the pinned arm64 tarball, on `127.0.0.1:18096`;
  **the SSO plugin v4.0.0.4** in its `data/plugins/SSO-Auth/`. Both hashes
  are in `data/setup.json`; the run checks them before using either.
- On this arm64 host the tarball's apphost and dlls carry no code
  signature, which the OS refuses (`Killed: 9`). `run.sh` signs the
  directory ad-hoc (`codesign --force --deep --sign -`), changing no code;
  without it the server is killed as the session's first attempt was.
- Kim's computer made `House` on the page at `localhost`; Alex, Bo and Cy
  joined. Kim lent Jellyfin with `svrngs share 18096 --name jellyfin
  --kind jellyfin --sign-in-with-svrngs`, and the plugin was configured
  through Jellyfin's own API with the issuer, client id and secret `share`
  printed, `DoNotValidateEndpoints`, `DisableHttps`, and the login button
  in Jellyfin's Branding "Login disclaimer" (`data/plugin-settings.json`,
  `data/share.out`).
- Alex (Chromium) opened Kim's lend at his own computer's address. Bo
  (WebKit as an iPhone) opened a device link made on Bo's computer
  (`svrngs link House --device --origin <page>`), which joins as Bo, asked
  no name, then "Kim’s jellyfin" from the list. Cy joined and Kim removed
  Cy before Cy opened the lend.

## Against the predictions

`data/verdicts.json` is the judge's; the numbers are from `data/raw.json`.

| | predicted | as it ran | verdict |
|---|---|---|---|
| 1 | Alex in as himself, nothing typed, film plays | **failed**: the app's login page and its **Sign in with SSO** came (127 ms to the app, 313 ms to the button), one press, and then the plugin's callback page stayed on "Logging in..." for 60,000 ms. No Jellyfin account, no film | failed |
| 2 | Bo on WebKit in the same way | **failed**: the device link joined Bo in 2,219 ms and the lend opened in 4,204 ms, the SSO button came, one press, then the same "Logging in..." for 60,000 ms | failed |
| 3 | two people stay apart | **failed**: neither signed in, so there are no two accounts to keep apart | failed |
| 4 | a removed member is refused before any sign-in | **passed**: Cy's open was refused in 5 ms with "You are no longer in this group, so nothing of it is open to you."; Jellyfin's login page was not offered; no `Cy-` user was made; Kim's door minted no code | passed |
| 5 | a code cannot be made to name another | **passed**: Bo's browser asked with `sub=alex` and forged `x-svrngs-key`/`x-svrngs-person`; the code redeemed at 200 with `sub` `c4b05374…`, `preferred_username` `Bo-c4b05374b122cfa9`, `name` `Bo`, `groups` `["member"]`, and the same code again answered 400 `invalid_grant` | passed |

`scripts/check.sh` was **never-ran** this session: the row stopped for the
operator at the fork below, and the next session's close is where it runs.
The trust floor is **unchanged** (`scripts/trust_floor.txt` not modified).
`scripts/lean_check.py` passed inside the caps it holds: the repository
17,374 of 17,375 and svrngs-core 3,152 of 3,153 (`data/lean.txt`), both
under the operator's 17,728 and 3,153.

## What it found

**The plugin's browser half needs the app's own origin, and the door gives
each lend an origin of its own.** The callback page is Jellyfin's
`WebResponse.Generator` (plugin v4.0.0.4, `SSO-Auth/WebResponse.cs:464` and
`:506`). It builds every address from `GetRequestBase`
(`SSO-Auth/Api/SSOController.cs:1283-1313`), whose `Host` is
`Request.Host.Host` — and the door sets that Host to the app's own
`localhost:<lend port>` (`crates/svrngs-node/src/door.rs:1498`,
`own_address` at `:744`). The browser, however, is at
`<label>.localhost:<api port>` (`crates/svrngs-node/src/origin.rs`). The
callback page the browser gets at the lend origin then holds:

```
url         <label>.localhost:<api>/sso/OID/redirect/svrngs?code=…&state=…
iframe      http://localhost:18096/web/index.html
absolute    http://localhost:18096/sso/OID/Link/svrngs/
            http://localhost:18096/web/index.html
            http://localhost:18096/sso/OID/Auth/svrngs
stored      ["_deviceId2"]
credentials null
```

(`data/raw.json`, `people.alex.callback`.) The iframe is another origin, so
the `jellyfin_credentials` its `/web/` writes never appear on the page's;
the page waits for them for ever (`WebResponse.cs` `main`, its
`jellyfin_credentials` loop), and its final `location.replace` would leave
the lend origin anyway. The app's server half is sound: the token exchange
reached the provider and was answered (the first run's only refusal was the
run's own secret parse, the sentence's final period, fixed in `run.sh`),
and the planted bar's redemption got the right claims.

This is not a setting: `SchemeOverride` and `PortOverride` change the
scheme and the port, never the host. The door rewrites `Location` headers
into paths (`as_path`, `door.rs:774-788`) and rewrites no body
(SIGN_IN_WITH_SVRNGS §5); this hop is a body. On this host a phone is
WebKit in a viewport on the lender's computer, so the absolute
`localhost:18096` would still answer; it is the two origins, not the
phone, that stop it.

**The lender is told two contradictory things.** `share` prints both
"jellyfin asks each person to sign in once, with an account you make them"
(a known kind's accounts) and "It signs people in with svrngs" plus the
issuer and client (`data/share.out`). The first line is false for this
lend. Not a bar of this row; recorded for the turn-on's words.

## The fork for the operator

Meeting the plugin's browser half needs one of:

1. the door to pass the browser's Host to the app for this lend (or every
   lend), which undoes the per-lend origin that keeps one lend's cookies
   and storage from another's (u5-2; `origin.rs`'s own doc comment);
2. a body rewrite of the app's absolute addresses, which SIGN_IN_WITH_SVRNGS
   §5 refuses and which would be written in Jellyfin's words;
3. an adapter in svrngs for Jellyfin's callback, which fence 6 refuses;
4. a different, pinned Jellyfin SSO plugin whose callback page uses
   relative addresses, or Jellyfin dropped from the trial.

The run's evidence and this question are `NEEDS_HUMAN.md` beside this
README. `oidc-forgejo` depends on this row; it is not run here.
