# Windows, full parity — due diligence

Status: surveyed 2026-11-19, starting at `a6f091c05982` with a dirty
tree — the share-binding queue's work is uncommitted in
`crates/svrngs-node/src/` (claim ring-a86928d156c12253), and `uid.rs`,
this survey's centre, is theirs until it lands: implementation rechecks
every symbol cited below after that queue. The contract companion, where
each Windows fact is numbered W1-W6 and the spike is planned, is svrn-docs
`rings/current/SHARE_BINDING_WINDOWS.md`; this file is the survey, the
estimate and the operator's decisions. Neither has run on Windows: every
external fact there is cited to a source read, and none is believed
before the spike.

## Recommendation

Full parity, in six rows, spike first: a Windows computer joins, serves,
reads and **lends with the same binding promise as Linux and macOS**
(SHARE_BINDING I1-I6), through a third reader in `uid.rs` beside
sock_diag and libproc. The decisions stay put: `binding.rs` is
platform-free by design ("moves to `eudoxia-community` whole") and moves
to Eudoxia with SHARE_BINDING §5.9 whether or not Windows comes; Windows
only supplies reads. The Mac taught the shape (processes, not cookies:
`Read { lasting: false }`, `Took::Lineage`); Windows buys Linux's sight
with it (every socket, every account, `dwOwningPid`, W1). Nothing on the
wire changes, so no version fence is owed.

