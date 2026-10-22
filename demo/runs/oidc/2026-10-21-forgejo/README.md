# Forgejo through svrngs — what the run gave, 2026-10-21

Row oidc-forgejo (`ralph/next/oidc/STATE.md`; the trial's bar 1,
SIGN_IN_WITH_SVRNGS §7 1, §5a). The setup and predictions are
`PREDICTIONS.md`, committed (`c02568bc`) before the driver and before any
data. The first take is kept at `data-first-take/`: the driver read
Forgejo's landing page as its login page and timed out; that is the
driver's fault, not the app's. The take below follows the landing page's
own **Sign in** and reaches the app's account page on both engines. Every
person is a program; one host and one run are not a rate (AGENTS.md 7).

## How it ran

```
W=/tmp/oidc-forgejo bash demo/runs/oidc/2026-10-21-forgejo/run.sh
```

- **Forgejo 17.0.0**, the Homebrew bottle `forgejo--17.0.0.arm64_golden_gate`
  (sha256 `d997331b207cf62e274ab7d15cf668c242152d1146292491360f8053f48238f6`),
  binary sha256 `fb508a2fcf4411e7792284224f7be8ae9c41fa6f550926912b22b0aff33404f1`,
  on `127.0.0.1:18097` with its own work path and SQLite database. Stock
  settings: `ROOT_URL = http://localhost:18097/`,
  `[oauth2_client] ENABLE_AUTO_REGISTRATION = true`, `USERNAME =
  preferred_username`, `REGISTER_EMAIL_CONFIRM = false`, and the `svrngs`
  OIDC source made by Forgejo's own `admin auth add-oauth`
  (`data/forgejo.ini`, `data/forgejo-auth.txt`; secrets redacted).
- **The row's Podman premise is false on this Mac** (STATE.md `427f406a`):
  Podman's containers run in its own Linux machine, so their `127.0.0.1`
  is that machine's, not this computer's (SIGN_IN_WITH_SVRNGS §8 8), and
  the provider's one issuer string is
  `http://127.0.0.1:<api port>/oidc/<lend>`
  (`crates/svrngs-node/src/oidc/provider.rs:62-65`). The design's path for
  a Mac is the app outside a container; the run uses the pinned bottle and
  **no Docker**. `data/podman-probe.txt` keeps the container's side: its
  host-network `127.0.0.1` refuses the API, and only
  `host.containers.internal:<api>` reaches it (with the caller read as
  loopback) — an address the one issuer string cannot name.
- Kim's computer made `House`; Alex, Bo and Cy joined. Kim lent Forgejo
  with `svrngs share 18097 --name forgejo --kind forgejo
  --sign-in-with-svrngs --limit 8`; the issuer was
  `http://127.0.0.1:51314/oidc/forgejo`, the client id
  `X7-pSH_e9oPljOAJ3V-MeZqfn4sYYbfC` (`data/share.out`; the secret is
  redacted). Alex was Chromium 147.0.7727.15; Bo was WebKit 26.4 as an
  iPhone. The drive ran 05:57:21 to 05:59:52, 151 s, the 1-minute load
  1.88 at the start and 1.60 at the end (`data/steps.log`).

## Against the predictions

`data/verdicts.json` is the judge's; the numbers are from `data/raw.json`.

