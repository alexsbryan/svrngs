# A group's folder, synced both ways on its members' disks — 2026-10-09

The mesh-Dropbox spike (`docs/DESIGN.md` §8 "Then", §9's folders row, §10;
the operator, 2026-10-09; queue `ralph/next/sync/STATE.md`, row
sy-1-build-run): **does a group's folder, a real directory on each member's
disk, sync both ways — added, edited, deleted by anyone — stay available
while its author sleeps, keep both sides of a conflict, and give no byte to
anyone outside the group?** Group folders' spike (`demo/runs/gf/2026-10-09/`)
proved one holder's read-only folder streamed, verified, into one Jellyfin,
nothing beyond the folder leaving its holder. This one asks what a Dropbox
needs that it did not: writes by anyone, both ways, on disk; a file kept
while its author sleeps; conflicts kept; members only.

This file is sy-1's first commit: the question, what exists at HEAD, the
index chosen, the slice, the bars and their plants, how each number is
measured, a prediction each, and the line estimate, committed before any
code or data of the spike. None of it moves after its data; what the data
gives is written below it, in later commits. One host, one run of each: not
a rate (AGENTS.md 7). A run that could not finish is could-not-judge, never
a pass (AGENTS.md 6). A member that did not answer is never an empty folder,
and a refusal is never an empty answer.

## 1. What exists at HEAD (`e4e5437`), and what does not

### Exists — group folders' spike, reused before anything is written again

- **The folder's act.** `RailAct::Folder { name, files }`
  (`crates/svrngs-core/src/lib.rs:439`), each file by path, size and BLAKE3
  hash (`SharedFile`, `crates/svrngs-core/src/folder.rs:18-27`), shipping at
  `FOLDER_LINE_VERSION`, 5 (`lib.rs:712`). Each key's newest list per name is
  its folder; an empty list stops it (`folder.rs:58-84`). It is one
  holder's list, replaced whole: it cannot say who changed a file, from
  which version, nor that a file was deleted by someone other than its
  holder.
- **The provider.** iroh-blobs 0.103 in the node, the store under
  `<data>/blobs` (`crates/svrngs-node/src/folder.rs:87-142`), a file taken
  in by reference (`ImportMode::TryReference`, `:169`) without following a
  link (`symlink_metadata`, `:159`). Every get is asked of the record on
  each request (`provider`, `:321-366`): served only while a standing folder
  of this computer's names the hash (`shares`, `:219-224`).
- **The verified fetch.** `hold` makes a range of a hash held here,
  verified, asking each holder in turn, a holder that did not answer within
  `ANSWER_WITHIN`, 3 s, passed over and named (`folder.rs:234-285`; `fetch`
  `:287-318`).
- **`share --folder` and its stop**, asked of the running node by the
  record API (`crates/svrngs/src/cli.rs:260`, `:291-293`, `:548`;
  `FolderAsk`, `folder.rs:538-542`).
- **The record's membership, and the gate that asks it.** Who stands is
  computed from the seed and the `Admit`/`Remove` acts alone
  (`crates/svrngs-core/src/membership.rs:118`, `:178`). The accept loop
  serves the member protocol and the blobs protocol only to a dialer
  standing in a group held here, and closes the rest (`admit_dialer`,
  `crates/svrngs-node/src/node.rs:739-762`, applied at `:285`).
- **The record's sync.** A write nudges a round at once, and the round
  pushes and pulls with each peer (`crates/svrngs-node/src/ring_sync.rs:28-49`,
  the nudge `node.rs:197-201`); every 60 s otherwise (`ring_sync.rs:104`).

### Does not exist

- **A change by anyone.** No act names a file's change, by whom, from which
  version, or its delete.
- **A directory kept in step.** Nothing watches a directory, nor writes
  another member's change into one.
- **Conflicts.** Nothing tells two edits from one version apart from an edit
  made after another.
- **A synced file served by a member who did not write it.** `shares` asks
  only this computer's own folders (`folder.rs:219-224`).

### The index: svrngs's record, not iroh-docs

iroh-docs 0.101.0 (cited from the cargo registry's
`iroh-docs-0.101.0/`) is n0's multi-writer sync, and takes this branch's
iroh, iroh-blobs and an iroh-gossip. Three of its facts decide against it
for this spike:

