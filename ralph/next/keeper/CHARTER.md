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
- **svrngs's half** (svrngs-renew) is estimated at +180 Rust and +30 JS,
  measured from main's counts (see "Alongside app-state").
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

## Alongside app-state (PR 7)

`app-state` (https://github.com/alexsbryan/svrngs/pull/7, "A shared SQLite
database for each carried app") is worked from the other machine in the
commons (peer `cbc855627108`). It branched from the same `e5d9813b`, and it
is a draft until all of its parts are done. Whichever branch lands second
rebases onto main (linear history, never a merge commit). Until then, stay
out of its way:

- **Its budget is its own.** PLAN §1 on app-state records the operator's
  pool for its whole build: +1,450 of all code but JavaScript and +80 of
  JavaScript (2026-11-22). It has already raised the non-JS cap to 25,735
  (252573fa). Keeper's envelope is a separate approval. When the branches
  meet, each cap in `scripts/lean_check.py` is main's plus both branches'
  deltas, and each branch's `Size:` paragraph stands for its own delta.
- **Its economizing pass is in progress,** in `lend.rs` (`stamp`, through
  `lend::all`), `door.rs` (`lend_for`) and the home page's JavaScript.
  svrngs-economize cuts only in files app-state hasn't changed. Read
  `git fetch origin app-state && git diff --name-only main...origin/app-state`
  when the row starts, and leave out every file it names.
- **svrngs-renew's hunks:**
  - in `lend.rs`, only `Terms`, `Hold` and `Answer`;
  - the door's branch in `door/bound.rs`, which app-state doesn't touch.

  Anything else it needs in a file app-state changed goes in a hunk of its
  own, cited in its commit, so the rebase is mechanical.
- **The shared files, resolved by their meaning:**
  - `Cargo.toml` members and `scripts/tests/test_touched.py`'s package list
    take both crates, in sorted order: `-p svrngs-core -p svrngs-db
    -p svrngs-keeper -p svrngs-node`.
  - Each `Cargo.lock` is regenerated, not merged by hand. Both branches
    already moved `url` to 2.5.9.
  - `docs/PLAN.md` §1 and `ARCHITECTURE.md` keep both entries.
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

## The release: available by default (2026-11-22)

The operator, in order:
- "'Linux, command line, native programs' -- that's clearly not anywhere
  close to a complete release";
- "Keep vs Give is an implementation detail … End users have clean
  interface. We should make both hold strongly";
- "make a deterministic deployment plan that we can execute on our service
  topology", which "might have to be a folded function";
- a reviewer's production handoff, to incorporate;
- "use svrngs --> the program that offers this service for other
  programs";
- "Size raise fine, release only after packaged proofs".

The design is svrn-docs `rings/current/KEEP_IT_RUNNING_RELEASE.md` at
`4f77680`. Its §4 holds the settled decisions, each with its grounds, and
its §5 the release line. Where a row and the spec disagree, the spec wins:
correct the row in a commit to this file's STATE.md, citing it.

**The envelope.** The operator approved the raise as an envelope over the
counts after avail-economize's cuts (24,636 non-JS, 8,650 JS): first at most
+435 of all code but JavaScript and +30 of JavaScript (spec §6). Then, on
2026-11-22, "Ok raise it", at most **+885 of all code but JavaScript, a
ceiling of 25,521**, with JavaScript still +30 (8,680). That came after
avail-plan landed +304 and avail-publish +124 against estimates of ~+140 and
~+40, which left 7 lines before the terminal row (demo/runs/economize/
2026-11-22-plan/SURVEY.md). Each row spends only as far as it
lands, with a `Size:` paragraph. Past the envelope, stop with
`needs-human`. The counts at `04e5873b`, rebased on main `789707dc`: core
3,471 of 3,539, all code but JavaScript 24,655 of 24,656, JavaScript 8,668
of 8,689.

**The release gate.** No release before HUMAN-release-hosts: the packaged
proofs on beefymac and a real Fedora laptop, and the people sessions. The
release itself is the operator's (PLAN §2).

## Run it with svrngs (2026-11-22, the design that stands)

After the build ballooned, the operator ruled from first principles:
- "I prefer bulletproof systems that work without flash over perfect bells
  and whistles", which dropped the live takeover;
- "If we just stop trying to do magic, what's the most industry standard
  devex expectation", then "Yes this is all I wanted. It just has to be
  clear in the UI / UX for the lightly technical user, but otherwise
  that's the bulletproof system".

So the release has two explicit steps, as pm2, `brew services` and
systemd have:
1. **Run it with svrngs,** with a command the person gives or confirms.
2. **Share it.**

svrngs never adopts, stops or signals a program it didn't start. A busy
port is refused in plain words. The design is svrn-docs
`rings/current/KEEP_IT_RUNNING_RELEASE.md` at `0080f3f`, which supersedes
the earlier revisions' takeover and gate. A partial avail-containers
session on the takeover design is in `git stash@{0}`; avail-run-containers
reuses its `containers.rs`.

**The envelope now.** The operator, 2026-11-22: "Go", to the spec's ask of
about +360 non-JS and +90 JS for its rows 2–8, after avail-unmagic's
removal (about −155). The ceilings are:
- **all code but JavaScript, 25,727:** the branch's whole growth since
  e5d9813b held to about +1,210;
- **JavaScript, 8,741:** about +48 since the branch.

Each row spends only as far as it lands, with its `Size:` paragraph. A row
that would cross a ceiling stops with `needs-human`; it never raises past
it.
