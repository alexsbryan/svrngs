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

**Since: the plants out (syp-2).** The switch ran at the runs' commits,
sy-1's at `ea0f3a5` (§7) and syp-1's rerun at `ff62f07`
(`demo/runs/syp/2026-10-10/no-stream/`), and is deleted from product code
at syp-2-plants-out's commit (`ralph/next/syp/STATE.md`). From that commit
each plant is a test's red run, the guard taken out of the code and the
test of `crates/svrngs-node/tests/folders.rs` watched failing: `deaf`,
`a_change_by_any_member_reaches_another_s_directory`; `own-only` and
`open-gate`, `a_synced_file_is_served_by_a_member_who_did_not_write_it_and_to_no_one_outside`;
`later-wins`, `both_sides_of_a_conflict_are_kept`; and group folders'
three that guard the synced folder too, `unchecked`,
`a_hash_no_synced_folder_places_is_refused_though_the_store_holds_it`,
`follow-links`, `a_link_in_a_synced_directory_is_not_followed`, and
`unbounded`, `a_holder_asleep_is_waited_on_for_a_bound_then_the_next_asked`.
`run.py` still sets the variable, which nothing reads after that commit:
its plant phases there run the guard in, and are not a plant's run.

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

## 8. What the run gives (sy-2)

**A group's folder, a real directory on each member's disk, synced both
ways: an add, an edit and a delete by any of three reached the other two
within 2 s; a file stayed reachable with its author killed; two edits made
apart were both kept; and a node outside the group was given no byte.**
Written after sy-1's data (§7; every number below is in `run/summary.json`
by its key, `run/events.jsonl`, or the file named), by §3's bars, each read
for what it guards. All four passed, each plant failing first with its
guard out (§7, "The plants"). The index is the record, one act per scan's
changes, and the bytes move by group folders' provider and verified fetch,
unchanged (§1).

| Bar | Letter | Spirit — what it guards | Data |
|---|---|---|---|
| S1. Both ways | passed | held: any member's change reaches every other's disk, on one host | add 1.06 s, edit 1.63 s, delete 1.36 s, bound 10 s (`S1`); with Ana's scan out her add reached no one in 10 s (`S1-plant`) |
| S2. Its author asleep | passed | held for this run: a file is reachable while one member who holds it is up; a store that loses what it held is a hole in it (4) | Cy held `s2.bin` 4.47 s after her command, from Ben (`S2`), Ana passed over at 3 s (`cy.log:76`); Ben serving only what he wrote, from no one in 10 s (`S2-plant`) |
| S3. Conflicts kept | passed | held for an edit made while its writer's node was stopped (2) | 17.0 s after Ben's start, bound 30 s: `s3.txt` Ana's `36f7c2fd…`, `s3 (conflict Ben).txt` Ben's `eb066e59…`, in all three (`S3`); with the later edit winning, Ben's `05513869…` in none (`S3-plant`) |
| S4. Members only | passed | held for a node in no group held here; a member of another group Ana holds was not asked (1); a member writing outside the folder was not asked (3) | Dee refused on both protocols, 0 hits in her data dir, 0 of the planted file anywhere (`S4`); with the gate out, the bytes given whole, `payload` 65,536, and 2 hits (`S4-plant`) |

**Where the letter and the spirit part**, each judged by the spirit:

1. *S4 asked one outsider; group folders' second gap is this one's too.*
   Dee stood in no group of Ana's. The gate admits a dialer standing in any
   group held here (`admit_dialer`, `crates/svrngs-node/src/node.rs:760`);
   the provider serves a hash any synced folder in any group held here
   places (`shares`, `crates/svrngs-node/src/folder.rs:265-273`, through
   `placed`, `synced.rs:67-78`); and the node writes each scan's act into
   every group held here (`write`, `folder.rs:236-258`) and places every
   group's folder of that name in the one directory (`apply`,
   `synced.rs:152-156`), fetching from the members of every group
   (`:157-162`). The run held one group (`events.jsonl`, setup: `new
   Shared`, two joins), where the two are one. Read from the code, not run;
   as group folders' §8, fifth part. Candidate 2.
