# Lessons from BitTorrent, and the smallest package that acts on them

Design brief, 8 November 2026. The package was run the same day
(`demo/runs/swarm/2026-11-08/README.md`, §7 here says what held): part D is
built and part C is the PLAN rule; part A failed step 1 (BBR3 no better than
Cubic on this path) and did not land, and a smaller receive window on blob
fetches landed in its place, which holds for one fetcher and not for four, and
turns across fetchers landed on 9 November for the four
(`demo/runs/swarm/2026-11-09/README.md`); part
B missed step 3's bar, is over the 35 counted lines the operator gave the
package, and lies as a patch (`demo/runs/swarm/2026-11-08/parts/`). `DESIGN.md`
stays the current design.
This brief neither changes PLAN's order nor approves a size-budget raise.

File and line citations are to `main` at `847e991b`. Dependency citations are
to the versions `Cargo.lock` pins: iroh 1.0.2, iroh-blobs 0.103.0, noq-proto
1.0.1.

## 1. The short version

BitTorrent's durable lessons for svrngs are about being a good guest on a home
connection and about not piling every request on one computer. Its incentive
machinery is the part to refuse: it exists because strangers defect, and it
ranks people.

Priorities used here:

- **P0**, harm or a hard limit today: the household's uplink (lesson 1) and the
  load on a file's writer (lesson 2).
- **P1**, cheap to get right now and costly later: what a hash means (lesson 3),
  slow holders (lesson 4), a file nobody can currently supply (lesson 5).
- **P2**, a test and no code: whether the record can be read without svrngs
  (lesson 6).

The package in §5 covers P0 and P1 in about 95 counted lines, with **no change
to the record**. The key to its size is that iroh-blobs can already fetch any
range of a single blob and report which ranges a peer holds, so BitTorrent's
pieces can be slices of the hash svrngs already stores.

## 2. What svrngs already has

These need no work. They are listed so nobody rebuilds them.

| BitTorrent | svrngs | Where |
|---|---|---|
| Downloading is uploading | People join for themselves and the commons is what that leaves behind | `NORTH_STAR.md` §1 |
| A downloader becomes a source | Each member that asked for a blob carries it for the rest | `DESIGN.md:546` |
| Verified pieces (BEP 3), per-file Merkle trees (BEP 52) | BLAKE3 verified ranges over iroh-blobs | `DESIGN.md:1303` |
| Resume from another peer | A holder that stops mid-file leaves what is lacking to the next | `crates/svrngs-node/src/folder.rs:489` |
| Magnet link (BEP 9): a name plus a hint | The join token: the issuer's key, a secret, a relay only when needed | `PLAN.md:839` |
| Tracker moved into the swarm (BEP 5, BEP 11) | The record says who stands and which relay to use | `DESIGN.md:1307` |

## 3. The lessons

### Lesson 1 (P0): uploads must yield to the household

Clients filling a home uplink were a large part of why people turned BitTorrent
off. A full upstream buffer adds seconds of latency to everything else in the
house. µTP (BEP 29), built on LEDBAT (RFC 6817), was the answer: treat rising
queue delay as the signal to back off.

svrngs today: "Carrying spends the carrier's upload, uncapped"
(`DESIGN.md:550`). Nothing in `crates/*/src` sets a congestion controller, so
every connection runs noq's default, Cubic
(`noq-proto-1.0.1/src/config/transport.rs:581`). Cubic grows until packets drop,
which on a home router usually means after the buffer has filled. A person
whose call stutters because a friend fetched a folder has a reason to leave,
and NORTH_STAR §1 depends on that person having no such reason.

### Lesson 2 (P0): spread the load off the writer

A swarm's capacity grows with its demand only if requests spread. In svrngs,
every member asks the file's writer first (`crates/svrngs-node/src/synced.rs:676`)
and works through holders one at a time (`folder.rs:509`). `hold` only reorders
holders that recently failed to answer (`folder.rs:507`).

The arithmetic, for 1 GB, 8 computers, and 40 Mbit/s uplinks with downlinks
faster than that:

