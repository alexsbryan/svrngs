# A — Sam's films, three homes at once

Gate: three of three people, each at home on their own computer, play a
different film from Sam's lent library at the same time for ten minutes, each
seeking at least three times, and every seek plays again within five seconds,
whoever else is seeking; Sam's Jellyfin password is in nothing any of them
received, and Jellyfin's list of users is the same after the run as before.
Shown after U3 (DESIGN §8).

**Before the run.** Sam: a computer at home running Jellyfin with at least
three films and one viewer account whose password only Sam's computer holds,
and svrngs. Alex, Bo and Cy: a computer each, in three homes on three
networks, with svrngs put there by the operator beforehand — installing is not
what this demo shows, and Demo B has none. Each of the three browsers records
the session as a HAR file, and every screen is recorded. Before anyone starts:
Sam's upload measured once and written down, and Jellyfin's list of users
exported.

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
