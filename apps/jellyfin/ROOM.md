# Jellyfin on the TV in the room

For whoever lends Jellyfin, and whoever's computer is in the room with
the TV. Measured on one computer, a browser standing in for the TV
(`demo/runs/u6/2026-10-06-jellyfin-screen/`, u6-2); the real TV is the
next run's.

**The TV, once.** On a computer in the room, on the TV's WiFi, in the
group: `svrngs room jellyfin`. It prints the address the room opens
Jellyfin at, such as `http://192.168.1.4:48719/`, the same each time it
runs. On the TV, install Jellyfin's own app from its store and type that
address into it, once. Leave the command running while the TV plays.

**The TV signed in, with nothing typed on its remote.** On the TV, "Use
Quick Connect": it shows a code. On your phone, open the group's link,
open Jellyfin from the list and sign in; then your picture at the top
right, then Quick Connect, and type the code. The TV is then signed in
as you. Jellyfin comes with Quick Connect on; if the TV does not offer
it, in Jellyfin: your picture at the top right, then Dashboard, General,
"Enable Quick Connect on this server", and Save.

**A film on the TV from someone else's phone.** Whoever lends Jellyfin,
for each person who will: in Jellyfin, your picture at the top right,
then Dashboard, Users and their picture; tick "Allow remote control of
other users", and Save. Without it, the TV is not in their list. Then on
their phone, in Jellyfin: the button at the top that casts ("Cast to
Device"), the TV by its name and the name it is signed in as, and play
the film.

**Pause and seek from the TV's own remote.** Once their film starts, a
phone signed in as someone other than the TV loses the TV: the TV keeps
playing, but that phone cannot pause or seek it. Jellyfin does this
on its own, with svrngs nowhere between them. The phone of whoever the
TV is signed in as keeps it: the same button, then Remote Control.
