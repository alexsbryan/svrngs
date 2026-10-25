# svrngs capability — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided

- **The rule** (2026-10-22). "the consent an app needs is set by what it can
  do as you, not by how it arrived". "libaries are best when they have this
  capability based focused rather than this arbitrary product style
  bucketing". The exploration is svrn-docs
  `rings/current/WHAT_CAN_ACT_AS_YOU.md`.
- **Carried apps are trusted with what they read** (that note, §5). The
  containment floor keeps out foreign code and accidental reach. It does not
  stop a pinned app the person chose from sending out what it reads, and the
  docs say exactly that.
- **The note's §7, all four** (2026-10-22): "Your yes per app" for
  `--reads`; "Only carry what you signed"; "First open is the yes"; "Yes,
  keys everywhere".
- **Light consent** (2026-10-22): "make this as lightweight as possible so
  it's not onerous friction".
- **One deliverable** (2026-10-23): "I'd like to bundle it all in with" the
  note. Sealing waits for its trigger ("Out, keep its trigger"); the interim
  version-pin fix is dropped for authorship ("Drop it"); an app declaring
  what it needs waits ("Wait, with a trigger").
- **The phone's pass key** (2026-10-23, on capability-pass-key's measured
  trade-off): "K now, H with signed". The key stays readable to a lend's
  own pages, said in DESIGN §3 under §5's trust; the uncopyable key and
  chained pass land with capability-signed.
- **Authorship by signature** (2026-10-24, on capability-authorship's
  design, `demo/runs/authorship/2026-10-24/DESIGN.md`, 3e2aba45): yes, as
  §2-§5 and §9-§10 write it. F1, a lent app writing as the person asking:
  the person's own device signs what the app names in `x-svrngs-append`,
  within the lend's declared `--kind` (§6's recommended). F2, the chained
  pass at a computer on an older svrngs: today's raw-bytes pass for an open
  that reaches one, its writes waiting until a computer on the new svrngs
  takes them (§11's recommended). The ceiling for capability-signed and
  capability-keys: "+535 / +180".

## Decide

- The exact test and fix for each row, within its fence.
- Whether a check belongs in the door, the fold or the app, by where the
  decision lives already.
- Where a person's yes to an app is kept, by where the door keeps one today.
- The words a person reads, held to svrn-docs `VOICE.md`.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08).

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:
- **the budget:** past the operator's ceiling (2026-10-23, "+265 / +3
  now"): at most 17,884 repository and 3,153 svrngs-core through
  capability-what-acts, each row raising only what it lands; a row past its
  estimate's range (`demo/runs/economize/2026-10-23-capability/SURVEY.md`
  §6) stops. Then, for capability-signed through capability-release
  (2026-10-24, "+535 / +180"): at most 18,325 repository and 3,330
  svrngs-core, each row raising only what it lands, to one past where it
  lands; a row past the design's range for it (`demo/runs/authorship/
  2026-10-24/DESIGN.md` §14, +278 to +534 and +101 to +178 for both rows)
  stops;
- **a change to what an act is:** capability-authorship's design, before any
  product code changes the format;
- **a fix that would break a path a person uses today**, or add a step that
  blocks one, with no way around it;
- **any widening of the trust floor**;
- **publishing**, pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**