This needs no hosted service, no new cryptographic grant, no protocol
change and no core lines. It does need a Windows VM (the spike's, then
the release test's), one new dependency (`windows-sys`), the second
`unsafe` *site* under the one exemption `lean_check.py` already allows,
an install/release story (install.ps1, a `windows-x86_64` tarball, a
signing decision), and its own gate tier. Each is the operator's, §
"Decisions".

## What exists, and what can serve

| Existing surface | Evidence | Use |
|---|---|---|
| The reader seam, platform-free | `binding.rs` (approve/check, "no system's types"); `uid.rs`'s `linux`/`mac` halves and its `#[cfg(not(any(linux, macos)))]` stubs (:702-723) | A third half. The stubs mean the crate compiles degraded on Windows today; nothing crashes, `share` refuses `NoReader` (`share.rs:273,334`) and session's localhost check passes all comers (`session.rs:539`) |
| The macOS decision shape | `binding.rs` `Took::Lineage`, `Read.lasting=false`; `uid.rs` `waited`, `ACCEPT_WITHIN`, the ntstat canary (:925-953) | Windows inherits lineage approval and the accept-wait; its canary plants a UDP row |
| Per-lend approval storage | `lend.rs` `Approved` (serde, `deny_unknown_fields`), `share.rs` `reapproved` | Unchanged: `pid@start` with absolute FILETIME starts (W2/W4), safe across reboots where Linux's is not |
| Service keep-at-login, three managers in one crate | commonwealth-ai `svrn/crates/sovereign-service` @ `3a05f1e89`: launchd/systemd/`schtasks` arms, `data/windows/SvrnmeshDaemon.xml` ("per-user, no elevation… the exact analogue of a launchd LaunchAgent"), stop-as-success semantics | Fork for `svrngs restart`'s Windows arm (cli.rs:1011-1026) and install's keep step, `ported from` on every file |
| Self-update, platform-keyed | `update.rs` `platform_of` (:337-348, macos/linux only), `apply_at`'s rename (:226-229), `File::try_lock` claim (:433-444, std, cross-platform) | Add `windows`; tar.exe ships in Windows 10 1803+ (the `tar -xzf` at :204 may hold; verify in the tier). A running exe cannot be replaced: rename it aside, then place the new one — its own commit, behaviour kept apart |
| The gate | `scripts/check.sh` (bash, `lockf`, `ulimit`, `caffeinate`); no CI anywhere (`ls .github/workflows` — none) | A Windows tier is written, not extended: see "Gate" below |

Node paths are under `crates/svrngs-node/src/`, command paths under
`crates/svrngs/src/`, as LENDING_SCOPE's convention.

### Upstream inventory

Checked commonwealth-ai at `3a05f1e89` (also where SHARE_BINDING's
upstream facts sit):

- `svrn/crates/sovereign-service/src/lib.rs` and
  `data/windows/SvrnmeshDaemon.xml` — the whole three-manager service
  story including the Windows Scheduled Task, its rationale comment and
  its crash-loop brake. Fork; do not rewrite.
- `code/crates/sovereign-cli-dev/src/code_fieldglass/mod.rs:709-715`
  and `code_capability_graph.rs:780-786` — a three-way browser opener
  (`open`/`explorer`/`xdg-open`); two lines of pattern, worth matching
  in `home.rs opened` (:1104) but not a fork.
- No socket-ownership reader exists there for any platform, Windows
  included: the only `target_os = "windows"` uses are the above and a
  template include. As with LENDING_SCOPE, this is a scoped inventory,
  not a semantic-clone result: nothing to fork for the reader; it is
  svrngs's own third half, written to `uid.rs`'s existing pattern.

## Policy and representation

- **Whose a socket is.** `Owner::Uid` stays the closed set it is: same
  account decided exactly by the token opening (W4), another account's
  `NotMine` by its denial — a decision where the Mac's is an inference.
  Another account's *name* is not read unelevated; the shown answer says
  "another account's program". Linux's exact uid is not matched, and
  that difference is said in the shown words, not hidden.
- **Socket identity.** No cookie on Windows (W2): `lasting: false`,
  approval by `(pid, start)`, the Mac's shape. A restart holds the
  share until one click, on every platform now.
- **The one `unsafe`.** `lean_check.py`'s `UNSAFE_HOME` allows exactly
  one `unsafe_code` exemption in `uid.rs` (`found > (rel == UNSAFE_HOME)`
  fails a second). The windows-sys calls live inside that same one
  `kernel(Ask)` function, cfg'd arms within one body — not a second
  exemption. If the shape will not hold, the rule change is asked for,
  not fallen into.
- **New dependency.** `windows-sys` (Microsoft, zero-dependency, raw
  FFI, feature-gated: IpHelper, Threading, ToolHelp, Security,
  Foundation), pinned like `netlink-sys` is (`=0.8.8` style). Parsing
  stays svrngs's: ports in network byte order, states, FILETIMEs read
  in safe Rust where the type does not already carry them.
- **Secrets and modes.** `declared.rs`'s 0o600/0o700 modes
  (:127-185) have no Windows meaning: the cfg arm drops them and the
  trust-floor commit says plainly that Windows protection is the
  profile directory's ACLs (a per-user `%LOCALAPPDATA%` data dir), as
  `SVRNGS_PACKAGE`'s packaging row says the pacman story in words.
- **Size budget.** Windows lines compile only under `cfg(windows)` but
  count fully against `.` (the count reads the tree as-is). Caps today:
  core 3,540, `.` 23,102, js 8,652 (`BUDGET`, lean_check.py:836-844);
  the tree is dirty with the share-binding queue's landing, which comes
  first. Raises are incremental, to measured+1 per row, per the
  operator's standing rule. One gap to decide: `CODE` counts
  `.rs .ts .py .sh` — `install.ps1` would not count unless `.ps1` is
  added, and consistency says it is.

## Enforcement and portability checklist

The compile tier's cfg gates, each named (the spike proves them green
on the target triple; `cargo check --target x86_64-pc-windows-msvc`
from a Windows VM or `x86_64-pc-windows-gnu` from here for
type-checking):

1. `node.rs:131-149` `asked_to_stop` — already cfg'd; Ctrl-C arm holds,
   and a Scheduled Task stop arrives as a console event
   (`tokio::signal::windows` under the "signal" feature already on).
2. `apps/give/src/main.rs:46,155-159` — libc signal handlers need a
   console-ctrl arm or Give stays off Windows (its own small row).
3. `update.rs:227` chmod 0o755 and `:229` rename — the rename-aside
   dance; `platform_of` gains `"windows"`.
4. `cli.rs:1012` `MetadataExt` uid and `:1015` launchd/systemd — the
   schtasks arm, forked from sovereign-service; `NotKept` refusal keeps
   its meaning.