- From the writer alone: 7 copies × 200 s = 1,400 s, about 23 minutes.
- The standard lower bound for peer-to-peer distribution (Kumar and Ross, 2006)
  is `max(F/U_writer, N·F/ΣU)` = `max(200 s, 7·8,000/(8·40) s)` = 200 s, about
  3.3 minutes.

This is arithmetic, and it assumes every computer is on. The measured starting
point is narrower: 100 MiB reached each of two members at 83 MB/s from one
holder, and a member fetching from another member that holds the file is
unmeasured (`PLAN.md:456`).

### Lesson 3 (P1): don't overload what a hash means

BitTorrent v2 (BEP 52) changed what an infohash names, and needed hybrid
torrents to carry both worlds. An append-only record has the same problem with
no way out: `Change.now` is `(ContentHash, u64)` with no format
(`crates/svrngs-core/src/synced.rs:104`), and every entry already written stays
in the record forever.

The cheapest way to honor this lesson is not to change the meaning at all. If a
different form is ever needed, it arrives as a new closed variant (AGENTS 9),
and `hash` keeps meaning the BLAKE3 of the file's bytes.

### Lesson 4 (P1): drop slow peers

Each byte received resets the 3 s timer (`folder.rs:597`, `ANSWER_WITHIN` at
`node.rs:111`). That is deliberate, since a large file's holder is not asleep,
but it means a holder trickling from hotel WiFi keeps the fetch for as long as
it trickles, while a member on fiber who has the same file is never asked. The
writer-first order makes this the likely case and not the rare one.

### Lesson 5 (P1): a dead torrent with a live tracker

A torrent's listing can outlive its last seeder, and the record has the same
shape. It spreads quickly and is small, the bytes are large and depend on one
computer, so a writer who closes their laptop right after saving leaves an entry
nobody can fetch.

Today this is a warning in the log (`synced.rs:679`), and the step retries
(`synced.rs:274`, between `RETRY_FIRST` and `RETRY_UP_TO`). The person waiting
is told nothing, as far as a narrow search of the home page's sources shows.
BitTorrent clients show availability as a figure. svrngs can say it in a
sentence.

Seedboxes also showed the fix that needs no protocol: a computer that never
sleeps, which is just another participant. A group can have this today by
keeping one member's computer on.

### Lesson 6 (P2): a short spec outlives its client

BEP 3 is short enough that many independent clients exist, and that is much of
why the protocol has lasted. "Each ring keeps its record on its members'
computers and can leave together, intact" (`NORTH_STAR.md` §3) depends on the
same property. PLAN already parks publishing the wire until someone sets out to
write a second implementation (`PLAN.md:449`). The cheap test does not have to
wait: give a blind agent `DESIGN.md` §6 alone and ask it for a record reader.

## 4. What to refuse

- **Tit-for-tat and share ratios** (Cohen, 2003). They exist because strangers
  defect. Private-tracker ratios rank people, and fail NORTH_STAR test 3.
- **A DHT or public swarms.** Membership already answers who to ask, and a
  public swarm would show members' addresses to strangers.
- **Rarest-first.** It protects large swarms that lose seeders. In a small group
  it can wait until that failure is counted.
- **Fixed blocks to dedupe edits, on BitTorrent's authority.** BitTorrent never
  solved this: an insertion shifts every later fixed block. Blocks help in-place
  edits only. That row stays parked on its own trigger (`PLAN.md:456`).

## 5. The package

| Part | Concern | Change | Counted lines (estimate) |
|---|---|---|---|
| A | Lesson 1 | BBR3 in place of Cubic | ~4 |
| B | Lessons 2 and 4 | Large files fetched in slices, each from a holder that has it | ~60 |
| C | Lesson 3 | A rule, no code | 0 |
| D | Lesson 5 | The home page says which files are waiting, and on whom | ~30 |

### Part A: BBR3

Set `.transport_config(...)` with a `Bbr3Config` controller factory where the
endpoint is built (`crates/svrngs-node/src/guest.rs:142`), and the same in
`dial.rs:59`, because a dial that passes its own config does not use the
endpoint's (`iroh.rs:362-363`). The uploader's side is what matters: a blob
provider sends on a connection it accepted.

