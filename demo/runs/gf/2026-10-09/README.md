# Group folders, second spike: a folder's films by verified range — 2026-10-09

Group folders' second spike (`docs/DESIGN.md` §9's row, §3 "Lent apps" call
7, §8 "Then"; the operator, 2026-10-09; queue `ralph/next/folders/STATE.md`,
row gf-0-rule): **do a group's films, shared as a folder, play in one
Jellyfin with their bytes fetched by verified range from whoever holds them,
and nothing else leaving a holder?** The first run (svrn-docs
`rings/runs/strm-2026-10-05/`, `457f120`) put Jellyfin 12.1.0 over two `.strm`
stubs pointing at a range server: every browser played the MP4 straight from
the stub's URL, WebKit remuxed the MKV through Jellyfin's own HLS, and it
failed its bar 2, 93.0% and 85.8% of the MP4's bytes going to the browser,
on Jellyfin's per-play `bytes=0-` probe of a 47 MB file (its `README.md`,
`run/summary.txt`). This spike puts what DESIGN §9 names in the range
server's place: the folder's file list in the group's record, its bytes
content-addressed and fetched by verified range from whoever holds them
(iroh-blobs 0.103), served to Jellyfin by the meeting point's own node.

This file is gf-0: the question, what exists at HEAD, the slice, the bars
and their plants, how each number is measured, a prediction each, and the
line estimate, committed before any code or data of the spike. None of it
moves after its data; what the data gives is written below it, in later
commits (gf-1). One host, one run of each: not a rate (AGENTS.md 7). A run
that could not finish is could-not-judge, never a pass (AGENTS.md 6). A
holder that did not answer is never an empty folder, and a refusal is never
an empty answer.

## 1. What exists at HEAD (`2bf0e1b`), and what does not

### Exists

- **The record and its acts.** A group's journal holds signed acts, the
  closed set `RailAct` (`crates/svrngs-core/src/lib.rs:346-434`): `Record`
  carrying an app's payload (`:348`), `Correct`, `Seal`, `Admit`, `Remove`
  and `Invite` (`:429-433`), the infra's own acts each a variant carrying
  no payload. Each kind ships at the line version that first reads it
  (`:651-665`, `line_version` `:680-694`), so a new kind is a new version.
  Sync carries every act to every member's computer each round, every 60 s
  (`crates/svrngs-node/src/ring_sync.rs:104`), and at once after a write,
  the door's nudge (`crates/svrngs-node/src/node.rs:196-201`).
- **`share` and lends.** `svrngs share <port>` lends a localhost service and
  `svrngs unshare <name>` stops it (`crates/svrngs/src/cli.rs:111`, `:128`,
  parsed `:302-318`). A lend is a declaration file the door reads on every
  request (`crates/svrngs-node/src/lend.rs:2-20`; `Lend` `:220-239`;
  `declare` `:406`, `withdraw` `:452`), so a stop takes at the next request.
  Lends never enter the record: what a computer lends is asked of it
  (`lend.rs:18-20`; DESIGN §8, "Lends are asked, not stored").
- **The node's iroh 1.0.2 endpoint.** One endpoint per node, its key the
  person's signing key, advertising every `Alpn` (`node.rs:205-209`;
  `crates/svrngs-core/src/alpn.rs:14-37`: `Member`, `Redeem`, `Lend`).
  iroh is pinned `=1.0.2` (`crates/svrngs-node/Cargo.toml:21`; `Cargo.lock:1715-1716`).
  The accept loop admits by ALPN and dialler (`node.rs:274-282`,
  `admit_dialer` `:729-752`) and serves an axum router per protocol, and
  nothing else: `Admit` is `Serve(axum::Router)` or `Refuse`
  (`crates/svrngs-node/src/iroh.rs:313-319`).
- **The asleep-computer bound.** Another computer is asked for at most
  `ANSWER_WITHIN`, 3 s (`node.rs:98-103`), where an asleep one's dial gives
  up at about 10 s; one that has not answered by then is named as not
  answering, never dropped (`node.rs:479-494`, `offered_by` `:636-649`).
- **A Jellyfin of the spike's own.** `/Applications/Jellyfin.app` (its
  `Info.plist` says 12.0; the run records what `/System/Info/Public`
  answers) started with its own data, configuration, cache and log
  directories, its own web directory and ffmpeg, and its own port, as
  `apps/jellyfin/check/check.sh:123-124` starts one. No container: the
  first run's probe figures carried Docker's network buffering (its
  `README.md`, `validate/`), which this run does not.
