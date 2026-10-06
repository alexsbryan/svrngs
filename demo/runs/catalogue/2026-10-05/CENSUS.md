# The catalogue's census — DESIGN §1's apps against the canary's routes

u5-catalogue-dry (ralph/next/u5/STATE.md), written and committed before any
app below runs. Each app DESIGN §1 names (`docs/DESIGN.md:26-31`), against
the conformance canary's nine routes (`apps/web-conformance/serve.mjs:13-28`,
judged in `crates/svrngs-node/tests/conformance.rs:207-217`), with what it
leans on cited from its own docs, fetched 2026-10-05; and a prediction for
each way a person opens it (DESIGN §7): on a computer (each app at
`http://<label>.localhost:<port>/`), on a phone with the wildcard (each at
`https://<label>.<page host>/`), and on a phone on the floor (every app under
`<lends host>/l/<label>/`). A prediction is one of **works**, **works after
a setting** the lender types once (named), or **does not** (why).

## What the predictions rest on (svrngs at `42c3705`)

- **The floor takes the base off.** On the floor a lend's worker serves only
  paths under `/l/<label>/` and hands the app the rest
  (`web/lend.js:67-75`), so a page asking for `/x` gets the lends host's
  `/x`, never the app's (the canary's `root`, "needs the wildcard",
  `conformance.rs:220`). An app's own base-URL setting cannot help: the app
  never sees the `/l/<label>` it would need to match. Every app that
  serves its pages at its root therefore **does not** work on the floor.
- **The door asks the app at `localhost:<its port>`** (`crates/svrngs-node/src/door.rs:521-533`,
  `:937`), and turns a redirect to that address back into a path
  (`:535-553`). It does not rewrite the browser's `Origin`, and it rewrites
  no address inside a page's body.
- **A computer's lend origin carries the viewer's own port**, which is
  ephemeral (`demo/runs/u5/2026-10-05-jellyfin-in/data/alex-bare-command.out`:
  `localhost:51404`), so no setting a lender types once can name it.
- **On a phone the runtime forwards the headers the lend's worker sees**
  (`web/lend.js:182-188`); the Fetch standard adds `Origin` after a service
  worker has handled the request, so an app reached from a phone is
  predicted to see no `Origin` at all. Unverified until the run.
- **An app that keeps its own accounts asks each person to sign in once**
  with an account the lender made (DESIGN §3, "Lent apps", call 4); one that
  takes the person from a request header can let them straight in, the
  door's `x-svrngs-person` (`crates/svrngs-node/src/door.rs:107`) named in its
  setting — the column "header sign-in".

## The census

Routes: **root** `/`-absolute paths · **ws** WebSocket · **range** read by
Range · **upload** a large body uploaded · **worker** its own service worker
· **isolated** SharedArrayBuffer under COOP/COEP · **host** a Host check or a
redirect built from it · **cookie** a cookie sign-in · **storage** local
storage. Two the canary lacks, named where an app leans on them: **origin**,
the browser's `Origin` compared with the app's Host; **body-url**, absolute
addresses built from the Host or a configured URL written into its pages.

