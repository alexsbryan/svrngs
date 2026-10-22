# Jellyfin on the TV in the room

For whoever lends Jellyfin, and whoever's computer is in the room with
the TV. Measured on one computer, a browser standing in for the TV
(`demo/runs/u6/2026-10-06-jellyfin-screen/`, u6-2); the real TV is the
next run's.

**The TV, once.** On a computer in the room, on the TV's WiFi, in the
group: open your home page, find Jellyfin in your group's list, and press
**Show it on a TV here**. The row then says the address to type into the
TV's app, such as `http://192.168.1.4:48719/`, the same each time. (In a
terminal, `svrngs room jellyfin` does the same.) On the TV, install
Jellyfin's own app from its store and type that address into it, once.
While it's on, anyone on this WiFi can open Jellyfin there as you. It
stays on whenever svrngs runs on this computer and it is on this WiFi,
also after a restart; on another WiFi it is off. Turn it off on the same
row.

**The TV signed in, with nothing typed on its remote.** On the TV, "Use
Quick Connect": it shows a code. On your phone, open the group's link,
open Jellyfin from the list and sign in; then your picture at the top
right, then Quick Connect, and type the code. The TV is then signed in
as you. Jellyfin comes with Quick Connect on; if the TV does not offer
it, in Jellyfin: your picture at the top right, then Dashboard, General,
"Enable Quick Connect on this server", and Save.

**The simplest way: one account for the room.** Whoever lends Jellyfin
makes one account for the room, in Jellyfin: your picture at the top
right, then Dashboard, Users and the plus; a name such as `room`, a
password, every library ticked, and Save. The TV signs in with it by
Quick Connect (above, from a phone signed in as `room`), and that name
and password go to everyone in the room. Every phone signed in as `room`
finds the TV under the button at the top that casts, by the TV's name
among every phone signed in as `room`, and starts, pauses and seeks it,
with nothing ticked (u6-2's run, `c-same`: a phone signed in as the TV's
own account, which had no permission ticked; five phones in turn in
u6-9's, `demo/runs/b/2026-10-06/`). What it costs: the room shares one
account, so a film one person plays shows as played for all, and starts
where they left it (u6-9's run: the second to pick a film started it
631 s in).

**Or an account for each person: a film on the TV from someone else's phone.** Whoever lends Jellyfin,
for each person who will: in Jellyfin, your picture at the top right,
then Dashboard, Users and their picture; tick "Allow remote control of
other users", and Save. Without it, the TV is not in their list. Then on
their phone, in Jellyfin: the button at the top that casts ("Cast to
Device"), the TV by its name and the name it is signed in as, and play
the film.

**Pause and seek, with an account for each person, from the TV's own remote.** Once their film starts, a
phone signed in as someone other than the TV loses the TV: the TV keeps
playing, but that phone cannot pause or seek it. Jellyfin does this
on its own, with svrngs nowhere between them. The phone of whoever the
TV is signed in as keeps it: the same button, then Remote Control.
