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

## Same-image full gate at `789707dc` — passed

`REVIEW-audit-available-5.await.log:1–3,435–545` records the detached base,
the image `372d88c39c15eecf3b80ccf7deb501f1ee58fd649e57175450a0884830b46832`
(also `-4.await.log:3`), the warm browser build, and the unchanged full gate:

```text
passed     tests (379 s)
passed     browser (97 s)
passed     pages (11 s)
note       took 1400 s in all
Full gate exit 0
```

Neither candidate browser failure reproduced in this main run. The two
WebKit worker-stop observations are still could-not-judge (lines 513 and
528); neither concerns the navigation target abort or the relay fallback.
One green base run cannot establish a keeper regression; attribution of
both candidate failures is unresolved. The gate prints
browser's aggregate time on a pass, so this log cannot say how close the
base came to the fallback bound. The candidate's failed full gate stands.

Rechecked on the same candidate sources (`git diff --exit-code 2b1a7040
0df95c6b`, excluding this audit and the queue state): the `789707dc` diffs
for the gate, floor, floor checker and CSP still exited 0;
`python3 scripts/trust_check.py` reported `13 policies, 13 listeners, 17
gates`, and `python3 scripts/lean_check.py` reported 25,715/25,719 non-JS and
8,731/8,731 JS, both within the charter ceilings.

### Next diagnostic: browser-test overlap

Run the browser tier once per trial, in the fixed order base/candidate/
candidate/base under the same image and `TOKIO_WORKER_THREADS=4` (fresh
container and shared browser pair per trial, no other heavy work under the
gate lock). The diagnostic uses the browser tier's usual `--skip playback::`
selection (`scripts/check.sh:114–123`); it only adds libtest JSON output and
captures timestamps, case status, active test count and visible load. Record
the Home case's span beside each WebKit case, and the relay's first direct
selection, payload and timings. The new isolated Run/Share scenario is
ignored in the normal tier; the existing Chromium Home case adds two Share
confirmation clicks (`browser/main.rs:1984–1988,2193–2199`). If Home ends
before the relay case starts, those clicks cannot overlap its measured
fallback on that trial. Whole-test overlap would show opportunity for
contention, not causation. Read the target abort separately from the timing
miss. This diagnostic cannot qualify the release.

### Browser-test overlap results at `ab551667`

`REVIEW-audit-available-6.await.log:3,646–1084` records the same gate image
and 32 visible CPUs. The candidate's sources and checks match `2b1a7040`
(`git diff --exit-code 2b1a7040 ab551667`, excluding this audit and queue
state). Results in the declared order:

| Revision | Browser pass/fail; Cargo exit | WebKit fallback | Home ended before relay test by | Home overlap with WebKit navigation |
|---|---|---:|---:|---:|
| `789707dc` | 43/1; 101 (gzip) | 3901 ms | 7.085 s | 7.958 s |
| `ab551667` | 44/0; 0 | 3841 ms | 7.003 s | 8.037 s |
| `ab551667` | 44/0; 0 | 3770 ms | 0.286 s | 9.065 s |
| `789707dc` | 44/0; 0 | 3859 ms | 1.152 s | 8.225 s |

All four channel-close cases selected WebRTC first and returned 206 with
all 6,291,456 bytes and the expected digest. All four navigation cases
passed without the `WKPage._onTargetCreated` abort. The one base failure
was `a_phone_opens_a_lend_that_answers_only_in_gzip::webkit` (line 737);
the diagnostic does not identify its cause. The wrapper's exit 0 means
the trials completed; the first browser suite itself failed.

The Home test did not overlap the relay test in these trials. Its overlap
with the navigation test occurred on both revisions without an abort.
Neither earlier failure reproduced here, so these trials cannot attribute
them or justify a test change.

The candidate's focused audit still passes:

```text
67 tracked markers, 67 ledger entries; missing=[]; extra=[]
trust floor: 13 policies, 13 listeners, 17 gates
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   25,715   most  25,719
  JavaScript                 8,731   most   8,731
```

