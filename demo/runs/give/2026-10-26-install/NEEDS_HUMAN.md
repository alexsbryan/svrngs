# give-install: a raise of the repository cap, 19,109 to 19,121

**The ask:** land the raise below, from the operator's seat (a worker's edit
of `BUDGET` is refused as a CI bypass, give-tcp-1.out). give-install is past
the ceiling the operator set for give-token through give-play, so no range
pre-approves it.

## Why the lines

`web/install.sh` and `scripts/build-release.sh` are counted in the
repository ledger (`scripts/lean_check.py` CODE takes `.sh`). The row asks
both: install.sh "offers Give's keeper program as an option", from "the page
host beside svrngs, with its checksum in SHA256SUMS", and build-release.sh
builds it.

| Commit | What | `.` by lean_check |
|---|---|---|
| 2ce1fb4c | the row opened | 19,107 of 19,109 |
| 60565068 | the economizing pass over install.sh, behaviour kept: four statements in fewer lines (`demo/runs/economize/2026-10-26-give-install/SURVEY.md`) | -7, 19,100 |
| 2d68d696 | `--give`: fetching and checking, placing, stopping and keeping at login made one function each and run for svrngs and Give; Give's agent's PATH and its wait as it stops; build-release.sh builds `-p give` and sums both | +20, 19,120 |
| 2e6ca3b6, 8402864a | Give's page and its program (`apps/`, not counted) | 0 |

Net +13 over 19,107. What the pass could not find is in the survey: the
rest of install.sh says each step once, and the relay's printed unit
differs from the user unit in four lines. The change shares rather than
copies; copying the steps for Give would be about 30 lines.

## The four ledgers

- **core** 0 (3,241 of 3,242)
- **library** +13 net (+20 after the pass's -7)
- **apps/** +101 by lean_check's rule (machine.rs, keep.rs, podman.rs,
  shelf.rs, page/index.js), and tests
- **surface** the install line gains one option, `--give`; an app author's
  surface is unchanged

## The ready diff

```diff
--- a/scripts/lean_check.py
+++ b/scripts/lean_check.py
@@
 # The page after joining (the operator, 2026-10-26: the shelf unticked and,
 # just joined, folded; "What acts as you" folded; the group's things
 # first), after its own economizing pass, the page's heading set by one
 # assignment (-3): +1, at 19,108, so the cap lowered to one past it.
-BUDGET = {"crates/svrngs-core": 3242, ".": 19109}
+# give-install, after its own economizing pass over install.sh (-7,
+# demo/runs/economize/2026-10-26-give-install/SURVEY.md, 60565068): Give's
+# program as install.sh's --give, fetched, checked, placed and kept at
+# login by the functions svrngs's own steps became, and built beside
+# svrngs in build-release.sh (2d68d696): +13 net and 0, at 19,120
+# (demo/runs/give/2026-10-26-install/NEEDS_HUMAN.md).
+BUDGET = {"crates/svrngs-core": 3242, ".": 19121}
```

## Also for the operator to know

- **The line names the published page.** Give's page reads only its own
  origin, so it cannot read the page this computer's svrngs came from (the
  data dir's `origin`, which Give does not reach, CHARTER fence 4). It
  gives `curl -fsSL https://group.svrnme.sh/install.sh | sh -s --
  'https://group.svrnme.sh/' --give`, README.md:22's line with `--give`.
  A computer of a group on another page host would get svrngs and Give
  from the published page by it, and install.sh keeps that page in the
  data dir.
- **Rosetta, never-ran here:** making a machine with Rosetta on, and a
  machine with it off. This Mac's one machine runs with Rosetta on, and on
  applehv a second would stop it. The Podman Desktop setting the words
  name ("Turn on Rosetta in Podman Desktop’s settings, then start Podman
  again") was not opened here.
- **The gate:** `scripts/check.sh` once, at the close; its size step fails
  until the raise lands (the await log keeps the run).

## The director's verdict, 2026-10-26

Reproduced: `python3 scripts/lean_check.py` in a detached worktree at each
commit gives `.` 19,107 at 2ce1fb4c, 19,100 at 60565068 (the pass, -7) and
19,120 at 2d68d696 and at HEAD 34c80912; core 3,241 of 3,242. `cargo fmt
--all --check` is clean at HEAD, so the count is the one the raise lands on.

Why this is the operator's: the ceiling (STATE.md give-economize, THE
OPERATOR DECIDED 1) covers give-token through give-play, and give-install
names no range of its own. CHARTER, "Leave for the operator": a row past its
ceiling or its estimate's range. AGENTS.md "Done": a size-budget raise is the
operator's.

The options:

1. **Land the raise, 19,109 to 19,121** (the ready diff above). Cost: +12 on
   the cap; 19,121 is still under the ceiling's 19,233.
2. **Land it and widen the ceiling's rows** to give-install and give-docs,
   inside 19,233. Cost: the same +12; give-docs (documents, expected 0) then
   needs no park on size.
3. **No `--give` in install.sh**: Give's program a separate download a person
   fetches by hand. Cost: reverts 2d68d696 and 2e6ca3b6 and saves 20 lines,
   but the page asks a lightly technical person for a second step, against
   the charter's "clean and fluent interface".

**Recommendation: 1.** The pass is on record (SURVEY.md), the growth shares
svrngs's steps rather than copying them, and the total stays inside what the
operator already allowed the give work. Then the row's one `scripts/check.sh`
closes it.