2. *S3 kept an edit made while its writer's node was stopped*, which §3's
   letter allowed: Ben's edit was written to his directory while his node
   was down (`events.jsonl`, S3: Ben down at …304.086, his write at
   …304.088), and found at his start by its size and mtime against what he
   last held (`unchanged`, `synced.rs:38-44`), then sent (`ben.log:155`).
   That line names no base (`synced.rs:133`), a trace short of AGENTS.md
   1, and candidate 9's; both edits kept as heads says Ben's base was the
   entry both edited, not Ana's edit. Two nodes running and unable to
   reach each other take the same path — the once-a-second scan sends with
   the base held — read from the code, not run.
3. *A member can write outside the folder, on every other member's disk.*
   A change's path is a string its writer names (`Change.path`,
   `crates/svrngs-core/src/synced.rs:34`), placed as written
   (`synced.rs:91`), joined to the directory (`synced.rs:184`) and the
   fetched bytes exported there (`:203`); nothing refuses `../` or an
   absolute path, which `Path::join` takes in place of the directory. A
   conflict's name carries its writer's person (`beside`, `:114-124`),
   which `Name::parse` refuses only empty, past 64 bytes or with a control
   character (`crates/svrngs-core/src/lib.rs:207-220`), so a `/` in it is
   a directory. S4 guards what leaves the group; this is what a member's
   act does to another member's disk, which no bar asked. Read from the
   code, not run. Candidate 7.
4. *A member's store gave nothing of files its directory still held*, three
   times (§7, "Seen, not asked"): Ana's `s2-plant.bin` and `s2.bin` after
   her `kill -9` (`ana.log:13741`, `:13745`, `done=false payload=0`), and
   Cy's `s2.bin` (`cy.log:2481-2482`) after SIGTERM stops alone. S2 passed
   because Ben's store held `s2.bin`; had his lost it as Cy's did, a file
   on two members' disks would have reached no one. Why is not
   instrumented by this run: a store's loss is to be reproduced before
   anything is changed (AGENTS.md 2). Candidate 8.
5. *A refusal named at the connection, not on the record's answer.* Dee's
   member connection and her ask for the bytes were each told "closed by
   peer: not admitted (code 0)"; her ask for the record over that
   connection failed as `IncompleteMessage`, its words not naming the
   refusal (`run/dee.out`). The refusal is the connection's close, named
   there; the ask over it says only that its answer broke. Said; it waits
   (§9).

**The measurements, no bar, the operator's call.**

- *M1, 1,000 files of 4 KiB*: Ben and Cy held all 1,000 after 395.27 s,
  Eve, joining later, after 242.24 s, where §5 predicted about 20 s. The
  time is the fetches, one file after another (`apply`'s loop,
  `synced.rs:183-214`, each file's pieces awaited in turn, `:193-198`): 0.38
  s a file at Ben, 0.25 s at Eve (§7), on a host at load 40.38
  (`events.jsonl:1`). The index: Eve's journal 133,738 bytes once she held
  the folder, its longest line 80,300; one later change moved 1,166 bytes
  of it, against about 700 predicted, and reached her in 7.1 s. Every
  change stays in the journal: §9's compaction row, unchanged.
- *M2, 1 MiB rewritten inside 100 MiB*: Ben and Cy each received
  104,857,600 bytes, the whole new version, against 1,048,576 edited, as
  §5 predicted: a version is fetched by its own hash, and nothing of the
  old one counts. The first version reached both in 96.23 s, the edit in
  74.34 s. Whether a member's every edit costing each other member the
  whole file is acceptable is the operator's call (§9, "Not candidates").

So a group's folder synced both ways on its members' disks does what a
Dropbox's folder does for three members on one host: anyone's change
everywhere within 2 s, a file kept while its author sleeps, a conflict kept
beside its file with its writer's name, and no byte to a node outside the
group. What the run did not reach: a second group held by one member, a
path that leaves the folder, a store that keeps what it held across a stop,
a folder of many files in reasonable time, a large file's small edit, and
everything off this host. The spike cost 543 lines where §6 estimated
about 480, 13% over. One host, one run of each: not a rate (AGENTS.md 7).
DESIGN §3, §8 and §9 are rewritten for it on this branch.

## 9. What makes a mesh Dropbox production (sy-2)

Each part sy-1 counted (§7, by `scripts/lean_check.py`'s rule,
`396b0b6..4926df0`), and what production keeps, changes or drops of it:

| Part | Lines | Production |
|---|---|---|
| the synced act and its layout, in svrngs-core | 132 | keeps; a path is parsed, not a string (7); the layout read per group (2) |
| the node's scan, write and apply, and what it keeps | 180 | keeps; writes one group's act and applies one group's layout (2); its kept state refused by name when unreadable, and the directories it syncs kept (9); fetches several files at once (10). The plants' lines, 5 (`synced.rs:74-76`, `:88` and its close), drop |
| the provider serving synced files, the split share and the plants | 44 | keeps the provider's rule and the split; the plants, 10 (`folder.rs:62-66`, `:113-115`, `node.rs:756-759`), drop — each stays a test's red run |
| wiring: `share --folder --both-ways` | 7 | keeps; names its group (2) |
| the run's script, `run.py` | 180 | the run's |

**Against group folders' six** (`demo/runs/gf/2026-10-09/README.md` §9).
The synced folder writes each file onto each member's disk (`apply`'s
export, `synced.rs:203`), so an app on that computer reads its own
directory; it never asks the loopback endpoint (`bytes`,
`crates/svrngs-node/src/folder.rs:424-535`), which today binds on every node
that runs folders (`start`, `:122-145`).

