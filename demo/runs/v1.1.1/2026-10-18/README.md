# V1.1.1, released 2026-10-18: what was checked

V1.1.1 is V1.1 (`a555a225`) with the Linux build's allocator fix and
nothing else of main's (the operator, 2026-10-18: "Let's do the patch
release"). It's built from `6db0cccf`, tagged `v1.1.1`, a commit on no
branch:

- `67e34d45`: main's `23c182ae`, the economizing cut before the fix
  (door.rs's two dialer guards made one, 3 lines);
- `4bafddcc`, `2218f88a`, `f5832c46`: the runs the fix cites,
  `demo/runs/twentyfive/2026-10-17/` and `demo/runs/leak/2026-10-17/`
  (main's `4ddc48c7`, `b6cb0522`, `0f48da03`);
- `93e81c7b`: main's `61a06614`, mimalloc as the `#[global_allocator]`
  under `target_env = "musl"`. DESIGN §7's build sentence and
  `build-release.sh`'s header say so in V1.1's words. The cap is 16,558 →
  16,556 at a count of 16,555;
- `9a2bc4ff`, `6db0cccf`: the fix's results (main's `f8af6107`,
  `0cea24ff`).

Every person here is a program, and one run is not a rate.

## The check, at 6db0cccf

`scripts/check.sh` exited 0 in 1,078 s, all 15 steps passed, in a
worktree with `npm ci --prefix web` (`check.out`). No rerun was needed.

## The live site

Published to Vercel svrngs-page (scope svrnmesh) by deploy-page.sh's own
steps: build-web.sh, build-release.sh, `web/vercel.json`. `SHIPPED` is
`N0` (`crates/svrngs/src/cli.rs:251`), so there was no relay check.
Production deployment `dpl_Bd2VCBn2boDTQXqueNiocdWnimtJ`
(svrngs-page-553y49199-svrnmesh.vercel.app), aliased to group.svrnme.sh.

Read back:

- `get/SHA256SUMS` is the build's, byte for byte (`live/SHA256SUMS`), and
  all four live tarballs match it.
- `install.sh` and `index.html` are as built.
- The page's policy is the trust floor's: connect-src
  `'self' wss: https:` (`scripts/trust_floor.txt:48-54`), and its one
  127.0.0.1 is in a comment. The host sends
  `content-security-policy: frame-ancestors 'none'`.
- `.env.local`, `.vercel/project.json` and `vercel.json` answer 404.
  Afterwards, the CLI's `.env.local`, `.vercel/` and `.gitignore` were
  deleted from the upload directory.

What people download:

- **Fedora,** fetched as install.sh does: the linux-x86_64 tarball checks
  against the live SHA256SUMS, and its `svrngs` is `e0282fe8…`, the
  binary the leak run below measured (`live/binaries.sha256`).
- **This Mac:** the darwin-aarch64 tarball checks, its `svrngs` is
  `ab894915…`, and `--help` is V1.1's byte for byte (`live/help.txt`
  against the kit's V1.1 binary). Only the Linux binary carries
  mimalloc's symbols.

## Leak: 25 computers on Fedora, the founder asked back to back

`demo/runs/leak/2026-10-17/leak.py run --cond v111-fast --bin <the
release's linux-x86_64 svrngs>` at `error` (`leak-fast/`). It ran before
the deploy, on the same bytes.

| phase | reads | KiB kept a read | founder RSS | founder CPU |
|---|---:|---:|---|---:|
| A, idle 120 s | 0 | | 97.8 → 94.4 MiB | 1.3% |
| B, `/v1/lends` back to back, 60 s | 45,003 | **0.43** | 95.1 → 114.6 MiB | 206% |
| C, idle 60 s | 0 | | 114.4 → 121.3 MiB | 0.9% |
| D, `/v1/rail/people`, 60 s | 228,492 | −0.03 | 123.9 → 116.8 MiB | 34% |
| E, idle 60 s | 0 | | 116.7 → 118.0 MiB | 1.0% |

V1.1's release kept 21.3 KiB a read in the same phase, 69 → 699 MiB
(`demo/runs/leak/2026-10-17/`, condition ii). It's green: under 1 KiB a
read.

## Smoke: the 25-computer burst, the wall at one read a second

`demo/runs/twentyfive/2026-10-17/run.py --mode burst --layout linux
--wall-every 1`, from a scratch copy of the kit whose `bin/` held the
live binaries. There were three runs:
- `smoke/first/`, V1.1.1;
- `smoke/pair/`, V1.1.1 then V1.1, each with `ticks.py` on Fedora reading
  every computer's CPU ticks and RSS from `/proc` once a second
  (`pair.sh`, as it ran).

| line | V1.1.1, first | V1.1.1, pair | V1.1, pair | V1.1, 2026-10-17 |
|---|---|---|---|---|
| 1. every join in | 24/24, median 3.34 s: failed | 3.42 s: failed | 3.33 s: failed | 3.32 s: failed |
| 2. lit on the wall, median | 3.77 s | 3.84 s | 3.79 s | 3.79 s |
| 3. lend on the wall, median | 0.39 s | 0.52 s | 0.50 s | 0.48 s |
| 4. wall settles after last join | 3.35 s | 4.30 s | 3.41 s | 3.33 s |
| 5. wall reads, p50 | 4 ms | 2 ms | 2 ms | 2 ms |
| 6. matrix whole after last share | 27.97 s | 29.48 s | 27.74 s | 27.07 s |
| 7. greetings right | 75/75 | 75/75 | 75/75 | 75/75 |
| 8. cpu and memory | **failed** | **failed** | passed | passed |

Line 1 fails as V1.1's did. Its bound is a median under 3 s, and a joiner
waits about 3.1 s for n0's relay (`demo/runs/twentyfive/2026-10-17/README.md`).

**Line 8 fails on V1.1.1, twice, on two readings:**
- **Memory.** The founder's RSS peaked at 98.3 and 101.9 MiB, against a
  bound of 80 MiB, set before mimalloc was in the build. V1.1 peaked at
  66.8. That's the fix's known cost: mimalloc holds more at rest.
- **Another computer's CPU in the window after settling:** 19.9% and
  10.0%, against 5%; V1.1 read 0.0. That reading is `ps`'s `time`, which
  counts whole seconds, so each 10% is one second boundary crossed in a
  10 s window. The window also holds the matrix, which runs after the wall
  stops at +20 s.

`ticks.py`'s readings (`smoke/pair/*/ticks.jsonl`, 100 Hz) say where the
CPU went:

| per computer that isn't the founder, median (max) | V1.1.1 | V1.1 |
|---|---|---|
| CPU in its first second | 0.17 s (0.40) | 0.02 s (0.03) |
| from then until the wall settles: join, share | 0.21 s (0.62) | 0.15 s (0.22) |
| the 20 s of the wall's steady reads | 0.09 s (0.16), 0.4% | 0.12 s (0.14), 0.6% |
| after the wall: the matrix, each asking all 24 at once, and 3 visits | **0.72 s (1.34)** | 0.23 s (0.26) |
| the whole run | 1.23 s (1.81) | 0.51 s (0.61) |
| peak RSS | 59.2 MiB (67.6) | 45.2 MiB (47.4) |

At the wall's pace, the founder used 1.5% under V1.1.1 against 2.4%, and
the rest used the same. In the leak runs' phase B, V1.1.1's founder spent
less CPU a read than V1.1's: 206% at 763 reads a second, against 190% at
501. But a computer that asks all 24 others for the first time, as the
matrix does, spends about three times V1.1's CPU doing it: half a second
more. So does its first second. It's a one-time cost of first contact,
not of steady reads. Why mimalloc costs that much there isn't measured:
first-touch page faults in its per-thread heaps, on 32 worker threads, is
a guess. A laptop runs fewer worker threads than Fedora's 32 cores give.

## Left as found

Fedora's `~/v111`, `~/twentyfive` and `ticks*` are removed, and nothing
of the runs is left running. `~/.svrngs` and `~/svrngs` there were never
touched. The release commit is kept by its tag. The worktree is removed.
