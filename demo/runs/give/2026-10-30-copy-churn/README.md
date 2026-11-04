# How much a kept app's state changes — 2026-10-30

Give takes a copy of a kept app's state every 5 minutes
(`apps/give/fold/src/lib.rs:65`, `EVERY = 5 * 60`), as one tar per state
volume (`apps/give/src/runtime.rs:358-410`), and the group's folder sends
each new tar, whole, to every member. This pass measured what actually
changes between those copies, for three apps run from their publishers'
images, to size the proposed fix: a copy as one blob per file plus a
manifest, sent only to the computers that keep.

## How it ran

- The Fedora box (`ssh alexbryan@fedora`), podman 5.8.7 rootless, its
  storage on btrfs (`findmnt -T ~/.local/share/containers`), so
  `cp --reflink=always` worked for every copy.
- Images, pinned by their multi-arch index digest (`skopeo inspect --raw |
  sha256sum`, recorded in `scripts/apps.sh`):
  - **n8n 2.45.7** (the image's `org.opencontainers.image.version`),
    `ghcr.io/n8n-io/n8n@sha256:9a8d8dce…`, 1.21 GB. Docker Hub refused the
    pull of `docker.n8n.io/n8nio/n8n` ("toomanyrequests: You have reached
    your unauthenticated pull rate limit"); `ghcr.io/n8n-io/n8n:latest` is
    n8n's own registry and gave the same amd64 manifest as Docker Hub's
    mirror (`8e7fd6d6…` in both indexes).
  - **Uptime Kuma 1.23.17** (`/app/package.json`),
    `mirror.gcr.io/louislam/uptime-kuma@sha256:70233f4a…`, 478 MB: the same
    index digest the Mac pass recorded for `louislam/uptime-kuma:1`
    (`../2026-10-30-tutorial-pass/mac/kuma/notes.txt`), fetched through
    Google's Docker Hub mirror for the same rate limit.
  - **Open WebUI 0.12.5**, `ghcr.io/open-webui/open-webui@sha256:03443b8d…`,
    4.64 GB, already on the box, run as
    `../2026-10-30-tutorial-pass/fedora/q3-open-webui/compose-c-own-image.yaml`
    has it (`WEBUI_SECRET_KEY_FILE=/app/backend/data/.webui_secret_key`,
    data at `/app/backend/data`), started empty.
- Each app ran in its own pod `svrngs-churn-<app>`, published on 127.0.0.1
  only, with its state in one named volume `svrngs-churn-<app>-data`
  (`scripts/apps.sh`). Kuma's pod also held `python:3.12-slim` running
  `python3 -m http.server --bind 127.0.0.1 8000`, and `scripts/kuma_setup.js`
  (adapted from the Mac pass's `kuma.js`) made the first account and two
  HTTP monitors of it, every 20 s (`kuma/setup.log`: `"add1":true,"add2":true`).
- `scripts/run.sh <app>` did the rest, the three apps side by side, 40 s
  apart. Ready: n8n and Kuma after 2 s, Open WebUI after 91 s. Settled
  when 3 minutes had passed since ready and the volume had grown under 1 MB
  in the last minute (`*/run.log`, "settled"). Then 7 copies 5 minutes
  apart, each **the way Give takes one**: `podman pod pause`, a fresh
  `<vol>-snapshot` volume, `podman unshare cp -a --reflink=always
  <vol>/. <copy>/`, `podman pod unpause`, then `podman volume export` of the
  copy to `<k>.tar` and the copy removed (`runtime.rs:448-470`). The pause
  lasted 205–292 ms for a running app (`*/run.log`, `paused_ms=`), Open
  WebUI's 1.1 GB included.
- One light use at minute 12, so window 2→3 holds it and the other five are
  idle (`*/use.log`):
  - n8n: owner account set up (200), one workflow created (200, a manual
    trigger and a Set node, inactive; `list workflows 200 1`).
  - Kuma: nothing extra; its monitors write anyway.
  - Open WebUI: the first account (200, `role admin`), one chat saved with a
    user message and no model (200), one settings change (200).
  The three light-use scripts were tried first on throwaway pods
  (`scripts/trial.sh`, `svrngs-churn-trial-*`, removed after).
- `scripts/analyze.py` read every tar with Python's `tarfile`, hashed every
  member, and wrote `<app>/manifests/` (`path size sha256` per file, in the
  tar's own order) and `<app>/windows.json`; `scripts/figures.py` made
  `table.md` from those. The hashes of the two key files (n8n's `config`,
  which holds its `encryptionKey`, and `.webui_secret_key`) are withheld in
  the manifests here. No password, token or environment value was printed.

## What changed, per app

All per-window numbers are in `table.md`.

**n8n** (tar 6.65 MB, 7 files). Idle, the copies were byte-identical: copies
0, 1 and 2 all `e4578a44…`, copies 3–6 all `308dedc2…` (`n8n/run.log`).
The light use changed 3 files, none added or removed: `database.sqlite-wal`
(4.14 MB, 20 of its 4 KiB blocks differ), `database.sqlite-shm` (32 KB, 5
blocks) and `n8nEventLog.log` (530 → 1108 bytes). `database.sqlite` itself
(2.47 MB) did not change in 30 minutes: everything n8n wrote stayed in the
WAL, which is a fixed 4,136,512 bytes in every copy (`n8n/manifests/`).

**Uptime Kuma** (tar 1.47 MB, 3 files). Every copy differed from the one
before, idle or not (`kuma/run.log`, 7 different sha256s). Each window
changed the same 3 files: `kuma.db-wal` (a fixed 1,194,832 bytes, 169–188
blocks differ), `kuma.db` (233→246 KB over the run, 6–11 blocks) and
`kuma.db-shm` (32 KB, 1–5 blocks): 1.46–1.47 MB of whole files a window,
0.74–0.83 MB of differing 4 KiB blocks. The light-use window looks like the
others, since there was no extra use.

**Open WebUI** (tar 1,118 MB, 148 files). Idle, the copies were
byte-identical: 0–2 all `861405cf…`, 3–6 all `ffba4c4c…`
(`owui/run.log`). The account, chat and setting changed 2 files:
`webui.db-wal` (54 → 169 KB) and `webui.db-shm` (32 KB); `webui.db` (729 KB)
and `vector_db/chroma.sqlite3` did not change. 1,115 MB of the 1,118 MB is
model cache the first start fetched into the data dir:
`cache/embedding/models` 967 MB and `cache/whisper/models` 148 MB, plus
`cache/tiktoken` 1.7 MB (summed from `owui/manifests/snaps-6.txt` by
directory). None of it changed after settling.

## Why a tar changes when nothing meaningful did

In this run, **it didn't**: a tar changed only when a file's bytes changed.

- `podman volume export` is deterministic for an unchanged volume. With the
  pod stopped, exporting the same volume twice, and copying it then exporting
  twice the way Give does, gave four byte-identical tars for each app
  (`*/run.log`, "stopped export-a/-b" and "copy copy-a/-b": n8n all
  `e19c8a33…`, Kuma all `f75d25f0…`, Open WebUI all `ffba4c4c…`).
- The copy keeps mtimes: `cp -a` from `<vol>/.` preserves them, and the
  export of the copy equals the export of the volume (`windows.json`,
  "determinism", `det/export-a.tar vs det/copy-a.tar`: identical).
- Podman's tar has no root entry and lists members in sorted order (each
  manifest is in the tar's own order, e.g. `n8n/manifests/snaps-0.txt`:
  `config` … `storage/`), so neither the copy volume's own directory nor
  the order of entries moves. `analyze.py` found 0 entries whose header
  alone changed (mtime, mode, owner) and no change of order in any window,
  for all three apps (`table.md`, "header-only entries", "order changed").
- What does change without a meaningful change is **inside SQLite's files**.
  The `-wal` files are fixed-size and rewritten from their start after each
  checkpoint, so a few new rows rewrite a large span of them: Kuma's
  heartbeats (2 monitors, every 20 s) touch 6–11 pages of `kuma.db` a
  window but 169–188 blocks of its 1.19 MB WAL. The `-shm` file, SQLite's
  shared-memory index of the WAL, changes whenever the WAL does, and is
  rebuilt from the WAL when a database is opened, so a copy doesn't need it.
- Logs and journal files played almost no part: n8n's `n8nEventLog.log` grew
  578 bytes with the light use, and its empty `crash.journal` is removed at
  a clean stop (`snaps/6.tar vs det/export-a.tar`: "removed crash.journal").
- Stopping an app can change its state: Kuma checkpointed at stop and
  removed its WAL and shm, so the stopped volume's tar is 249 KB against
  1.48 MB running (`kuma/run.log`). n8n kept its 4.14 MB WAL at stop.

Every copy here was consistent, as far as SQLite can say: folding each
copy's WAL into its database in a scratch directory passed `PRAGMA
integrity_check` for all 21 copies (`fold_wal.txt`, `scripts/fold_wal.py`).
This answers, for these apps idle or lightly used, the tutorial pass's open
question of whether a sqlite copied with its WAL is consistent; a pause
mid-transaction under heavy writing was not tested.

## (a) and (b), in MB/hour and GB/day

From `table.md` (`scripts/figures.py`). Group of 20 for (a), 3 holders for
(b), 12 copies an hour.

| app | (a) nominal: per member, 20 members | (a) as measured: per member, 20 members | (b) idle: per holder, 3 holders | (b) one light use, per holder |
|---|---|---|---|---|
| n8n | 79.8 MB/h, 38.3 GB/day | 13.3 MB/h, 6.4 GB/day | 0, 0 | 4.17 MB once |
| Uptime Kuma | 17.6 MB/h, 8.5 GB/day | 17.6 MB/h, 8.5 GB/day | 17.6 MB/h, 1.27 GB/day | (same as idle) |
| Open WebUI | 13,416 MB/h, 6,440 GB/day | 2,236 MB/h, 1,073 GB/day | 0, 0 | 0.24 MB once |

- **(a) nominal** sends every tar. **(a) as measured** sends only the tars
  that differed from the one before, since files move by hash: here 1 of 6
  for n8n and Open WebUI (the light-use window) and 6 of 6 for Kuma. So
  today's design already moves nothing while an app is truly idle, and the
  cost is per use: **one use of Open WebUI sends 1.12 GB to each member,
  22.4 GB across 20**, and one use of n8n 6.65 MB each, 133 MB across 20.
  An app that writes on its own, like Kuma, pays every window.
- **(b)** sends the changed and added files plus the manifest, only when
  something changed. Idle n8n and Open WebUI move nothing. Kuma, always
  writing, moves its three SQLite files whole every window: 1.47 MB, so
  17.6 MB/h per holder; (b) cuts its group total from 8.5 to 1.27 GB/day
  only by sending to 3 holders instead of 20, not by sending less each.
  A light use costs 4.17 MB per holder for n8n (the WAL, whole) and
  0.24 MB for Open WebUI, of which 42 KB is the manifest of 272 entries.
- A new holder first receives the whole state once: 6.6 MB, 1.5 MB and
  1,118 MB.

## (c) What dominates (b), and what would shrink it

- **The SQLite WAL, sent whole, dominates every (b) figure**: n8n's 4.14 MB
  WAL is 99% of its light-use window; Kuma's 1.19 MB WAL is 81% of every
  window; Open WebUI's 169 KB WAL plus its manifest is all of its window.
  The data that actually changed is far smaller. Folding the WAL into the
  database in the copy (`fold_wal.txt`) leaves this many 4 KiB pages
  changed: n8n's use 18 pages (74 KB); Kuma 6–11 pages a window (mean 34 KB,
  0.41 MB/h per holder, 0.03 GB/day for 3 holders); Open WebUI's use 26
  pages (106 KB).
- **Model caches dominate Open WebUI's state**, 1,115 of 1,118 MB. They
  never change, so under (b) they cost only the first transfer to each
  holder, and under (a) they are the whole of the 22.4 GB a use costs.
- Logs don't matter for these three (n8n's event log, 578 bytes a use).

Candidates, named, not built:

1. **Leave `-shm` out of a copy.** SQLite rebuilds it from the WAL; it is
   32 KB of every changed window for all three apps.
2. **Fold the WAL into the database in the copy, never in the app's
   volume**, after the pause and before hashing: the copy's database then
   changes by the pages that changed (24–106 KB here) and the WAL drops out.
   It needs SQLite to recognise the files, and is the largest single cut
   for Kuma (1.47 MB → 34 KB a window, as pages).
3. **Send changed blocks of a changed file**, not the whole file. Fixed
   4 KiB blocks already halve Kuma (9.4 MB/h per holder) and take n8n's use
   from 4.17 MB to 0.11 MB; WAL frames are 4,120 bytes (24-byte header plus
   a page), so fixed 4 KiB blocks straddle them, and content-defined chunks
   would align better. After candidate 2, page-sized blocks of the database
   are exact.
4. **Leave caches out of state.** Open WebUI's `cache/` refills itself on a
   start (it fetched it on this run's first start), so leaving it out would
   take its state from 1,118 MB to about 3 MB, at the cost of a fetch on
   each new keeper. That is a card-level choice (which paths are state).
5. **Send the manifest as a change**, when a copy's manifest is mostly the
   same as the last one's: Open WebUI's 42 KB manifest was 17% of its
   light-use window.

## Left on the box

Every pod, container and volume this pass made was removed. `podman ps
--all` shows only the four containers that were there before
(`sovereign-vulkan`, `sovereign-rocm-7.2.4`, `cw-jellyfin`, `palim`),
`podman pod ps` none, `podman volume ls` only the three `svrngs-gate-*`
volumes; nothing listens on 15678, 13001 or 18090. The 12 GB of tars were
deleted; `~/svrngs-churn` (672 KB: the scripts, run logs and analysis) is
left. Images left, with their sizes: `ghcr.io/n8n-io/n8n@sha256:9a8d8dce…`
1.21 GB and `mirror.gcr.io/louislam/uptime-kuma@sha256:70233f4a…` 478 MB
(pulled for this pass); `ghcr.io/open-webui/open-webui:v0.12.5` 4.64 GB and
`python:3.12-slim` 124 MB (already there).

## Not tried

A busy app (many writes a minute), a pause landing mid-transaction, any app
not on SQLite (Postgres, flat files), Open WebUI with a model answering
chats, longer than 30 minutes (SQLite's checkpoints, log rotation), and
Give's own keeper program (the box's checkout has no `apps/give`, as the
tutorial pass found); `run.sh` follows `runtime.rs` by hand.
