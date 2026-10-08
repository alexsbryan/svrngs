# A — Sam's films, three homes at once

Gate: three of three people, each at home on their own computer, play a
different film from Sam's lent library at the same time for ten minutes, each
seeking at least three times, and every seek plays again within five seconds,
whoever else is seeking; Sam's Jellyfin password is in nothing any of them
received, and none of them made an account: every user in Jellyfin's list
after the run was there before it, or was made by a program Sam runs in
front of Jellyfin for a person who came, one each (`apps/jellyfin/README.md`;
`runs/mw/2026-10-08/README.md` §6). Shown after U5, in Jellyfin's own player
through the lend (DESIGN §8).

**Before the run.** Sam: a computer at home running Jellyfin with at least
three films, set up as `svrngs share` tells Sam (DESIGN §3, "Lent apps",
call 4, as u5-3 decides it), and svrngs. Alex, Bo and Cy: a computer each, in three homes on three
networks, with svrngs put there by the operator beforehand — installing is not
what this demo shows, and Demo B's newcomers do none. Each of the three browsers records
the session as a HAR file, and every screen is recorded. Before anyone starts:
Sam's upload measured once and written down, and Jellyfin's list of users
exported.

**The steps as the design has them now** (DESIGN §3, "Joining" and "Lent
apps, as they come", sign in once), which nobody is told; the dry run of
2026-10-05 took them on one computer, Alex on a computer and Bo and Cy on
phones through the page (`runs/a/2026-10-05/`). Set up before the run: Sam
runs `svrngs share 8096 --upload <the upload written down>`, which keeps
nothing of Jellyfin's and prints how to make each person an account in
Jellyfin's own pages; Sam makes Alex, Bo and Cy one each, every library
ticked, and the export of Jellyfin's users is taken after that. From the
link on: Sam runs `svrngs new <group>`, which prints the group's link,
`svrngs://join/<token>`, and keeps running (with `--origin <page>` it also
prints the link on a page address, for a phone); Sam sends each person the
link and their Jellyfin name and password. Each of the others:
`svrngs join <link>` on their computer; then `svrngs` alone, which prints
"Sam's jellyfin, in your browser:" and an address of the lend's own,
`http://<label>.localhost:<port>/`, opened in their browser (`join` prints
no lends; the bare command asks for them as it starts). Jellyfin's own page
asks them to sign in, once, with the name and password Sam sent; then they
pick a film, play it, and seek with its own seek bar.

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
any user it did not hold is one Sam's program made, one per person who came,
or it is an account someone made. Write the record in `runs/a/<date>.json`
before looking at anything else — `participants` 3, `succeeded_in_time` the
people who met the bar, `accounts_created` the accounts someone made, from the
comparison — then the notes: where each person who did not finish got stuck. A failed run is kept, not rerun until it passes.
