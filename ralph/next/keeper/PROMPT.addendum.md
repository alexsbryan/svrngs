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
- Tests that need Podman go in the `svrngs-gate` container, exactly as
  `scripts/gate/README.md` gives it, under `flock /tmp/svrngs-gate.lock`.
- `systemctl --user`, `loginctl` and real units need the host. A unit's
  host checks use units named `svrngs-keeper-test-*` and remove them.
  Never touch the operator's own units or the installed svrngs.

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
