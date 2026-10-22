# Jellyfin through its SSO plugin — setup and predictions, 2026-10-21

Row oidc-jellyfin (`ralph/next/oidc/STATE.md`). Kim lends a real Jellyfin
from this host with `svrngs share 18096 --sign-in-with-svrngs`; the
Jellyfin SSO plugin is the stock client (SIGN_IN_WITH_SVRNGS §5a, §7 2).
Alex, on another computer, and Bo, on a phone, open it through the group
and are in as themselves, having typed nothing in Jellyfin. This is
written before the driver and before any data. `README.md` here will say
what the run gave against each line. One host and one run are not a rate
(AGENTS.md 7). Every person is a program, so no people bar is judged.

## 1. Setup

`run.sh` writes its data to `data/`; `W` is the scratch dir (else
`/tmp/oidc-run`). `scripts/check.sh` runs none of this: it needs Jellyfin
and the plugin.

- **Jellyfin 10.11.11**, the arm64 macOS tarball
  `https://repo.jellyfin.org/files/server/macos/stable/v10.11.11/arm64/jellyfin_10.11.11-arm64.tar.xz`
  (sha256 `27c0bcbeac7a698cf0fd034e347f124f69083bc4463816099b52bebead614a47`),
  run on `127.0.0.1:18096` with its own data, config, cache and log dirs
  and one 60 s film. The server is pinned by the tarball's hash, the film
  by the ffmpeg command in `run.sh`.
- **The Jellyfin SSO plugin, release v4.0.0.4**,
  `https://github.com/9p4/jellyfin-plugin-sso/releases/download/v4.0.0.4/sso-authentication_4.0.0.4.zip`
  (sha256 `c09f16ba31059a434ddd7f811e4f9608d4b4c4514cc80a5bf1ca33bee61e1107`),
  installed in `<Jellyfin data>/plugins/SSO-Auth/` before the server starts,
  so Jellyfin's own loader reads it.
- **The lender's plugin settings**, as §5a names them: the issuer, the
  client id and secret `share` prints, `DoNotValidateEndpoints` (the
  authorize address and the token address are on two ports), and
  `DisableHttps` (its http rule, for `localhost` and `127.0.0.1`). The
  lender also puts the plugin's own login button in Jellyfin's Branding
  "Login disclaimer" (`/sso/OID/start/svrngs`), as the plugin's README
  says; it is the one setting the lender types, and no person types
  anything. Set through Jellyfin's own API, with the lender's admin token.
- **svrngs** at this commit, `cargo build -p svrngs`. Kim's computer makes
  the group `House` on the page served by `web/tests/serve.mjs` on
  `localhost`; Alex, Bo and Cy join by the client link, each with a `HOME`
  and a data dir of its own. Kim lends Jellyfin from Kim's computer.
- **The page.** `http://localhost:<page port>/`; a computer opens Kim's
  lend at its own door's address, `<label>.localhost:<api port>`, and a
  phone opens the group's link and then "Kim’s jellyfin" from the list, as
  a phone does (`web/lend.js`). Both are this host's loopback.

## 2. The cast and the story

1. Kim's computer makes `House`, and Kim lends Jellyfin with
   `--sign-in-with-svrngs`; `share` prints the issuer, the client id and
   the secret. The driver configures the plugin with them and puts the
   login button in Jellyfin's Branding disclaimer.
2. **Alex, another computer (Chromium).** Alex opens `Kim’s jellyfin` at
   Alex's own computer's address. Jellyfin's login page shows the lender's
   **Sign in with SSO**. One press: the browser goes through the plugin,
   the door answers the authorize path, and the plugin signs Alex in.
   Alex is on Jellyfin's home as Alex's own account, and plays the film.
3. **Bo, a phone (WebKit as an iPhone).** Bo opens the group's link, gives
   "Bo" on svrngs's own page (the group's join, not Jellyfin), presses
   "Kim’s jellyfin" in the list, then **Sign in with SSO**. Bo is in as
   Bo's own account and plays the film.
4. **Cy, a member who is removed.** Cy joins and runs, and Kim removes Cy.
   Cy then opens Kim's lend. The door refuses before Jellyfin is asked.
5. **A planted ask.** Bo's browser asks the authorize path with
   `sub=alex` in the query and forged `x-svrngs-key`/`x-svrngs-person`
   headers naming Alex. The door must mint the code for Bo, the asker; the
   code, redeemed at the token route, names Bo and never Alex, and a
   second redemption of it is refused.

## 3. Predictions

1. **Alex is in as himself, having typed nothing.** Within 60 s of opening
   the lend: Jellyfin's login page offers the lender's **Sign in with
   SSO**; after one press, Jellyfin's home renders with the film. The
   browser's stored sign-in names a Jellyfin user whose name begins
   `Alex-` and ends in the first 16 hex of Alex's svrngs key. Nothing was
   typed in Jellyfin. The film plays.
2. **Bo's phone is in the same way.** The group's page lists "Kim’s
   jellyfin"; one press in Jellyfin, no typing, and the stored sign-in
   names a user beginning `Bo-`. The film plays in WebKit.
3. **Two people stay apart.** Alex's and Bo's Jellyfin users are two
   distinct accounts, each browser's token is its own account's, and
   Jellyfin's `/Users` names both, each by its own person. Neither token
   opens the other's account (`/Users/Me` under each token names the same
   account the browser stores).
4. **A removed member is refused before any sign-in.** After Kim's
   removal, Cy's request to the lend is refused by the door with words Cy
   can act on, and Jellyfin's login page is never shown. Jellyfin's
   `/Users` gains no `Cy-` account, and the Jellyfin log for the window
   after the removal holds no request the door made for Cy. So no code is
   minted and no sign-in reaches the app.
5. **A code cannot be made to name another.** The planted ask (Bo's
   browser, `sub=alex`, Alex's forged headers) answers a code whose token
   names Bo's `sub`, `preferred_username` and `name`, not Alex's. The
   second redemption of that same code is refused `invalid_grant`. The
   identity in the ask is never the code's person.
6. **The trust floor is unchanged** (`scripts/trust_floor.txt`) and
   `scripts/lean_check.py` stays within 17,728 and 3,153. No new listener:
   the authorize path is on the door's existing origin and the four
   provider routes on the record API's.

## 4. Verdicts

Each bar is `passed`, `failed`, `could-not-judge` or `never-ran`
(AGENTS.md 6), judged from `data/raw.json` by `judge.py` into
`data/verdicts.json`. A bar whose evidence did not happen is
`could-not-judge`, never a pass. If Jellyfin or the plugin cannot be
installed or run here, the run says could-not-judge and stops for the
operator, as the row says.
