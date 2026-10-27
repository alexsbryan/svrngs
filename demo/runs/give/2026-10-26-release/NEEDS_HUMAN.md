# The give release: prepared, for the operator to publish

give-docs (`ralph/next/give/STATE.md`) prepares the release and stops:
publishing is the operator's (CHARTER, "Leave for the operator"; fence
10). Nothing below has been run.

## What it is

`RELEASES.md` "Next, not released yet": this queue (a game or anything on
a port, Play, Give, the open from your computer, the token for programs)
and the two paragraphs already there from after V1.2 (the home page's
open naming what an app does, a5044249; two people of one name in Money
and Events, keys-names). `README.md` beside this file maps each claim in
the notes to its row's commit.

## The decisions

1. **Publish it, and under what name.** The notes call it "the next
   release". V1.3 is the name assumed below; the name is yours.
2. **A size cap for Give** (PLAN §2: "A cap on Give's own lines", whose
   trigger was "Give lands"). `apps/` is not counted by
   `scripts/lean_check.py`, and each row reported Give's lines instead
   (PLAN §1, give-app-runtime about 1,430, give-app-keep about 1,160,
   give-app-move +303, give-app-takeover +406, give-app-scenes +7,
   give-install +101, give-page-host about +30). Whether Give gets a
   cap of its own, and where, is yours; nothing waits on it to publish.

## The publish, as `scripts/deploy-page.sh` and V1.2 ran it

From `<commit>`, the commit `scripts/check.sh` last passed at (this row's
close; the await log of give-docs names it), in a worktree of its own so
the loops' tree and `target/` stay apart:

```
cd ~/dev/svrngs
git worktree add --detach ../svrngs-v1.3 <commit>
cd ../svrngs-v1.3
npm ci --prefix web                 # a fresh worktree has no web/node_modules
scripts/check.sh                    # optional: the close already passed at <commit>
rm -rf target/debug                 # the release build needs the disk
git tag v1.3 <commit>
scripts/deploy-page.sh              # the page and eight tarballs (svrngs and give, four platforms each), then vercel deploy --prod
curl -fsS https://group.svrnme.sh/get/SHA256SUMS   # read back: eight tarballs, this build's sums
rm -rf .vercel .env.local           # what the Vercel CLI leaves in the upload dir, if present
cd ~/dev/svrngs && git worktree remove ../svrngs-v1.3
git push origin main v1.3
```

`deploy-page.sh` runs `scripts/build-release.sh`, which now builds `-p
give` beside `svrngs` for each target and lists both in `SHA256SUMS`
(2d68d696). It reads `SHIPPED` (`crates/svrngs/src/cli.rs`, `N0` at this
commit), so this build meets its groups at n0's relays and publishes at
once.

Then `RELEASES.md`'s heading "Next, not released yet" becomes "V1.3,
<date>", with "V1.3 was built from commit `<commit>`." at its end, as
V1.2's is.

## What to know before you publish

- **Give's program has been cross-built for one platform of four.**
  give-install built `cargo zigbuild --release --locked -p give --target
  x86_64-unknown-linux-musl` (9 s, 2d68d696's body). Otherwise it was
  built natively, on this Mac and on the Fedora box
  (`demo/runs/give/2026-10-26-cards/README.md`), never cross-built for
  macOS on Intel or Linux on aarch64. A target that fails stops
  `build-release.sh` before anything is uploaded.
- **Every program that uses the record API needs the token.** A script of
  a person's own, or an app outside this repository, that asked
  `127.0.0.1:<api port>` with nothing else is refused 401 `no_token` once
  its computer updates (give-token, 662803be). The notes say so for
  developers; svrngs's own apps and command already send it.
- **Each person opens svrngs from their computer once after updating.**
  A bookmark from before has no session, so it shows "Open svrngs from
  your computer" until the opener runs (give-session, c219e643, 192a22c1).
  `install.sh` puts the opener in place, so the update line itself does
  it; a computer updated another way needs `svrngs open` once.
- **Update every computer.** A computer on V1.2 lists a port a newer one
  lends as an app to open; opened, the lender's door answers "<name> is
  lent as a port, not a page" (`crates/svrngs-node/src/door.rs:1297`), and
  it has no Play. No row tested an older computer against a raw lend.
- **Bringing Give to a group needs a terminal**, `give page <group>`, run
  once by anyone (`apps/give/README.md`); Give is not on the home page's
  shelf (`crates/svrngs-node/src/shelf.txt`), and the program's line is
  pasted into a terminal as svrngs's own is. The fluency run that would
  count what that costs a newcomer is never-ran (PLAN §2).
- **Never played with people.** The two-home game sessions
  (`demo/factorio.md`, `demo/minecraft.md`) and Give's people runs
  (`demo/give.md`) wait for you (PLAN §2), never-ran. Every scene in the
  gate is a program on one host; the one rehearsal ran Factorio on one
  host for 9 minutes (`demo/runs/factorio/2026-10-26-rehearsal.json`).
- **A Mac whose Podman machine has Rosetta off** cannot keep Factorio,
  whose card runs an amd64 build; Give refuses to start it there and its
  page says to turn Rosetta on (give-install (c), 8402864a), never-ran on
  this host, whose machine has it on.
