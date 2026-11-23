# Availability release audit

`REVIEW-audit-available` at `a9db4471`, after the [packaged Fedora VM
proof](README.md).

## Port ledger — passed

Compared `python3 scripts/ports.py` with the tracked markers from
`git grep -InE 'ported from commonwealth-ai [^[:space:]]+@[0-9a-f]{7,40}' HEAD -- ':!docs' ':!ralph' ':!scripts/ports.py'`.
The comparison checks local paths and line numbers and upstream paths and
revisions. Omitted the launchd entry from the listing **in memory** first,
then compared the actual listing:

```text
planted omitted entry: FAIL: missing: [('apps/svrngs-keeper/src/launchd.rs:2', 'svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0')]; extra: []
unmodified ledger: PASS: 67 matching markers (path, line, upstream path and revision)
apps/svrngs-keeper/src/launchd.rs:2  <-  svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0  unchanged; here: changed in 2 commit(s)
apps/svrngs-keeper/src/systemd/tests.rs:1  <-  shared/crates/host-kit/src/service/tests.rs@8ed8d99b0  unchanged; here: changed in 1 commit(s)
apps/svrngs-keeper/src/systemd.rs:2  <-  shared/crates/host-kit/src/service.rs@8ed8d99b0  unchanged; here: changed in 5 commit(s)
```

## CSP and trust floor — passed

The keeper branch was rebased on `789707dc` (CHARTER, “The release”).
`git diff --exit-code 789707dc a9db4471 -- scripts/trust_floor.txt scripts/trust_check.py scripts/check.sh scripts/gate`
and the following CSP diff both exited 0 with no output:

```sh
git diff --exit-code 789707dc a9db4471 -G 'Content-Security-Policy|content-security-policy|[a-z]+-src|frame-ancestors|base-uri|form-action' -- apps crates web
```

There is one floor difference from the original `e5d9813b` base:
`9167f112`, already in `789707dc`, changes the relay listener's rule at
`scripts/trust_floor.txt:133` and gives its approved `Trust floor:` reason.
There are no keeper changes to that file. `python3 scripts/trust_check.py`
compared the served CSPs with the pins and exited 0:

```text
trust floor: 13 policies, 13 listeners, 17 gates
```

## Release envelope — passed

`python3 scripts/lean_check.py` exited 0:

```text
Lines of code, tests and apps/ not counted, and the most each may have:
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   25,719   most  25,719
  JavaScript                 8,727   most   8,731
  Swift                          0   most       0
```

The actual counts and the repo caps fit the CHARTER's final ceilings
(“The envelope now”): 25,727 non-JavaScript and 8,741 JavaScript, leaving
8 and 14 lines respectively from the actual counts at `a9db4471`.

## Rebuilt gate at `b4daf708` — failed

The second full run (`REVIEW-audit-available-2.await.log`) rebuilt
`svrngs-gate`, then ran with the main `.git` mounted and
`TOKIO_WORKER_THREADS=4`. The rebuilt image supplied `ss utility,
iproute2-6.1.0`; the keeper's `--test run` passed 3/3. The full gate passed
`tests`, `types`, `typed lint`, and every other tier except `browser`:
`42 passed; 2 failed; 4 ignored; 2 filtered out`. The full gate exited 1.
The conformance canary still reports the two worker-stop observations as
could-not-judge: WebKit has no stop-worker command.

- WebKit navigation: Node exited 1 in Playwright's
  `WKPage._onTargetCreated`, the same provisional-page assertion reproduced
  before this branch ([2026-11-19 flake record](../../flakes/2026-11-19/README.md)).
- WebKit channel close: `/offer` answered 200 at `13:15:53.429Z`, but no
  `goes direct` appeared. After the driver's 20 s wait, all 6 MiB arrived
  with SHA-256 `db17adeaed9388036c8d3cde3ba1a6ef3f2c8f173d47c35ab5100df076e84241`.
  The 721 ms tail and 71 ms gap described a relayed fetch, not a measured
  fallback. The first full run instead reached the fallback and missed
  the 4 s bound by 6 ms. Both runs are failures for different reasons.

The same image's exact replays in that log used `CARGO_BUILD_JOBS=4` and
`TOKIO_WORKER_THREADS=4`:

| Exact replay | Base `789707dc` | Tip `b4daf708` |
|---|---|---|
| Relay seek (first 1,423 B, ms) | 1/3 passed: 9497, 2495, 6474 | 3/3 passed: 2586, 2854, 4375 |
| WebKit channel close (fallback ms) | 3/3 passed: 3809, 3761, 3715 | 3/3 passed: 3791, 3965, 3772 |

