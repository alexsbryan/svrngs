# B — movie night

Gate: four of five newcomers get from the link to the film on the TV within
three minutes, unassisted.
Shown after U6: five friends at Dee's, Sam's library across town, the film
on Dee's TV in Jellyfin's own app, reached through Dee's computer
(DESIGN §8; §3, "The room").

**Before the evening**, each set up once and not timed. Sam, at home across
town: a computer running Jellyfin with at least three films, lent as
`svrngs share` tells Sam (DESIGN §3, "Lent apps", call 4), and a group that
names a page origin, so its link opens in a phone's browser; in Jellyfin's
own pages, one account for the room, every library ticked (the room
account, `apps/jellyfin/ROOM.md`); then Jellyfin's list of users exported. Dee: in Sam's
group on a computer at home, on the WiFi the TV is on; that computer runs
the room door for Sam's Jellyfin (u6-1 names its command), which prints
the address the room reaches it at; Dee's TV has Jellyfin's own app from
its store, that address typed into it once and signed in by Quick
Connect from Dee's phone with the room account (`apps/jellyfin/ROOM.md`,
u6-2).
The five: a phone each, on Dee's WiFi, never in Sam's group, nothing
installed. Every phone's screen and the TV are recorded; Sam's and Dee's
svrngs logs and Jellyfin's log are kept.

**The steps as the design has them now**, which nobody is told; u6-9's dry
run takes them on one computer and writes here the ones it found. From the
link on: Sam sends the group's link, on its page address, and the room
account's name and password. Each newcomer opens the link on their
phone, gives their name and is in (DESIGN §3, "Joining"); opens Sam's
jellyfin from the list, at an address of its own (DESIGN §7); signs in to
Jellyfin's own page, once, with the name and password Sam sent; picks a
film, and plays it on Dee's TV from Jellyfin's own "Play on", and any
phone in the room pauses and seeks it, all signed in as the room (u6-2's
run, `c-same`; seat, 2026-10-07: one account for six and no permission
ticked, where an account each loses the TV once a film starts); the TV's app kept
to the room's address in u6-2's run (DESIGN §3, "The room", call 4).

**What Sam is told, and nothing else:** "Share your Jellyfin with the group,
and send the link to the five at Dee's."

**What Dee is told, before the evening, and nothing else:** "Put Sam's
Jellyfin on your TV."

**What each of the five is told, and nothing else:** "Open the link Sam
sent, and put a film from Sam's library on the TV."

**What is watched.** The five go one at a time, the TV playing one film at a
time. A person's clock starts when they open the link and stops when the
film they picked plays on the TV, read from the recordings; within three
minutes they succeed. The operator watches in the room and says nothing; any
help ends that person's attempt and goes in the notes. The notes also
record what Dee typed before the evening, and where Dee got stuck.

**After.** Compare Jellyfin's list of users with the export: unchanged.
Write the record in `runs/b/<date>.json` before looking at anything else —
`participants` 5, `succeeded_in_time` the people who met the gate,
`accounts_created` from the comparison, `key_prompts` the times a phone
asked for anything DESIGN §1 keeps from people, `assisted` — then the notes:
where each person who did not finish got stuck. A failed run is kept, not
rerun until it passes.
