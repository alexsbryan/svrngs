# svrngs-keeper — try it however you like, then keep it running here

The operator, 2026-11-22: "people doing all the stuff they do to run
programs and test them out … when you want to move to production you just do
something simple … that will just run your program no fuss, keep it alive,
restart it on boot (all configurable)", then "the shared runtime should be a
clean package", named svrngs-keeper, and "lean and scrappy".

The design is in svrn-docs `rings/current/`:
- `LOCAL_HOSTING.md`: the scene;
- `HOSTING_AND_GIVE_BOUNDARY.md`: the owners;
- `KEEP_SHARING_ON_RESTART.md`: the renewal rule;
- `KEEP_SHARING_BUILD.md`: the budget.

The spike behind them is `local-hosting-spike/`, with its five no-login
VM boots and the Mac runs.

**Scrappy means** Linux first, command line first, and native programs
first (Sam's model). The page, the Mac, and Give's containers on OS
lifetime come after the demo.

## The user flow, as this branch builds it

Sam has run llama-server in a terminal and lent it:

```
$ llama-server -m ~/models/qwen2.5-7b.gguf --port 8101
$ svrngs share 8101 --program llama-server --name coder --group house
…
coder runs from a terminal: closing it, or restarting this computer, stops it.
To keep it running whenever this computer is on: svrngs-keeper keep 8101 --lend coder
```

**Promote it.** One command, one yes:

```
$ svrngs-keeper keep 8101 --lend coder
svrngs-keeper will run coder: llama-server with your model
~/models/qwen2.5-7b.gguf on port 8101, context 4096.
It starts when this computer starts, before anyone logs in, and starts again
if it stops (4 times a minute at most). The llama-server in your terminal
stops, and this one takes its place. Keep it running? Type y, then press Enter: y
coder runs here now, and it answers on port 8101.
Now answering on coder's port: the systemd unit svrngs-keeper-coder.service.
Keep sharing coder with it? Type y, then press Enter: y
coder stays lent to house whenever this computer runs it.
```

The second question is svrngs's own `svrngs approve`, run in Sam's
terminal. A unit approved this way is granted to stay lent across its
restarts.

The first time, keeping a card before login asks once more: *"To start it
before you log in, your account needs to run when nobody is logged in.
Turn that on? Type y…"* (linger, `set-self-linger`). On no, the line says
*"It starts when you log in"*.

**Restart the computer:** nothing to do. Alex's link opens once coder
answers.

**Manage it:**

```
$ svrngs-keeper list
coder   running since 09:14 · starts when this computer starts · 0 restarts · port 8101
$ svrngs-keeper stop coder        # stopped until you start it; lending it pauses
$ svrngs-keeper start coder
$ svrngs-keeper remove coder      # no longer run here; your model and its card stay, and it says where
$ svrngs unshare coder            # Stop lending: coder keeps running for you
```

**When it can't:**
- **Something it doesn't know:** it shows the command it would run, with
  no environment carried, and lists by name the settings Sam's terminal
  added.
- **The copy doesn't answer** within 30 s: it says so, and starts Sam's
  original again exactly as it was.
- **It declines** a port that changes at each start, a program that
  wants typed input, or an environment it can't read, and says which.

## Baseline

Branched from main at `e5d9813b`. Lean: core 3,465/3,539, non-JS
24,517/24,517, JS 8,693/8,693. `apps/` is uncounted (lean_check.py:973).

## Estimates

These are the figures the launch approved, in code lines as
`scripts/lean_check.py` counts them:

| Row | Product | Counted in svrngs | Tests | Sessions |
|---|---|---|---|---|
| keeper-move | 0 new (1,951 moved) | 0 | 0 new | 1 (done) |
| keeper-units | ~525 (195 forked) | 0 | ~270 | 1–2 |
| keeper-promote | ~350 | 0 | ~250 | 2 |
| svrngs-economize | cuts | ratchets | 0 | 1 |
| eudoxia-cgroups | ~15 (eudoxia) | 0 | ~60 | 1 |
| svrngs-renew | ~180 Rust, ~30 JS | the same, less the cuts | ~250 | 2 |
| keeper-demo | ~150 (run driver) | 0 | — | 1–2 |
| REVIEW-audit-keeper | repairs | 0 | — | 1 |

## Rows

- [x] keeper-move 909bd73b — depends [] — ONE UNIT, behaviour kept:
  - `apps/svrngs-keeper`, a library crate in the workspace. Give's
    `card.rs`, `check.rs`, `podman.rs`, `runtime.rs` and `machine.rs` move
    into it unchanged, with only their paths fixed.
  - Give depends on it and re-exports those modules, so its 13 call sites
    stay as they are.
  - Give's tests stay in Give and pass unchanged through the re-exports:
    `check.rs`, `machine.rs` and `keeper.rs` share its Podman stand-in
    (`tests/fake`), and `machine.rs` also uses Give's `shelf`.
  - **The keeper's own `tests/surface.rs`:** it links no svrngs product
    crate and no Give.
  - **Give's surface test** refuses any dependency named `svrngs…`. It
    admits `svrngs-keeper` by its exact name, and refuses every other.
  - Give's README and lib doc say where the runtime went.

  read: apps/give/{Cargo.toml,src/lib.rs,tests/surface.rs}; Cargo.toml
  members.
  check: Give's tests and scenes green, unchanged;
  `cargo test -p svrngs-keeper`; clippy; fmt. Both surface tests watched
  red on a planted `svrngs-node` dependency.

- [x] keeper-units 0f02ef741 — depends [keeper-move] — ONE UNIT:
  - **Fork** commonwealth-ai `shared/crates/host-kit/src/service.rs` and
    its tests @8ed8d99b0 as `systemd.rs`, with the ported-from line. That
    gives `SystemdUser`: install, uninstall, state, `ensure_linger`, and
    specifier escaping.
  - **The card gains one field:** `x-svrngs.native`, with a command, a
    folder and environment names, and no services. A native card renders
    to `svrngs-keeper-<name>.service`, carrying the spike's settings:
    `Restart=on-failure`, `RestartSec=2`, `StartLimitBurst=4` in 60 s,
    `SuccessExitStatus=143`, `WantedBy=default.target`.
  - **The binary `svrngs-keeper`:** `keep <card>`, `list`, `stop`,
    `start`, `remove`.
  - **Where things go.** Kept cards live in
    `~/.local/share/svrngs-keeper/<name>/`, with the previous one kept for
    a later "go back". Linger is asked for once, and a refusal is a
    warning, as host-kit has it.

  read: host-kit service.rs and tests; card.rs's `x-svrngs`; the spike's
  units.
  check: argv and unit text by stand-in systemctl and loginctl, as
  host-kit's tests do. On this host, a kept native card:
  - kill -9 brings it back;
  - `stop` stays stopped past 10 s;
  - `remove` leaves its data;
  - `keep` run twice resets nothing.

- [x] keeper-promote 2e78b0939 — depends [keeper-units] — ONE UNIT:
  - `keep <port>`. The port's holder is read with `ss -ltnpH`; the
    keeper links no svrngs reader.
  - **A program it knows** has a card it ships (`cards/llama-server`),
    filled from what the program reports: `/props` `model_path`,
    `default_generation_settings.n_ctx`, and the port.
  - **Anything else** is copied: the executable (keeping its first
    argument as its name), the arguments, the folder, and the parent chain
    up to the shell, so a wrapper script is adopted, not its child. No
    environment is carried; the settings the terminal added are listed by
    name.
  - **The handover:** show, ask, stop the original (by pid@start), start
    the unit, then verify. The same executable must listen on the same
    port and answer within 30 s. If it doesn't, start the original again
    with its arguments, folder and full environment, and say so.
  - Then it runs `svrngs approve <lend>` in the same terminal, when that
    port is lent: `keep <port> --lend <lend>`, which also names the card.
    No svrngs command says which lend a port is (the loopback API's routes,
    crates/svrngs-node/src/surface.rs:158-161), and the lend files are
    lend.rs's schema, so svrngs, which knows the lend, gives its name.
    Without `--lend` it ends by saying the `svrngs approve` to run.

  read: the spike's `cmd` reads (four ways to start llama-server).
  check: on this host:
  - a relative model path and a `~` path, each composed from `/props`;
  - a wrapper script, composed;
  - a renamed python process, copied with its name;
  - a planted copy that needs an environment variable: rolled back, with
    the original answering again.

- [x] svrngs-economize c2c69a5cc — depends [] — ONE UNIT, behaviour kept: the
  economizing pass over the code svrngs-renew extends:
  `crates/svrngs-node/src/door/bound.rs`, `share.rs`, `uid.rs`,
  `home/page/lent-row.js` and `lend-change.js`. It cuts in none of the
  files app-state (PR 7) has changed. Read
  `git diff --name-only main...origin/app-state` at the start, and leave
  those files out (CHARTER, "Alongside app-state"). At 2026-11-22 that
  rules out `lend.rs` and `door.rs`.
  - Preserving cuts only, each its own commit, each ratcheting the caps
    in `scripts/lean_check.py` down to one past where it lands.
  - Record the baseline, the candidates and each cut in
    `demo/runs/economize/2026-11-keeper/SURVEY.md`.

  read: AGENTS.md "Done"; svrn-docs rings/current/ECONOMIZING.md; the six
  files.
  check: each cut's focused tests (`cargo test -p svrngs-node --lib door`,
  `share`, `uid`; `node web/tests/run.mjs unit`); fresh lean counts.

