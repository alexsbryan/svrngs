# The founder's memory under the wall's reads, 2026-10-17

Fedora (x86_64, 32 cores), then this Mac (Apple M2 Max, 12 cores), a
fresh group of 25 computers each condition, brought up by the 25-computer
kit's `fleet.py` (`demo/runs/twentyfive/2026-10-17/`). `leak.py` drives the
founder's loopback API in phases and reads its memory each second: `/proc`
on Linux; on macOS, ps's RSS and footprint's dirty memory, which is what
macOS's malloc gives back when it frees. The predictions
in `PREDICTIONS.md` were written before any data. One run of each
condition: a spread, not a rate (AGENTS.md 7).

## Verdict: the allocator in the release's Linux build, not svrngs's code

Every `/v1/lends` read keeps 11–90 KiB in V1.1's Linux release, and that
memory never comes back. The same commit doesn't keep it when built
against glibc or rustc's own musl. It does keep it when built with zig
0.17.0 through cargo-zigbuild 0.23.4, which is how `build-release.sh` builds
Linux (`scripts/build-release.sh:9-11,28` at `a555a225`). Tasks, threads and
fds stay flat throughout, so nothing in svrngs is left running.

| condition | binary | B's reads | KiB a read | RSS through B | CPU in B |
|---|---|---:|---:|---|---:|
| i | V1.1 release, kit's debug log | 51,306 | 20.1 | 64 → 1,070 MiB | 233% |
| ii | V1.1 release, `error` (as installed) | 29,556 | 21.3 | 67 → 682 MiB | 191% |
| iii | V1.1 release, `error`, B 5 min | 290,188 | 11.6 (13.1 first third, 9.6 last) | 67 → 3,341 MiB | 205% |
| viii | V1.1 release, `error`, one read a second for 10 min | 599 | 89.8 (114 first third, 69 last) | 67 → 120 MiB | 1.5% |
| iv | a555a225 + probe, glibc (x86_64-unknown-linux-gnu) | 49,083 | 0.14 | 62 → 69 MiB | 223% |
| v | a555a225 + probe, rustc's musl | 33,261 | 0.02 | 37 → 38 MiB | 509% |
| vi | a555a225 + probe, zig 0.17.0 musl, as the release | 33,486 | 19.4 | 68 → 704 MiB | 203% |
| vii | vi + `#[global_allocator]` mimalloc 0.1.52 | 33,856 | −0.5 | 135 → 117 MiB | 207% |

And on the Mac, with V1.1's darwin-aarch64 binary (`a2259c89…`), built by
cargo on macOS's own malloc:

| condition | B | B's reads | KiB a read: RSS, footprint | RSS through B | CPU in B |
|---|---|---:|---|---|---:|
| mac-i | back to back, 60 s | 10,263 | 0.01, 0.00 | 44.7 → 44.8 MiB | 181% |
| mac-ii | one read a second, 600 s | 595 | 1.48 (4.36 first third, −0.65 last), −1.72 | 42.6 → 43.4 MiB | 7.6% |

And the fix, `61a06614`, on Fedora: its Linux x86_64 binary built on the
Mac as `build-release.sh` builds it (`cargo zigbuild --release --locked -p
svrngs --target x86_64-unknown-linux-musl`, sha256 `6be2cfca…`), zig's
musl with mimalloc 0.1.52 as the global allocator:

| condition | B | B's reads | KiB a read | RSS through B | CPU in B |
|---|---|---:|---|---|---:|
| ix | back to back, 60 s | 33,423 | 0.39 (0.59 first third, 0.14 last) | 96.6 → 109.4 MiB | 210% |
| x | one read a second, 600 s | 599 | 45.5 (39.1 first third, −71.5 last) | 89.8 → 116.4 MiB, high-water 133.5 | 1.5% |
| xi | one read a second, 1,800 s | 1,798 | −2.07 (−5.58 first third, −1.22 last) | 95.8 → 92.2 MiB, high-water 102.0 | 1.3% |