| gf's row | For a mesh Dropbox |
|---|---|
| 1. A stub that reaches its viewer | drops: the bytes are on the viewer's disk |
| 2. A folder at one level, served only there | carries over, wider: the synced act, its layout, its fetch's holders and the provider's rule each per group, not across every group held here (§8's first part) |
| 3. Who the endpoint answers | drops with the endpoint, if the operator drops streaming; carries over unchanged if not |
| 4. Stubs in step with the record | drops: no stubs |
| 5. What the meeting point keeps | changes: every member's store copies each version it takes (`ImportMode::Copy`, `synced.rs:96`) and keeps each it fetched, and drops none; and the store also lost what it held across a stop (§8's fourth part). Candidates 5 and 8 |
| 6. A holder that sends wrong bytes, planted | carries over unchanged: the apply's fetch is the same `hold` (`synced.rs:195`). Tests only |

Whether a group's folder is streamed by range (group folders), synced onto
every disk (this spike), or both, is the operator's: a film folder synced is
every member holding every film, M2's whole copies at each edit; a synced
folder streamed is no folder on anyone's disk. The rows below are for the
synced folder; rows 1, 3 and 4 of group folders' stay as they are written
there for a streamed one.

Candidate rows, the operator's to queue, numbered as group folders' where
one carries over and from 7 where new. Each check is a bar rerun with its
plant, red first:

- **2. A synced folder is one group's.** `share --folder <dir> --both-ways`
  names its group; its act is written there alone; the directory holds
  that group's layout alone; its files are fetched from that group's
  members and served by the provider only to a dialer standing there.
  With group folders' row 2, one change: the level is the group's.
  Evidence: §8's first part. Check: Ana in two groups, each with a folder
  `Shared`; a member of the other refused her hash by name and her
  directory holding the one group's files alone, then the same with the
  per-group rule out.
- **5. A store that keeps what the layout places, and no more.** A version
  no head names, nor the directory holds, is dropped from the store at the
  next record change; a conflict's other side and a file the directory
  holds stay. Check: M2 rerun, each member's store holding the edit's
  version and not the first, measured in bytes; with the drop out, both.
- **7. A path under the folder, parsed.** A change's path is a type of the
  core's, made only from a relative path of named parts — never empty,
  `.`, `..`, a root or a prefix — on admission and on the write that makes
  it, and a conflict's name made from it and a person whose `/` cannot
  make a part. Evidence: §8's third part. Check: a member's act naming
  `../outside` refused by name at admission, nothing written beside the
  directory; with the parse out, the file written there. A test of
  svrngs's, red first.
- **8. A store that keeps what it held across a stop, reproduced first.** A
  test of svrngs's: a file taken and a file fetched, the node stopped by
  SIGTERM and by `kill -9`, started, its provider asked for each. If it
  reproduces, the fix is its row's; if not, the run's step that lost them
  instrumented and rerun. Evidence: §8's fourth part. Check: S2 rerun with
  Ben's node stopped and started before Cy's start, Cy holding the file
  from him.
