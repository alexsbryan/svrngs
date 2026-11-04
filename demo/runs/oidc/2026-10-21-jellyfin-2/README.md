# Jellyfin through its SSO plugin — what the second take gave, 2026-10-21

Row oidc-jellyfin (`ralph/next/oidc/STATE.md`). The first take,
`../2026-10-21-jellyfin/`, failed bars 1-3 and is kept as it ran. The
director decided the general fork in the standard reverse-proxy form
(d0c416e4): the door tells a lend's service the browser's origin in
`X-Forwarded-Host`/`-Proto`, the lender sets Jellyfin's stock
`KnownProxies`, and the front channel accepts the lend's own origin as the
redirect (`1efa9a2d`, its cap `bbef1ba2`). The setup and predictions are
`PREDICTIONS.md`, committed (`e9810f84`) before the driver and before any
data. Every person is a program; one host and one run are not a rate
(AGENTS.md 7).

## How it ran

```
W=/tmp/oidc-run bash demo/runs/oidc/2026-10-21-jellyfin-2/run.sh
```

- **Jellyfin 10.11.11**, the pinned arm64 tarball, on `127.0.0.1:18096`;
  **the SSO plugin v4.0.0.4** in its `data/plugins/SSO-Auth/`. Both hashes
  are in `data/setup.json`; the run checks them before using either.
- On this arm64 host the tarball's apphost and dlls carry no code
  signature, which the OS refuses (`Killed: 9`). `run.sh` signs the
  directory ad-hoc (`codesign --force --deep --sign -`), changing no code.
- **The lender's setting this take adds:** Jellyfin's stock
  `KnownProxies` = `127.0.0.1`, in its own `network.xml` before the server
  starts, read back after Jellyfin's setup rewrote it (`data/network.xml`).
  The plugin's other settings are §5a's: `DoNotValidateEndpoints`,
  `DisableHttps`, and the login button in Jellyfin's Branding "Login
  disclaimer" (`data/plugin-settings.json`).
- Kim's computer made `House` on the page at `localhost`; Alex, Bo and Cy
  joined. Kim lent Jellyfin with `svrngs share 18096 --name jellyfin
  --kind jellyfin --sign-in-with-svrngs` at `svrngs` 6b8374b2
  (`data/steps.log`); the issuer was `http://127.0.0.1:51675/oidc/jellyfin`
  and the client id `_t6ytzNOllAo64NqDiu7B2Y20JVIWuv4`
  (`data/share.out`).
- Alex (Chromium 147.0.7727.15) opened Kim's lend at his own computer's
  address; Bo (WebKit 26.4 as an iPhone) opened a device link made on Bo's
  computer and then "Kim’s jellyfin"; Cy joined and Kim removed Cy before
  Cy opened the lend. The drive ran 04:54:56-04:55:13 with the load at
  3.00 and then 2.78 (`data/steps.log`).

## Against the predictions

`data/verdicts.json` is the judge's; the numbers are from `data/raw.json`.

| | predicted | as it ran | verdict |
|---|---|---|---|
| 1 | Alex in as himself, nothing typed, film plays | **passed**: the lend opened in 130 ms, the login page came in 342 ms with **Sign in with SSO**, one press, sign-in 490 ms after it, no typing, as `Alex-57a5060242a2e5fc`; the film played in 1,378 ms | passed |
| 2 | Bo on WebKit in the same way | **passed**: the device link joined Bo in 2,110 ms, the lend opened in 4,296 ms, the login page came in 1,308 ms, one press, sign-in 876 ms after it, no typing, as `Bo-c6b4506b0b341c4b`; the film played in 4,607 ms | passed |
| 3 | two people stay apart | **passed**: `Alex-57a5060242a2e5fc` and `Bo-c6b4506b0b341c4b` are distinct users and distinct tokens, and each token's `/Users/Me` names its own account | passed |
| 4 | a removed member is refused before any sign-in | **passed**: Cy's open was refused in 6 ms with "You are no longer in this group, so nothing of it is open to you."; Jellyfin's login page was not offered; Jellyfin's `/Users` stayed `kim`, `Alex-57a5060242a2e5fc`, `Bo-c6b4506b0b341c4b` with no `Cy-` user; Kim's door answered 2 authorize asks before the removal and 2 after, so it minted Cy no code | passed |
| 5 | a code cannot be made to name another | **passed**: Bo's browser asked with `sub=alex` and forged `x-svrngs-key`/`x-svrngs-person`; the code redeemed at 200 with `sub` Bo's key, `preferred_username` `Bo-c6b4506b0b341c4b`, `name` `Bo`, `groups` `["member"]`, nonce `n-planted`, and the same code again answered 400 `invalid_grant` | passed |

## The data, and every number above

- `data/raw.json`: the engines, each step's ms, the two users and their
  `/Users/Me` answers, the film's play, Cy's words and the users before and
  after, the planted ask and its claims, and each browser's log. The two
  access tokens the driver held to read `/Users/Me` are redacted
  (`"redacted"` in the file): a token stays in the lender's svrngs and on
  the wire only to the app (SIGN_IN_WITH_SVRNGS §3 4), and the answer each
  read is kept in `people.<who>.me`.
- `data/verdicts.json`: the judge's four verdicts for the five bars.
- `data/network.xml`: the lender's `KnownProxies` =
  `<string>127.0.0.1</string>`, read back from Jellyfin's config.
- `data/setup.json`: the two pinned hashes and versions, the trust floor's
  verdict, and `data/lean.txt`'s reading.
- `data/share.out`, `data/plugin-settings.json`, `data/steps.log`,
  `data/ffmpeg.txt`, `data/film.txt`, `data/jellyfin-log/`: the lender's
  words, the plugin's settings, the run's timeline, and the app's log.

## What it found

**The forwarded origin is what the plugin's browser half needed.** With
`KnownProxies` naming this computer, the plugin's callback page builds its
`redirect_uri`, iframe and `location.replace` from `X-Forwarded-Host` and
`-Proto` — the lend's own origin — so the callback page and its iframe are
one origin and the stored sign-in is written. Alex and Bo are signed in on
the first press with nothing typed, and the two accounts stay apart.

**The phone's worker was telling the door it was `svrngs`.** A browser
hides `Host` from a worker's request headers, and `web/src/http.rs` used
its `UNNAMED_HOST` fallback, so Jellyfin built Bo's callback as
`https://svrngs/web/index.html`. That is an instrument of the phone, not
Jellyfin's words: `web/lend.js` now names the address the browser asked
(34596567). The earlier attempt of this take that found it had Alex
passing and Bo failing; the commit body holds its values, and the recorded
run is after the fix.

**The driver raced the callback page's own navigation.** An attempt read
the stored sign-in between the plugin's localStorage write and its
`location.replace`, and lost the context ("Execution context was
destroyed"); it now reads it on `/web/` after the app's own navigation
(6b8374b2). The bar and the judge are unchanged.

Found and not a bar: `share` still prints both "jellyfin asks each person
to sign in once, with an account you make them" and "It signs people in
with svrngs"; the first line is false for this lend, as in the first
take's README.

## The close

`scripts/check.sh` **passed** every step at this tree: principles pinned,
lean and RUST_ARCH, trust floor, the app-noun gates, rustfmt, clippy,
tests, canaries, the ceiling canary, the web's rustfmt and clippy, the
browser, joining in a browser, the conformance canary, and the app laws.
The trust floor is **unchanged** (`data/setup.json`), and
`scripts/lean_check.py` reads the repository **17,422 of 17,423** and
svrngs-core **3,152 of 3,153** (`data/lean.txt`), both inside the
operator's 17,728 and 3,153.