Idle at the end of A, the release's founder was 66.8–67.4 MiB and Aiko
50.2–51.4 MiB (ii, viii); the fix's founder 88.7–95.2 MiB and Aiko
76.8–83.5 MiB (ix–xi). That is mimalloc's price on a 32-core machine:
about 20–30 MiB more for every Linux computer, asked or asking.

- **Idle phases keep it.** C (60 s after B) moved +12 to +1,388 KiB in
  every condition. Nothing was given back.
- **`/v1/rail/people`, which dials no one, costs nothing.** D did
  209,560–448,060 reads at 0.00–0.01 KiB each in every condition.
- **Tasks, threads and fds are flat.** In iv–vii the runtime's alive tasks
  sit at 118–123 at rest, and 157–168 at the end of a B phase (that
  read's asks in flight). Threads stayed at 66–67 and fds at 19–22 in
  every condition.
- **Members alone don't grow it.** A, idle for 120 s with all 25 in, grew
  1.6–2.8 MiB under zig's musl, 3.9 MiB under glibc and 0.1 MiB under
  rustc's musl.
- **The asked side barely grows.** Aiko, one of the 24 answering every
  ask, went 50.7 → 73.8 MiB over iii's 290,188 asks, about 0.08 KiB an
  ask.

## Against the predictions

1. Log backlog: **refuted, as predicted.** The writer is synchronous
   (`crates/svrngs/src/main.rs:14-20`), and `error` grew as much as
   debug: 21.3 against 20.1 KiB a read.
2. A per-read leak on the asking side: **true for the release binary,
   false for svrngs's code.** Growth is linear and never returned, but it
   comes from the libc the release links. The linearity bound (last
   minute ≥ 70% of the first) came to 73% in iii.
3. Allocator: **true, though not as predicted.** I predicted a plateau,
   and zig's musl malloc doesn't plateau: in iii it kept growing for 5
   minutes, 9.6 KiB a read in the last third.
4. Member growth: **false, as predicted.** A grew under 2 MiB in four of
   seven conditions, and under 4 MiB in all of them.
5. The local read: **false, as predicted.** D cost 0.00–0.01 KiB a read.
6. Threads and fds flat: **true, as predicted.**
7. Scaling with computers asked: **never ran.** Once 2 was refuted for
   the source, it wouldn't have changed the answer.
8. The Mac, fast: **flat, so the fix is gated to musl.** 0.01 KiB a read
   in RSS and 0.00 in footprint, under the 1 KiB rule. The Mac read about
   170 times a second against Fedora's 500: one machine of 12 cores
   carrying all 25.
9. The Mac, paced: **held.** 1.48 KiB a read in RSS and −1.72 in
   footprint, under 5, against Linux's 89.8 (viii). No idle phase raised
   it by more than 0.5 MiB.
10. The fix, fast: **held.** 0.39 KiB a read, under 1; CPU 210%, inside
    150–260%; RSS at the end of A 95.2 MiB, under 200.
11. The fix, paced for 600 s: **failed as written.** 45.5 KiB a read
    over B, against under 5. The founder sat at 89–98 MiB for 300 reads,
    stepped to 129 MiB about 330 s in, and fell back to 116 MiB by B's
    end. Aiko, who only answers, rose too, from 76.8 to 123.8 MiB by C's
    end, and was 119.3 MiB at E's. (12's own text gives these in
    thousands of KiB: 92 → 134 → 119, and 78.7 → 126.7.) What stepped is
    not known: no read of the founder's own changed then.
12. The fix, paced for 1,800 s: **held.** The founder was 92.5 MiB 600 s
    into B and 92.2 MiB at its end, within 15%; its high-water 102.0 MiB
    and Aiko's 93.2, each under 160. Against the release's 53 MiB climb
    in 600 s (viii), this run gave back 3.6 MiB over 1,800. With x, it
    reads as mimalloc's heaps filling to a ceiling once, not reads kept:
    x's step is one excursion in two paced runs, not a rate.

## What it means for the evening

