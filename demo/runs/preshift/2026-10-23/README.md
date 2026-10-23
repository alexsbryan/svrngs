# The evening review, tried on svrngs queues — 2026-10-23

The operator, 2026-10-23: "A pre-shift sweep and evening review, tried on one
svrngs queue first." A ralph row that finds an operator's gate at night parks
until morning. The sweep does not answer gates. It names, in daylight, the
gates tonight's ready rows would reach.

    python3 demo/runs/preshift/2026-10-23/preshift.py <queue> [--json]
    python3 demo/runs/preshift/2026-10-23/test_preshift.py

## Why it is here and not in scripts/

`scripts/lean_check.py` counts `scripts/*.py` in the repository's budget, and
the repository is at 17,634 of 17,635 (`data/` runs below; lean_check exit 0
with these files in the tree). `scripts/preshift.py` would break `check.sh`,
and a raise is the operator's (AGENTS.md "Done"). Here, it is this run's
driver (lean_check.py's docstring: demo/runs/ apart). Moving it to
`scripts/` costs 592 counted lines by lean_check's rule (docstrings count): the operator's call. Moved, its
`--root` default becomes `parents[1]` (preshift.py, `main`).

## What it reads

- Rows: ralph's grammar, copied from commonwealth-ai `scripts/ralph.py:57-60`.
  Ready means `[ ]` or `[~]` with every depend `[x]` (`State.current`).
  Parked means `ctl/parked/<id>.md` (`parked_ids`).
- Fences: `## Leave for the operator` and `## What this queue does not
  decide` bullets, `**Leave for the operator** (...): a; b; c.` paragraphs,
  and `## The budget`. A fence item gives terms: its backticked names, its
  bold lead, and comma phrases of up to three words. An item with none is
  listed as "not checked".
- Ceiling: the fence's budget item. It comes in five shapes, `data/census.txt`
  over all 33 queues, 32 with a CHARTER.md (ceiling has none):
  - relative: guests at the park, "+150 repo, +60 core", and leave. It is
    resolved over lean_check's BUDGET at the done economize row's commit.
  - absolute: "past 16,925 for the repository or 3,076 for svrngs-core".
  - the caps as they stand: "past the caps", "raising the size budget",
    models' "Zero".
  - "past the ceilings": canon, groups and v1ui. Resolved through the
    charter's decided `**The budget.**` bullet.
  - unparsed: arch's per-row "+60 over the count it started at", and
    capability's "one ceiling for the bundle, asked after the economizing
    pass". grapevine and v1patch have no budget item in a fence.
  - bash's and v1's "— but for bash-devices, which may take svrngs-core to
    3,080" is said, not applied.
- Headroom: a charter number is read as a lean_check cap, so left = ceiling
  - 1 - count. The test pins this against the park's own "24 left, none in
  svrngs-core".
- Premises: each named path exists, or its parent does for one the row
  makes; each cited line is inside its file; each command's script exists;
  no command matches the deny list in queue.toml's `settings`.
- Present: `HUMAN-` rows, and "live demo", "with people", "operator runs" and
  the like, unless followed by "never-ran" or "waits".

## The failing input: guests-link, 2026-10-19

The tree is 65c7fbb3, the commit before the park a3212c59. It is from `git
archive`, with lean_check's reading there kept in
`fixtures/lean-65c7fbb3.txt`. The run is `data/guests-at-65c7fbb3.md`, exit
4. The sweep reads the ceiling as 16,728 and 3,069 (16,578 + 150 and
3,009 + 60 over 93d17fe79's BUDGET). That leaves 24 in the repository and 0
in svrngs-core, which is what NEEDS_HUMAN.md says. guests-link stated no
estimate (STATE.md:67 at 65c7fbb3). So the sweep flags it as could not
judge: "no estimate stated; 24 left … 0 left … any line in svrngs-core
exceeds it". It does not say "will park". The row was knowable as a
question ("how big?"), not as a park. With the park's own measure planted
in the row ("about 118 repository lines, 7 of them svrngs-core lines"),
the sweep puts it under "ask now" and exits 3 (test_preshift.py).

## Today's trial

capability and stall each have one ready row. capability has the most open
rows (11), and it runs after stall. stall's row is in flight under a running
loop now. guests is the queue the park came from.

| queue | present | ask now | could not judge | clear | exit |
|---|---:|---:|---:|---:|---:|
| capability | 0 | 1 | 1 (the queue's ceiling) | 0 | 3 |
| stall | 0 | 0 | 1 | 0 | 4 |
| guests (today, done) | 0 | 0 | 0 | 0 (no ready rows) | 0 |
| guests at 65c7fbb3 | 0 | 0 | 1 | 0 | 4 |

- capability-economize plans its own stop: "then NEEDS_HUMAN.md asking the
  operator for one ceiling for the whole bundle, and stop" (STATE.md:46).
  The charter's ceiling is not yet a number (CHARTER.md:43).
- streaming-stall: there is 0 left under either cap, so any counted line
  (its debug traces in crates/ included) is over. It stated no estimate.
  It parks if the cause is outside svrngs (STATE.md:12).
- guests today: the queue is done. Its fence's 16,925 and 3,076 now sit
  710 and 74 under the count, because later queues raised the caps. The
  ceiling is stale, and no row is ready.

## What a static sweep cannot see

- The size of what a row will write. Only a stated estimate is compared,
  and guests-link's own text gave none. Its "(Changes: the node, the home
  page, the phone's page.)" spent 7 svrngs-core lines, so a row's scope
  words do not bound where its lines land.
- Judgment fences: "anything whose evidence you could not reproduce", "a
  new office or policy". They are listed as not checked.
- Term matching is by words, with a crude negation window. It can miss a
  paraphrase. stall's "the cause is outside svrngs" does not match the
  fence term "cause outside svrngs", and only the row's own conditional
  NEEDS_HUMAN.md caught it.
- Whether a conditional park's condition holds.
- Tonight's state. A row ahead of it may change the counts first. Rows are
  judged one by one against the counts now, and only the ready ones.