| | predicted | as it ran | verdict |
|---|---|---|---|
| 1 | Alex in as himself, nothing typed | **failed**: the lend opened in 75 ms, **Sign in** brought Forgejo's login page in 3 ms with **Sign in with svrngs**, one press, no typing (0 local sign-in posts) — then `/user/link_account`, the app's **"Complete new account — Username, Email address"** page; no Forgejo user was made | failed |
| 2 | Bo on WebKit in the same way | **failed**: the phone's page listed "Kim’s forgejo", the lend opened in 2,270 ms, the login page in 11 ms with the same source, one press, no typing — then `/user/link_account` with **"Complete new account — Username, Email address — Complete account"**; no Forgejo user | failed |
| 3 | two people stay apart | **could-not-judge**: no two accounts happened; Forgejo's own list holds only `kim` (`{"id":1,"login":"kim"}`) | could-not-judge |
| 4 | a removed member is refused before any sign-in | **passed**: Cy's open was refused in 5 ms with "You are no longer in this group, so nothing of it is open to you."; Forgejo's login page was not offered; Forgejo's users stayed `kim` with no `Cy-`; Kim's door answered 2 authorize asks before the removal and 2 after, so it minted Cy no code | passed |
| 5 | a code cannot be made to name another | **passed**: Bo's browser asked with `sub=alex` and forged `x-svrngs-key`/`x-svrngs-person`; the code redeemed at 200 with `sub` Bo's key, `preferred_username` `Bo-db4c8c751d306c70`, `name` `Bo`, `groups` `["member"]`, nonce `n-planted`, `iss` the issuer and `aud` the client; the same code again answered 400 `invalid_grant` | passed |

The browser stayed on the lend's own origin through both sign-in attempts
(`data/raw.json`, `stayed` true); the door's redirects were the relative
paths it makes. The **canonical-URL warning** Forgejo shows is expected:
`ROOT_URL` is the address the door asks it at, `localhost:18097`, while
the browser sees `<label>.localhost:<api port>`. It did not stop the
OIDC round trip; the flow reached Forgejo's callback and stopped at its
account page.

## What it found

**Forgejo 17.0.0 cannot auto-register a person svrngs sends without an
email.** Its OIDC callback creates the account only when the OIDC user has
an email:

```
routers/web/auth/oauth.go:1119-1127 (tag v17.0.0)
    if gothUser.Email == "" {
        missingFields = append(missingFields, "email")
    }
    ...
    if len(missingFields) > 0 {
        // we don't have enough information to create an account automatically,
        // so we prompt the user for the remaining bits
        showLinkingLogin(ctx, gothUser)
```

The provider sends no `email` (SIGN_IN_WITH_SVRNGS §8 4; the `share` line
says "it tells the app no email address, because svrngs has none"), so the
callback answers `/user/link_account` and asks the person to type a
username and an email. That is the trial's bar 1 **failed**, not a host
fault: Forgejo runs, the OIDC round trip works, and the app's own stock
client refuses to finish without a claim svrngs does not have. §5a's
table put Forgejo among the apps served without an email; the run
falsifies that row for 17.0.0. The design's own rule (§8 4) is that no
`email` is sent until `MAIL.md` is built, so Forgejo waits on `MAIL.md`
with Grafana, Open WebUI and Mealie. The next step is the operator's
(`NEEDS_HUMAN.md`).

**The first take was the driver's, not the app's.** It read Forgejo's
landing page as the login page and timed out looking for the source; that
data is kept (`data-first-take/raw.json`: "no sign-in... Explore Help
Register Sign in"). The landing page's own **Sign in** goes to
`/user/login?redirect_to=%2f`, whose page carries **Sign in with
svrngs**; the driver now follows that address, which types nothing.

## The close

- **No Docker**: the app is the pinned Homebrew bottle; the only
  containers are the probe's, and they run no app (`data/podman-probe.txt`).
- **The trust floor is unchanged** (`data/setup.json`).
- **`scripts/lean_check.py`**: svrngs-core **3,152 of 3,153**, the
  repository **17,421 of 17,423** (`data/lean.txt`), both inside the
  operator's 17,728 and 3,153.
- **`scripts/check.sh`** runs at this close under `ralph-result await`;
  its log is the session's `oidc-forgejo-*.await.log`. The run adds no
  product code, only `demo/runs/`, which the budget excludes.

## What the operator has to decide

Forgejo's stock client needs an `email` claim; the design says no email
until `MAIL.md`. Either Forgejo waits on `MAIL.md` — the trial's second
app is not served, and `oidc-retire` has nothing to retire for it — or the
no-email rule changes. `NEEDS_HUMAN.md` holds the question, what was
tried, and the evidence.
