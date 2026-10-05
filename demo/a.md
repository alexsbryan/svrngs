# A — Sam's films, three homes at once

Gate: three of three people, each at home on their own computer, play a
different film from Sam's lent library at the same time for ten minutes, each
seeking at least three times, and every seek plays again within five seconds,
whoever else is seeking; Sam's Jellyfin password is in nothing any of them
received, and Jellyfin's list of users is the same after the run as before.
Shown after U5, in Jellyfin's own player through the lend (DESIGN §8).

**Before the run.** Sam: a computer at home running Jellyfin with at least
three films, set up as `svrngs share` tells Sam (DESIGN §3, "Lent apps",
call 4, as u5-3 decides it), and svrngs. Alex, Bo and Cy: a computer each, in three homes on three
networks, with svrngs put there by the operator beforehand — installing is not
what this demo shows, and Demo B has none. Each of the three browsers records
the session as a HAR file, and every screen is recorded. Before anyone starts:
Sam's upload measured once and written down, and Jellyfin's list of users
exported.

**The steps as the design has them now** (DESIGN §3, "Joining"; §8, U3's
inventory), which nobody is told; the dry run of 2026-10-04 took them on one
computer (`runs/a/dry-2026-10-04/`). Sam: `svrngs new <group>`, which prints
the group's link, `svrngs://join/<token>`, and keeps running; then, beside it,
`svrngs share 8096 --upload <the upload written down>`, which asks for what
Jellyfin takes on each request and keeps it on Sam's computer. Sam sends the
link. Each of the others: `svrngs join <link>` on their computer; then
`svrngs` alone, which prints "Sam's jellyfin, in your browser:" and a local
address, `http://127.0.0.1:<port>/lend/jellyfin/`, opened in their browser
(`join` prints no lends; the bare command asks for them as it starts). Jellyfin's
own web player stopped at its sign-in page, and a member's sign-in logged the
lend out for everyone (its `README.md`); Demo A plays in it as it comes all
the same (operator, 2026-10-05; DESIGN §3, "Lent apps, as they come"). u5-3
and u5-4 take the way past that sign-in, u5-2 moves the local address to one
of the lend's own, and u5-9 writes the steps from the local address on
(DESIGN §8, U5; §11, risk 7).

**What Sam is told, and nothing else:** "Share your Jellyfin with the group,
and send the link to Alex, Bo and Cy."

**What each of the others is told, and nothing else:** "Open the link Sam
sent. Watch any film from Sam's library that the other two aren't watching,
and skip around in it."

**What is watched.** A person's clock starts when they open the link. They
succeed when their film is playing while the other two are, for ten minutes,
and they have sought three times with each seek playing again within five
seconds, read from their screen recording. The operator watches on a call with
screens shared and says nothing; any help ends that person's attempt and goes
in the notes. The notes also record, per stream, its bitrate and the share of
Sam's upload the door told it (Sam's node's log).

**After.** Search the three HAR files and the three computers' svrngs logs for
Sam's password: zero matches. Compare Jellyfin's list of users with the export:
unchanged. Write the record in `runs/a/<date>.json` before looking at anything
else — `participants` 3, `succeeded_in_time` the people who met the bar,
`accounts_created` from the comparison — then the notes: where each person who
did not finish got stuck. A failed run is kept, not rerun until it passes.