iroh exposes the setting (`iroh-1.0.2/src/endpoint/quic.rs:419`) but re-exports
only the controller traits (`quic.rs:68-71`), so this takes a direct
`noq-proto = "=1.0.1"` dependency, the version the lock already holds.
`Bbr3Config` is at `noq-proto-1.0.1/src/congestion.rs:12`.

BBR3 paces to a model of the path instead of filling the buffer, so a small
flow in the same house keeps its latency. It does not yield: it still competes
for a fair share. The bar in §6 tests the symptom, which is latency.

Alternatives considered:

| Alternative | Why not first |
|---|---|
| A LEDBAT controller (~90 lines; the `Controller` trait is public) | A controller is chosen when a connection is accepted, before its ALPN is known (`iroh-1.0.2/src/endpoint/connection.rs:164`, `:558`). So it would apply to lends as well, and a movie would fall behind any other upload in the house. It is the fallback if BBR3 fails the bar. |
| iroh-blobs' `ThrottleMode::Intercept` | One message per chunk of usually 16 KiB (`provider/events.rs:616`) through a channel of 32 (`folder.rs:635`), about 8,900 a second at the 139 MB/s measured. Its events carry a connection id, not the connection, so reading RTT needs more plumbing. |
| Extending the declared upload (`DESIGN.md:1318`) | A person who declared nothing would be unprotected, and that is the person this is for. A number nobody chose would break "no number stands in for one nobody chose" (`DESIGN.md:1327`). |

BBR3 changes every connection, lends included, which is why §6 reruns them. If
it lands, the BBR3 pacing estimate parked at `PLAN.md:453` becomes cheaper.

Measured 2026-11-08, step 1: the ping's p95 was 261 ms under both Cubic and
BBR3 and the bar 140, so part A did not land. `perf_quic` shows the sender's
cwnd growing past 800 MB under noq-proto 1.0.1's BBR3 and no loss, so the
queue near 250 ms is held by iroh's stream receive window, not by the
controller, which is why the two controllers read the same. A 512 KiB window
on the fetcher's dial then held one fetcher to 110 ms and let four reach 430
ms, about fetchers × window (`demo/runs/swarm/2026-11-08/README.md`). That
window landed (`iroh.rs`, `BLOBS_RECEIVE_WINDOW`, 6 counted lines). It applies
to blob fetches alone, so lends keep theirs. Per stream it caps a path at 210
Mbit/s at 20 ms and 52 Mbit/s at 80 ms. Several fetchers at once, or several
files at once, added up. Turns across fetchers on the holder's side then
bounded that (`folder.rs`, `Sends`; `demo/runs/swarm/2026-11-09/README.md`):
four at once read 116 ms, the fetch no slower, and many medium files over an
open 40 ms path took 41% longer. The LEDBAT fallback above stays for someone
who wants svrngs always to yield (PLAN §2, the transfer row).

### Part B: slices

Files of at most one slice keep today's path, untouched, so the results for many
small files still stand. Larger files, in `hold` (`folder.rs:499`), given the
file's size:

```text
SLICE = 4 MiB
start = BLAKE3(own key ‖ file hash) mod number of slices

loop, walking the slices from start and wrapping:
    skip a slice held whole here
    ask every candidate what it holds, in parallel, within ANSWER_WITHIN
    keep candidates whose bitfield covers this slice, and that are not asleep
    none: move to the next slice; a whole pass with none ends the fetch
    choose: a holder not yet measured in this fetch, else the fastest measured
    get blob_ranges(hash, this slice's missing ranges) from it
    record that holder's rate; a silent holder is marked asleep as today
```

What makes it work:

- **The start point is the piece-selection strategy.** Members begin at
  different places in the file, so the writer sends each part roughly once
  and members trade the rest. It is a pure function of two hashes, with no random
  number source and nothing to configure.
