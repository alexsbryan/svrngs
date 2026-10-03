# D1 — Split the pizza

Gate: `n-plus-one` — four of five people new to svrngs, unassisted, settle an
expense through the bot within three minutes of joining the channel, with no
account created and no key prompt (DESIGN D9 E0). Shown after e0-3.

**Before the run.** A Discord server made for the run, one channel bound by the
operator's svrngs host, and the bot's greeting visible. Five people who have not
heard of svrngs, recruited outside the operator's circle; none has seen the bot.
Screen recording on for every participant who agrees, and on the host's log.

**What each person is told, and nothing else:** "Join this channel. You and the
others had pizza for 60 and Sam paid. Use the bot to settle it."

**What is watched.** The clock starts when a person's join message appears. A
person succeeds when an expense they entered or confirmed is in the group's
record and they can read who owes whom from the bot. The operator says nothing
and touches nothing; any help ends that person's attempt and is written in the
notes.

**After.** Fold the group's journal and compare with every balance the bot
posted (`balances_equal_fold`). Write the record in `runs/d1/` before looking at
anything else, then the notes: where each person who did not finish got stuck.
A failed run is kept, not rerun until it passes.
