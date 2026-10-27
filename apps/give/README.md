# Give

A group keeps an app on one member's computer at a time. It runs in
rootless Podman, from the app's own published image, and copies of its state
go to the other members (svrn-docs `rings/current/GIVE.md`). Give is its own
small package on svrngs's public surface. It links no svrngs crate
(`tests/surface.rs`), and it reaches svrngs through four things: the record
API, `svrngs share` and `svrngs unshare`, the synced folder, and its own
carried page.

This much has landed: the runtime on the computer that keeps an app, an
author's check of a card, the one fold of who keeps what, Give's program for
computers, and the carried page with the shelf, "Start it for the group",
"Move it to…", "Keep it here?" and "Start instead". Give's scenes in the
gate, and installing its program, come next (`ralph/next/give/STATE.md`).

## A card

An app is a folder holding:

- **`compose.yaml`**: the app's own compose file with an `x-svrngs` block of
  five fields: `title`, `about`, `door` (a port, and http, tcp or udp),
  `state` (the named volumes to keep), and `asks` (each with a link, and the
  environment its yes sets).
- **`nouns.txt`**: the app's words, which svrngs's code never uses.
- **`README.md`**.

`cards/minecraft` and `cards/factorio` are the first two cards. Health is
compose's own `healthcheck`. Give refuses five fields, each by name:
`build`, `privileged`, a custom network, host networking, and a bind mount
from the host. Any field of compose that give does not run is refused too,
never dropped.

## Checking a card

    cargo run -p give -- check [--agree] apps/give/cards/<kind>

The check pulls the card's image by digest and starts the app. It waits for
the app to be healthy, snapshots it, and stops it. Then it restores the
snapshot, compares the restored state with the snapshot file by file, and
starts the app again until it is healthy. Last, it removes everything it
made. If a step fails, the check names it. `--agree` is your yes to the
card's asks, once you have read them. On a Mac, if Podman's machine is
missing or stopped, the check makes it or starts it, sized for the card.

## Keeping an app

A computer keeps apps for its groups when it has Podman and runs Give's
program beside svrngs:

    give page <group>     # once, by anyone: brings Give to the group
    give keep             # on each computer that keeps apps; leave it running

`give page` puts Give's page in the group's folder, so every member can open
it from their home page with nothing to install. The page lists the shelf.
**Start it for the group** asks what the app's makers ask of whoever runs
it, and the app is then kept by the computer the page was open on. Each row
says who keeps the app: "kept by your computer", or "kept by Cy’s computer".

`give keep` says in each group that this computer can keep apps. It keeps
each app the group's record names this computer for, and only those. It
lends the app through svrngs (`svrngs share`) once it is healthy, copies
its state into the group's folder every 5 minutes and as it stops, and
keeps this computer's 3 newest copies of each app. Each copy is said in
the group's record, so everyone can see when the app was last copied.
When the record names another computer, it stops lending the app, copies
it one last time and stops it. That last copy is kept apart and never
removed. Ctrl-C stops everything it keeps, each with a last copy.

**Move it to…** on an app your computer keeps offers it to another computer
that runs `give keep`. That computer's person is asked what the app's makers
ask of whoever runs it, and nothing moves without their yes. On the yes,
your computer stops lending the app, copies it to the group one last time,
writes the move and stops it. The other computer starts it from that copy,
and only once its own copy of it is whole.

**Keep it here?** If the computer keeping an app goes 15 minutes without
copying it, everyone's page says when that computer last answered. Anyone
whose computer runs `give keep` is offered the newest copy, with the
app's makers' terms again. On their yes, their computer starts the app
from that copy once the copy has fully arrived. If the copy doesn't
become healthy, or stops within a minute, their computer stops it and
says so. The page then offers the copy before it. A copy that didn't
start is never tried again unless a person chooses it.

**Start instead.** When a computer that kept an app comes back after
another took it over, it starts nothing and lends nothing. It copies the
state it had to the group, and every page lists that copy as "Minecraft
as Cy’s computer had it at 14:10". Anyone whose computer can keep may
start that copy instead, which is the same yes as above. Nothing is
merged and nothing is thrown away: the state the app had before is
copied too, and listed the same way.

Any member's computer that runs `give keep` may keep an app for now. Who
may keep is the operator's to decide (`demo/runs/give/2026-10-26-keep/QUESTION.md`).

A lend reaches everyone in every group its person is in, under one name
(`svrngs share`), so one computer keeps one app of a kind at a time.

## The files

- `src/card.rs`: a card, read once, with each refusal naming its field.
- `src/runtime.rs`: start, health, snapshot, restore and stop, for one card
  under one name, on Podman's command line (`src/podman.rs`). It names no
  svrngs type.
- `src/machine.rs`: Podman's machine on a Mac.
- `src/check.rs`: `give check`.
- `fold/`: who keeps each app, from give's acts in the record's order; the
  program links it, and the page runs `page/vendor/fold.js`, the same code
  built to JavaScript by `build.sh` and pinned in `page/vendor/README.md`.
  `fold/cases.json` is run against both builds (`fold/tests/cases.rs`,
  `fold.test.mjs`).
- `src/keep.rs`: `give keep`, its `plan` decided apart from doing it;
  `src/record.rs`, `src/lend.rs`: the record API and `svrngs share`, all of
  svrngs it reaches; `src/shelf.rs`: give's own cards and any `--card`.
- `src/page.rs`, `page/`: Give's page, and `give page`, which brings it.
- `src/main.rs`: the three commands.
- `tests/`: the stand-in card (busybox's httpd) and the tests;
  `page.test.mjs`, the page in Chromium; svrngs's scene of Give,
  `crates/svrngs/tests/scenes/give.rs`, plays it with real svrngs. The tests that
  need Podman name their pods and volumes `svrngs-test-*`, and remove them.
  Where Podman does not answer, each of those tests fails as never-ran and
  gives its reason.
