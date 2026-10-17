**Answered by the operator, 2026-10-16: "Final from this computer, through V1".** groups-keys closes as surveyed; a key per group on computers goes to DESIGN §9 with this survey as its evidence.

# groups-keys — the budget, for the operator

**The question:** a key per group on a computer, shape C, needs about
220–330 repository lines, and the queue's +200 has 41 left. Do you raise the
ceiling for it, or does "final from this computer" stay through V1?

## The evidence

- The survey: `SURVEY.md` beside this file (commit 64ed421b). It lays out
  five shapes, each with its cost and the file:line it touches.
- The budget left, `python3 scripts/lean_check.py` at 64ed421b:

  ```
  crates/svrngs-core     2992 / 2993 lines
  .                     16295 / 16296 lines
  ```

  The ceilings are your +40 and +200 over groups-economize's caps
  (2,953 and 16,136, `df996e3b`): 2,993 and 16,336. So 1 svrngs-core line
  and 41 repository lines are left. groups-tabs used 22 lines and
  groups-leave 137.

- The shapes, from the survey's §4:

  | Shape | svrngs-core | repository | meets the bar |
  |---|---|---|---|
  | A, a signing key derived per group | B's or C's | B's or C's +10 | only with a kept epoch: a rejoin derives the cut key again |
  | B, an `Admit` naming both keys | 30–45 | 150–225 | yes, with a new line version |
  | C, an endpoint per key the computer holds | 0 | 220–330 | yes |
  | D, the cut key admitted again | 10–20 | 20–40 | no: the row's own plant |
  | E, a second svrngs on the computer | 0 | ~10 | no: two home pages |

## What I recommend

**C, with the repository's ceiling raised by about 330 (to 16,666), and
svrngs-core's left as it is.**

- It changes no line of svrngs-core and no byte of the record.
- Every existing group keeps its members' keys. Only a group this computer
  was cut from ever gets a second key, so nothing a computer holds is
  migrated.
- "One key signs and dials" stays true key by key, as it already is on a
  phone (`web/store.js`, "One key per person per group").
- The cost at run time is one more relay connection per group a person
  rejoined.

The estimate comes from the sites, not a spike. If the raise is given, (b)
starts with C's runtime part (a signer per group) and stops again if C
passes the ceiling.

**The other way: keep "final from this computer" through V1.** Then the
row closes as surveyed. groups-run already takes that branch ("or told it is
not possible yet where it did not"). Per-group keys on computers would move
into DESIGN §9, with this survey as their trigger's evidence. That costs
nothing now. A person who leaves can still rejoin from a phone, which has a
key per group.

## What I did not do

- I built nothing for (b). The row says (b) stops at NEEDS_HUMAN where the
  shape does not fit. The charter also leaves a raise past the ceilings to
  you ("The budget: past the ceilings").
- I changed no cap.