- [x] eudoxia-cgroups 2d97a99e9 — depends [] — ONE UNIT, in `~/dev/eudoxia` on
  branch `keeper-cgroups`, under eudoxia's own AGENTS.md:
  - `binding::Chosen` gains a cgroup form, every socket on the address
    made in one of a set of cgroups, beside the process form, with the
    same `Refused::Another` when one isn't.
  - `check` is unchanged: what took a connection is still judged by the
    approved cgroups.
  - Then svrngs on this branch takes it through
    `[patch."https://github.com/alexsbryan/eudoxia"]` with
    `eudoxia-community = { path = "../eudoxia/community" }`, in its own
    commit here. Compile only; nothing uses it yet.

  read: eudoxia community/src/binding.rs and community/tests/binding.rs;
  svrn-docs local-hosting-spike/probe/src/main.rs `renew`.
  check: eudoxia's tests, each new case planted:
  - every socket in the set: approved;
  - one outside it: `Another`;
  - none on the address: `Nothing`;
  - the process form unchanged.

  `cargo check --workspace` here with the patch.

- [x] svrngs-renew 1581a2f35 — depends [svrngs-economize, eudoxia-cgroups] — ONE
  UNIT, svrngs's half (K1+K2 of KEEP_SHARING_BUILD):
  - **Name the job.** `Answer::Job` from the socket's cgroup path and the
    unit's `Description`: the approval says "the systemd unit
    svrngs-keeper-coder.service", not `pasta.avx2`.
  - **The grant.** `Terms::keep`, written when an approval's holders lie
    in one non-transient user unit (unless `--ask-after-restart`): the
    unit plus a hash of its files under `~/.config`.
  - **The door.** On `Held::Another` with `keep`, beside `hold`: read the
    unit (`/usr/bin/systemctl --user show`, cached by cgroup id), compare
    the hash, approve with `Chosen::Cgroups`, compare-and-set, check,
    carry. It never runs on `Unjudged` or `Empty`, never over `held`, and
    traces every branch.
  - **The row.** `Hold::Changed` and its words.

  Allowance: CHARTER's envelope, at most +180 Rust and +30 JS over the
  post-economize counts, landed with a `Size:` paragraph. Past it, stop
  with needs-human.

  read: door/bound.rs; share.rs approve; lend.rs Terms, Hold, Answer;
  eudoxia binding.rs; the spike's probe renew.
  check, each planted:
  - a crash and a restart are renewed;
  - another unit is refused, with 0 bytes;
  - a changed unit file gives `Changed`;
  - a held lend stays held;
  - a stopped lend stays stopped;
  - a transient scope is never granted.

