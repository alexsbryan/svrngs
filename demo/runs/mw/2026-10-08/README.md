# Jellyfin's zero sign-in as middleware — 2026-10-08

The middleware spike (DESIGN §8 "Then" :1093; §9 :1133; §3 "The room",
call 6 :396; §10 :1146-1149; queue `ralph/next/mw/STATE.md`, first commit
`5e877c9`): can the four rows u5-3 counted for Jellyfin's zero sign-in
(§9 :1131; u5-3 finding 4, `demo/runs/u5/2026-10-05-jellyfin/README.md`)
be done instead by an ordinary program on the lender's computer, in front
of Jellyfin, lent as `svrngs share <its port>`, with no diff under
`crates/` or `web/`? This file is mw-0: the inventory, the contract, the
bars, the kill clause and a prediction per bar, committed before any
middleware or data commit of the queue, as u5-3's rule was (`9a637e5`).
None of it is moved after its data; what the data gives is written below
it, in later commits (mw-3).

## 1. The inventory (AGENTS.md 11)

Upstream read at commonwealth-ai `d4356181c` (HEAD, 2026-10-07), with
`git -C ~/dev/commonwealth-ai show d4356181c:<path>`. `mesh_media/` is
`cmnwlth/crates/sovereign-cli-mesh/src/mesh_media/`; DESIGN §8 U5's
inventory (:921) cites it at `e1a43bcd2` under `sovereign/`, and
`git show d4356181c:<new> | diff - <(git show e1a43bcd2:<old>)` prints
nothing: moved by `6c51bba85` ("Top level programs"), unchanged, so every
line U5's inventory cites stands.