- **Bitfields are BitTorrent's HAVE message.** `Remote::observe`
  (`iroh-blobs-0.103.0/src/api/remote.rs:542`) streams a peer's bitfield. Part B
  takes the first item and drops the stream, which is one round trip per holder
  per slice. A 1 GB file is about 238 slices.
- **The hash does not change.** `GetRequest::blob_ranges`
  (`iroh-blobs-0.103.0/src/protocol.rs:554`) asks for ranges of the BLAKE3 the
  record already names. A partly held file is verified the same way as a whole
  one.
- **One get per file at a time.** `FETCH_AT_ONCE` (`synced.rs:53`) stays the
  limit across files, so this adds no concurrent writes to a single store entry.
- **Lesson 4 falls out.** A slow holder is measured and moved last, so it costs at
  most one slice per fetch: 4 MiB at 50 KB/s is about 80 s, once, where today it
  can be the whole file.

Authority: the provider refuses `Observe` today by dropping its answer
(`folder.rs:639`). Part B answers it under the same check as a get,
`folders.serves(hash, dialer, now)`, so a dialer who may not fetch a hash may
not learn what is held of it either. That check reads placement from the record,
so a member holding part of a file already passes it. Get-many stays refused.

Why not `HashSeq` and iroh-blobs' `SplitStrategy::Split`, the first idea: `Split`
parallelizes across a sequence's children and not within one blob
(`iroh-blobs-0.103.0/src/api/downloader.rs:440`), so it would need every file
stored as a sequence, which is the change to the record lesson 3 warns about.

Every decision traces at debug (AGENTS 1): the slice chosen, each candidate kept
or passed over and why, the holder chosen, and its measured rate.

As built and run (2026-11-08), `hold_sized` chooses a holder by the slice's
place among those that cover it, passing one that fails or takes
`SLICE_WITHIN` (15 s) for the rest of the fetch, rather than measuring each:
step 4's cost of a slow holder is 15 s, not 80 s shared six ways. It did not
reach step 3's bar. Its cost was 93 counted lines after unifying `hold` and the
slice loop; the code wins over the sketch above.

### Part C: a rule

No code. If blocks ever land, an entry names its form with a new closed variant
and `hash` keeps its meaning. The rule sits on the blocks row at `PLAN.md:456`.

### Part D: say what is waiting

`fetch_placed` returns `None` for two different failures: no member gave the
file (`synced.rs:679`), and it could not be written to disk (`synced.rs:692`).
Part D makes that a closed set (AGENTS 9), so the count below includes only the
first. A step then records, per folder, how many files wait on each writer
(`Placed.entry.key`), and clears it when none do. The group's status carries it
to the home page, beside the folder lines `dirs_of` already feeds
(`folder.rs:697`).

The words follow VOICE.md. A draft:

> 3 files are still only on Ana's computer. They'll arrive when it's back on.

It appears only while files are waiting, and says only what the group would
need to know to fix it, so it ranks no one. A status that cannot be read is
reported as such and never as nothing waiting (AGENTS 6).

## 6. Proving it, cheapest first

Each step can end the work below it. Runs go on the Fedora box, with network
namespaces and `tc` shaping uplinks to 40 Mbit/s behind a buffer deep enough to
hold about 500 ms, and are recorded under `demo/runs/swarm/<date>/`. One run
each. One run is not a rate (AGENTS 7), so these decide whether to build and do
not stand as people bars.