In all six exact channel-close runs, the selected path was WebRTC before
the close and the 6 MiB digest matched. These small samples document runs,
not rates. On the same candidate sources, exact WebKit navigation passed
6/6; the unchanged browser tier alone reported
`44 passed; 0 failed; 4 ignored; 2 filtered out` (89.57 s). Those replays
do not change the second full run's verdict. The assigned row's next
check is three interleaved exact trials of both WebKit cases at base and
tip, followed by one controlled full gate.

After the four fixes, `python3 scripts/lean_check.py` reported 3,471/3,539
core, 25,719/25,719 non-JS and 8,731/8,731 JS, leaving 8 and 10 actual lines
under the release ceilings. The trust check still reported `13 policies,
13 listeners, 17 gates`. The diffs of `scripts/check.sh`, the floor and its
checker, and CSP lines from `789707dc` still exited 0; the gate image diff
adds `iproute2` to the existing package list.

## Controlled gate at `1581f2ba` — failed

`REVIEW-audit-available-3.await.log` records the image digest and both
revisions at line 3. Of the interleaved WebKit trials, navigation passed
3/3 at `789707dc` and 2/3 at `1581f2ba`: the latter's first trial exited
101 from Playwright's `WKPage._onTargetCreated` assertion (lines 566–591).
All six channel-close trials went direct before closing and passed.
The unchanged full gate passed `browser`, but exited 1 for two failures:

- `tests`, lines 1339–1359: the Share verb saved the two selected IDs in
  ID order, not selection order. Asking for both orders makes the check
  deterministic. The conditional publication test also saw
  `Selected(["house-id", "trip-id"])` instead of
  `Selected(["trip-id", "house-id"])`. Retaining the selected order but
  removing the reviewed-ID guard made the mismatched-audience plant fail.
  With `698433df`, the gate container's `verbs` passed 48/48, `tcp` 8/8,
  `udp` 7/7, `availability_plan` 4/4, and `surface` 15/15.
- `pages`, lines 1659–1679: the WebKit acts driver forced Node to exit
  with a write pending: `EOF while parsing a string`, column 8192. In the
  same container, adding a temporary 1 MiB string to the exact test's
  browser report gave EOF at column 16384, exit 101. After letting Node
  exit normally (`2d320f05`), the same plant passed 1/1, exit 0. With the
  plant removed, `pages` passed 3/3.

At `2d320f05` the port ledger still matched all 67 tracked markers.
The `789707dc` diffs for `scripts/check.sh`, the trust floor and its
checker, and the CSP lines each exited 0 with no output. `trust_check.py`
reported `13 policies, 13 listeners, 17 gates`. The final focused count
was core 3,471/3,539, non-JS 25,715/25,719, and JS 8,731/8,731, below the
release ceilings of 25,727 non-JS and 8,741 JS. The full gate is still owed
on the repaired candidate.

## Controlled gate at `2b1a7040` — failed

`REVIEW-audit-available-4.await.log:3` pins the same gate image and the base
`789707dc`. The full gate exited 1 after 1,276 s; only `browser` failed:

```text
passed     tests (374 s)
failed     browser (90 s)
           the bytes after the channel closed must arrive over the relay within 4 s, took 4010 ms:
           test result: FAILED. 43 passed; 1 failed; 4 ignored; 0 measured; 2 filtered out; finished in 83.95s
passed     pages (11 s)
```

This time the worker had selected WebRTC before the close (line 1215), and
the 206 response had all 6,291,456 bytes and the expected SHA-256 above
(lines 1216–1236). This is a timing failure at `webrtc_page.rs:183–189`,
distinct from the earlier run where no direct path was selected.

The exact interleaved trials in `-3.await.log` and `-4.await.log` gave:

| Revision | WebKit navigation | Channel close `fallbackMs`, in trial order |
|---|---|---|
| `789707dc` | 6/6 passed | 3753, 3856, 3802; 3896, 3898, 3855 |
| `1581f2ba` | 2/3 passed | 3803, 3812, 3769 |
| `2b1a7040` | 2/3 passed | 3793, 3761, 3842 |

Every focused channel-close trial selected WebRTC before the close and
passed the byte, digest and timing checks. Navigation hit Playwright's
`WKPage._onTargetCreated` assertion once at each candidate (`-3.await.log:574`,
`-4.await.log:1051`), separately from the latest full-gate failure. The
same navigation abort was reproduced on earlier main (see the flake record
above); these few trials do not establish its frequency or a keeper effect.

Next: one unchanged full gate at the rebase pin `789707dc`, in the same
image, with the same mounts and worker count. It can demonstrate a matching
base failure if one occurs; a single green base run cannot establish a
keeper regression. The result on `2b1a7040` remains failed either way.