- [x] keeper-demo 3cda16539 — depends [keeper-promote, svrngs-renew] — ONE UNIT,
  evidence:
  - Sam's scene, end to end, in the spike's Fedora VM. Sam and Alex are
    two accounts, each with a node, met at a local `svrngs relay`.
  - The steps: llama-server in a terminal, `svrngs share`,
    `svrngs-keeper keep 8101 --lend coder`, then three no-login reboots, a kill -9,
    `svrngs unshare` and a reboot.
  - Alex's request is timed each time, and Sam typed nothing after the
    promotion.
  - Recorded in `demo/runs/keeper/<date>/`.

  check: every case passed, failed, could-not-judge or never-ran, named
  in its run.

- [x] HUMAN-eudoxia-push 80c38bd — depends [svrngs-renew] — the operator:
  - Push eudoxia's `keeper-cgroups` commit, then say its rev.
  - A worker then re-pins `eudoxia-community` in
    crates/svrngs-node/Cargo.toml to that rev and removes the `[patch]`,
    in its own commit. The gate fetches eudoxia from its public repository
    (scripts/gate/README.md), so the audit needs it pushed.
  - No worker ticks this row or pushes.

- [x] keeper-linger-ask b7fdb8493 — depends [keeper-demo] — ONE UNIT: `keep` asks
  before it lets this account run with nobody logged in, as "The user flow"
  above has it. keeper-demo found it doesn't
  (demo/runs/keeper/2026-11-22/README.md, "Found on the way"). `keep` turned
  linger on with no question, and before the yes it said "It starts when you
  log in", read before it changed linger.
  - The first `keep` on an account that doesn't linger asks: "To start it
    before you log in, your account needs to run when nobody is logged in.
    Turn that on? Type y, then press Enter:".
    - **Yes:** linger, through the fork's `ensure_linger`.
    - **No, or no terminal to ask:** no linger. The unit is enabled for
      login, and the line says "It starts when you log in". A later `keep`
      asks again only while linger is off.
  - The plan's "It starts …" line says what will be true after that answer,
    never what was true before it.

  read: apps/svrngs-keeper/src/{main.rs,kept.rs,systemd.rs}; "The user
  flow"; the demo README.
  check, on the stand-in loginctl (tests/kept.rs), each planted first (the
  ask skipped, so linger is always turned on):
  - y turns linger on, and the line says "when this computer starts";
  - n doesn't, and it says "when you log in";
  - no terminal doesn't;
  - an account that already lingers isn't asked.

