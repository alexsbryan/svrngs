<!-- section: keeper after=intro -->
Read this queue's CHARTER.md and STATE.md, and the svrn-docs designs it
names, before any work. The operator decided the design and launched this
queue. Keep marching through the assigned unit; another step boundary is not
a new approval gate. A cap decision is the operator's, under CHARTER's
envelope, not a reason for a workaround.

**No check.sh before the very end** (the operator's rule for queues,
2026-11-18: "let's just save that for the very end").
- Before REVIEW-audit-keeper, run no `scripts/check.sh` in any form.
- Close each unit on its row's own focused checks.
- The commit hook's `check.sh --fast` is the only check.sh a unit meets.

**Where things run on this computer:**
- Builds, tests and commits go in the toolbox:
  `toolbox run -c sovereign-vulkan bash -c "cd ~/dev/svrngs-keeper && …"`,
  and commits with `git commit -F <file>`.
- Thread-heavy suites (`-p svrngs`'s `verbs` and `scenes`, svrngs-node's
  door and ring tests) run with `TOKIO_WORKER_THREADS=4`. Each node in a test
  starts runtimes sized to this computer's 32 cores. svrngs-renew's `verbs`
  run held 1,880 threads and panicked with "OS can't spawn worker thread …
  os error 11". The toolbox's task limit is 8,192 since then (it was
  2,048).
- Tests that need Podman go in the `svrngs-gate` container, exactly as
  `scripts/gate/README.md` gives it, under `flock /tmp/svrngs-gate.lock`.
- `systemctl --user`, `loginctl` and real units need the host. A unit's
  host checks use units named `svrngs-keeper-test-*` and remove them.
  Never touch the operator's own units or the installed svrngs.

**The full gate from this worktree.** This checkout is a git worktree,
whose `.git` points into `/home/alexbryan/dev/svrngs/.git`. In the gate's
`podman run`, add `-v /home/alexbryan/dev/svrngs/.git:/home/alexbryan/dev/svrngs/.git`,
or git in the container finds no repository, and lean_check and trust_check
fail for the mount alone (commons lesson d789eff5;
`/tmp/keeper-audit/gate.sh` is a working copy).

**beefymac** (`ssh beefymac`, the operator's Mac; approved for this
queue's Mac rows):
- Only a scratch folder, `~/svrngs-keeper-scratch`.
- Only launch agents and containers named `svrngs-keeper-test-*`, removed
  at the end of the unit, with the commit saying what was removed.
- Never the operator's installed svrngs, its groups, or `~/Library/LaunchAgents`
  entries not of this queue's naming.
- No reboot of beefymac without the operator: a "reboot" demo there is
  recorded by the operator in HUMAN-release-hosts, and a worker uses
  `launchctl` restarts and sign-out/sign-in only if the operator has said
  so in this file.

**xenia** runs on DeepSeek Flash at `--variant max` (AGENTS.md, "Ralph
model effort"), as PLAY_LEND_GIVE_UX §4's runs were.

**Never touch the operator's real svrngs groups or lends.** svrngs runs for
a check only as a scratch node: `SVRNGS_DIR` and `HOME` in a scratch
folder, as svrn-docs LEND_SANDBOX §6 B5 does.

**Decisions:**
- **A row's allowance is its estimate; the caps decide** (the operator,
  2026-11-19). For svrngs's half, the envelope is CHARTER's.
- **An advisory that blocks commits is judged by reachability** (675b0cc1).
  - If none of its affected functions is called, add a `deny.toml`
    exception whose reason gives that evidence, with a review date two
    weeks out.
  - If one is called, upgrade when a fix exists, as 4e7bb257 did for url.
    Otherwise stop.