| App | Leans on (its docs) | Header sign-in (its docs) | Computer | Phone, wildcard | Phone, floor |
|---|---|---|---|---|---|
| Jellyfin | root (its web at `/web/`, its API at the root), ws (`/socket`), range, cookie-less token in storage — measured, `demo/runs/u5/2026-10-05-jellyfin/` | none: sign in once (u5-3) | works (measured, u5-4) | works (measured, u5-4) | not run; a Base URL is supported (jellyfin.org/docs/general/post-install/networking/, "Base URL") but the floor takes it off: predicted does not |
| Navidrome | root, range (streams), cookie/token, storage | yes: `ND_EXTAUTH_TRUSTEDSOURCES` and `ND_EXTAUTH_USERHEADER`, default `Remote-User` (navidrome website `docs/usage/configuration/options.md:203-204`, `docs/usage/integration/authentication.md:26-32`) | works after a setting: `ND_EXTAUTH_USERHEADER=X-Svrngs-Person` and `ND_EXTAUTH_TRUSTEDSOURCES` naming the address the door reaches it from; then no sign-in | same | does not (root) |
| Immich | root ("Immich does not support being served on a sub-path", immich.app/docs/administration/reverse-proxy/), ws ("enable websockets", same page), upload (its assets), range (video), cookie, storage | none named in its docs | works | works | does not (root) |
| Audiobookshelf | root, ws (socket.io), range (audio), upload, cookie/token, storage | none named in its docs | works | works | does not (root) |
| Calibre-Web | root, cookie (Flask session), upload (books), range (reads) | yes: "Allow Reverse Proxy Authentication", a header name typed in its admin page (calibre-web wiki `Setup-Reverse-Proxy.md:5-9`) | works after a setting: the header named `X-Svrngs-Person`, each person's user made first ("the user must exist beforehand", `:9`) | same | does not (root) |
| Nextcloud | root, host (`trusted_domains`, docs.nextcloud.com config_sample_php_parameters), cookie, upload (WebDAV `PUT`, `MKCOL`, `MOVE` — methods the canary does not send), worker, storage | none in core | works (`localhost` is the trusted domain its container installs with; the door asks on `localhost:<port>`) | works | does not (root) |
| Paperless-ngx | root, ws (its status socket), upload (`post_document`), cookie (Django session and CSRF), **origin**: `PAPERLESS_CSRF_TRUSTED_ORIGINS`, "a list of trusted origins for unsafe requests (e.g. POST)" (paperless-ngx `docs/configuration.md:482-488`) | yes: `PAPERLESS_ENABLE_HTTP_REMOTE_USER` and `PAPERLESS_HTTP_REMOTE_USER_HEADER_NAME` (`docs/configuration.md:629-661`) | **does not**: its sign-in is a POST whose `Origin`, `http://<label>.localhost:<port>`, is not its Host, `localhost:<port>`, and the port is the viewer's and changes, so no trusted origin typed once names it | works (no `Origin` reaches it from a phone) | does not (root) |
| Actual | root, **isolated** (SharedArrayBuffer, which its docs tie to HTTPS: actualbudget.org/docs/config/https/), worker, storage (its budgets in the browser), a token in a header | its "Authenticating with HTTP Headers" (actualbudget.org/docs/config/oauth-auth/, its navigation) carries a password in a header, not a person | works after a setting: the server's password, set once by the lender and typed once by each person (Actual keeps one password without OpenID) | same | does not (root) |
| Firefly III | root, cookie (Laravel session, CSRF token), **body-url** (Laravel builds addresses from the Host or `APP_URL`) | yes: `AUTHENTICATION_GUARD=remote_user_guard`, `AUTHENTICATION_GUARD_HEADER` (firefly-iii docs `how-to/firefly-iii/advanced/authentication.md:38-40`) | does not: links in its pages name `http://localhost:<port>` (body-url), uncertain until run | works after a setting: `APP_URL=https://<label>.<page host>` | does not (root) |
| Mealie | root, cookie/token, storage, upload (images) | none in its docs; `BASE_URL` is "used for notifications and the OIDC callback url" (docs.mealie.io backend-config) | works | works | does not (root) |
| Vikunja | root, worker, storage (token), upload (attachments); `service.publicurl` "used … for the communication between api and frontend" (vikunja.io/docs/config-options/) | none in its docs (OpenID) | uncertain: works if its page reaches its API at its own address; else does not | works after a setting: `VIKUNJA_SERVICE_PUBLICURL=https://<label>.<page host>/` | does not (root) |
| Grocy | root, cookie, `BASE_URL` (grocy `config-dist.php:68`) | yes: `ReverseProxyAuthMiddleware`, `REVERSE_PROXY_AUTH_HEADER` (`config-dist.php:90-96`) | works | works | does not (root) |
| Forgejo | root, cookie (CSRF), ws (its notifications), host and **body-url**: `ROOT_URL`, "useful if the internal and external URLs don't match" (forgejo.org config-cheat-sheet) | yes: `ENABLE_REVERSE_PROXY_AUTHENTICATION` (same page) | uncertain: its links are relative to its sub-URL but its clone URLs and some redirects are `ROOT_URL`'s | works after a setting: `ROOT_URL=https://<label>.<page host>/` | does not (root) |
| BookStack | root, cookie (Laravel), **body-url**: `APP_URL` is required (BookStack `.env.example.complete:27`) and every address it writes is built from it | none in core (`AUTH_METHOD`, `.env.example.complete:153`: standard, ldap, saml2, oidc) | does not: `APP_URL` cannot name a port that changes per viewer | works after a setting: `APP_URL=https://<label>.<page host>` | does not (root) |
| Home Assistant | root, ws (`/api/websocket`, every state it shows), worker, storage (its tokens), its sign-in a redirect back to the page's own address | none in core | works | works | does not (root) |

## The five the run takes, and the act for each

Spanning the routes: ws (Home Assistant, Immich, Paperless), range and a
large upload (Immich), WebDAV methods (Nextcloud), isolated and worker and
storage (Actual), origin (Paperless). Bo, on a phone through the page by
the link, in WebKit as an iPhone and in Chromium, in both phone ways; each
signs in once with an account (or for Actual the password) the lender made,
since none of the five is run with a header sign-in.

| App | Act | Wildcard | Floor |
|---|---|---|---|
| Home Assistant | a state the lender changes shows live on Bo's page | works | does not (root) |
| Immich | takes a 100 MB video and plays it, seeking | works | does not (root) |
| Nextcloud | takes a file from the browser | works | does not (root) |
| Actual | opens a budget | works (password typed once) | does not (root) |
| Paperless-ngx | takes in a PDF | works | does not (root) |

What would falsify these: an app that renders and does its act on the
floor (then the floor carries more than `web/lend.js:67-75` reads as); any
of the five failing its act with the wildcard; Paperless's sign-in refused
on a phone (then `Origin` reaches it, and the origin route is the canary's
to gain).