- [x] REVIEW-audit-keeper 2bca25e65 — depends [keeper-demo, HUMAN-eudoxia-push, keeper-linger-ask] — the unchanged full
  `scripts/check.sh` in the svrngs-gate container, exit 0. The ported-from
  lines are listed by `scripts/ports.py`. CSP and the trust floor are
  unchanged, and the counts are within each allowance.

## The release: available by default (svrn-docs rings/current/KEEP_IT_RUNNING_RELEASE.md)

The spec at svrn-docs `4f77680` is this section's design, and each row
names its part of it; read the spec's §2 and §3 before any row. Rows marked
(keeper) are under `apps/` and uncounted. The rest spend CHARTER's release
envelope.

- [x] avail-economize 480b53c69 — depends [] — ONE UNIT, behaviour kept: spec §5 row
  R3.
  - The pass over `surface.rs`, `home.rs`, `cli.rs`, `lend.rs`, `uid.rs`,
    `door/bound.rs`, `job.rs`, `lent-row.js` and `running-here.js`.
  - It leaves out every file app-state (PR 7) changed (CHARTER,
    "Alongside app-state").
  - Each cut is its own commit and ratchets the caps.
  - Recorded in `demo/runs/economize/<date>-available/SURVEY.md`.

  check: each cut's focused tests; fresh lean counts.

- [x] avail-plan befdaf978 — depends [avail-economize] — ONE UNIT: spec §3.1, §3.6,
  §3.7 P1, P2, P4; row R-plan.
  - `crates/svrngs-node/src/plan.rs` and `plan/words.rs`, both pure: the
    closed event types, `step` and `plan`, the return points, the method,
    the token, and every string as a function of the outcome.

  check, each planted first:
  - P1: the same events give the same plan and token, and folding a prefix
    then the rest equals folding the whole;
  - P2: the table test over the product of the closed sets;
  - P4: a `lean_check.py` rule naming `plan.rs` as pure.

  Demo: the pure P2 fixtures preview §2.1's words for each kind and act on
  nothing. Run `svrngs share <port> --plan` on the host once R1 wires the
  terminal (release spec §5 R1; today's `crates/svrngs/src/cli.rs:888–925`
  approves and writes the share, with no preview branch).

- [x] avail-keeper-facts 009ecd0df — depends [] — ONE UNIT (keeper): spec §3.4, §3.7
  T1; row R2.
  - Facts as JSON with the keeper's own token; `--yes`; `list --json`;
    `restart`; `start` resets a failed unit; `Nice=10`.
  - Recoverable takeover: lock, stage, `takeover.json`, stop the recorded
    pid@start, verify, enable. Reconcile on every call.

  check, each planted first: an interruption at each stage is reconciled on
  this host, and a retry never stops a second process (T1).

- [x] avail-plan-economize 2a81dc014 — depends [avail-plan] — ONE UNIT, behaviour kept: an economizing pass over the planner avail-plan landed (`crates/svrngs-node/src/plan.rs`, `plan/words.rs`, and their wiring in `lib.rs` and `lean_check.py`). The operator, 2026-11-22: "Do it", after avail-plan landed +309 counted lines against the spec's ~+140 (befdaf97's `Size:`), leaving 126 of the envelope's +435 for publish (~+40), terminal (~+65), page (~+30) and Mac renewal (~+150).
  - Preserving cuts only, each its own commit, each ratcheting the caps:
    repeated shapes folded, enums and matches that say the same thing
    twice, words built from one table.
  - The table test (`tests/availability_plan.rs`, P1, P2) and P4 hold
    unchanged throughout.
  - Recorded in `demo/runs/economize/<date>-plan/SURVEY.md`, which ends by
    re-estimating the remaining rows against what is left. It names the
    spec's cuts that would close any gap (spec §6: the Mac container
    witness −60, Restart −10).

  check: `cargo test -p svrngs-node --test availability_plan` green after
  each cut; P4's lean rule; fresh lean counts.