| Upstream | What it does there | Becomes |
|---|---|---|
| `mesh_media/viewer.rs:304-463` (`provision`), with `:65-75` (`viewer_id_in`), `:85-110` (`call`), `:143-196` (the policy merged and read back) and `:216-296` (`already_provisioned`); its tests `:465-746` | finds or makes one non-admin Jellyfin user by a fixed name (`VIEWER_NAME`, `:52`), with a password it invents and never keeps (`:316-319`), reset when the user exists (`:378-396`); writes the read-only policy over the origin's own and reads it back (`:398-436`); signs in by `AuthenticateByName` (`:438-452`) and answers `MediaBrowser Token="…"` (`:459-462`), all with one elevated credential given once (`:301-303`) | **forked** into `apps/jellyfin/mw/`, the middleware's first module: a user per person in place of one viewer (u5-3 finding 4, its first piece). It lands unchanged in a commit of its own, `ported from commonwealth-ai cmnwlth/crates/sovereign-cli-mesh/src/mesh_media/viewer.rs@d4356181c` on it, its own tests passing as upstream's do (DESIGN §4); then it changes: the name from the person the door names, a user per `x-svrngs-key`, its token kept by the middleware rather than declared. It stands outside sovereign's crates: it uses `reqwest`, `serde_json`, `uuid`, `tokio` (its tests) and `tracing`, and names `commonwealth_media` only in a doc comment (`:42`) |
| `mesh_media/viewer.rs:6-12` | an `/Auth/Keys` key is admin-equivalent: zero `UserId`, passes elevation | the key the lender gives the middleware once, the only thing that can make a user (`:301-303`); the bar that no byte holds it (§3 below) is about this key |
| `cmnwlth/crates/commonwealth-transport/src/iroh_identity_forward.rs:1-27` | the door's forward on upstream: request heads rewritten, the identity under `x-mesh-*` set, "the origin (or a shim beside it)" deciding per member (`:11`); responses "never parsed", a byte copy (`:20-22`); `Transfer-Encoding` of any kind ends the connection (`:22-27`) | **not forked**: already svrngs's door (DESIGN §8, U1's inventory), and it cannot be the middleware's — a middleware rewrites answers (finding 3's token) and Jellyfin's are chunked. The middleware's forwarding (bodies streamed both ways, a WebSocket carried) is written new on `hyper`, since upstream has no forward that reads answers, and it links no svrngs crate: its contract is HTTP (§9 :1133), so it is not an SDK |
| `shared/crates/host-kit/src/shell/files.rs:38-61` (`serve_file`) | a dev server's HTML with a `<script src>` put before `</head>` (`:50-58`), for `ring show`'s shim (`ring_cmd/show.rs:227-232`) | **not forked**: five lines of a file server, not a forward. The middleware writes jellyfin-web's stored sign-in by putting a script of its own first in `<head>` of the page it forwards, where every script of jellyfin-web's own is `defer` (`/Applications/Jellyfin.app/Contents/Resources/jellyfin-web/index.html`, 12.0.0 on this host), so it runs before them; as svrngs's own lend worker puts its shim (`web/lend.js:334-341`) |
| tag `e0-bot` (U5's inventory, DESIGN :923) | the door masked a lend's secret in bodies, never measured | **not forked**, and nothing of it is in upstream's crates at `d4356181c` (`git grep -n -i "mask\|redact"` over `commonwealth-transport`, `commonwealth-media` and `sovereign-cli-mesh` finds only `iroh.rs:58`, an import of `configured_proxy_redacted`, a proxy address redacted for a log). The middleware's swap of each person's token in what Jellyfin answers (finding 3: `PlaybackInfo`, `master.m3u8`, `main.m3u8`, 43 places) is written new |
| `cmnwlth/crates/commonwealth-transport/src/iroh_routed_forward.rs:245-262` (`mint_tie`, `tied_pubkey`) | 32 random bytes per registration, handed only to its registrant; a request at a local port without the tie "is a caller that typed `x-mesh-*` at a local port — its claim is not believed" | **not forked**: §9 :1126 keeps it with its trigger, and it is the operator's (fence 4). What mw-1 records of it is §5 below |

**The language and the place.** Rust, as the fork is: `apps/jellyfin/mw/`,
a package with a `[workspace]` of its own, so the root workspace
(`Cargo.toml:3`) neither builds, lints nor tests it and `scripts/check.sh`
stays free of Jellyfin (fence 2); `apps/` is apart from the size budget
(`scripts/lean_check.py:14`, `:49`), and `scripts/ports.py` lists its pair
as any other. Its run and checks are `apps/jellyfin/check/check.sh`'s, as
u5-3's and u5-4's were. Jellyfin is GPL-2.0-only: the middleware talks to
it over HTTP and its API and copies none of its code.

## 2. The contract it leans on (`crates/svrngs-node/src/door.rs` at `5e877c9`)

The lender runs `svrngs share <the middleware's port>` and presses Enter
alone at the password asked (`crates/svrngs/src/cli.rs:802-807`): the lend
carries no secret. Every request the middleware is asked comes from the
door's `forward` (`door.rs:947-1101`) to `127.0.0.1:<its port>`
(`:1039`), and holds:

- `x-svrngs-person` (`:107`) and `x-svrngs-key` (`:111`), set by the door
  from the verified dialer and the register (`:972-990`), after every
  header the client typed under `x-svrngs-` is stripped (`:468-496`,
  `rewrite_headers`; module docs `:10-20`). `x-svrngs-key` is the person's
  key as lowercase hex; a holder on a phone is the person who signed the
  pass (`:59-66`), and the lender's own browser is this computer's key
  (`Who::Own`, `:609`, `:735`).
- `x-svrngs-declared-share-bps` (`:116`), when the lend declares an upload
  (`:1020-1025`). The middleware needs it for nothing.
- The client's own `authorization`, untouched, since no secret fills it
  (`rewrite_headers` displaces only the names a secret declares,
  `:475-488`): jellyfin-web's `MediaBrowser Client=…, Device=…,
  DeviceId=…, Version=…, Token=…`, its device with it.
- `Host: localhost:<the middleware's port>` (`:995`, `own_address`
  `:527-530`), and the lend's own `Origin` told as that address (`:996-1000`,
  `own_origin` `:539-553`). The middleware passes both on, so a redirect
  Jellyfin builds to its asker's address comes back as a path (`as_path`,
  `:558-572`, read against the lend's port, `:1061-1069`).

What the door does that the middleware must not do again: the identity
above; refusing a path that can step out of its lend (`:737-744`), a
dialer in no group held here (`:750-754`), a path the lend refuses
(`:788-797`), and a newcomer past the lend's limit (`:800-821`); counting
and telling the share (`:822-834`, `:1070-1077`); a lend's secret
displacing the client's `authorization` (`:475-488`) and a response header
echoing it dropped (`drop_echoed_secret`, `:574-595`, called `:1060`) —
neither does anything here, the lend having no secret, and the lender's
key, which the middleware holds, is the middleware's to keep out of what
it answers. Bodies stream, `Range` passes and a WebSocket is spliced by
the door on its hop (`:21-33`, `:1045-1059`); the middleware carries the
same on its own. The loopback record API is part of the contract (§9
:1133); this spike is predicted to need none of it.

Which of u5-3's four pieces (finding 4) each part takes over:

| u5-3's piece | Where finding 4 put it | In the middleware |
|---|---|---|
| a Jellyfin user per person, made with a key the lender gives once | `apps/jellyfin/` | the forked `provision`, run on the first request whose `x-svrngs-key` it has not seen (mw-1) |
| the door carrying a secret per person, not per lend (`door.rs:497-516`, `secret_headers`, one per lend) | the door | the middleware keeps each person's token by `x-svrngs-key` and writes it into the `Token="…"` of the client's own `authorization`, its device kept; the lend has no secret (mw-1) |
| jellyfin-web's stored sign-in (`jellyfin_credentials`) written at the lend's origin before its page loads | the infra (`origin.rs`, `web/lend.js`, `web/open.js`), the script in `apps/jellyfin/` | the middleware's answer to jellyfin-web's page carries a script, first in `<head>`, writing it: the person's user id, the server's id, the lend's own origin as the address, and a stand-in for the token (mw-2) |
| the person's token kept out of the bodies Jellyfin writes it into | the door, rewriting a streamed body | the middleware swaps the person's token for the stand-in in what Jellyfin answers, and the stand-in back for the token in what it is asked — header, query and socket address (mw-2) |

## 3. The bars, and the kill clause

§9 :1133, word for word: "a newcomer plays from the list with no sign-in
in both engines on a computer and a phone; no byte a person receives holds
the lender's key; two people's plays stay apart." Each is judged on four
pairs — a computer and a phone (the page's link), each in Playwright's
Chromium and WebKit — run by `apps/jellyfin/check/check.sh pair` extended
for a lend through the middleware (mw-2), against a scratch Jellyfin 12.0.0
on this host (`check.sh up`), two people new to it, Ana and Ben, as u5-4's:

1. **A newcomer plays from the list with no sign-in.** Each person, from
   the list (the bare command's address on a computer, the page's list on
   a phone) to a `<video>` that moves within 30 s of play (u5-3 (a)), is
   never asked "Please sign in" and types nothing; then every seek through
   jellyfin-web's own seek bar plays again within 5 s, Ana's five and
   Ben's three (Demo A's bar, `apps/jellyfin/check/judge.py` bar 2).
2. **No byte a person receives holds the lender's key.** Every response
   header, body and socket frame each browser received, searched by
   `apps/jellyfin/check/scan.py` for the key the lender gave the
   middleware, the lender's password and token, and the other person's
   token: none found, as `judge.py`'s "no secret" reads it. Whether a
   person's bytes hold their own Jellyfin token is recorded, not judged.
3. **Two people's plays stay apart**, read as u5-3 (a) measured "left
   alone": while one seeks, the other's `<video>` has no `emptied` or
   `loadstart`, never goes back more than 2 s and never stands still 5 s,
   in every phase, the converted play included; both are still signed in
   as themselves afterwards; and Jellyfin holds each as a user of their
   own on a device of their own (`devices_apart`, u5-3 finding 1).

**The kill clause.** The spike is killed, not bent, by either: (K1) any
diff under `crates/` or `web/` since this queue's first commit —
`git log --diff-filter=A --format=%h -- ralph/next/mw/STATE.md` prints
`5e877c9`, so `git diff 5e877c9..HEAD -- crates web` prints anything; or
(K2) a need for a header the door does not set today — it sets
`x-svrngs-person`, `x-svrngs-key` and `x-svrngs-declared-share-bps`,
`Host` and the lend's own `Origin` (§2), and a lend's secret in
`authorization` (`cli.rs:797`). A row that finds such a need writes it down
with its evidence and goes to the verdict; it does not make the change.

**What the rule gives** (mw-3): *kept* if bars 1 to 3 each passed in all
four pairs and neither K1 nor K2 fired; *killed* if K1 or K2 fired, naming
the change, or a bar failed, naming it. A pair that could not finish is
could-not-judge, never a pass: it is run again, or it goes to the
operator. If every bar held but a seek over 5 s, the seat does not judge
it after the data: it goes to the operator in `QUESTIONS.md`, as u5-3's
one slow seek did. One host, one run of each: not a rate (AGENTS.md 7).

## 4. Predictions, 2026-10-08, before any run

- **Bar 1: passes in all four pairs; one seek on a phone in WebKit may
  miss.** jellyfin-web given `jellyfin_credentials` before its scripts
  run opens signed in and plays, as it did in u5-3's probe with the
  driver's stored sign-in (finding 4; `probe-*-placeholder.json`). On a
  phone the lend's worker reads the page whole and puts its shim at
  `<head>` (`web/lend.js:334-341`), ahead of the middleware's script:
  predicted harmless. Seeks on a computer within 1 s; on a phone, through
  n0's relay, one miss in the four pairs would not surprise (u5-3:
  7,871 ms; u5-4: 6,389 ms; §11 risk 4), and the middleware adds a hop on
  loopback only.
