# start-economize: 187 short of 15,800

The operator asked for the repository back under 15,800 (2026-10-13, "Back
under 15,800, then ratchet"). The row's survey found 50 lines it could cut
with behaviour kept (SURVEY.md). It cut all of them and lowered the caps to
where they landed (0b5bccf):

| | At 9267bc1 | After the cuts | Cap now |
|---|---:|---:|---:|
| repository | 16,036 | **15,986** | 15,987 |
| svrngs-core | 3,121 | 3,121 | 3,122 |

`python3 scripts/lean_check.py` at e894f42: `crates/svrngs-core 3121 /
3122`, `. 15986 / 15987`, exit 0. **15,986 − 15,799 = 187 lines short.**

The row allows this ending: "where the survey finds less than that,
everything it found cut, the ratchet made at what was reached, and
NEEDS_HUMAN.md saying how far short and what remains". Nothing is waiting
on this file. The next rows run on the lowered caps (3f17dba).

## Why the instruments found no more

- **The compiler.** After every `pub` was demoted and the product and the
  page rebuilt, five items reached only by tests were left. lean-3
  (8012ec3f9) had already taken what nothing reached. Nothing in
  svrngs-core went unreached.
- **The clone report.** At 0.95 the only exact groups are three-line
  accessors on different types. At 0.90, apart from neighbours and test
  code, there is one pair, and it is two deciders that disagree (below).
  lean-4 (a) had already merged the clone pairs (732285d..e978122).
- **The open findings.** The ones that name two readers or two deciders
  either save no line or change what a person is told (#136, #135, #113).
  For the rest, lean-4 already said why not (#42, #48, #63, #89).

## What remains, and what each needs

None of these is a cut this row may make. Each one changes what a person
sees, changes what a test asserts, or was found by reading rather than by
the row's instruments. Line counts marked ~ are my estimates; nothing was
measured for them.

| What | Lines | What it changes | Needs |
|---|---:|---|---|
| origin.rs:117-153 `kept_to_reads` and surface.rs:237-271 `reads`: one decider of "may the page of `app` reach `label`" | ~25 | which page a silent lender refuses, and in what words: today `kept_to_reads` refuses could-not-judge (`away`), while `reads` lists the lender as `unanswered` | a follow-up row whose test asserts one reading of an absence, red first |
| cli.rs:333-391 `parse` (at e894f42): the ten value flags each read, parse and wrap a refusal the same way, so one helper could do it | ~15 | nothing a person sees, since every message is kept | a decision: the clone report does not find a shape repeated inside one function, so this is "said twice" found by reading. If that counts, it is a move for a row like this one |
| cli.rs `Verb::Remove` and `Verb::Handover`, the same three fields: one variant with a kind | ~6 | the `Verb` the tests assert (tests/remove.rs, tests/joined.rs) | a follow-up row |
| #113: home.js and app.js each walk question.js's `uses` | ~5 | one "not lent" line ("groups" against "group") | a follow-up row |
| svrngs's own relay, parked (crates/svrngs/src/relay.rs 142, `Verb::Relay` and its usage text, scripts/deploy-relay.sh 43): the rest of the gap and more | ~200 | deletes the relay the operator chose to build before v1 and keeps parked behind `SHIPPED` (DESIGN §7; relay-park) | **not recommended.** I list it only because it is the one thing that size in the product a person does not run today |

The first four together come to ~51, which would leave the repository at
~15,935 and still 136 short. **Closing the gap means removing behaviour
the operator chose to have, not removing waste.**

## The question

Do you accept 15,987 and 3,122 as the caps, with the first three rows
above queued as follow-ups wherever they fit? Or do you want a cut that
changes behaviour to reach 15,800, and if so, which one? Until you say,
the caps stand where the cuts landed, and every raise is asked only after
a pass of its own (AGENTS.md "Done"; DESIGN §5).

## The answer

The operator, 2026-10-13: "Accept 15,987; fold follow-ups into onair".
The caps stand at 15,987 and 3,122 (0b5bccf). The first four rows of the
table above became onair-economize, the onair queue's pass before any
raise (491936f, `ralph/next/onair/STATE.md`); the parked relay stays.