- [x] avail-publish 9629f16d1 — depends [avail-economize, avail-plan-economize] — ONE UNIT: spec §3.2, §3.7
  W1, W2; row R-publish.
  - A share's revision, with tombstones on withdrawal.
  - One conditional publish through `lend::mutate`: binding, grant,
    audience and limit together.
  - `RunningNotShared`.
  - The revisions and tombstones live where app-state's `lend::all` doesn't
    look (PR 7, `lend.rs:904`, which lists every `<name>.json` in the lends
    folder as a declaration, as its `stamp` walks): another suffix, or a
    folder of their own. Commons lesson 5d939213.

  check, each planted first:
  - `lend::all` over a folder holding a tombstone lists no extra lend;
  - a concurrent withdrawal wins;
  - a stale confirmation is refused and never resurrects a withdrawn
    share;
  - a failed write leaves the job running, unshared, and the next plan is
    `Reuse`.

- [x] avail-terminal 9ef329ffb — depends [avail-plan, avail-keeper-facts, avail-publish] — ONE UNIT: spec §§2.1–2.2, §3.2; row R1, its code.
  - `plan/run.rs`, the executor at the edge: it replays at confirm, gives
    `ReviewAgain` with no side effects, and feeds each owner's answer back
    as a `Done` event.
  - `svrngs share` through the plan.

  check, planted first: partial success. A failed write leaves "Running
  here; not shared", and the next review shares it with no restart.

Rows from here follow the spec at svrn-docs `0080f3f`, "run it with svrngs" (the operator, 2026-11-22: "Yes this is all I wanted … that's the bulletproof system"). Each row's design and grounded estimate is the spec's §5 row named in it; where they disagree, the spec wins. svrngs never stops or signals a program it didn't start (spec §3.4, N1).

- [x] avail-unmagic 3c3646180 — depends [avail-terminal] — ONE UNIT: spec §5 row 1. Remove the live takeover, the adoption and the copying from a live process.
  - **Keeper:** `takeover.rs`; the stop, verify and identity paths of
    `host.rs`; `promote.rs`'s adoption, declines, copy card and
    environment; `facts.rs`, reduced to the prefill; `main.rs` `at_port`'s
    handover; `kept.rs`'s takeover hooks. The tests go with them.
  - **Counted:** the takeover and live-process arms of `plan.rs`,
    `plan/run.rs`, `plan/words.rs` and `cli.rs`.
  - Its commit carries a `Size:` paragraph, measured.

  check:
  - N1, a `lean_check.py` rule that svrngs never stops or signals a process
    it didn't start: planted against today's tree first (it must fail on
    `host.rs` `stop`), then green after the cut;
  - every remaining test of svrngs-keeper and svrngs-node green;
  - lean counts.

- [x] avail-run-linux eb93617c8 — depends [avail-unmagic] — ONE UNIT: spec §2.2–§2.4, §5 row 2.
  - `svrngs-keeper run <name> [--folder] -- <command>` writes a native card
    and its unit.
  - It refuses a busy port with spec §2.2's sentence.
  - Preserve llama-server's prefill (`facts.rs:26–36`) and `Nice=10`
    (`unit.rs:40`); Run adds the 30 s answer check.
  - Prove `start` resets a failed unit: already implemented in
    `apps/svrngs-keeper/src/kept.rs:283–296` at `21dc739e` (spec §7.8
    predates it).
  - svrngs gets `svrngs run` through the plan. **No new authorization
    mode** (QA, oc-cruu42, 2026-11-22). Run starts the unit, verifies that
    its listener answers (the 30 s check), and only then approves that
    listener through the existing `share::approve`, with its `Terms::keep`
    grant, in the same press. The spec's `Unapproved`-with-grant arm and
    grant-before-bind are not built.
  - **Only the unit's own listener is approved,** read fresh at the save:
    every socket on the port, and what took the dial, must lie in the
    unit's cgroup (as renewal judges, `door/bound.rs`). So a different
    listener between verification and the save is refused, never approved.
  - **Two failures, said apart** (QA, 2026-11-22):
    - verified running, then the save failed: "Running here; not shared",
      and a retry of Share makes no lifecycle call;
    - verification failed: "Not shared; svrngs couldn't verify coder is
      answering", unless the service manager separately reports it
      running.

  Demo, first: Sam stops his terminal copy and runs coder with svrngs, and
  Alex's link keeps working through three no-login reboots in the spike's
  VM and a kill -9. Then the busy-port refusal, and a failed save giving
  "not shared" with no second run.
  - **Unjammed** (keeper-lead with QA oc-cruu42, 2026-11-23, after three
    sessions ended with no commit; the operator: "clear up the jam").
    - `24bfe771`, the keeper's half, stands.
    - Sessions 2–4's uncommitted svrngs half was +325 counted against this
      row's +60. It is saved, not applied, at
      `ralph/next/keeper/ctl/avail-run-linux-wip.patch`. Take only what the
      cuts below leave.
  - **Two steps, literally** (QA's cuts; they supersede the earlier bullets
    where they differ):
    - `svrngs run <name> --port <port> -- <command>` runs the program and
      shares nothing. Sharing stays the existing `svrngs share`.
    - Re-running an app that is already shared keeps its reviewed terms
      through the existing conditional publication. If carrying the terms
      over costs more than a few lines, require an explicit Share instead.
    - No second share flow in `cli.rs`: no `--group` and no audience on
      Run.
  - **No planner scaffolding.**
    - Don't build a made-up `State` (dummy binding, unknown starts) to feed
      the share planner.
    - Add no `Event::Run` or `Method::Run` unless a concrete preview reads
      them.
    - Run is one concrete call into the keeper, not a generic executor and
      save closure. Only an actual Share uses the revision and token
      machinery.
  - **The keeper call.** One bounded call in `plan/run.rs`, with its
    `NODE_COMMAND` entry restored in the shape it had before 3c364618
    (`git show 9599ab05:scripts/lean_check.py`, line 46). Use the
    packaged sibling path, not a bare `Command::new("svrngs-keeper")` that
    searches PATH.
  - **The grant check is real.** The draft's `job_grant` has
    `let belongs = true`. The real cgroup check must stand, with red and
    green evidence in the commit.
  - **Allowance: +90 counted, at most.** If the row can't fit, stop with
    `needs-human` and per-file counts.
  - The demo's model is at `/tmp/keeper-demo/qwen2.5-0.5b.gguf`. Search
    nowhere else: a refused read ends the session.