The CSP diff and the diff of `scripts/check.sh`, the floor and its checker
against `789707dc` both exited 0 with no output. The counted totals sit 12
non-JavaScript and 10 JavaScript lines below CHARTER's release ceilings.

## Operator decision: the candidate full gate is still failed

Should the release stay blocked pending a separate investigation of the
WebKit gate failures, or should the operator explicitly authorize another
qualification path? `ralph/next/keeper/STATE.md:870–875` requires an
unchanged full pass; the diagnostic is useful evidence but does not meet
that requirement. QA's disposition is recorded in `ce02fe1a`.

- `flock /tmp/svrngs-gate.lock bash /tmp/opencode/keeper-audit-webkit.sh 2b1a7040`
  ran the unchanged `scripts/check.sh` with the main `.git` mounted and
  `TOKIO_WORKER_THREADS=4`. The candidate exited 1: the WebKit fallback
  took **4010 ms** against the **4000 ms** bound, with 43 browser tests
  passed and 1 failed (`-4.await.log:1163–1307`).
- `flock /tmp/svrngs-gate.lock bash /tmp/opencode/keeper-audit-main.sh`
  ran the same full gate on `789707dc` in the same image. It exited 0
  (`-5.await.log:435–545`). The navigation abort was reproduced on
  earlier main; no same-image control here reproduced the candidate's
  exact fallback miss.
- `flock /tmp/svrngs-gate.lock bash /tmp/opencode/keeper-audit-browser-overlap.sh ab5516674ca7624d3dc70eb6987a4a869e728c7a`
  produced the four browser-only outcomes above (`-6.await.log:646–1084`).
  Both candidate trials passed, and the lone failure on base was a
  different case. No release check was relaxed.

## Final candidate: focused checks at `35881a42`

The release-tier replay has its own [record](../../release/2026-11-23/README.md).
For the final source candidate, the `scripts/ports.py` comparison above
rejected the in-memory deletion before accepting the actual listing:

```text
planted omission: FAIL missing: [('apps/svrngs-keeper/src/launchd.rs:2', 'svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0')]
actual: PASS 67 tracked markers, 67 listed; missing: [] extra: []
```

Both diffs against the rebased main `789707dc` exited 0 with no output:

```sh
git diff --exit-code 789707dc HEAD -- scripts/trust_floor.txt scripts/trust_check.py
git diff --exit-code 789707dc HEAD -G 'Content-Security-Policy|content-security-policy|[a-z]+-src|frame-ancestors|base-uri|form-action' -- apps crates web
```

`python3 -B scripts/lean_check.py` and `python3 -B scripts/trust_check.py`
both exited 0 in `sovereign-vulkan`:

```text
Lines of code, tests and apps/ not counted, and the most each may have:
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   25,723   most  25,723
  JavaScript                 8,741   most   8,741
  Swift                          0   most       0
trust floor: 13 policies, 13 listeners, 17 gates
```

These counts and caps fit CHARTER's final ceilings: 25,727 non-JavaScript
and 8,741 JavaScript, leaving four lines and zero lines respectively.

The locked `/tmp/keeper-audit/gate.sh` preflight used image
`3080239cdc83294d04a49ff0594eac163d39da784116911641cd5272d3fe49d1`.
Inside it, `git rev-parse HEAD` resolved `35881a4254428fdb0085acb9609ac61da34d1993`
through the main `.git` mount, and `ss -V`, `node --version` and
`rustc --version` reported iproute2-6.1.0, v24.22.3 and 1.95.0.
The final full gate verdict is still owed.

## Full gate at `69996642` — failed; type fix verified in isolation

`REVIEW-audit-available-10.await.log:1–2` pins the same image and the
revision. The unchanged full gate exited 1 after 1,620 s. `types` was the
only failing step; the same run passed `browser`, `relay fallback`,
`keeper e2e`, `typed lint`, `pages`, and `scenes`. The conformance canary's
two WebKit worker-stop observations remained could-not-judge.