| Step | Observation | Bar | Its plant | If it fails |
|---|---|---|---|---|
| 0 | A member fetches from another that holds half a file, stopped mid-fetch | Exactly the held ranges arrive, verified | Flip a byte in the half-holder's data file: the fetch rejects it | Upstream's tests for this are commented out (`iroh-blobs-0.103.0/src/tests.rs:101`). Part B falls back to serving only from whole holders: slower, same format |
| 1 | 1 GB fetch across one shaped uplink, ping from the sender's namespace | Ping's p95 within 100 ms of idle, RFC 6817's target; the fetch at most 25% slower than Cubic (a proposed figure) | Cubic must miss the ping bar. **If Cubic passes, drop part A** | BBR3 fails: the LEDBAT fallback, and its effect on lends becomes a decision |
| 2 | A's reruns: a lend's stream and perf-2's 100 MiB fetch | No regression against the recorded runs | Not new checks | BBR3 applies to blobs only if a way to choose per connection is found; else drop part A |
| 3 | 8 computers, 256 MiB, all uplinks shaped | Under a third of today's time. Arithmetic says today is about 6.3 min and the bound 54 s | Today's `hold` must miss the bar. **If it doesn't, drop part B** | Keep today's fetch |
| 4 | As step 3, one non-writer holder throttled to 400 Kbit/s | Still under the bar; the trace shows that holder serving at most one slice per fetcher | Today's order, with that holder first in it, must miss the bar | Rank by more than this fetch's rates |
| 5 | Test: the writer stops after its entry spreads | The line appears, names the writer, and goes when the file arrives | A disk failure must not produce the line | Part D is wrong |

Part B also takes a test that a dialer standing in no group holding the hash is
refused an `Observe`, its plant being the provider with that check removed.

## 7. What is known and what is not

| Claim | Status |
|---|---|
| Fetch order, timer reset, default Cubic, `Observe` refused, no record format field | Read in the code cited |
| iroh-blobs can observe and fetch ranges of one blob | Run: `perf_partial`, step 0 passed |
| A partly held file serves its ranges correctly | Run once, step 0 (`perf_partial`): 10,190,848 of 20,000,000 bytes, exactly the held range, verified; a flipped byte refused by the provider's own verification, a clean range beside it served |
| 23 minutes vs 3.3 minutes | Arithmetic, assumes everyone is online |
| Today's `hold` is that slow between homes | Run once: 8 computers, 256 MiB, 40 Mbit/s uplinks, 399.8 s (arithmetic said 376) |
| Cubic bloats this path; BBR3 does not | Run once: Cubic 261 ms, BBR3 261 ms, idle 40.1, bar 140.1. BBR3 fails |
| The queue is the stream receive window's, not the controller's | Run once each with 512 KiB on the fetcher's Blobs dial (`parts/blobs-receive-window.patch`, 6 counted lines, not landed): one fetcher ping p95 110 ms, passing; four fetchers 430 ms, failing, about fetchers × window. A per-dial window bounds one fetcher, not several |
| The package's slices reach a third of today's time | Run once: 201 s against 399.8 s, a bar of 133 missed (step 3) |
| BitTorrent history | From the BEPs, RFC 6817, Cohen (2003) and Kumar and Ross (2006) |
| Line counts | Estimates, before any code; as built: part A 7, part B 93 (with its small files' path folded in), part D 27 counted lines |

## 8. Cost

At `847e991b` the size budget stood at 20,722 of 20,723 lines for the repository,
3,510 of 3,511 for `svrngs-core`, and 8,387 of 8,396 for JavaScript
(`scripts/lean_check.py`).
Part D's roughly 8 lines of JavaScript just fit; the core is untouched. The
repository-wide cap has one line of room against about 95, so the package needs
an economizing pass, and any remainder is a raise for the operator to approve
(AGENTS, "Done"). As landed: +33 counted lines against the operator's +35, part
D 27 and the window 6, `.` to 20,754. The turns across fetchers then cost 77,
after an economizing pass cut 44: `.` to 20,787.

## 9. Limits

- BBR3 keeps queues short and does not give way. A person who wants svrngs to
  always lose to their other traffic needs the LEDBAT fallback.
- If the slow computer is the writer and nobody else has the file, nothing here
  helps until another member holds some of it.
- A holder that deliberately sends one packet every few seconds still costs a
  slice's time. Members are not adversaries here.
- `blob.rs` carrying stays whole-file. Phones still serve nothing: a browser is
  relay-only and cannot listen (`DESIGN.md:1349`).
- An edit still moves the whole file (`PLAN.md:456`).
- PLAN parks part B until a member waiting on a large file is counted between
  computers (`PLAN.md:456`). Step 3 shows the gain and is not that count.