- [x] avail-run-busy-witness 7295df83a — depends [avail-run-linux] — ONE UNIT (keeper, plus the demo): fix Run's false success, then close QA's busy-port evidence gap (oc-cruu42, 2026-11-23). At fabda979, `demo/runs/keeper/2026-11-23-run-linux/run.py:57–70` samples `/health` only before Run. It checks the SSH child's liveness, the managed unit's MainPID (0) and Alex's origin. None of those shows the original listener untouched, or nothing written. Patch that scene, with no new harness, to record in its JSONL, before and after the refused Run:
  - the port owner's PID and start time, from `ss` and `/proc/<pid>/stat`;
  - the hash of a stable localhost response body;
  - that the unit file and the keeper's job record are absent, and
    `systemctl --user show -p LoadState` says `not-found`;
  - the existing share's declaration and revision, which are equal.

  Then rerun that scene in the scratch VM and update the README's row. Demo: the busy-port case with all four witnesses equal or absent.

  **First, Run's false success** (QA, 2026-11-23).
  - Today `apps/svrngs-keeper/src/native_run.rs:97–105` calls a Run
    verified from `host::wait_for_answer` alone: an HTTP status on either
    loopback. If another program binds the port after the precheck and
    the new service exits, Run still says coder is running. Nothing is
    published wrongly, since Share checks ownership itself, but Sam is
    told something false.
  - The fix: claim "running" only with owned-listener evidence, a listener
    on the port inside the unit's cgroup. Otherwise report the manager's
    state and the endpoint's readiness as two separate observations.
  - Reuse what renewal and Share already read. Add no new authorization or
    parsing stack.
  - Plant: a foreign listener on the port while the unit has exited. Today
    it reads verified (red). After the fix it is not (green).
  - Size: keeper only, a few lines. Counted: none expected.

  **And the readiness test's flake** (QA, 2026-11-23). At 8f7f16d0,
  `apps/svrngs-keeper/tests/run.rs`'s
  `readiness_needs_an_answer_and_llama_must_finish_loading` failed its 200
  case once and passed on an immediate rerun.
  - The test server reads only `GET /health ` and closes with the rest of
    the request unread, which can reset the connection before the reply is
    read.
  - QA's test-only fix reads the whole request header before replying. It
    is at `ralph/next/keeper/ctl/readiness-flake.patch`; `git apply` it.
  - Commit it alone, as a test fix. Its body gives QA's evidence: with the
    patch, the 20 focused keeper tests and 10 more runs of the case
    passed. The original failure was intermittent, and no reset was
    captured directly.