5. `cli.rs:1074-1079` `stty` echo — a console-mode arm.
6. `home.rs:1104` `opened` — `ShellExecuteW` or `explorer.exe`; the
   `browse` trait boundary is unchanged.
7. `declared.rs:127-185` modes — the ACL posture above.
8. `oplog.rs:357-367` — already carries a non-unix arm.
9. Tests: `install.rs` (1,648 lines of launchd/systemd stubs) needs a
   Windows tier beside it, not inside it; the `PermissionsExt`/
   `symlink` test sites (`folders.rs:227,652,1021`,
   `confined_tests.rs:14`, `declared/tests.rs`, `lend/tests.rs`,
   `oidc/tests.rs`, `identity_tests.rs`, `tests/share.rs`,
   `tests/verbs/share.rs`, `common/as_main.rs`) cfg-gate the same way
   `uid_tests.rs` already does per platform.
10. W5's compile proof: iroh, `netdev`, `notify`, `cap-std`,
    `rustls-platform-verifier` (the #160 report is retested at pin 0.7),
    `netlink-sys`/`libc` staying off the Windows graph.

The door, sweep, choices, OIDC, sessions and pages are TCP/HTTP/JS and
move nothing: no wire change, no version bump, no migration — lend files
never travel between computers, and `Approved` is lender-local.

## Releases and the gate

- `scripts/build-release.sh` gains `windows-x86_64`
  (`x86_64-pc-windows-msvc`; built on the VM — MSVC does not cross from
  the Mac — or cargo-xwin, the operator's toolchain call).
  `web/install.sh` gains an `install.ps1` twin spelling the same
  `RELEASE_FILE`; `update.rs`'s sums line takes the new tarball name.
  Signing: darwin builds are signed and notarized today; Windows
  unsigned means SmartScreen's first-run warning — an Authenticode
  certificate costs money yearly and is the operator's, with "ship
  unsigned, say so on the page" a real option.
- The gate: `check.sh` is bash with `lockf`, `ulimit`, `caffeinate` —
  macOS-shaped, and there is no CI to extend. The Windows tier is its
  own runner on the VM (a small `windows-check` step-set: the reader's
  fixture and unit tests, the install/service tier, the release test
  harness), its steps said **passed / failed / could-not-judge /
  never-ran** exactly as `check.sh` says them, with rows in
  `docs/CHECKS.md` and `docs/seams.tsv` naming the Windows-only seams'
  home. The full Unix gate stays the whole gate where it runs; the
  Windows rows are owed their own never-ran honesty, not a silent
  absence (ARCH 5-6).
- `svrngs-fam` (the Omarchy panel) is out of scope: the home page serves
  Windows approval, as it serves macOS's today.

## The environment, concretely

Two machines and one new VM; the Mac's role changes not at all.

- **The Mac keeps the day-to-day loop.** A Windows `record` module
  compiles for tests on any platform, as the Linux and mac ones do
  (`#[cfg(any(target_os = "macos", test))]`, uid.rs:728): the table
  parsers and their fixture tests run in this Mac's gate against bytes
  captured once on the VM, and `binding.rs`'s tests already run here.
  Most of the reader is written and tested on the Mac, in seconds.
- **A Windows VM joins the Fedora box's lab** — beside Omarchy's a, b
  and c, KVM, ~8 GB as each of them takes
  (`share-binding-release-test/README.md`), OpenSSH's server on, driven
  by that harness's `guest.py`/`run.py` pattern. It is the Windows
  everything else: the spike ("Proof before landing"), the one-shot
  fixture capture, the MSVC compile tier (`x86_64-pc-windows-msvc`
  does not cross from the Mac; the gnu target via zig type-checks here
  for the fast loop), the real-socket tests as `#[ignore = "drives
  real Windows sockets: run on the Windows VM by hand"]` (the
  `systemd-run` tests' pattern, uid_tests.rs:410-453), the
  `windows-check` runner over SSH, and the release build and release
  test, as V1.12.2's ran on a and b.
- **Two checks before provisioning it**: `uname -m` on the Fedora box —
  x86_64 gives native-speed Windows x86_64 under KVM, which settles the
  arch question naturally; and the license, since an unactivated
  Windows boots the spike but not a shipped release test's record.

## Economy and estimate

Ranges from the inspected sites, not permission to spend them; the
share-binding queue's landing re-baselines every count first.

| Slice | Non-JS added | JS added |
|---|---:|---:|
| Compile tier: cfg gates, restart/schtasks fork, update's Windows arm | 95–200 | — |
| The reader: tables (TCP/UDP, v4/v6), token/start/parents, canary, shown answers, `waited`, `Owner` wiring | 280–420 | — |
| install.ps1, build-release/install twins (uncounted today; `.ps1` decision above) | — | — |
| Gate tier runner, CHECKS/seams rows (scripts count against `.`) | 20–40 | — |
| **Product total** | **395–660** | **0** |

svrngs-core: zero planned. Test lines, separate as always: fixture and
unit 250–450 (the "read where it holds it" pattern, plants named),
Windows install/service tier 300–700, release-test harness beside
`share-binding-release-test/` in svrn-docs (a run's driver, uncounted).
Each landing raises its cap to measured+1, asked with its reason (the
standing rule); no raise here funds a second approval UI, a Windows
panel, or MSIX/winget packaging.

## Proof before landing

No existing test framework changes; every new gate watched failing on
its plant first (AGENTS 5), every decision branch traced at `debug`
(target `uid`, glassbox).

1. **The spike** (svrn-docs SHARE_BINDING_WINDOWS §4): each W1-W5
   measurement and the UDP double-bind matrix, recorded with its run,
   before any reader code.
2. **Fixture tests**: a captured table's bytes read where the reader
   holds them; planted offsets, planted absences, the canary refused.
3. **The refusals fail on plants**: the pid match skipped; the
   four-tuple match skipped; `NotMine` forged; `NoReader` unreachable.
4. **The release test** on the Windows VM: §9.1's items 1-2 with the
   same zero-byte bars (old link, takeover, recovery one click), a
   service-account Jellyfin for the other-account case, and a
   Windows-hosted Factorio for item 3.
5. **A people run**: a Windows computer joining a group and lending,
   recorded under `demo/runs/` (ARCH 7): the demo the six rows move.

## Order of work

1. The operator's decisions (below) — the VM first; nothing runs
   without it.
2. The spike, recorded in svrn-docs; the contract page's **to measure**
   list emptied or its design amended.
3. Compile tier (the ten gates above), proved green on the triple.
4. The reader, fixture tests and refusals, behind the one exemption.
5. install.ps1, the release target, `restart`, update's dance.
6. The gate tier, the release test, the people run, the re-review asked
   with them (SHARE_BINDING's re-review gains the Windows rows).

## Decisions for the operator

Each in plain words, with what each choice changes:

1. **The VM**: a Windows VM on the Fedora box's lab, beside Omarchy's
   a/b/c ("The environment, concretely"), and its license. Everything —
   spike, tier, release test — waits on it; `uname -m` on the box
   settles x86_64 (native under KVM, most installed Windows) against
   aarch64 (needs an ARM host, Qualcomm laptops only).
2. **The dependency**: `windows-sys` pinned, FFI inside the one
   existing `unsafe` exemption. Refusing it means no reader, which is
   the degraded tier, not parity.
3. **The budget**: product +395–660 against `.` 23,102, incremental
   per row after the share-binding landing; and whether `.ps1` joins
   the counted kinds (consistency says yes).
4. **Signing**: Authenticode (yearly cost, no SmartScreen warning)
   or unsigned with the warning said on the page.
5. **The toolchain**: build the Windows release on the VM (simple,
   slow, honest) or cargo-xwin from the Mac (fast, one more moving
   part).
6. **Scope of the first milestone**: this document recommends full
   parity in the order above, with the degraded tier (member, not
   lender) available at row 3 as a shippable checkpoint if the reader
   slips — the inverse ordering (degraded first, reader later) spends
   the same total and delays the promise.
