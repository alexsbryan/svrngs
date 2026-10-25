# B — movie night

Gate: four of five newcomers get from the link to the film on the TV within
three minutes, unassisted.
Shown after U6: five friends at Dee's, Sam's library across town, the film
on Dee's TV in Jellyfin's own app, reached through Dee's computer
(PLAN §4, "Demo B"; DESIGN §3, "The room").

**Before the evening**, each set up once and not timed. Sam, at home across
town: a computer running Jellyfin with at least three films, lent as
`svrngs share` tells Sam (DESIGN §3, "Lent apps", call 4), and a group that
names a page origin, so its link opens in a phone's browser; in Jellyfin's
own pages, one account for the room, every library ticked (the room
account, `apps/jellyfin/ROOM.md`); then Jellyfin's list of users exported. Dee: in Sam's
group on a computer at home, on the WiFi the TV is on; on that
computer's home page Dee opens Sam's Jellyfin to this WiFi, **Open it to
this WiFi** on its row, or `svrngs room jellyfin`, which says the address
the room reaches it at; Dee's TV has Jellyfin's own app from
its store, that address typed into it once and signed in by Quick
Connect from Dee's phone with the room account (`apps/jellyfin/ROOM.md`,
u6-2).
The five: a phone each, on Dee's WiFi, never in Sam's group, nothing
installed. Every phone's screen and the TV are recorded; Sam's and Dee's
svrngs logs and Jellyfin's log are kept.

**The steps as the design has them now** (DESIGN §3, "Joining" and "The
room"), which nobody is told; the dry run of 2026-10-06 took them on one
computer, the TV a browser at the room address and the five phones
through the page (`runs/b/2026-10-06/`), and the movie-night scene plays
them on every gate, a newcomer's phone for the five
(`crates/svrngs/tests/scenes/night.rs`). Set up before the evening: Sam
runs `svrngs new <group> --origin <page>`, which prints the group's link,
`svrngs://join/<token>`, and the same link on the page's address, for a
phone, and keeps running; beside it, `svrngs share 8096`, which prints how
to make an account in Jellyfin's own pages; Sam makes one for the room,
every library ticked, and the export of Jellyfin's users is taken after
that. Dee: `svrngs join <link>` on the computer on the TV's WiFi, then
**Open it to this WiFi** on Sam's jellyfin's row of the home page, or
`svrngs room jellyfin`, which prints "Sam's jellyfin:" and
`http://<an address on Dee's WiFi>:<port>/`, the same each time, and keeps
running; that address typed into Jellyfin's app on the TV, "Use Quick
Connect" there, and its code typed on Dee's phone, which opened the link
and signed in to Jellyfin as the room. From the link on: Sam sends the
five the link on its page address and the room account's name and
password. Each opens the link on their phone, gives their name and is in;
opens Sam's jellyfin from the list, at an address of its own; signs in to
Jellyfin's own page, once, with the name and password Sam sent; taps a
film on Jellyfin's home; taps the button at the top that casts ("Cast to
Device") and the TV by its name, in a list that also holds every phone
signed in as the room; and taps Play. The film plays on the TV, from
where anyone last left it; the same button's Remote Control pauses and
seeks it from their phone.

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