- [x] avail-run-containers 37939176a — depends [avail-run-linux] — ONE UNIT (keeper), time-boxed proof: one explicit, volume-only card on Linux, run by svrngs (QA, oc-cruu42, 2026-11-22, with keeper-lead; this replaces spec §5 row 3's stash port). It reuses:
  - the card (`compose.yaml` plus `x-svrngs`) and `card::check`;
  - `runtime::App` (`pull`, `start`, `wait_healthy`, `health`, `stop`);
  - `machine` and `podman`.
  The Give and check call paths stay unchanged. Keeper needs an opt-in
  seam in `runtime.rs:172–202,517–534`: the existing start supplies no
  atomic ownership mark, and the existing stop targets a reusable name.
  Set the ownership mark during pod creation; inspect it and target the
  immutable Podman ID for cleanup. Proof 2 checks the same-name refusal.

  None of Give's custody policy (`keep.rs:585–611`) is taken.

  Kept off this row: the import of a hand-run container (`stash@{0}`),
  Quadlet translation, and host binds (`card.rs:543`, which is the
  operator's to decide).
  - **Proof 1, first, before any adapter is budgeted.** In the spike's VM,
    after a real reboot, fresh reads tie both the published listener and
    the accepted connection to the managed job's cgroup. Checked through
    Alex's unchanged link, with a marker written before the reboot and
    read after it.
    - If plain Podman started from a unit breaks that, measure the smallest
      OS delegation option (`--cgroups=split`, `Delegate=yes`).
    - Then stop and report with `needs-human`. Don't add an engine witness
      by default.
  - **Proof 2.** Kill the foreground adapter after the detached pod
    starts, and check the restart.
    - `App::start` makes the pod by name, and `App::stop`
      (`runtime.rs:517`) removes a pod by name. So a leftover pod must
      neither wedge recovery nor let cleanup touch a foreign pod of the
      same name.
    - Do it at two moments: once the pod is ready, and after the detached
      start but before readiness. Each time, the next invocation recovers
      only its own pod and keeps its volumes, though the normal stop path
      never ran.
    - Prove ownership before any cleanup, and show no volume is lost. A
      foreign pod of the same name is the negative control: it stays
      running and untouched.
    - Liveness is not readiness: a single unhealthy or unreadable result
      never tears down a running pod.
  - **Only if both hold:** the adapter, a unit whose main process is
    `svrngs-keeper serve-card <name>`.
    - Readiness gates publication, not restarts, so a slow or briefly
      unhealthy app is never torn down.
    - The container's own exit is the restart signal where the OS can see
      it.
    - One restart owner. No retry loop or migration policy in the adapter.

  Demo, only if both proofs hold: one named volume-only app (Actual, if
  its card fits) by the same link and a data marker, through a reboot.
  Otherwise sharing this run stays, and managed containers wait for the
  operator. The native baseline on Linux and macOS is unchanged.

- [ ] avail-run-page — depends [avail-run-linux] — ONE UNIT: spec §2.1–§2.3, §5 row 4.
  - The share line's two kinds; the form with its one command box and its
    prefill; the busy sentence; every state.
  - Two routes over `run.rs` `confirm`. The words come from `plan/words.rs`
    only.
  - **Not on the page this release** (QA's recommendation, adopted
    overnight 2026-11-23): no Restart (automatic recovery, plus the
    existing Stop and Start, cover it), and no Change how it runs. To
    change a command, stop running it with svrngs and Run again.

  Demo: the first demo again, from the page, in WebKit.
  - **Answer to the park** (keeper-lead, with QA oc-cruu42's allowance,
    2026-11-23). `5ab554fb` and `03f2fe26` stand.
    - **The whole row may add +120 counted and +85 JavaScript** since
      `a0698aa1`. That leaves +60 and +54 from `03f2fe26`. Over that, cut
      further, or park with measured deltas. Never borrow the Mac rows'
      margin.
    - **Don't take `/tmp/opencode/avail-run-page-measured.patch`
      wholesale.**
  - **The form is explicit and native.** No automatic model or command
    prefill on the page this release: no Prefill branch, no facts JSON
    shuttle, no card-to-command round trip, no `run_form` or `plan.rs`
    export.
    - The page's HTTP decodes only a typed `{name, port, folder,
      command}`.
    - It forwards those fixed fields through the same packaged keeper call
      as the CLI, with one argv-quoting parse in the keeper. The CLI
      already gets argv from its shell.
    - No raw argv and no arbitrary keeper verb from the page. Keep the
      local home route's user authorization, and its test refusing a
      request from elsewhere.
  - **States:**
    - project the keeper's existing `list --json` onto the existing
      running rows;
    - show only what the keeper observed;
    - keep Stop (running) apart from Unshare (sharing);
    - Start and Stop send closed actions to the keeper's owned verbs.
    - Deferred: a Remove control, richer status copy (a bad command gets
      the existing terminal recovery instruction), Restart and Change.
  - **Demo, in WebKit, in the gate container** (PROMPT.addendum, "From
    the toolbox, Podman and browsers are on the host"):
    - one native Run, verify, then the existing Share;
    - a busy or failed Run that publishes nothing;
    - one Start and Stop.

- [x] avail-run-stop-gap cc33fbc9c — depends [avail-run-containers] — ONE UNIT (keeper): a Stop during the wrapper's restart gap leaves the pod serving (QA oc-cruu42 at b94aa922).
  - `serve-card … --stop` with no MAINPID returns early. `KillMode=process`
    leaves the pod behind on purpose, so Stop pressed after the wrapper
    dies, and before systemd restarts it, would leave Actual serving after
    Sam stopped it. That breaks C3: stopped stays stopped.
  - Fix: an explicit Stop cleans the owned pod by its label and full ID
    (`runtime::stop_owned`), whether or not there is a MAINPID.
  - Plant: kill the wrapper, then Stop before the restart. Today the pod
    still answers (red). After the fix, the pod is gone, the unit stays
    stopped through a restart of the user manager, its volume and marker
    are kept, and a same-name foreign pod is untouched (green).
  - Size: keeper only. Counted: none expected.

- [ ] avail-run-mac — depends [avail-unmagic] — ONE UNIT (keeper): spec §5 row 5.
  - Fork commonwealth-ai `svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0`,
    lines 296–395, as `launchd.rs`: unchanged first, with ported-from lines.
    Then grow it to the job verbs, as `systemd.rs` is.
  - The busy check and the prefill by `lsof` and `ps`.

  Demo on beefymac (PROMPT.addendum, "beefymac"): llama-server on Metal,
  run by svrngs; kill -9; a stop that stays stopped. Reboots and sign-ins
  are HUMAN-release-hosts'.

- [ ] avail-run-mac-renew — depends [avail-run-mac, avail-publish] — ONE UNIT: spec §5 row 6.
  - A launch agent's grant: its label plus its plist's hash, judged by
    lineage.
  - No Podman-machine container witness: managed Mac containers are
    deferred (see avail-run-mac-containers). A Mac container stays
    shareable while it runs, as today.

  check: B2's Mac twins, each planted, on beefymac.

- [x] avail-run-mac-containers — depends [avail-run-mac-renew] — DEFERRED, not built (QA oc-cruu42's recommendation, adopted overnight 2026-11-23, for the operator to reverse). Managed Mac containers add the Podman machine's lifecycle, and an endpoint-identity proof beyond native macOS. A Mac container stays shareable while it runs. The native baseline on Linux and macOS is unchanged. No commit.

- [ ] avail-ship — depends [avail-run-page, avail-run-containers, avail-run-mac-renew] — ONE UNIT: spec §5 row 8.
  - `build-release.sh`, `install.sh` (always) and the Arch package, as
    Give's lines are.
  - Uninstall says what svrngs still runs.
  - The guides: models.md, lend.md and troubleshooting.md, in spec §2's
    words.

  Demo: a fresh VM, installed by the line.

- [ ] avail-rehearse — depends [avail-ship] — ONE UNIT, evidence: spec §5 row 9, its strangers.
  - xenia on the built page: DeepSeek Flash at `--variant max`, the world
    being svrn-docs `play-lend-give/world/`.
  - The plant runs first: the two line kinds' words swapped. Can a stranger
    tell "shared while you run it" from "run by svrngs"?
  - Recorded in `demo/runs/available/strangers-<date>/`.

- [ ] avail-proof — depends [avail-ship, avail-rehearse] — ONE UNIT, evidence: spec §5 row 9, its packaged proofs, in the VM. In `demo/runs/available/<date>/`, predictions first, each case given one of the four verdicts:
  - after a reboot, Alex's old link reaches the same app and data, a marker
    written before the reboot and read after;
  - stop sharing ends Alex's open stream within 2 s, with a test that
    plants the withdrawal itself (`tests/door/live.rs`);
  - a concurrent withdrawal; a failed save; the busy port.

- [ ] HUMAN-release-hosts — depends [avail-proof] — the operator. The
  release waits on this row (the operator, 2026-11-22: "release only after
  packaged proofs"):
  - beefymac with the install line, including three reboots and sign-ins
    with Alex's link and the data marker;
  - a real Fedora laptop: a GPU model before sign-in, a passphrase disk
    (`AfterUnlock`), and the login screen's idle suspend;
  - the people sessions (spec §5 R9, "People").

  No worker ticks this row, installs on the operator's own svrngs, or
  touches a real host but beefymac's scratch folders.

- [ ] REVIEW-audit-available — depends [avail-proof] — the unchanged full
  `scripts/check.sh` in svrngs-gate, with the main `.git` mounted
  (PROMPT.addendum). It also checks:
  - `scripts/ports.py` lists every ported-from line;
  - CSP and the trust floor are unchanged;
  - the counts are within CHARTER's release envelope.

## Not in this release

Spec §5, "Out": starting on request (N1), running while this computer is
off (N2), another account's service (N3), UDP coming back by itself,
compose projects with their own networks, before login on a Mac, and
Windows.
