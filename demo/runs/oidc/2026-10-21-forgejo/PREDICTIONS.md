# Forgejo through svrngs — setup and predictions, 2026-10-21

Row oidc-forgejo (`ralph/next/oidc/STATE.md`). The trial's bar 1
(SIGN_IN_WITH_SVRNGS §7 1, §5a): Forgejo is the app whose stock settings
name an OpenID Connect issuer, and a member who opens it through the group
is in as themselves with nothing typed. This is written before the driver
and before any data; `README.md` here will say what the run gave against
each line.

**The row's Podman premise is false on this Mac**, and the state file says
so (427f406a): Podman's containers run in its own Linux machine, so
`127.0.0.1` there is that machine's, not this computer's
(SIGN_IN_WITH_SVRNGS §8 8, `rings/current/SIGN_IN_WITH_SVRNGS.md:340-348`),
while the provider's one issuer string is
`http://127.0.0.1:<api port>/oidc/<lend>`
(`crates/svrngs-node/src/oidc/provider.rs:62-65`) and Forgejo's own server
must reach it. The design's path for a Mac is the app outside a container.
Forgejo 17.0.0 is run from the Homebrew bottle, pinned by sha256, and no
Docker is used. The container route is kept in `data/podman-probe.txt` as
the reason: from a container on this host, `127.0.0.1:<api>` refuses, and
only `host.containers.internal:<api>` reaches the API (with the caller
read as loopback), an address the one issuer string cannot name.

`README.md` will say what each of the five bars gave, and every number in
it is in `data/raw.json`, `data/verdicts.json` or `data/setup.json`.
One host and one run of each are not a rate (AGENTS.md 7). Every person is
a program, so no people bar is judged.

## 1. Setup

`run.sh` writes its data to `data/`; `W` is the scratch dir (else
`/tmp/oidc-forgejo`). `scripts/check.sh` runs none of this: it needs
Forgejo and the browsers.

- **Forgejo 17.0.0**, the Homebrew bottle for arm64 (bottle
  `forgejo--17.0.0.arm64_golden_gate.bottle.tar.gz`, sha256
  `d997331b207cf62e274ab7d15cf668c242152d1146292491360f8053f48238f6`),
  its binary sha256
  `fb508a2fcf4411e7792284224f7be8ae9c41fa6f550926912b22b0aff33404f1`,
  on `127.0.0.1:18097` with its own work path and SQLite database.
  `run.sh` reads the version and hash back before using it.
- **Forgejo's stock settings** (§5a): `ROOT_URL =
  http://localhost:18097/` (the lend's address the door shows it),
  `[oauth2_client] ENABLE_AUTO_REGISTRATION = true`, `USERNAME =
  preferred_username`, `REGISTER_EMAIL_CONFIRM = false`, and the
  `svrngs` OIDC source made by Forgejo's own `admin auth add-oauth`
  with the issuer, client id and secret `share` prints. The lender fills
  this form once; no person types anything.
- **svrngs** at this commit, `cargo build -p svrngs`. Kim's computer makes
  the group `House` on the page served by `web/tests/serve.mjs` on
  `localhost`; Alex, Bo and Cy join by the client link, each with a
  `HOME` and a data dir of its own. Kim lends Forgejo from Kim's computer
  with `svrngs share 18097 --name forgejo --kind forgejo
  --sign-in-with-svrngs --limit 8`.
- **The page.** `http://localhost:<page port>/`; a computer opens Kim's
  lend at its own door's address, `<label>.localhost:<api port>`, and a
  phone opens the group's device link and then "Kim’s forgejo" from the
  list, as a phone does (`web/lend.js`). Both are this host's loopback,
  as in `demo/runs/guests/2026-10-20-run/`.

## 2. The cast and the story

1. Kim's computer makes `House`, and Kim lends Forgejo with
   `--sign-in-with-svrngs`; `share` prints the issuer, the client id and
   the secret. The driver tells Forgejo's own auth source and switches
   auto-registration on.
2. **Alex, another computer (Chromium).** Alex opens `Kim’s forgejo` at
   Alex's own computer's address. Forgejo's login page offers **Sign in
   with svrngs**. One press: the browser goes through Forgejo, the door
   answers the authorize path, and Forgejo signs Alex in. Alex is on
   Forgejo as Alex's own account, having typed nothing.
3. **Bo, a phone (WebKit as an iPhone).** Bo opens Bo's device link, then
   "Kim’s forgejo", then **Sign in with svrngs**. Bo is in as Bo's own
   account, having typed nothing.
4. **Cy, a member who is removed.** Cy joins and runs, and Kim removes Cy.
   Cy then opens Kim's lend. The door refuses before Forgejo is asked.
5. **A planted ask.** Bo's browser asks the authorize path with
   `sub=alex` in the query and forged `x-svrngs-key`/`x-svrngs-person`
   headers naming Alex. The door must mint the code for Bo, the asker; the
   code, redeemed at the token route, names Bo and never Alex, and a
   second redemption of it is refused.

## 3. Predictions

1. **Alex is in as himself, having typed nothing.** Within 60 s of opening
   the lend: Forgejo's login page offers the lender's **Sign in with
   svrngs**; after one press, Forgejo's home renders signed in. The
   browser stays on the lend's own origin (`<label>.localhost:<api>`), and
   Forgejo's own `/api/v1/user` names a login beginning `Alex-` and ending
   in the first 16 hex of Alex's svrngs key. Nothing is typed in Forgejo.
2. **Bo's phone is in the same way.** The group's page lists "Kim’s
   forgejo"; one press in Forgejo, no typing, and Forgejo's
   `/api/v1/user` names a login beginning `Bo-`.
3. **Two people stay apart.** Alex's and Bo's Forgejo logins are two
   distinct accounts, each `/api/v1/user` names its own login, and
   Forgejo's admin user list holds both. Neither is the other.
4. **A removed member is refused before any sign-in.** After Kim's
   removal, Cy's request to the lend is refused by the door with words Cy
   can act on, and Forgejo's login page is never shown. Forgejo's user
   list gains no `Cy-` account, and the door mints Cy no code (Kim's log's
   authorize answers do not rise).
5. **A code cannot be made to name another.** The planted ask (Bo's
   browser, `sub=alex`, Alex's forged headers) answers a code whose token
   names Bo's `sub`, `preferred_username` and `name`, not Alex's. The
   second redemption of that same code is refused `invalid_grant`.
6. **The trust floor is unchanged** (`scripts/trust_floor.txt`) and
   `scripts/lean_check.py` stays within 17,728 and 3,153. No new listener:
   the authorize path is on the door's existing origin and the four
   provider routes on the record API's. No Docker is used.

## 4. Verdicts

Each bar is `passed`, `failed`, `could-not-judge` or `never-ran`
(AGENTS.md 6), judged from `data/raw.json` by `judge.py` into
`data/verdicts.json`. A bar whose evidence did not happen is
`could-not-judge`, never a pass. If Forgejo cannot be installed or run
here, the run says could-not-judge and stops for the operator, as the row
says.

**What would falsify these:** Forgejo's login page writes the authorize
address or the callback as an absolute `http://localhost:18097/…` in a
body or a link the browser follows, so the browser leaves the lend's
origin and the door is bypassed (the catalogue census's `body-url` risk,
`demo/runs/catalogue/2026-10-05/CENSUS.md`); auto-registration refuses
without an `email` claim; the callback's state cookie is not carried by
the door; or Forgejo's OIDC client rejects the ES256 token or the plain
http issuer.
