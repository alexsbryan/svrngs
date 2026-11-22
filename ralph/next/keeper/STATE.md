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
To keep it running whenever this computer is on: svrngs-keeper keep 8101
```

**Promote it.** One command, one yes:

```
$ svrngs-keeper keep 8101
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

- [ ] keeper-units — depends [keeper-move] — ONE UNIT:
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

- [ ] keeper-promote — depends [keeper-units] — ONE UNIT:
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
    port is lent.

  read: the spike's `cmd` reads (four ways to start llama-server).
  check: on this host:
  - a relative model path and a `~` path, each composed from `/props`;
  - a wrapper script, composed;
  - a renamed python process, copied with its name;
  - a planted copy that needs an environment variable: rolled back, with
    the original answering again.

- [ ] svrngs-economize — depends [] — ONE UNIT, behaviour kept: the
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

- [ ] eudoxia-cgroups — depends [] — ONE UNIT, in `~/dev/eudoxia` on
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

- [ ] svrngs-renew — depends [svrngs-economize, eudoxia-cgroups] — ONE
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

- [ ] keeper-demo — depends [keeper-promote, svrngs-renew] — ONE UNIT,
  evidence:
  - Sam's scene, end to end, in the spike's Fedora VM. Sam and Alex are
    two accounts, each with a node, met at a local `svrngs relay`.
  - The steps: llama-server in a terminal, `svrngs share`,
    `svrngs-keeper keep 8101`, then three no-login reboots, a kill -9,
    `svrngs unshare` and a reboot.
  - Alex's request is timed each time, and Sam typed nothing after the
    promotion.
  - Recorded in `demo/runs/keeper/<date>/`.

  check: every case passed, failed, could-not-judge or never-ran, named
  in its run.

- [ ] HUMAN-eudoxia-push — depends [svrngs-renew] — the operator:
  - Push eudoxia's `keeper-cgroups` commit, then say its rev.
  - A worker then re-pins `eudoxia-community` in
    crates/svrngs-node/Cargo.toml to that rev and removes the `[patch]`,
    in its own commit. The gate fetches eudoxia from its public repository
    (scripts/gate/README.md), so the audit needs it pushed.
  - No worker ticks this row or pushes.

- [ ] REVIEW-audit-keeper — depends [keeper-demo, HUMAN-eudoxia-push] — the unchanged full
  `scripts/check.sh` in the svrngs-gate container, exit 0. The ported-from
  lines are listed by `scripts/ports.py`. CSP and the trust floor are
  unchanged, and the counts are within each allowance.

## After the demo, not now

- The page: **Running here for good**, and promoting from a row.
- Give's image cards rendered to Quadlets, so a kept Give app survives
  its keeper's reboot.
- The Mac, forking commonwealth-ai `sovereign-service`'s LaunchAgent half,
  and renewal by launchd job.
- The promise words, K4.
- The home-access Flatpak scope, which is the operator's.
