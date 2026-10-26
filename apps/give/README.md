# Give

A group keeps an app on one member's computer at a time. It runs in
rootless Podman, from the app's own published image, and copies of its state
go to the other members (svrn-docs `rings/current/GIVE.md`). Give is its own
small package on svrngs's public surface. It links no svrngs crate
(`tests/surface.rs`), and it reaches svrngs through four things: the record
API, `svrngs share` and `svrngs unshare`, the synced folder, and its own
carried page.

This much has landed: the runtime on the computer that keeps an app, and an
author's check of a card. The fold of who keeps what, the keeper program and
the carried page come next (`ralph/next/give/STATE.md`).

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

## The files

- `src/card.rs`: a card, read once, with each refusal naming its field.
- `src/runtime.rs`: start, health, snapshot, restore and stop, for one card
  under one name, on Podman's command line (`src/podman.rs`). It names no
  svrngs type.
- `src/machine.rs`: Podman's machine on a Mac.
- `src/check.rs`, `src/main.rs`: `give check`.
- `tests/`: the stand-in card (busybox's httpd) and the tests. The tests that
  need Podman name their pods and volumes `svrngs-test-*`, and remove them.
  Where Podman does not answer, each of those tests fails as never-ran and
  gives its reason.
