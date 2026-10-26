# give check: the stand-in and the two cards, on a Mac and on Linux

2026-10-26, row give-app-runtime (`ralph/next/give/STATE.md`), at ae481b71
with the cards as committed beside this record.

The command, from the repository:

    cargo build -p give
    target/debug/give check --name svrngs-test-standin apps/give/tests/standin
    target/debug/give check --agree --name svrngs-test-minecraft apps/give/cards/minecraft
    target/debug/give check --name svrngs-test-factorio apps/give/cards/factorio

`--agree` is the yes to Minecraft's one ask, its EULA, which the operator's
worker gave for a check that ran the server for under a minute and then
removed it.

## What ran

| Host | Stand-in | Minecraft 26.3 | Factorio 2.1.23 |
|---|---|---|---|
| This Mac: macOS 27.0.1, arm64, podman 6.0.1, applehv machine (8 CPUs, 24 GiB), volumes on XFS (`host.txt`) | passed, 25 s, reflink, paused 958 ms | passed, 28 s, reflink, paused 967 ms | passed, 16 s, reflink, paused 903 ms (amd64 build, under Rosetta) |
| The Fedora box: Fedora 44, amd64, podman 5.8.7 rootless, no machine, volumes on btrfs (`fedora.txt`) | passed, 14 s, reflink, paused 285 ms | passed, 76 s, reflink, paused 345 ms | passed, 39 s, reflink, paused 278 ms |

Each run printed every step passed: read, asks, machine, pull by digest,
start, healthy, snapshot, stop, restore ("the state is the snapshot's"),
start again, healthy again, and remove. Afterwards no pod or volume named
`svrngs-test-*` was left on either host. The pause is the CLI's view:
`pod pause`, the copy in podman's storage, a `stat` of each volume's own
folder, and `pod unpause`. That is several podman calls, so it reads longer
than GIVE.md §7's 105 to 180 ms for the copy alone.

On Linux, give ran podman locally and copied with `podman unshare cp`. On
the Mac, it copied inside the machine (`podman machine ssh -- podman
unshare cp`). Both paths are in `src/runtime.rs` `storage_shell`.

## What did not pass first, and why

- **Factorio's arm64 build segfaults on this Mac.** Its first check failed
  "healthy: server stopped, exit 139". By hand, the image's entrypoint ran
  `/bin/box64 /opt/factorio/bin/x64/factorio --create …` and printed
  "Segmentation fault (core dumped)". The arm64 build carries Factorio's x64
  binary under box64. The machine's page size is 4096 (`getconf PAGESIZE`).
  `podman run --platform linux/amd64` of the same digest bound 34197 and
  saved the map on stop, under the machine's Rosetta
  (`/proc/sys/fs/binfmt_misc/rosetta`: enabled). So the card names
  `platform: linux/amd64` (ae481b71).
- **A restore lost the volume's own folder.** Factorio's rootless build,
  restored, exited 1 with "Could not move file factorio-current.log".
  `podman volume export` writes no entry for the volume's root, so it came
  back 0:0 755 and not 0:0 777. Snapshots now record it (445279d5).
- **`pod rm --force` of a running pod failed in podman 6.0.1**, on
  netavark's teardown, and left the pod. Removal now stops the pod first
  (f768e1bb).

## Never-ran

- **A plain copy, where the filesystem has no reflink (ext4).** Both hosts'
  volumes were on filesystems with reflink, so `--reflink=never` never
  ran. A host with podman's storage on ext4 would run it.
- **Making podman's machine on a Mac.** This Mac's machine is running, and
  podman runs one machine at a time, so making another would stop it. The
  plan and its commands are tested against a podman that records them
  (`apps/give/tests/machine.rs`).
- **A player joining.** Health here is each card's own check: Minecraft's
  `mc-health` answering, and Factorio's port bound. A player connecting
  through svrngs's door is give-app-keep's scene 1.
- **A Mac with Rosetta off in Podman.** Podman 5.6 and later turn it off by
  default (GIVE.md §4). This Mac had it on, so how Factorio's amd64 build
  runs without it is unknown here.