- **9. What a node keeps of its syncing.** The directories it syncs kept in
  its data dir, so a restart resumes them (today the command is asked
  again at each start, §2); its kept state, `synced-<folder>.json`, read
  or refused by name, never read as empty — today an unreadable one is
  read as nothing held (`synced.rs:82-85`), so every file is sent again
  with no base and placed beside every other member's entry for it as a
  conflict (the layout's rule, `crates/svrngs-core/src/synced.rs:130-146`;
  read, not run); and a change sent names its base in its trace (§8's
  second part). Check: the kept state planted unreadable, the node
  refusing to sync that directory by name; with the refusal out, every
  file conflicted in every directory.
- **10. Several files fetched at once.** The apply fetches each file after the
  last (`synced.rs:183-214`); in its place a bound of fetches at once, the
  row's. Evidence: M1, 395.27 s for 1,000 files. Check: M1 rerun against
  a bound the operator sets from that number; with one at a time, the
  bound missed.

Group folders' row 6 carries over as its row, unchanged.

**Since: streaming dropped (syp-1).** The operator, 2026-10-10, chose the
synced folder alone, "Drop streaming + share": group folders' endpoint
(`bytes`, cited above) and their one-way share are deleted on this branch
(`ralph/next/syp/STATE.md`, syp-1-no-stream), so rows 1, 3 and 4 of theirs
wait with a mounted folder (below; DESIGN §9). S1-S4 ran again after it,
each plant failing first and each bar passing
(`demo/runs/syp/2026-10-10/no-stream/`).

Not candidates, said so, each with what brings it back:

- **A large file's small edit costing the whole file** (M2): fetching by
  the parts that changed is a version as a list of chunks' hashes, a new
  layout in the record and the fetch. The operator's call after M2, or a
  group's edits to a large file measured too costly.
- **A phone in the folder**: a phone holds no directory a person edits as a
  computer's, and iOS suspends an app soon after it leaves the screen
  (DESIGN §9, "a native phone app"). After the computer (DESIGN §10,
  2026-10-09), with the phone replica spike, once its endpoint is shown to
  stay reachable.
- **A mounted folder**: every file listed from the layout and fetched on
  open, by verified range as group folders' endpoint fetches — the two
  shapes met — needs a file system of svrngs's (macFUSE, or Apple's File
  Provider), a dependency and a process apart. Waits for a member whose
  disk cannot hold the group's folder, counted.
- **Many groups, many folders**: each folder is walked once a second
  (`both_ways`, `synced.rs:53-65`) and each step and each provider answer
  reads every group's layout from every act (`placed`, `:67-78`), unmeasured
  here. A watcher of the file system in place of the walk, and a layout
  kept in place of read, wait for a step measured near its second.
- **The journal's growth**: every change stays, 1,166 bytes a change at M1;
  DESIGN §9's compaction row, its trigger unchanged.
- **The record's refusal named on its answer** (§8's fifth part): waits for
  a person refused who is told nothing.
- **A member removed keeps what they hold**: what has reached a member's
  disk, a removal does not take back, as group folders' stop (its §8,
  third part). Said; never a candidate.
- **Real links and a rate**: the operator's run with a group in the wild
  (DESIGN §9's row), after these rows.

**The budget.** By `python3 scripts/lean_check.py` at `8bf279d`: 15,707 of
15,750 on this branch, svrngs-core 2,873 of 3,000. A mesh Dropbox's
production drops the plants' 15 (above) and group folders' 21 and its
digest's 3 (its §9), and, with streaming dropped, the endpoint's 138
(`folder.rs:420-567`, by the same rule) and the rest of its wiring; it adds,
by the same rule, about 50 for one group's folder (2), 30 for the store's
drop (5), 30 for the parsed path (7), 20 for the store kept across a stop
(8, until it is reproduced), 35 for what a node keeps (9) and 15 for
fetches at once (10): about 180, svrngs-core's part about 40. That is about
15,710 with streaming dropped, svrngs-core about 2,913. sy-1 ran 13% over its
estimate (§7) and group folders 52% (its §7); the same misses on the 180
make about 15,730 and 15,800. With streaming kept, group folders' rows 1
and 3 add their 10 and 40 and the endpoint stays: about 15,900. Either is
past main's 15,200 (DESIGN §5); the raise is the operator's, with the
merge (the queue's fence 5).
