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

- [ ] avail-terminal — depends [avail-plan, avail-keeper-facts, avail-publish] — ONE UNIT: spec §§2.1–2.2, §3.2; row R1, its code.
  - `plan/run.rs`, the executor at the edge: it replays at confirm, gives
    `ReviewAgain` with no side effects, and feeds each owner's answer back
    as a `Done` event.
  - `svrngs share` through the plan.

  check, planted first: partial success. A failed write leaves "Running
  here; not shared", and the next review shares it with no restart.

- [ ] avail-page — depends [avail-terminal] — ONE UNIT:
  spec §2.1, §2.2, §3.7 O3, O4; row R4.
  - The page renders the review and the result svrngs sends, through two
    routes. No string lives in the page.

  check:
  - O3, a lean rule: svrngs only reads jobs, and one module runs the
    keeper;
  - O4, `vocab_check.py`: no mechanism word on the page;
  - the first demo again, from the page, in WebKit.

- [ ] avail-containers — depends [avail-keeper-facts] — ONE UNIT (keeper):
  spec §5 row R5.
  - A rootless Podman container or pod becomes a Quadlet, verified by the
    unit's cgroup.
  - First, a plant: today a container's port copies pasta
    (`promote.rs`, no container case).

  Demo: Actual, and an Immich-shaped pod, through a reboot in the VM.

- [ ] avail-mac-native — depends [avail-keeper-facts] — ONE UNIT (keeper):
  spec §5 row R6.
  - Fork commonwealth-ai `svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0`,
    its LaunchAgent half, as `launchd.rs`: unchanged first, with ported-from
    lines.
  - Then reads by `lsof` and `ps`, and §3.4 on the Mac.

  Demo on beefymac (PROMPT.addendum, "beefymac"): llama-server on Metal,
  kill -9, and `launchctl kickstart -k` of its agent. The reboots and
  sign-ins are the operator's, in HUMAN-release-hosts.

- [ ] avail-mac-renew — depends [avail-mac-native, avail-publish] — ONE
  UNIT: spec §3.3, §3.7 B2; row R7.
  - A LaunchAgent's label plus its plist's hash, judged by lineage.
  - A Podman-machine container by name and image digest, with the engine
    read before and after the dial.
  - Whether a sandboxed Mac app can reach Podman's socket is answered
    first; until it is, a Mac container is `ThisRun`.

  check: B2's Mac twins, each planted, and Alex's link after the agent's
  restart and a kill -9 on beefymac. After a reboot, it is
  HUMAN-release-hosts'.

- [ ] avail-mac-containers — depends [avail-mac-renew] — ONE UNIT (keeper):
  spec §5 row R8.
  - Spike M0 on a Podman machine's restart policy, then the keeper's Mac
    containers.

  Demo: Actual on beefymac.

- [ ] avail-ship — depends [avail-page, avail-containers, avail-mac-containers] — ONE UNIT: shipping, after the code (the operator, 2026-11-22: "a lot of coding ahead of us before needing to talk about fit and finish").
  - svrngs-keeper built by `scripts/build-release.sh`, always installed by
    `web/install.sh`, and in `packaging/arch`.
  - The guides: models.md, lend.md and troubleshooting.md, in spec §2's
    words.

  check: `scripts/release.sh build` lists the keeper's archives beside
  svrngs's. install.sh, run on a scratch HOME, puts both on PATH. The Arch
  package's file list names it.

- [ ] avail-rehearse — depends [avail-ship] — ONE UNIT, evidence: spec §5 rows R0 and R9's strangers, on the built page.
  - xenia strangers (DeepSeek Flash, `--variant max`) on the built home
    page, served by xenia's world (svrn-docs `play-lend-give/world/`). The
    availability rehearsal (`prototype/availability/`, not ours) is the
    reference to compare against.
  - The plant runs first: the two choices' labels swapped. Then Hana ("you
    restart tonight: make sure Cy can use coder tomorrow") and Ivo, two runs
    each.
  - Recorded in this repository's `demo/runs/available/strangers-<date>/`,
    the way PLAY_LEND_GIVE_UX §4 records them. Workers on opencode can't
    edit `~/dev/svrn-docs` (`.opencode/opencode.json`), and the svrn-docs
    folder `play-lend-give/runs/availability-2026-11-22/` is an earlier
    session's start, left as it is.

  read: PLAY_LEND_GIVE_UX §4; the rehearsal's README.
  check: the plant is told apart from the real labels, or the round says
  could-not-judge; each count is reported as a rehearsal, never a people
  result.

- [ ] avail-proof — depends [avail-ship, avail-rehearse] — ONE UNIT, evidence: spec §5 R9, the parts that are
  automatic. In `demo/runs/available/<date>/`, predictions first, each case
  given one of the four verdicts:
  - the gated cases: a takeover interrupted at each stage, a VM power-cut
    between stop and start among them; a withdrawal concurrent with a
    confirmation; a failed publication; a stale confirmation after a
    withdrawal;
  - the packaged path in the VM: the Arch package, and the install line on
    Fedora Workstation's image. After a reboot, Alex's old link reaches the
    same app and the same data, a marker written before the reboot and read
    after. Stop sharing leaves Sam's app answering on localhost, and Alex's
    open stream ends within 2 s, with a test that plants the withdrawal
    itself (`tests/door/live.rs`, spec §3.5);
  - Sam's scene: three no-login reboots in the spike's VM
    (`demo/runs/keeper/2026-11-22/`'s driver), with the page and the
    terminal.

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
