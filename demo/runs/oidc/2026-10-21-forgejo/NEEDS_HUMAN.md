# Needs the operator — Forgejo cannot auto-register without an email

**Question.** Forgejo 17.0.0's stock OIDC client creates an account only
when the OIDC user carries an `email` claim; svrngs sends none by design.
Does Forgejo wait on `MAIL.md` (the trial's second app is not served, and
`oidc-retire` has nothing to retire for it), or does the no-email rule
change?

Row `oidc-forgejo` (`ralph/next/oidc/STATE.md`), whose stop clause is
"if Forgejo cannot run here, say could-not-judge and stop for the
operator". The trial's bar 1 is SIGN_IN_WITH_SVRNGS §7 1; the design's
no-email rule is §8 4 (`rings/current/SIGN_IN_WITH_SVRNGS.md:306-313`).

## What was tried

1. **The row's Podman route, first.** On this Mac Podman's containers run
   in its own Linux machine; their `127.0.0.1` is the machine's, not this
   computer's (SIGN_IN_WITH_SVRNGS §8 8,
   `rings/current/SIGN_IN_WITH_SVRNGS.md:340-348`). Probed against a
   server on this Mac's loopback:

   ```
   ## podman --network=host: its 127.0.0.1 is the Podman machine's, not this Mac's
   wget: can't connect to remote host (127.0.0.1): Connection refused

   ## podman, bridged: host.containers.internal reaches this Mac's loopback
   probe
   ```

   The same against the provider's own discovery: the host-network
   container refused; the bridged one answered by
   `host.containers.internal`, with the caller read as loopback
   (`data/podman-probe.txt`). The provider's one issuer string is
   `http://127.0.0.1:<api port>/oidc/<lend>`
   (`crates/svrngs-node/src/oidc/provider.rs:62-65`), which a container's
   `127.0.0.1` cannot name. The design's path for a Mac is the app outside
   a container, so the run used it (STATE.md `427f406a`).
2. **Forgejo 17.0.0, pinned**, from the Homebrew bottle (sha256
   `d997331b207cf62e274ab7d15cf668c242152d1146292491360f8053f48238f6`;
   binary `fb508a2fcf4411e7792284224f7be8ae9c41fa6f550926912b22b0aff33404f1`),
   `ROOT_URL = http://localhost:18097/`, auto-registration on:
   `[oauth2_client] ENABLE_AUTO_REGISTRATION = true`, `USERNAME =
   preferred_username`, `REGISTER_EMAIL_CONFIRM = false`
   (`data/forgejo.ini`, secrets redacted). The `svrngs` OIDC source was
   added by Forgejo's own `admin auth add-oauth` with the issuer, client
   id and secret `svrngs share --sign-in-with-svrngs` printed
   (`data/share.out`, `data/forgejo-auth.txt`).
3. **Both engines.** Alex on Chromium 147.0.7727.15 at Alex's own door;
   Bo on WebKit 26.4 as an iPhone through the group's page. Both pressed
   Forgejo's landing-page **Sign in**, then the login page's **Sign in
   with svrngs**, and typed nothing (`data/raw.json`; `typed` false,
   `local_login_posts` 0).

## Evidence

- Both sign-ins ended at `/user/link_account`, not signed in:
  - Alex: `"url": ".../user/link_account"`, page: **"Register new account
    Link to an existing account Complete new account Username Email
    address"** (`data/raw.json`, `people.alex`).
  - Bo: `"url": ".../user/link_account"`, page: **"Complete new account
    Username Email address Complete account"** (`data/raw.json`,
    `people.bo`).
  - Forgejo's own user list holds only `kim`
    (`data/raw.json`, `forgejo.users`); no `Alex-` or `Bo-` account.
  - Verdicts: `alex` failed, `bo` failed, `apart` could-not-judge,
    `removed` passed, `planted` passed (`data/verdicts.json`).
- Forgejo's source makes the email the condition, not a setting: tag
  v17.0.0, `routers/web/auth/oauth.go:1119-1127`
  (`https://codeberg.org/forgejo/forgejo/raw/tag/v17.0.0/routers/web/auth/oauth.go`):

  ```
  if gothUser.Email == "" {
      missingFields = append(missingFields, "email")
  }
  ...
  if len(missingFields) > 0 {
      // we don't have enough information to create an account automatically,
      // so we prompt the user for the remaining bits
      showLinkingLogin(ctx, gothUser)
  ```

  There is no fallback for a missing email and no config that supplies
  one.
- svrngs has no email to send: the claims are `sub`, `name`, `groups`,
  `iss`, `aud`, `exp`, `iat` (`crates/svrngs-core/src/token.rs:39-56`);
  the lender's own `share` line says "it tells the app no email address,
  because svrngs has none" (`data/share.out`). The design forbids a
  substitute address: "An address that receives nothing is a value the app
  treats as true", §8 4 (`rings/current/SIGN_IN_WITH_SVRNGS.md:306-313`).
- The design's §5a says the opposite of what the run found: "Without an
  `email` claim the redirect serves the Jellyfin plugin, Forgejo,
  Paperless and Nextcloud"
  (`rings/current/SIGN_IN_WITH_SVRNGS.md:236-237`). For Forgejo 17.0.0
  that row is false.

## What would settle it

- **If Forgejo waits on `MAIL.md`:** the design's §8 4 already says
  Grafana, Open WebUI and Mealie wait on mail; add Forgejo to that list,
  and the trial's second app is not served. `oidc-retire` then has no
  Forgejo work; the next step in the queue is the operator's.
- **If the no-email rule changes:** svrngs needs a true address for each
  person (the in-group address `MAIL.md` describes), which is a design
  change and a new piece of product, not a run.
- A third option — an app-specific path in svrngs that hands Forgejo a
  synthesized address — is refused by §8 4 and by the queue's fence that
  a stock client needing app code in svrngs is NEEDS_HUMAN.

## The close

The run's data, README and driver are committed. `scripts/check.sh` runs
at this close under `ralph-result await`; the tree has no product change
from this row (`demo/runs/` only). The trust floor is unchanged, and
`scripts/lean_check.py` reads svrngs-core 3,152 of 3,153 and the
repository 17,421 of 17,423 (`data/lean.txt`).
