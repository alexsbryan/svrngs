# The founder's memory under the wall's reads: predictions, before any data

Written 2026-10-17, before `leak.py` first ran. The data is in
`data-<condition>/`; `README.md` here says what each line came to.

## What is already known

- **The finding.** In `runs/2026-10-17-linux-burst/`, the wall read the
  founder back to back: 54,844 reads, each `/v1/rail/people` then
  `/v1/lends`. Priya's RSS between the sampler's windows, once everyone
  had joined (the last join started at +37.4 s):

  | window | reads | RSS grew | per read |
  |---|---:|---:|---:|
  | +34 to +44 s | 6,680 | 158 MiB | 24.2 KiB |
  | +44 to +54 s | 6,277 | 126 MiB | 20.6 KiB |
  | +54 to +64 s | 3,927 | 78 MiB | 20.2 KiB |

  The wall stopped at +60.7 s. RSS read 442.3 MiB at +64 s, so nothing
  came back in those 3 s. With one read a second
  (`runs/2026-10-17-linux-burst-wall1s/`), RSS was 41, then 64, then
  65.5 MiB.
- **The log is written synchronously.** V1.1's subscriber is
  `tracing_subscriber::fmt()` with `.with_writer(std::io::stderr)`
  (`crates/svrngs/src/main.rs:14-20` at `a555a225`). There's no
  `tracing_appender::non_blocking`, so there's no in-process queue that
  could back up: a slow disk slows the writer and holds no lines in
  memory. The default filter is `error` (`main.rs:17`; the CLI's help,
  `cli.rs:219`).
- **The installed service sets no `RUST_LOG`.** install.sh's launchd plist
  sets only `SVRNGS_DIR` (`install.sh:134`), and so does its systemd unit
  (`install.sh:148`). People run at `error`.
- **The Linux build is musl, static**, through zig (`scripts/build-release.sh:28`
  at `a555a225`), and V1.1 sets no `#[global_allocator]`. So it runs on
  musl's malloc, and `MALLOC_ARENA_MAX`, a glibc setting, does nothing to
  it. The Mac build uses macOS's own malloc, which this run can't measure
  (the canon loop is running on the Mac).
- **Connections are kept, not dialled per read.** There are 53
  `AddConnection` lines in the burst run's founder log for 54,846 reads
  (`logs/Priya.log`). A connection is kept for each computer
  (`crates/svrngs-node/src/iroh.rs:161-184`, `kept`). Each ask is a new
  bi-stream of it, carrying its own hyper HTTP/1 handshake, and the
  driver is spawned as a task (`iroh.rs:360-365`, `on_stream`). Each
  `/v1/lends` read spawns one task for each computer
  (`node.rs:620-633`, `lends_in`).
- **The other 24 computers answered the same 54,846 asks each** and stayed
  at 36-48 MiB (`summary.json`, `samples_linux`). Whatever grows, grows on
  the asking side.

## The hypotheses, each with what would show it

Each condition is a fresh group of 25 computers on Fedora, V1.1's binary
(sha256 `8d3c5020…`). Phases: A idle 120 s; B `/v1/lends` back to back,
60 s; C idle 60 s; D `/v1/rail/people` back to back, 60 s; E idle 60 s.
The founder's VmRSS, VmHWM, `smaps_rollup` Anonymous, threads, fds and
log size are sampled each second.

1. **Log backlog.** Predicted false, since the writer is synchronous
   (above). It holds only if B's KiB per read at the kit's debug level is
   at least 3 times that at `error`, *and* RSS falls back in C.
2. **A per-read leak on the asking side.** Predicted true.
   - B grows at 15-25 KiB a read at the kit's debug level, as the burst
     run did.
   - It grows at 10-25 KiB a read at `error`. The growth isn't in
     logging; only the time spent formatting goes away, so B does more
     reads a second.
   - Linear: in a 5-minute B, the last minute's KiB per read is at least
     70% of the first's.
   - C and E keep it: RSS falls by at most 5% in an idle phase.
3. **Allocator high-water.** Predicted false. It holds if a long B
   plateaus (last minute's KiB per read under 30% of the first's), or if
   C gives back more than 30% of what B took.
4. **Growth with members, whatever the reads.** Predicted false. A, idle
   with all 25 in, grows by under 2 MiB in its 120 s.
5. **The local read leaks too.** Predicted false. D, `/v1/rail/people`,
   which dials no one, grows by under 1 KiB a read.
6. **Threads and fds stay flat through B.** Predicted true. Connections
   are kept (above), so a leak would be in memory, as tasks, streams or
   buffers, not in sockets: within ±5 threads and ±5 fds.
7. **It scales with the computers asked.** With 5 computers (4 asks a
   read), B's KiB per read is a quarter to a third of 25 computers' (24
   asks a read), if each ask leaks about the same. *Guess*, run only if 2
   holds at `error`.

## What a verdict means here

The verdict is one of leak, log backlog, allocator, member growth or
could-not-judge. It is could-not-judge if a phase didn't run or the
sampler missed it, never a pass.

## After the verdict: the Mac, and the fix

Added 2026-10-17, after `README.md`'s verdict and before the runs these
lines judge. One exception: the Mac's fast-read condition (`mac-i-error`)
had started, and its first 30 s of B had been glanced at (flat), before
this section was written. Its line below is a decision rule, not a
prediction.

8. **The Mac, `mac-i-error`** (V1.1's darwin-aarch64, `error`, B back to
   back for 60 s). macOS's malloc is the system's own, not musl's. If B
   keeps more than 1 KiB a read in ps's RSS or in footprint's, the fix
   covers macOS too; otherwise the fix is gated to musl.
9. **The Mac, paced, `mac-ii-paced1s`** (B at one read a second for
   600 s). Predicted: under 5 KiB a read in both RSS and footprint,
   against Linux's 89.8 (viii).
10. **The fix on Fedora, `ix-fix`** (the fix commit, built as
    `build-release.sh` builds Linux x86_64, `error`, B back to back
    60 s). Predicted: under 1 KiB a read (vii: −0.5); CPU in B within
    150–260%, about the release's 191–205%; RSS at the end of A under
    200 MiB (vii: 135).
11. **The fix on Fedora, paced, `x-fix-paced1s`** (B at one read a
    second for 600 s). Predicted: under 5 KiB a read, against the
    release's 89.8 (viii).

The fix is green when 10 and 11 hold; red is the release's ii, iii and
viii, already measured.

## After 11: is the fix's rise a ceiling?

Added 2026-10-17, after `x-fix-paced1s` failed 11 as written (45.5 KiB a
read over B; the founder 92 → 134 → 119 MiB, and Aiko, who only answers,
78.7 → 126.7 MiB by C) and before the run below.

12. **The fix, paced for 30 minutes, `xi-fix-paced30m`** (B at one read a
    second for 1,800 s). Predicted, if mimalloc's heaps fill to a ceiling
    rather than keeping reads: the founder's RSS at B's end within 15% of
    its value 600 s into B; its high-water under 160 MiB; Aiko's RSS
    under 160 MiB throughout. The release at the same pace climbed 53 MiB
    in 600 s (viii), so it would be past 220 MiB by 1,800 s.