- **iroh-blobs 0.103.0**, in the cargo registry and not in the lock. It
  takes `iroh` `1.0.0` with no default features (its `Cargo.toml:167-169`),
  so it should resolve to the lock's 1.0.2; its default features add
  `rpc` (`:56-73`), which the slice does not need. Cited from
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iroh-blobs-0.103.0/`:
  - *its store*: a file store (`fs-store`) adding a file by path
    (`src/api/blobs.rs:265`) with an import mode that references the file
    in place rather than copying it (`src/api/proto.rs:644`), and reading a
    blob's bytes back by range (`reader` `:140`, `export_ranges` `:315`);
    BLAKE3 over a tree of 16 KiB groups (`src/store/mod.rs:18`);
  - *its tags*: a name keeps a blob from collection (`src/api/tags.rs:67`
    `set`, `:123` `delete`);
  - *its verified range fetch*: a request is a hash and chunk ranges
    (`src/protocol.rs:498-505`), fetched into the local store taking what
    it already holds into account (`src/api/remote.rs:496`, `fetch`; `:638`,
    `execute_get`); bytes that do not match the hash end the fetch with
    `LeafHashMismatch` or `ParentHashMismatch` (`src/get.rs:629-650`), and
    the store keeps only verified groups;
  - *its provider*: served on its own ALPN, `/iroh-bytes/4`
    (`src/protocol.rs:406`), by a protocol handler over an iroh
    `Connection` (`src/net_protocol.rs:72`, `:86-89`); each request can be
    intercepted and refused, with per-request transfer events
    (`src/provider/events.rs:49-74`, `RequestMode::InterceptLog`; `:168-176`)
    whose stats split payload bytes sent from hash pairs and header
    (`src/provider.rs:44-58`).

### Does not exist

- **A folder in the record.** No act names a folder or a file; nothing in
  `RailAct` names bytes by hash (`lib.rs:346-434`).
- **iroh-blobs in svrngs.** Not a dependency; the endpoint does not
  advertise its ALPN (`alpn.rs:27`), and the accept loop can hand a
  connection only to an axum router (`iroh.rs:313-319`), not to a
  protocol handler over a `Connection`.
- **A meeting point's local endpoint.** The node's loopback API answers the
  record, what an app asks of its computer, and each lend's origin
  (`node.rs:238-248`); no route answers a file's bytes by hash and range.
- **`share --folder` and its stop.** `share` takes a port and `unshare` a
  lend's name (`cli.rs:302-318`).

## 2. The slice

Three people, each a node of their own on this host with its own data
directory, in one group:

- **Ana** shares a folder (`svrngs share --folder <dir>`) holding an MP4 and
  an MKV made as the first run made them, since that run kept no media
  (`rings/runs/strm-2026-10-05/media/` is absent at `457f120`): its
  `run.sh:24-30`, 120 s of 720p H.264/AAC at 3 Mbit/s, faststart, about
  47 MB, and the same streams remuxed to MKV, with this host's ffmpeg
  (8.1.2); their sizes and hashes are recorded at the run. Beside the
  folder, outside it, is **the planted file**: 1 MiB of random bytes with a
  marker string every 4 KiB, and a symlink to it inside the folder, so a
  folder walk that follows links reaches it. Her share writes one act
  naming the folder's files, each by name, size and BLAKE3 hash; her node
  serves those hashes by iroh-blobs and refuses every other.
- **Cy** shares a folder holding a copy of the MP4, the same bytes, so the
  record names two holders of its hash.
- **Ben's computer is the meeting point.** His node reads the folders from
  the record and serves each named file at a local endpoint on loopback,
  `http://127.0.0.1:<port>/<hash>`, answering HTTP ranges from its own
  iroh-blobs store and fetching what it lacks, by verified range, from a
  holder the record names, as the response is read, never ahead of it by
  more than one piece. A Jellyfin of the spike's own runs on Ben's
  computer over `.strm` stubs, one per file, each holding that file's URL
  at Ben's endpoint; the stub writer lives in `apps/`, and the run's
  Jellyfin driving in `demo/runs/gf/` (fence 5). Browsers are Playwright's
  Chromium and WebKit on Ben's computer, driven as the first run's
  `driver.cjs` drove them: play, 4 s, then seeks to 25%, 60% and 10% of the
  film, 3 s apart (`driver.cjs:180-191`).

svrngs moves a folder's files by hash and never learns a film
(`scripts/vocab_check.py`); `scripts/check.sh` stays free of Jellyfin. The
slice shares at one level, to the group; the closed set of levels §9 names
is not built.

## 3. The bars and their plants

Judged in Chromium; WebKit's MP4 play is reported beside each with the same
numbers, as data. Each plant is watched failing with its guard taken out,
then refused (gf-1).