1. *Who may read is its own.* A replica is read by whoever holds its
   namespace id (`Capability::Read(NamespaceId)`, `src/sync.rs:186-191`),
   reconciled on its own ALPN, `/iroh-sync/1` (`src/net.rs:18`). Gating that
   ALPN by the record would leave two deciders of one question, the id and
   the record (fence 3; AGENTS.md 8).
2. *It keeps no base.* An entry's record is its length, hash and timestamp
   (`src/sync.rs:1127-1134`), ordered by timestamp then hash
   (`:1138-1145`): an edit made after seeing another and an edit made
   without seeing it look the same, so S3's "both versions kept" could be
   told from neither the later edit winning nor a false conflict.
3. *Its own sync beside the record's.* An engine, a gossip and its own
   authors to wire, where the record already carries signed acts to every
   member each round, at once after a write.

So the index is svrngs's record: a new act naming each change by path, its
bytes by hash (none for a delete), and the entry it replaces. Bytes move by
the iroh-blobs serving and fetch above, unchanged.

## 2. The slice

**The act.** `RailAct::Synced { folder, changes }`, at a new line version:
each change names a path under the folder, its bytes now by hash and size
(none for a delete), and its *base*, the entry it replaces — key, seq and
path — as the writer last held it (none for a new path). An entry is one
change, named by its writer's key, the act's seq and its path. **The
layout**, decided once in svrngs-core: an entry no other names as its base
is a head; a path's live heads are its file, the first by key under its
name and each other beside it, named as a conflict with its writer's
person (`notes (conflict Ben).txt`); a path whose heads are all deletes is
absent. Writing to a conflict's copy, or deleting it, names its entry as
base, so the conflict ends by the same rule.

