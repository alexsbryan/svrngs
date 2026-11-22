# svrngs-keeper: try it however you like, then keep it running here

## Direction and authorization

The operator, 2026-11-22, in one conversation:
- "people doing all the stuff they do to run programs and test them out
  … when you want to move to production you just do something simple and
  there's a svrngs production local hosting setup that will just run your
  program no fuss, keep it alive, restart it on boot (all configurable)";
- "the shared runtime should be a clean package", named svrngs-keeper;
- "Let's make a branch and start the work … lean and scrappy … just
  document the user flow";
- after the estimates (`STATE.md`, "Estimates"): "Eudoxia owns authority.
  Those resources work for me -- let's kick off work".

That last message is this queue's launch directive, for its rows and for
this envelope.

The design is in svrn-docs `rings/current/`:
- `HOSTING_AND_GIVE_BOUNDARY.md`: the owners;
- `LOCAL_HOSTING.md`: the scene;
- `KEEP_SHARING_ON_RESTART.md`: the renewal rule and who it defends
  against;
- `KEEP_SHARING_BUILD.md`: reuse and budget.

The evidence is `local-hosting-spike/`: five no-login VM boots, the Mac
runs, and the four ways to start llama-server, read.

In short:
1. **svrngs-keeper is Give's local runtime as a package of its own.** It
   knows no group and links no svrngs crate. Give depends on it, and
   svrngs calls its program.
2. **The OS keeps a card alive.** A systemd user unit, with linger for
   boot. The keeper writes the unit and supervises nothing.
3. **Promoting is one command,** `svrngs-keeper keep <port>`:
   - a known program's card is composed from what it reports;
   - anything else is copied, shown, verified, and rolled back if it fails.
4. **svrngs names the job, and renews a kept job's lend.** Renewal is an
   approval in which the job is what was chosen, made by a grant instead
   of a click. The decision is eudoxia's `binding::approve`, with a new
   `Chosen::Cgroups`: "Eudoxia owns authority".

The demo this moves: Sam runs llama-server in a terminal, lends it, and
promotes it with `svrngs-keeper keep 8101`. Then he reboots three times,
with nobody logged in, and Alex's link opens each time with Sam typing
nothing. A stop stays stopped, and **Stop lending** stays stopped.

## Budget

At `e5d9813b` (`scripts/lean_check.py`): core 3,465 of 3,539, all code but
JavaScript 24,517 of 24,517, JavaScript 8,693 of 8,693. `apps/` and tests
are uncounted (lean_check.py:973).

- **svrngs-keeper lives in `apps/`.** Its rows aren't held to the caps.
  Keep it lean all the same: what Give already does is moved, not
  rewritten, and what commonwealth-ai has is forked.
- **svrngs's half** (svrngs-renew) is estimated at +180 Rust and +30 JS.
  - The operator approved that envelope ("Those resources work for me",
    answering an estimate that named a raise of about +120–150 Rust and
    +25–30 JS after an economizing pass).
  - The economizing pass (svrngs-economize) comes first. Its preserving
    cuts are their own commits, and they ratchet the caps down.
  - svrngs-renew then raises the caps by what it still lacks, never past
    +180 Rust and +30 JS over the post-pass counts. Its commit gives a
    `Size:` paragraph: the counts, what the pass cut, and why the rest
    can't be found in the code there is (AGENTS.md "Done").
  - Past that envelope, stop with `needs-human`.
- **Tests aren't budgeted** (the operator, 2026-11-19). Write what a
  change needs.
- **No new trust-floor line, and no CSP change.** The keeper's program
  isn't svrngs's listener and opens none.

## Eudoxia

`Chosen::Cgroups` is eudoxia's.
- The work is in `~/dev/eudoxia`, cloned at the rev svrngs pins
  (`a0d3b0e`), on branch `keeper-cgroups`, under eudoxia's own AGENTS.md
  and its own checks.
- svrngs uses it on this branch through a
  `[patch."https://github.com/alexsbryan/eudoxia"]` to that path.
- Before the audit, the operator pushes the eudoxia commit
  (HUMAN-eudoxia-push). A worker then re-pins svrngs to the pushed rev and
  removes the patch. No worker pushes.

## Reuse

- **Inventory first** (AGENTS 11):
  - Give's runtime is moved (keeper-move, done).
  - commonwealth-ai `shared/crates/host-kit/src/service.rs` and its tests
    @8ed8d99b0 are forked, unchanged first, with ported-from lines.
- **The spike's units are the reference settings,** proved through five
  boots: `Restart=on-failure`, `RestartSec=2`, `StartLimitBurst=4` in 60 s,
  `SuccessExitStatus=143`, `WantedBy=default.target`, and loopback ports.
- **The spike's probe** is the prototype of the renewal rule (svrn-docs
  `local-hosting-spike/probe/src/main.rs` `renew`): ported, not linked.