- **B1. Verified, or refused by name.** Every byte the meeting point serves
  was verified against the hash the record names; and a holder serving one
  flipped byte is refused that range by name, the browser erring rather
  than playing it. *Plant:* Ana's node serves the MP4 with one byte
  flipped in its first MiB (its file changed after its store took it by
  reference, or a provider of the run's that flips it), Cy's node stopped
  so no other holder answers; Ben's endpoint refuses that range naming
  Ana, the hash and the range, sends no byte of the group that failed, and
  the browser reaches an error and never `playing`.
- **B2. The holder's bytes go to the viewer.** Of the bytes the holder
  uploads during the scan and one Chromium play of the MP4, at least 95%
  are bytes the browser received (the first run's bar 2).
- **B3. The scan fetches nothing** from any holder (the first run's bar 1).
- **B4. Seeks.** Three seeks in Chromium each resume within 2 s (the first
  run's bar 3).
- **B5. The probe.** The play's probe fetches at most 10 MB from the holder
  (the first run's bar 4).
- **B6. A holder asleep.** With Ana's node stopped, Ben's play of the MP4
  starts from Cy within 5 s of the request, and Ana is named unanswered,
  never an empty folder. *Plant:* Ana's node killed (`kill -9`), so no
  goodbye reaches Ben, as a computer that sleeps.
- **B7. Only what is shared leaves a holder.** The file beside the folder,
  and the MKV after `unshare`, are refused by hash at Ana's node, the
  stop's delay said, and a scan of every byte the meeting point received
  finds none of the planted file. *Plants:* the planted file and its
  symlink (§2); the MKV's hash asked of Ana's node after she stops sharing
  it.

**One measurement with no bar:** WebKit's MKV through Jellyfin's HLS — the
bytes Ana uploads, and the time to first frame, against the file's size.
What it says is the operator's call after the data.

## 4. How each number is measured

- **A holder's upload**, at its own node, by hash and range: every request
  its provider answers, logged from iroh-blobs' per-request events
  (`RequestMode::InterceptLog`) as the hash, the chunk ranges asked, the
  last offset sent, and payload and other bytes sent (`TransferStats`),
  with the time; a refused request logged as refused, with its hash.
- **A browser's bytes**, at the meeting point's endpoint: every HTTP
  request it answers, logged with its User-Agent, the hash, the range
  asked, the bytes written before the response ended or was cancelled, a
  BLAKE3 of those bytes, and the time. The browser's requests are those
  with the Playwright browser's User-Agent; Jellyfin's are its `Lavf`/
  ffmpeg ones, as the first run told them (`analyze.py:14`).
- **B1**: each range the endpoint served, its BLAKE3 against the same range
  of the source file; any range the endpoint served that Ben's store did
  not hold verified is a miss. The plant's refusal: the endpoint's log
  line and status, and the browser's `video.error` and events.
- **B2**: the holder's uploaded payload ranges for the MP4's hash, each cut
  at the last offset sent, from the scan's start to the play's end; the
  share is the bytes of those ranges the browser received at the endpoint
  (by offset, each byte once) over everything the holder sent for that
  hash, payload and hash pairs both.
- **B3**: requests at any holder's node between the scan's start and end,
  as the first run's scan window (`analyze.py:38-39`); the endpoint's
  requests in that window reported beside.
- **B4**: as the first run's `seekAndTime` (`driver.cjs:90-108`): from
  setting `currentTime` to the video advancing past the target, not paused,
  not seeking.
- **B5**: what the holder sent for the MP4's hash while Jellyfin's requests
  were the only ones open at the endpoint for it, before the browser's
  first.
- **B6**: from the play click to `playing`, the endpoint's log naming the
  holder each range came from; what Ben's folders answer names Ana's files
  and Ana as not answering.
- **B7**: the refusal of each planted hash at Ana's node, from her
  provider's log and the fetch's error; the stop's delay from `unshare`
  returning to the first refusal of the MKV's hash at her node, and to
  Ben's list no longer naming it. The scan: every file under Ben's data
  directory (the blob store, its partial blobs, the journal) and every
  byte his endpoint served, searched for the planted file's marker and
  its hash; and Ana's upload log, for any request of its hash answered.

## 5. Predictions

- **B1: pass.** iroh-blobs verifies each 16 KiB group before its store
  keeps it, so the flipped byte fails one group and the fetch ends with
  `LeafHashMismatch`; Chromium errors within 10 s of the click.
- **B2: pass, 97% or more.** Ben's store keeps what it fetched, so the
  browser's reads of the probe's bytes are not fetched again; hash pairs
  cost 64 bytes per 16 KiB, 0.4%; the rest of the miss is the endpoint's
  read-ahead past where the browser stopped, one piece.
- **B3: pass, 0 requests.** The first run's scan made none with stubs at a
  URL (`run/summary.txt`), and a stub's URL here is Ben's endpoint, not a
  holder.
- **B4: pass**, each seek under 500 ms: every hop is on this host.
- **B5: pass, under 3 MB.** ffprobe read 0.2-1.1 MB in the first run; what
  it does not read is not fetched, but for one piece and socket buffers.
- **B6: pass, about 4 s**, if the fetch asks a holder for at most
  `ANSWER_WITHIN` and Ben remembers Ana unanswered across Jellyfin's probe
  and the browser's request. The risk named: a live connection to a killed
  node waits on QUIC's idle timeout unless the fetch bounds it, and asking
  Ana anew for the probe and again for the play is two waits, about 7 s.
- **B7: pass.** The planted file and its link are never in Ana's store;
  the MKV's hash is refused at her node under 1 s after `unshare` returns,
  as a lend's stop takes at the next request (`lend.rs:2-7`), and Ben's
  list drops it after his next round, under 60 s. What Ben's store already
  holds of the MKV is not taken back; it is said.
- **The MKV in WebKit**: Ana uploads about the whole file, unpaced, as the
  first run's 54.7-57.5 MB through Jellyfin (`run/summary.txt`), and the
  first frame waits on the first HLS segment, under 3 s.

## 6. The lines, estimated

About 430 lines counted by `scripts/lean_check.py`'s rule, `apps/` apart,
against 671 left of 15,200 and 365 of svrngs-core's 3,000 at `b482e08`
(`python3 scripts/lean_check.py`: 14529 and 2635 at `2bf0e1b`):

| Part | Lines |
|---|---|
| the folder's act, in svrngs-core | about 70 |
| serving a folder's files by iroh-blobs, and refusing the rest | about 120 |
| the verified fetch and the local endpoint | about 150 |
| wiring: the ALPN, the accept loop, the node | about 40 |
| `share --folder` and its stop | about 50 |

A row that would pass either budget stops for the operator (fence 6).

## 7. The run, bar by bar (gf-1)

`run.sh` at `575a911`, 2026-10-09 23:15-23:22 UTC, every phase once
(`run/steps.log`); its numbers are `run/summary.txt`, from
`apps/jellyfin/folders/analyze.py`. Setup's load average was 30.24
(`steps.log:1`), above this host's 12 cores, and Jellyfin's
`/System/Info/Public` answered 12.0.0 (`steps.log:8`). One run of each:
not a rate.

| Bar | Its number | Prediction | Bar | Prediction |
|---|---|---|---|---|
| B1 | 119 ranges, 252,484,839 bytes served, none not the source's; the plant refused 6 times naming Ana, the flipped byte served 0 times, Chromium's `error` (MEDIA_ELEMENT_ERROR 4) 6.6 s after the click and never `playing` | errors within 10 s | passed | held |
| B2 | 0.9957: 41,254,912 of the 41,432,704 bytes Ana sent for the MP4 reached Chromium | 97% or more | passed | held |
| B3 | 0 holder requests, 0 endpoint answers, in a 21.5 s scan | 0 | passed | held |
| B4 | 113, 123 and 92 ms | each under 500 ms | passed | held |
| B5 | 2,106,000 bytes from Ana under Jellyfin's probe | under 3 MB | passed | held |
| B6 | `playing` at 3,977 ms, fetched from Cy, Ana named unanswered ("did not answer within 3s") | about 4 s | passed | held |
| B7 | the act names `exp.mkv` and `exp.mp4`, not `link.bin`; Ana served the planted hash 0 times; the scan of Ben's data dir, 37 files and 155,582,491 bytes, finds 0 hits; after `unshare`, Ana refused the MKV from 0.056 s and Ben's endpoint answered 502 from 0.016 s; Ben's list dropped her folder at 68.3 s | refused under 1 s; list under 60 s | passed | the refusal held; **the list missed, 68.3 s against 60 s** |

B1's 6.6 s is the error event against the click, by the driver's two
clocks (`run/results.jsonl`, phase D: the media request at
`performance.now()` 3,627 ms went out at 1791588006.174, the click at
1791588002.548); Jellyfin then fell back to its HLS, whose ffmpeg Ben's
endpoint refused as well, and Chromium waited out the driver's 45 s.
B7's stop has no bar on time; it is said. In C-plant, Ben's list dropped
the folder 8.2 s after `unshare` returned, in C 68.3 s; the run logs only
`folder` and `rail:folder` (`apps/jellyfin/folders/run.sh:33`), so which
sync round carried the stop to Ben is not in the data. What Ben's store
already held of the MKV stays there.

**WebKit's MP4**, as data: 0.9581 of what Ana sent reached it, 6,113,189
bytes, Ben's store holding the rest from Chromium's play; `playing` at
912 ms, seeks 46, 47 and 99 ms. **WebKit's MKV through Jellyfin's HLS**,
the measurement with no bar: Ana sent 47,247,899 bytes, 47,050,291 of
them payload, of a 47,050,291-byte file; `playing` at 1,518 ms; Jellyfin
read 53,574,683 bytes in 6 answers from Ben's endpoint. Predicted: about
the whole file, first frame under 3 s.

### Each plant, its guard out, then in

| Plant | Guard out | Guard in |
|---|---|---|
| a flipped byte served (D) | the first run's range server over Ana's file: 2 requests served the flipped byte, Lavf's and Chromium's, and Chromium played at 677 ms | Ben's endpoint: refused naming Ana, "stream reset by peer" (her store stops at the group before the flip) with Cy's "did not answer within 3s" |
| an unshared hash fetched (C) | `SVRNGS_GF_PLANT=unchecked`: Ana sent the MKV 75 times between `unshare` returning and Ben's list dropping it, 8.2 s; all 74 probes after it 206 (`run/C-plant-probe.jsonl`, `ana.log`) | refused from 0.056 s; all 198 probes after it 502 (`run/C-probe.jsonl`) |
| a file beside the folder (P) | `follow-links`: the act names `link.bin` (`run/P-plant-share.json`), Ben fetched all 1,048,576 bytes (`P-plant-fetch.txt`), the scan finds it in his store | not in the act, 0 served, 0 hits (B7) |
| a stopped holder's film (B) | `unbounded`: `playing` at 31,042 ms, Ana's dial timed out | 3,977 ms from Cy, Ana unanswered |

svrngs's own tests, each red first in its commit's body: the act,
`crates/svrngs-core/src/folder_tests.rs` (`bbf00e1`); the verified fetch
and the endpoint's refusals, `crates/svrngs-node/tests/folders.rs`
(`aee58ae`); `share --folder`, `crates/svrngs/tests/share.rs` (`1490f61`).

### The lines, against §6

By `scripts/lean_check.py`'s rule, `2bf0e1b` to `575a911`; the node's
`folder.rs` split at its functions (serving: `:1-232`, `provider`
`:319-366`; the fetch and endpoint: `hold`, `fetch` `:233-318`, `bytes`,
`range`, `Served` `:367-515`; its record API routes `:516-`):

| Part | Estimated | Counted |
|---|---|---|
| the folder's act, in svrngs-core (`folder.rs` 103, `lib.rs` 12, `admit.rs` 11, `alpn.rs` 2) | about 70 | 128 |
| serving a folder's files by iroh-blobs, and refusing the rest | about 120 | 220 |
| the verified fetch and the local endpoint | about 150 | 221 |
| wiring: `iroh.rs` 7, `node.rs` 10, `lib.rs` 1 (the ALPN is in the act's 2) | about 40 | 18 |
| `share --folder` and its stop: `cli.rs` 39, the node's routes 25 | about 50 | 64 |
| the run's script, `demo/runs/gf/2026-10-09/run.sh` | — | 4 |
| **total** | **about 430** | **655** |

`python3 scripts/lean_check.py`: svrngs-core 2763 of 3000, the
repository 15184 of 15200. `apps/jellyfin/folders/`, apart from the
budget: 978 by the same rule (`analyze.py` 322, `driver.cjs` 201, `run.sh`
192, `rangeserver.py` 99, `jf_setup.py` 92, `probe.py` 38, `strm.py` 34).

## 8. What the run gives (gf-2)

**A group's films, shared as a folder, play in one Jellyfin with their
bytes fetched by verified range from whoever holds them, and nothing else
left a holder.** Written after gf-1's data (§7; every number below is in
`run/summary.json`, by its key, or the file named), by §3's bars, each read
for what it guards. All seven passed in Chromium, and every plant passed
with its guard out before it was refused (§7, "Each plant"). The first
run's one miss is answered: of what Ana sent for the MP4, 0.9957 reached
Chromium, where the range server gave 93.0%; and every payload byte she
sent did, 41,254,912 of 41,254,912 (`A_chromium_mp4`,
`holder_payload_bytes` and `browser_bytes_of_what_holders_sent`). The
0.43% left, 1 − 0.9957, is the verification's own hash pairs and header,
which §5 put at 0.4%. The meeting point's store is why: Jellyfin's probe and the
browser asked for the same bytes, and the holder sent them once.

| Bar | Letter | Spirit — what it guards | Data |
|---|---|---|---|
| B1. Verified, or refused by name | passed | held for a holder whose file changed under her store; a holder whose program sends wrong bytes was not planted (1, below) | 119 ranges served, 252,484,839 bytes, none not the source's (`B1_served_ranges`); 6 refusals, each naming Ana, the flipped byte served 0 times (`B1`); Chromium's `error` at 6,646 ms, never `playing` (`run/results.jsonl`, phase D) |
| B2. The holder's bytes go to the viewer | passed | held: what is not the viewer's is the verification's | 0.9957, 41,254,912 of 41,432,704 bytes (`A_chromium_mp4`) |
| B3. The scan fetches nothing | passed | held | 0 holder requests, 0 endpoint answers, in 21.5 s (`B3`) |
| B4. Seeks | passed | held, on one host (4) | 113, 123 and 92 ms (`A_chromium_mp4.seeks_ms`) |
| B5. The probe | passed | held, the endpoint fetching one piece past what Jellyfin read | 2,106,000 bytes from Ana; Jellyfin read 1,048,576 in 1 answer (`A_chromium_mp4`) |
| B6. A holder asleep | passed | held | `playing` at 3,977 ms, fetched from Cy, Ana "did not answer within 3s", both folders listed (`B6`); unbounded, 31,042 ms (`B6_plant`) |
| B7. Only what is shared leaves a holder | passed | held where the bytes are; the record learns later, and the meeting point keeps what it fetched (3) | the act names `exp.mkv` and `exp.mp4`, not `link.bin`; 0 planted bytes served; Ben's data dir, 37 files and 155,582,491 bytes, 0 hits (`B7`, `B7_scan`); after `unshare`, Ana refused from 0.056 s, Ben's endpoint 502 from 0.016 s, his list 68.288 s (`B7_stop`) |

**Where the letter and the spirit part**, each judged by the spirit:

1. *B1 passed, and Ben's check was not the one that caught it.* The plant
   flipped a byte of Ana's MP4 at 500,000 after her store took it by
   reference (`run/steps.log`, 23:19:34), one of the two plants §3 allowed.
   Her store, reading the file against its own tree, stopped each send at
   the 16 KiB group before the flip: `done=false last=491520`, six times
   (`run/ana.log:287-302`). Ben's fetch ended "stream reset by peer"
   (`B1.why`), not `LeafHashMismatch`, as §5 predicted: no log of the run
   names a mismatch (`grep -ci mismatch run/*.log`, 0 each). That guards an
   honest holder whose disk changed. The bar guards a holder serving wrong
   bytes, which a changed program does past its store's check; Ben's
   verification of bytes that arrive wrong is iroh-blobs' `get`
   (`src/get.rs:629-650`), exercised neither here nor by svrngs's test,
   which plants the same way (`crates/svrngs-node/tests/folders.rs:235-248`).
   Held for the case planted; candidate 6 plants the other.
2. *B1's refusal is the browser's, not the person's.* Chromium erred and
   never played; Jellyfin then asked for its HLS, which Ben's endpoint
   refused as well, and the page waited out the driver's 45 s (`B1.events`,
   `B1.first`). Why — Ana's bytes did not verify, Cy did not answer — is in
   the endpoint's 502 and Ben's log, not before whoever pressed play, who
   sees a film that does not start. Said; it waits (§9).
3. *B7 passed where the bytes are; the record learns later, and the
   meeting point keeps.* Ana's node refused the MKV 0.056 s after `unshare`
   returned, and Ben's endpoint answered 502 from 0.016 s, since it asks her
   and she refuses. Ben's list named her folder for 68.288 s (`B7_stop`), and
   8.159 s with the guard out (`B7_stop_plant`); which round carried the stop
   is not logged (§7). §5 said under 60 s: missed, with no bar on it. What
   Ben's store fetched stays: 155,582,491 bytes in his data dir after the
   run (`B7_scan`), both films whole among them. What is not shared never
   left Ana; what has left, a stop does not take back. Candidate 5.
4. *Every hop was on one host, the browser on the meeting point.* The
   stub's URL, `http://127.0.0.1:18733/<hash>/exp.mp4`, is what Jellyfin
   handed the browser to play itself (`A_chromium_mp4.src`, `play_method`
   `DirectPlay`). A member watching Ben's Jellyfin through its lend from
   their own computer would be handed the same URL, which names their own
   loopback, not Ben's: read from the data, not run. Setup's load average
   was 30.24 on this host's 12 cores (`run/steps.log:1`). Candidate 1.
5. *The run's group was one.* `share --folder` signs its act into every
   group held here (`crates/svrngs-node/src/folder.rs:198-216`), and Ana's
   provider serves a hash to a dialer standing in any group held here
   (`crates/svrngs-node/src/node.rs:742`, `:758`), asking only whether a
   standing folder of hers names it, never whether it is shared where the
   dialer stands (`folder.rs:218-224`). The run held one group
   (`run/steps.log`: `svrngs new Films`, two joins), where the two are one.
   Candidate 2.

**The MKV, the measurement with no bar.** WebKit's remux through Jellyfin's
HLS took the whole file from Ana once, unpaced: 47,247,899 bytes sent,
47,050,291 of them payload, of a 47,050,291-byte file, while Jellyfin read
53,574,683 bytes in 6 answers from Ben's endpoint; `playing` at 1,518 ms
(`A_webkit_mkv`). As predicted. The first run's holder sent 54.7-57.5 MB
for the same remux (§5); here Ben's store took Jellyfin's rereads. What it
says: a film Jellyfin remuxes costs its holder the file once, at the first
play, and no more. Whether that is acceptable is the operator's call.

So a group's films, shared as a folder, play in one Jellyfin from whoever
holds them, every byte the meeting point served verified, a holder asleep
named and passed over within 4 s, and nothing beside the folder, nor after
its stop, leaving the holder. What the run did not reach is everything off
this host: a viewer on another computer, a second group, a real link. The
spike cost 655 lines where gf-0 estimated about 430 (§7), and leaves 16 of
15,200. One host, one run of each: not a rate (AGENTS.md 7). DESIGN §3, §8
and §9 are rewritten for it on this branch.

## 9. What makes folders production (gf-2)

Each part gf-1 counted (§7, by `scripts/lean_check.py`'s rule at `575a911`,
unchanged at `dfd00c3`), and what production keeps, changes or drops of it:

| Part | Lines | Production |
|---|---|---|
| the folder's act, in svrngs-core | 128 | keeps; the act gains its level (2). `Admission::holders`, 9 (`crates/svrngs-core/src/folder.rs:103-111`), is read only by its test while the endpoint decides holders itself across groups (`crates/svrngs-node/src/folder.rs:404-408`): one decider (2) |
| serving by iroh-blobs, and refusing the rest | 220 | keeps; the provider's gate asks where the dialer stands (2); the port is kept for the stubs (1); the plants' switch, `SVRNGS_GF_PLANT`, 21 (`folder.rs:45-56`, `:76`, `:97-102`, `:117`, `:157-160`, `:255-259`), drops — each plant stays a test's red run, as svrngs's are |
| the verified fetch and the local endpoint | 221 | keeps; who the endpoint answers changes (3); the BLAKE3 of each answer, the run's instrument, 3 (`Served.digest`, `:434`, `:460`, `:507`), drops |
| wiring | 18 | keeps |
| `share --folder` and its stop | 64 | keeps; names the level (2) |
| the run's script, in `demo/` | 4 | the run's |
| `apps/jellyfin/folders/strm.py`, apart | 34 | changes, in `apps/` (4) |
| the rest of `apps/jellyfin/folders/`, apart | 944 | the run's |

The lock took 30 crates for iroh-blobs, redb among them
(`git diff 2bf0e1b..dfd00c3 -- Cargo.lock | grep -c '^+name = '`); the
build's size was not measured.

Candidate rows, the operator's to queue (the queue's preamble). Each check
is a bar rerun with its plant, red first:

1. **A stub that reaches its viewer.** Every member's node answers the
   endpoint at one loopback port, kept across restarts — today
   `SVRNGS_FOLDER_PORT`, else any port (`folder.rs:103-106`), the run fixing
   18733 (`apps/jellyfin/folders/run.sh:32`) — so a stub's URL names, on
   each computer, that person's own node, which fetches by verified range
   from whoever holds the file: the bytes go from holder to viewer, and the
   meeting point serves Jellyfin's probe alone. The port taken by another
   program is refused by name. Evidence: §8's fourth part. Check: Ben's
   Jellyfin opened through its lend from Cy's computer, a second host, the
   MP4 playing, B2 and B4 rerun there with the holder's upload counted at
   her node.
2. **A folder shared at one level, and served only there.** The act names
   its level from the closed set §9 names (an enum; the row names the set,
   "the group" at least); `share --folder` writes it to the group named,
   not every group held here; the provider serves a hash only to a dialer
   standing where a standing folder of this computer's names it; and the
   holders of a hash are decided once. Evidence: §8's fifth part. Check:
   Ana in two groups, a member of the other refused her hash by name at her
   provider, answered with the dialer's check out.
3. **Who the endpoint answers.** It is served without the record API's
   guards (`folder.rs:123-126`; `crates/svrngs-node/src/api.rs:64-84`) and
   must answer a page, since the browser plays the stub's URL itself; so
   any page in this computer's browser, or any program on it, that names a
   hash reads the file, and a hash is no secret — the record names it, and
   the same bytes have it anywhere. In its place a stub's URL carries a mark
   the node made for it, and the endpoint refuses a request without one,
   naming why; its form is the row's, and with candidate 1 every member's
   node honours it. Check: a request with the hash alone refused, a stub's
   URL answered, and the hash alone answered with the guard out.
4. **Stubs in step with the record**, in `apps/` (fence 5). `strm.py`
   writes once, names each film by its extension as the run's driver needed
   (`apps/jellyfin/folders/strm.py:34`), and removes nothing. In its place a
   stub per file, by its name, written when `/v1/folders` names it and
   removed when it stops, then Jellyfin's library refreshed. Check: B3
   rerun after a share and after an `unshare`, the stub gone and the scan
   fetching nothing.
5. **What the meeting point keeps.** Ben's store kept every byte it fetched,
   a stopped file's among them (§8's third part). In its place a hash no
   standing folder names is dropped from the meeting point's store at the
   next record change, and the store held under a bound the row names.
   Check: after `unshare`, B7's scan of Ben's data dir finds none of the
   MKV's bytes, and with the drop out finds them.
6. **A holder that sends wrong bytes, planted.** A test of svrngs's whose
   holder sends a flipped byte on the wire — a provider of the test's, past
   any store — and Ben's endpoint refuses that range naming the holder and
   the mismatch. Tests only, no line of the budget. Evidence: §8's first
   part. Check: the test red against the first run's range server in Ben's
   place, as D's plant was (§7), then refused.

Not candidates, said so:

- **The person told why a film does not start** (§8's second part):
  Jellyfin's to show; svrngs names its refusal (`x-svrngs-refused:
  no-holder`, `folder.rs:424`). Waits for a person in the group's run who
  presses play on a film no holder gives and is told nothing.
- **The others' lists after a stop**, 8.2 and 68.3 s: the holder's refusal
  is what keeps the bytes (B7). Waits for a stopped film listed and failing
  to play, counted in the group's run.
- **The MKV's remux** (§8): the operator's call.
- **A phone watching** has no node to answer a stub's URL (candidate 1):
  after the computer (DESIGN §10, 2026-10-09, a computer first), with the
  phone replica spike (DESIGN §9, "a native phone app").
- **A folder of many files**: one act names every file again at each
  change (`ShareFolder`, `crates/svrngs-core/src/folder.rs:39-56`); the run
  had two. Waits for a folder whose act's size is measured too large to
  write at each change.
- **Real links and a rate**: the operator's run with a group in the wild
  (DESIGN §9's row), after these rows.

**The budget.** Production keeps about 600 of the spike's 634 (655 until
the audit after gf-2 named a file's hash by the core's `ContentHash`, 21
fewer, `ralph/REVIEW_FINDINGS.md` #108): the plants' 21, the digest's 3
and the run's script's 4 out, and `holders`' 9 folded into the one
decider. It adds, by the same rule, about 25 for the level (the enum, the
act, the flag, the write), 15 for the provider's gate, 10 for the kept
port, 40 for the endpoint's mark and 30 for the store's drop: about 720,
svrngs-core's part about 120 (2,741 of 3,000 on this branch, `python3
scripts/lean_check.py`). gf-0's estimate ran 52% short (430 to 655, §7);
the same miss on the 120 added makes about 780. That is the raise
folders need over main's count when they merge — the operator's, with the
merge (DESIGN §5; the queue's fence 6).

**Since: streaming dropped (syp-1).** The operator, 2026-10-10, chose
"Drop streaming + share" on the mesh-Dropbox spike's verdict
(`demo/runs/sy/2026-10-09/README.md` §8-§9): a group's folder is synced
onto its members' disks, not streamed. syp-1 (`ralph/next/syp/STATE.md`)
deleted the loopback endpoint (`bytes`, `range`, `Served`,
`SVRNGS_FOLDER_PORT`), the one-way share (`ShareFolder`, its act and line
version 5, `share --folder` without `--both-ways`, `unshare --folder`,
`GET /v1/folders`), `Admission::holders`, and the tests of them named
above. The provider, the verified fetch (`hold`) and the plants' switch
stay, the synced folder's now. This run and `apps/jellyfin/folders/` ran
at `575a911` and do not run after syp-1. Of the six rows: 1, 3 and 4 drop
with the endpoint and wait with a mounted folder (DESIGN §9), a folder not
on its viewer's disk; 2 carries over as the synced folder's row 2, the
level a group; 5 changes into its row 5, the store keeping what the
layout places; 6 carries over unchanged, against the synced folder's
fetch.