Replayed the `types` step (`scripts/check.sh:137`) in the same locked gate
container with `scripts/build-web.sh >/dev/null && node scripts/types.mjs`.
Before the fix it reproduced the failure:

```text
crates/svrngs-node/src/home/page/running-here.js(208,10): error TS7006: Parameter 'run' implicitly has an 'any' type.
types: 1 errors in 112 files
```

The tagged template does not contextually type the callback in its
interpolation. The JSDoc annotation at `running-here.js:208` takes the
parameter type from `found`, which consumes that callback. The same
replay, followed by `node scripts/typed-lint.mjs`, exited 0:

```text
types: 0 errors in 112 files
typed lint, jsconfig.json: 108 files, exit 0
typed lint, jsconfig.workers.json: 3 files, exit 0
```

The annotation adds no counted lines. `lean_check.py` still reports
3,471/3,539 core, 25,723/25,723 non-JS and 8,741/8,741 JS;
`trust_check.py` reports 13 policies, 13 listeners and 17 gates. The
CSP and trust-floor diffs against `789707dc` and the `scripts/check.sh`
diff against `35881a42` all exited 0 with no output. The port comparison
again rejected the in-memory omission of `launchd.rs:2`; the actual
ledger matched all 67 tracked markers, with none missing or extra.
The unchanged full gate is owed on the repaired revision.

## Squashed candidate: focused audit

The keeper was squashed onto main `05896fb0` in `eb58f807`. The Fedora,
Mac and full-gate results above apply to the archived branch, not this
candidate. On `923a4b9a`, the tracked marker comparison covered both port
sources recognized by `scripts/ports.py`:

```text
planted omission FAIL: [('apps/svrngs-keeper/src/launchd.rs:2', 'commonwealth-ai svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0')]
actual: 69 tracked, 69 listed; missing=[], extra=[]
```

`python3 -B scripts/trust_check.py` exited 0 with `13 policies, 13
listeners, 17 gates`. Both diffs below exited 0 with no output:

```sh
git diff --exit-code 05896fb0 HEAD -- scripts/trust_floor.txt scripts/trust_check.py
git diff --exit-code 05896fb0 HEAD -G 'Content-Security-Policy|content-security-policy|[a-z]+-src|frame-ancestors|base-uri|form-action' -- apps crates web
```

`python3 -B scripts/lean_check.py` exited 0: 3,529/3,539 core,
28,590/28,590 non-JS and 8,795/8,795 JS. Against the new main caps
(27,385 and 8,747), the candidate adds 1,205 non-JS and 48 JS lines.
CHARTER's original final ceilings of 25,727 and 8,741 were 1,210 and
48 above the original main caps at `e5d9813b` (24,517 and 8,693).
Applying just those keeper allowances to the new main caps, as CHARTER's
“Alongside app-state” directs, leaves five non-JS and zero JS lines.

The Mac replay needs the carried-SQLite gitlink from the same pin. A
scratch extraction of `git archive HEAD` alone gave `fatal: cannot change
to '…/vendor/cr-sqlite/core': No such file or directory` from the exact
Git read `crates/svrngs-db/build.rs` uses. The clone control with an invalid
gitlink gave `fatal: remote error: upload-pack: not our ref` (exit 128);
the candidate's gitlink produced HEAD `c3062b8d802eb9bc44237f5a8b987885adf5d149`
and the bundle manifest. The Mac driver stages and fetches those inputs
under the scratch root and beefymac's gate lock before its offline builds.