- **Bar 2: passes.** The middleware sends the lender's key only on its own
  calls to Jellyfin, never in an answer. Each person's own token: the
  first version is predicted to miss a place Jellyfin writes it, which the
  scan finds and mw-2 fixes before its judged run; judged run, none.
- **Bar 3: passes.** Each person is their own Jellyfin user; Jellyfin's
  sessions show the device jellyfin-web names in its `authorization`,
  which the middleware keeps, not one of the middleware's — the one part
  not known, since a token signed in under one `DeviceId` is then used
  under another; if Jellyfin holds to the token's, the middleware signs
  each person in once per device the client names, and the bar still
  holds. Left alone in every phase, as u5-3 found with an account per
  person (finding 1).
- **K1 and K2: neither fires.** The middleware needs `x-svrngs-key` to
  tell people apart, `x-svrngs-person` to name the user it makes, and the
  client's own `authorization`, which a lend with no secret leaves alone
  (§2): all set today.

## 5. The two questions §9 leaves open with it

Recorded, not answered (fence 4); mw-3 packages them in `QUESTIONS.md`.

1. **Demo A's unchanged list of users** (`demo/a.md:7`, `:16`, `:26`,
   `:54`; DESIGN §3, "Lent apps", call 4 :271-281). Users made per person
   break it by construction. mw-1 records Jellyfin's users before its run
   and after (`users-before.json`, `users-after.json`, as u5-3's data has
   them): how many the middleware made, under what names, and that a
   person's second ask makes none. mw-2 records the same over its four
   pairs, and whether a person who came through both a computer and a
   phone is one user.
2. **The per-process tie** (`mint_tie`, §9 :1126) that "verified" leans
   on. The door's identity is verified up to the middleware's port; past
   it, on loopback, nothing tells the door's request from any process's on
   the lender's computer. mw-1 records, not judges: what a process on this
   computer gets asking the middleware's port with no `x-svrngs-key`
   (refused, by mw-1's check) and with another person's key in it
   (predicted: that person's Jellyfin, as them, since the middleware holds
   no tie); and that the one secret a lend can carry today is
   `authorization` (`cli.rs:797`), which a tie put there would take from
   jellyfin-web's own header (`door.rs:475-488`), its device and the
   stand-in token with it — so a tie by the lend's secret is not free.
   mw-2 records whether any of its runs met a request at the middleware's
   port that did not come through the door.