At one wall read a second (viii), a Linux founder on V1.1 keeps about
90 KiB a read, about 5.4 MiB a minute: **about 630 MiB over 2 hours**,
7,200 reads, if viii's rate held. Its last third, 69 KiB a read, gives
about 500 MiB. A Linux computer always on and serving phones pays the
same per page that asks it what everyone lends.

The Mac build is `cargo build` on macOS's own malloc, and it doesn't keep
it (mac-i, mac-ii): a Mac at the projector stays at about 45 MiB.

With the fix, a Linux founder at the wall's pace stayed between 88 and
134 MiB and ended at 92–116 (x, xi), and stayed under 115 MiB through
33,423 reads back to back (ix). It
reaches people only in a release built from `61a06614` or later.

## The fix, and the check that shows it

The site is the release's Linux toolchain, not a line of svrngs. What
churns is each `/v1/lends` read:
- one spawned task per computer asked (`crates/svrngs-node/src/node.rs:617-627`,
  `lends_in`);
- and inside each ask, a fresh bi-stream, a hyper HTTP/1 handshake and a
  spawned driver (`crates/svrngs-node/src/iroh.rs:360-365`, `on_stream`);
- all of it spread over 32 worker threads.

Two ways out, each measured. The first landed in `61a06614`, gated to
`target_env = "musl"`, since macOS kept nothing (mac-i, mac-ii):
- **A global allocator for Linux:** `#[global_allocator]` mimalloc in
  `crates/svrngs/src/main.rs`, gated to Linux musl (`mimalloc.diff`). It
  stayed flat at the release's CPU (vii). The costs:
  - a C dependency, `libmimalloc-sys`;
  - a higher idle RSS: vii's guess was 120–160 MiB; measured on the
    fix, 88.7–95.2 MiB for the founder against 67, and about 80 MiB for
    a member against 51 (ix–xi).
- **Linking Linux with rustc's own musl instead of zig's:** flat (v), but
  at 2.5 times the CPU under the same reads (509% against 203%), since
  musl's own malloc is slow under contention. Cross-building `ring`'s C
  from the Mac still needs a musl C compiler.

The red check is `leak.py`'s B against the built Linux artifact: 25 or
fewer computers, 20,000 reads, failing above 1 KiB a read. On this data it
is red for the release (19–21 KiB) and green for vii and for the fix (ix,
0.39). It belongs beside `build-release.sh`, not `check.sh`, since it
needs a Linux host, the built binary and about seven minutes. It isn't
in the tree yet: as a counted script it would cost about 100 lines of the
budget, which is the operator's to give.

## Data

- `data-<condition>/`:
  - `summary.json`: each phase's numbers, `alive_tasks` where probed;
  - `samples.jsonl`: one line a second, the founder and Aiko.
- `run-mac-*.out`: what each Mac condition printed: nothing, since
  `leak.py` prints only when it fails. `build-*.out`: each scratch build
  on Fedora (`build-gnu.out` was `build.out` there, the build iv ran).
- `binaries.sha256`: each binary measured on Fedora; the release's is
  `8d3c5020…`, the fix's `6be2cfca…`. The Mac's is the kit's
  `bin/RECORD.txt`.
- `probe.diff`, `mimalloc.diff`: the scratch patches i–viii measured,
  never in the repo as such.
- `leak-since-viii.diff`: how `leak.py` here differs from the one i–viii
  ran on Fedora: the kit's path where it landed, and macOS's readings.
  Linux's readings and the phases are unchanged.

## Run it

From this directory, with the kit's binaries fetched into
`demo/runs/twentyfive/2026-10-17/bin/` as its README says:

```
python3 leak.py run --cond <name>                          # B back to back, 60 s
python3 leak.py run --cond <name> --b-secs 600 --b-every 1 # the wall's pace
python3 leak.py run --cond <name> --bin <a svrngs binary>  # any other build
```

Each condition works in `w-<name>/` here, removed at its end, and leaves
`data-<name>/`.

On Fedora, the work dir, the scratch builds, zig and cargo-zigbuild are
removed, as is the `x86_64-unknown-linux-musl` target rustup had added.
`cargo install` and the mimalloc build left their crates in Fedora's
`~/.cargo/registry` cache.