The existing main `.git` mount resolved the checkout's HEAD in the gate,
but `git -C vendor/cr-sqlite/core rev-parse HEAD` exited 128 (`not a git
repository: /work/vendor/cr-sqlite/../../../svrngs/.git/worktrees/svrngs-keeper/modules/vendor/cr-sqlite`).
The `/work` user-manager fixture stays at `/work`. With the checkout also
mounted at `/svrngs-keeper` and the same main `.git` at `/svrngs/.git`,
Git resolved the submodule HEAD to `c3062b8d802eb9bc44237f5a8b987885adf5d149`,
as it does on the host. An in-container `git submodule update` rewrote
`core.worktree` for `/work`, making host `git status` fail. The original
relative setting was restored. The preflight now initializes the gitlink
on the host side and verifies it through read-only `.git` mounts inside
the gate.

### Squashed release replay at `e63c4ba9` — failed in Actual stop-gap

`REVIEW-audit-available-14.await.log:76–120` records 23 of 24 release
cases passing. `linux-container-stop-gap.log:3–18` shows that Stop removed
the pod, it stayed stopped through a user-manager restart, and explicit
Start recovered the marker. After the unopposed crash, the root page
answered 200 but the first login returned `('000', '', False)`. The
release tier exited 1; that run did not reach the full gate.

A focused replay used the same release keeper (`sha256sum` starts
`b7efc7b81ff24d8b`, matching the archive manifest in the await log).
Capturing curl stderr showed both connection refusal (exit 7) and
connection reset (exit 56) during startup. The stop-gap driver now waits
on the login itself (`stop_gap.py`, `login_when_ready`) in the existing
16-second readiness window. Only no HTTP response is retried; an HTTP
answer must be `200`, `ok`, and include a token for the original marker.

The Fedora VM plant redirected only the first post-crash login to an
unbound loopback port. The wrong-marker control sent an incorrect
password to the actual login endpoint. Selected output:

```text
old driver, transport plant: attempt 1 status=('000', '', False)
AssertionError: ('000', '', False)                                   exit 1
new driver, wrong marker: attempt 2 status=('400', 'error', False)
AssertionError: ('400', 'error', False)                              exit 1
new driver, transport plant: attempt 1 status=('000', '', False)
new driver, transport plant: attempt 2 status=('200', 'ok', True)      exit 0
unplanted: Actual login did not answer, http=000 (twice after crash)
unplanted: an unopposed crash still recovers; restarts=1;
  status=["200", "ok", true]; same-name foreign pod untouched          exit 0
```

The driver discarded each VM disk, including its units, pods, drop-in
and volume. The focused audit again matched all 69 port markers after
rejecting an in-memory omission of `launchd.rs:2`. Trust check: 13 policies,
13 listeners, 17 gates; zero diff for CSP and the trust floor from
`05896fb0`, and for `scripts/check.sh` from `923a4b9a`. Lean check:
3,529 core, 28,590 non-JS, 8,795 JS, still five non-JS and zero JS lines
below the rebased ceilings. Both release tiers are owed on the repaired pin.

### Squashed release replay at `8729eb61` — failed in the Mac node suite

`REVIEW-audit-available-15.await.log:76–121` records 23 of 24 release
cases passing, including the repaired Actual stop-gap case. The release
tier exited 1 before the full gate. The Mac cleanup case passed.
`/tmp/opencode/keeper-release-squash-8729eb61/beefymac.log:1345–1370`
records the failing Mac node assertion and its inner status:

```text
assertion `left == right` failed: accepted late
  left: NotMine
 right: Uid(502)
test result: FAILED. 243 passed; 1 failed; 3 ignored
release-case mac-node-tests 101 darwin-arm64/cargo-test/svrngs-node
```

The fixture starts a thread that sleeps 30 ms before calling `accept`
(`crates/svrngs-node/src/uid_tests.rs:215–233`). The reader looks again
for up to 100 ms (`crates/svrngs-node/src/uid.rs:102–109,178–200`). The
failing run reports neither when the fixture accepted nor the retry
budget remaining; the assertion alone cannot distinguish a late fixture
from a missed socket read. The focused diagnostic measures both times
and rechecks the same socket after the accept if the first read failed.