**The node.** `svrngs share --folder <dir> --both-ways` keeps `<dir>` the
same as the folder of its name in the group: once a second, the node scans
the directory without following links, takes in each new or changed file
by reference (the provider's own import) and writes one act of the
changes; then it reads the layout and writes into the directory what it
lacks — fetched whole, verified, from any standing member, the writer
first — and deletes what the layout no longer places, never touching a file
changed on disk since the scan. What the node last wrote or read for each
path, and from which entry, is kept in its data dir, so an edit made while
it was stopped is found and sent with its base on start. The provider
serves a hash the record places in a synced folder, to a dialer standing
in the group, whoever wrote it.

**Who.** Four people, each a node of their own on this host with its own
data directory, through iroh's public relays, as group folders' run
(`apps/jellyfin/folders/run.sh:44-55`):

- **Ana, Ben and Cy** in one group, each syncing a directory of the same
  name, `Shared`, under the run's scratch dir. Beside Ana's `Shared`,
  outside it, is **the planted file**: 1 MiB of random bytes with a marker
  every 4 KiB, and a link to it inside `Shared`, so a walk that follows
  links reaches it.
- **Dee**, a node outside the group, given the group's id (the folder's
  is the group's id and its name) and a file's hash: a probe of the run's,
  under the node's tests (`crates/svrngs-node/tests/`), starts a node on
  Dee's data dir, asks Ana's node for the group's record on the member
  protocol and for the file's bytes on the blobs protocol, and keeps what
  it is given in that data dir.
- **Eve**, for the first measurement: joins after the folder holds 1,000
  files.

svrngs moves files by hash and path and knows no document, photo or film
(`scripts/vocab_check.py`). The slice writes to the folder of its name in
each group held here, as `share --folder` does (gf's §8, part 5); the run
holds one group. The command is asked again at each node start: which
directories a node syncs is not kept across a restart in the slice.

## 3. The bars and their plants

Each plant is watched failing with its guard taken out, then refused.

- **S1. Both ways.** A file Ana adds, an edit Ben makes and a delete Cy
  makes each reach the other two directories, byte-identical or gone,
  within 10 s. *Plant:* a change written while the watcher is out of the
  loop, never sent — Ana's node with its scan out (`deaf`), her add written.
- **S2. Its author asleep.** Ana adds a file while Cy's node is stopped;
  once Ben's directory holds it, Ana's node is killed (`kill -9`, no
  goodbye, as a computer that sleeps); Cy, started, holds the file within
  10 s, from Ben. *Plant:* each member keeping only what it wrote — Ben's
  provider serving only entries Ben wrote (`own-only`).
- **S3. Conflicts kept.** Ana and Ben edit one file while their nodes cannot
  reach each other: Ben's node stopped, Ben's edit written to his directory
  while it is down, Ana's edit written and reaching Cy, then Ben's node
  started. Reconnected, all three directories hold both versions, one under
  the file's name and the other beside it named as a conflict, no edit
  lost, within 30 s of Ben's start. *Plant:* the later edit simply winning —
  every node placing only a path's first head (`later-wins`).
- **S4. Members only.** Dee is refused the group's entries and the file's
  bytes, by name; a scan of Dee's data dir finds none of the file; and the
  planted file beside Ana's `Shared` never leaves her: a scan of Ben's,
  Cy's and Dee's data dirs and directories finds none of it. *Plant:* the
  gate that asks the record taken out — Ana's node admitting any dialer to
  the member and blobs protocols (`open-gate`).

The plants are one switch, read once at a node's start, as group folders'
(`SVRNGS_GF_PLANT`, `crates/svrngs-node/src/folder.rs:45-56`); never set
outside the run.

**Two measurements with no bar**, the operator's call after the data:

- **M1. 1,000 files reaching a member who joins later.** Ana's `Shared`
  given 1,000 files of 4 KiB; once Ben and Cy hold them, Eve joins and
  syncs: seconds from her command to her directory holding all 1,000,
  byte-identical; the index bytes the 1,000 took; then one later change of
  Ana's, and the index bytes it moved to Eve.
- **M2. A 1 MB edit inside a 100 MB file.** A 100 MiB file of random bytes
  synced to all three; then 1 MiB of it at offset 50 MiB overwritten in
  Ana's directory: bytes each of Ben and Cy received for the new version,
  against 1 MiB.

## 4. How each number is measured

- **A directory's state**, by the run's script polling each directory every
  100 ms: each file's path and BLAKE3, against what the step wrote. A
  bar's time is from the step's write returning to the first poll that
  matches in every directory named, or the bound passing (then the bar's
  number is the bound, failed).
- **Who gave what**, from each node's log at `folder=info`: each fetch's
  hash, holder, payload and other bytes (`hold`'s line,
  `crates/svrngs-node/src/folder.rs:268`), each provider answer, refusal,
  and each write and apply of the sync, with its path, hash and entry.
- **S1**: the three times, Ana's add at Ben's and Cy's, Ben's edit at Ana's
  and Cy's, Cy's delete at Ana's and Ben's.
- **S2**: from Cy's command returning to her directory holding the file,
  byte-identical; the holder from Cy's log.
- **S3**: all three directories' listings and hashes after Ben's start,
  both edits' hashes found in each, the one under the file's name and the
  conflict's name said; the time from Ben's start.
- **S4**: Dee's probe's answers, each refusal as the probe was told it;
  the scan: every file under Dee's data dir, and under Ben's, Cy's and
  Dee's data dirs and directories for the planted file, searched for the
  synced file's BLAKE3 hex and its first 64 bytes, and for the planted
  file's marker.
- **M1**: the time as S1's; the index bytes as Eve's journal file's size
  (`ring_oplog.jsonl`) before and after, and the act's line alone.
- **M2**: the payload and other bytes Ben's and Cy's logs say they fetched
  for the new version's hash.

## 5. Predictions

- **S1: pass, each about 3 s**: the scan's second, the nudged round's push
  through the relays, the receiver's second, a fetch of a small file.
- **S2: pass, about 6 s**: Cy's start runs a round at once, Ben giving
  Ana's act; Cy's fetch asks Ana first, the writer, waits out
  `ANSWER_WITHIN`, 3 s, then Ben. The risk named: the round itself waiting
  on Ana's dial.
- **S3: pass**: Ben's start scans before it applies anything, so his edit
  is sent with its base, Ana's entry; two heads, both kept. Which is under
  the file's name is the key order's, said, not chosen.
- **S4: pass**: Dee's connections closed at the accept on both protocols,
  given nothing; 0 hits in every scan.
- **M1: about 20 s** for the 1,000 (one act of about 150 KB, then 1,000
  fetches in turn over one connection); one later change about 700 bytes
  of index.
- **M2: about 100 MiB each**, a hundred times the edit: a fetch is by the
  new version's hash, whose tree the store has never held, so nothing of
  the old version counts.

## 6. The lines, estimated

About 480 lines counted by `scripts/lean_check.py`'s rule, against 587
left of 15,750 and 259 of svrngs-core's 3,000 (`python3
scripts/lean_check.py` at `e4e5437`: 15,163 and 2,741):

| Part | Lines |
|---|---|
| the synced act and its layout, in svrngs-core | about 90 |
| the node's scan, write and apply, and what it keeps of each path | about 170 |
| the provider serving synced files, and the plants | about 20 |
| wiring: the route, `share --folder --both-ways` | about 20 |
| the run's script, under `demo/runs/sy/2026-10-09/` | about 180 |

Dee's probe and svrngs's own tests are tests, apart. Group folders' spike
ran 52% over its estimate (430 to 655, its §7); the same miss here is about
730, past 587. A count that would pass either budget stops for the operator
(fence 5).

## 7. The run of 20:33, against the predictions

`python3 demo/runs/sy/2026-10-09/run.py` at `ea0f3a5`, exit 0 after 16m08s;
its data in `run/` (`events.jsonl`, `summary.json`, `dee*.out`, each node's
log). The host was loaded: load averages 40.38 43.17 63.64 at its start
(`events.jsonl:1`). The verdict, read for the bars' spirit, is sy-2's; this
section gives the numbers and where each is. A partial run of 20:28, kept in
`partial-2028/`, is not the bars' data (its commit `ea0f3a5` says why).

### The plants, each watched failing with its guard out

| Plant | Guard out | What the run saw (`summary.json`) |
|---|---|---|
| S1 | `deaf`: Ana's scan out | Ana's add reached neither Ben nor Cy in 10 s: `add_s` null, failed |
| S2 | `own-only`: Ben serves only what he wrote | Cy did not hold Ana's file in 10 s, from no one: `cy_s` null, `from` [], failed |
| S3 | `later-wins`: only a path's first head placed | after Ben's start all three held Ana's edit alone (`e59a14e6…`), Ben's (`05513869…`) in none: failed |
| S4 | `open-gate`: any dialer admitted | Dee's member connection open after 5 s; the bytes given whole, `payload` 65,536; the scan found 2 hits in Dee's data dir, `asked/blobs.db` and `asked/data/e0049707….data`: failed |

S4's plant shows a second refusal, not the gate's: with the gate out, Dee's
ask for the record was answered 403, 163 bytes, "e5b4aa9d… is not served to
72518b4c… here" (`/internal/ring/sync`, `ring_routes.rs:98`). The record's
route refuses a non-member on its own; the file's bytes have no such second
refusal, so the gate is the one that keeps them.

### The bars

| Bar | Predicted | Run | Number |
|---|---|---|---|
| S1 both ways | pass, about 3 s each | **passed** | Ana's add at Ben and Cy 1.06 s; Ben's edit at Ana and Cy 1.63 s; Cy's delete gone at Ana and Ben 1.36 s (bound 10 s) |
| S2 its author asleep | pass, about 6 s | **passed** | Cy held `s2.bin` byte-identical 4.47 s after her command, from Ben (bound 10 s) |
| S3 conflicts kept | pass | **passed** | 17.0 s after Ben's start (bound 30 s), all three directories: `s3.txt` Ana's edit `36f7c2fd…`, `s3 (conflict Ben).txt` Ben's `eb066e59…` |
| S4 members only | pass | **passed** | Dee refused on both protocols, 0 hits in Dee's data dir, 0 of the planted file anywhere |

- **S1** (`events.jsonl`, phase S1): each change within 2 s, under the 3 s
  predicted.
- **S2**: Cy asked Ana first, the writer, and passed her over at
  `ANSWER_WITHIN` — "a holder gave nothing of the range … did not answer
  within 3s" (`run/cy.log:76`) — then fetched it whole from Ben, verified,
  in 33 ms (`cy.log:77`, holder `f0e1461d…`, Ben's key, `events.jsonl`'s
  `keys`). The round did not wait on Ana's dial (the risk named).
- **S3**: Ben's start sent his edit with its base (`run/ben.log:155`, "a
  change sent"), then wrote Ana's under the name and his own beside it
  (`ben.log:159-160`, `beside=true`). Which is under the name is the key
  order's: Ana's key `7c1ce9b3…` sorts before Ben's `f0e1461d…`. 17.0 s
  against the bound of 30 s; the prediction gave no time.
- **S4**: Dee's three answers (`run/dee.out`): the member connection
  "closed by peer: not admitted (code 0)"; the record's ask, over that
  protocol, failed under it, `IncompleteMessage`, its own words not naming
  the refusal; the bytes "connection lost … closed by peer: not admitted
  (code 0)". **The scan**: every file under Dee's data dir
  (`/tmp/sy-run/dee`: `api`, `asked`, `blobs`) searched for `s4.bin`'s
  BLAKE3 hex, its 32 bytes and its first 64 bytes: 0 hits; Ben's and Cy's
  data dirs and directories and Dee's searched for the planted file's
  marker: 0 hits (`beside_hits`), in the plant's run as in the bar's.

### The measurements, no bar

- **M1. 1,000 files of 4 KiB.** Predicted about 20 s. Ben and Cy held all
  1,000 after **395.27 s**; Eve, joining later, after **242.24 s** from her
  command. The time is the fetches, one file at a time: Ben wrote the
  first at 03:36:20 and the last at 03:42:42 (`run/ben.log`, "written into
  the directory", 0.38 s a file), Eve from 03:42:44 to 03:46:53
  (`run/eve.log`, 0.25 s a file), on the loaded host. Index: Eve's journal
  133,738 bytes once she held the folder, its longest line, an act of
  Ana's changes, 80,300 bytes (Ana's scans took the 1,000 in two acts, seq
  12 and 13 at Ben's log); one later change reached Eve in 7.1 s and moved
  **1,166 bytes** of her journal, against about 700 predicted.
- **M2. 1 MiB rewritten inside 100 MiB.** Predicted about 100 MiB each.
  Ben and Cy each received **104,857,600 bytes** for the new version, a
  hundred times the 1 MiB edit (`summary.json`, from each log's "fetched,
  verified" lines for `2b41a956…`, 1 MiB ranges from Ana, e.g.
  `ben.log:2596`). The first version reached both in 96.23 s, the edit in
  74.34 s.

### Seen, not asked

A member's store gave nothing of a file its directory still held, three
times: Ana's `s2-plant.bin` and `s2.bin` after her `kill -9`
(`run/ana.log:13741`, `:13745`, `done=false payload=0`); and Cy's `s2.bin`
at 03:46:47 (`run/cy.log:2481-2482`), 12 minutes after she fetched and
verified it (`cy.log:77`), with only SIGTERM stops between
(`events.jsonl`). Eve took it from Ben (`run/eve.log:2017`). The other
`payload=0` answers in the logs are members asked for a file before they
had fetched it themselves. Why a stopped node's store loses what it held
is not instrumented by this run.

### The lines, by `scripts/lean_check.py`'s rule, `396b0b6..HEAD`

| Part | Estimated | Counted |
|---|---|---|
| the synced act and its layout, in svrngs-core (`synced.rs` 111, `lib.rs` 9, `admit.rs` 9, `membership.rs` 3) | about 90 | 132 |
| the node's scan, write and apply, and what it keeps (`svrngs-node/src/synced.rs`) | about 170 | 180 |
| the provider serving synced files, the split share and the plants (`folder.rs` 40, `node.rs` 4) | about 20 | 44 |
| wiring: `share --folder --both-ways` (`cli.rs`) | about 20 | 7 |
| the run's script, `run.py` | about 180 | 180 |
| **total** | **about 480** | **543**, 13% over |

`python3 scripts/lean_check.py`: 15,706 / 15,750, svrngs-core 2,873 /
3,000. Dee's probe (`crates/svrngs-node/tests/sy_dee.rs`) and the tests are
tests, apart.
