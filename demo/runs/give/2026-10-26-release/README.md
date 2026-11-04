# give-docs, 2026-10-26

The give queue's last row (`ralph/next/give/STATE.md`): svrngs's documents
say what the queue landed, and the release is prepared for the operator,
in `NEEDS_HUMAN.md`, never run.

## What changed in the documents

- DESIGN §1 names Give among the reference apps; §3 gains "Give", beside
  House rules: its own package on the surface alone, its keeper fact its
  own fold, every move and takeover a person's yes. §3's "A port, not a
  page", "The person's own programs are the trust boundary" and "A browser
  comes in through an open" landed with their rows (give-udp to
  give-session). §7 says how a computer that keeps apps gets Give: the
  same line with `--give`.
- ARCHITECTURE: the record API's token, the raw door and Play (`door/udp.rs`,
  `door/tcp.rs`, `play.rs`), and `apps/give/`; one cite of DESIGN §3's old
  heading, "Localhost is the trust boundary", given its name now.
- README: "Lend what you run" names UDP and TCP; "Keep a game for the
  group" links Give.
- PLAN §1: give-page-host and give-docs landed; §2: the queue's Waits, each
  with its trigger, and Give's people runs and cap.
- `apps/give/README.md`: who may keep is answered (the operator,
  2026-10-26), no longer the operator's to decide.
- `RELEASES.md`: four paragraphs under "Next, not released yet".
- svrn-docs `rings/current/GIVE.md`: a closing line pointing here.

## Each claim in the release notes, and its row's commit

| The notes say | Row | Commit |
|---|---|---|
| a UDP or TCP port is lent with `svrngs share <port> --udp`, or `--tcp` | give-udp, give-tcp | 36471346, 0847c192 |
| Play, "Connect to 127.0.0.1:<port>.", the same address each time | give-play | 86dd073d (`crates/svrngs-node/src/play.rs` `Open::said`) |
| listed under "What acts as you" with its Stop; nothing listens once off | give-play | 86dd073d (`crates/svrngs-node/src/home/home.js:379`) |
| someone removed is cut off within seconds | give-cut | 662e30be |
| one rehearsal, 9 minutes of Factorio on one desk, no desync, no drop | give-play-scene | d5f65cdb (`demo/runs/factorio/2026-10-26-rehearsal.json`) |
| Give keeps an app on one computer at a time, in Podman | give-app-runtime, give-app-keep | f768e1bb, cf5903e9 |
| its world copied every five minutes and whenever it stops | give-app-keep | cf5903e9, 3de28bf8 (`apps/give/fold/src/lib.rs` `EVERY`) |
| brought once with `give page <group>` | give-app-keep | 9ab61bf5 |
| "Start it for the group" asks what the makers ask, Minecraft's terms | give-app-keep, give-app-runtime | 9ab61bf5, d0e4098a |
| "Move it to…", its person's yes, started from the last copy | give-app-move | aed586a8, e48519e5 |
| quiet for fifteen minutes, "Keep it here?" with the newest copy | give-app-takeover | 3d700b44 (`apps/give/fold/src/lib.rs` `GONE`) |
| the old computer's later state a copy to "Start instead"; nothing merged or thrown away | give-app-takeover, give-app-scenes | 667fc60e, 4fa9003a |
| Give's page gives the program's line, svrngs's own with `--give`, from your group's page | give-install, give-page-host | 2d68d696, 2e6ca3b6, 527db088 |
| computers without Podman still play and hold copies | give-app-keep | cf5903e9 (copies through the group's folder, `apps/give/README.md`) |
| the opener in Applications or the applications menu; bookmarks keep working across restarts | give-session | 192a22c1, c219e643 |
| "Open svrngs from your computer" | give-session | c219e643 (`crates/svrngs-node/src/session.rs` `refused`) |
| a container on a Mac is refused | give-session | `demo/runs/give/2026-10-26-container/` |
| the record API asks for the token, 401 `no_token` naming the file | give-token | 662803be |
| `svrngs/acts` says `page` | give-page-host | db1130b3 |
| an app joins Give's shelf as one `compose.yaml` with five `x-svrngs` fields | give-app-runtime | d0e4098a (`apps/give/src/card.rs`) |

## The close

`scripts/check.sh`, once, at the commit that carries this file; its result
is in the row's await log (CHARTER fence 12), and `NEEDS_HUMAN.md`'s
`<commit>` is that commit.

## Published, 27 October 2026

The operator: "Let's fix the grapevine bug then release" and "minor version
bump, push main + the tag to github". give-redial landed first (bd344e4d);
its gate passed whole on this Mac at `d21578a6` (give-redial-3.await.log,
1,111 s), and no code changed after it (`git diff --stat d21578a6 HEAD --
crates web scripts apps Cargo.toml Cargo.lock` empty).

- A worktree detached at `d21578a6`, `npm ci --prefix web`, then
  `CARGO_INCREMENTAL=0 scripts/deploy-page.sh`, exit 0: eight tarballs,
  svrngs and give for darwin-aarch64, darwin-x86_64, linux-x86_64 and
  linux-aarch64, give cross-built for all four; deployment
  `dpl_J42faEd6upwwtjcb2pBxocy42qEC`, production.
- Read back: the live `get/SHA256SUMS` equals the built one; the
  darwin-aarch64 tarballs of svrngs and give hash to their lines;
  `https://group.svrnme.sh/` and a name under it send
  `Content-Security-Policy: frame-ancestors 'none'`; the live `install.sh`
  has `--give`. The upload directory's `.vercel` and `.env.local` removed.
- `git tag v1.3 d21578a6`; main and the tag pushed to GitHub.
