# A folder your whole group shares

svrngs can keep a folder the same on everyone's computer in a group: drop a
file in yours, and it appears in theirs. This page shows how to use the
folder every group already has, how to share another one, and what happens
when two people change the same file.

## Use your group's folder

Every group already has one. When you make a group or join one, svrngs
says where:

```
Elm's folder is /Users/ana/svrngs/Elm: what anyone in Elm puts there reaches this computer, and what you put there reaches theirs.
```

That's `~/svrngs/<group>`, on each computer in the group. Put a file in
yours, and within moments it's in Bo's and Cy's too. Change it or delete
it, and the change reaches theirs the same way.

The apps you bring from your home page live here too, under `apps/` (see
[the apps that come with svrngs](apps.md)). Leave that folder alone unless
you mean to change an app.

## Share another folder

Ana's band keeps its recordings in `~/Music/rehearsals`. Ana types:

```
~/.local/bin/svrngs share --folder ~/Music/rehearsals --group Band
```

(From here on, `svrngs` means `~/.local/bin/svrngs` if your terminal
doesn't find it.) svrngs answers with the folder's name, `{"name":"rehearsals"}`.

**The folder's name is what matters.** svrngs matches folders by their last
name, here `rehearsals`. So Bo and Cy each run the same line with a folder
of that name, wherever they keep it:

```
mkdir -p ~/Band/rehearsals
svrngs share --folder ~/Band/rehearsals --group Band
```

From then on the three folders are one. What's in Ana's arrives in Bo's
and Cy's, and anything they add arrives in Ana's. A folder of a different
name is a different folder: Bo's `~/Band/recordings` would stay Bo's own.

`--group` takes your group's name, or the first 8 letters of its id when
two of your groups share a name. svrngs has to be running on your computer
for this to work, and keeps the folder in step from then on, every time it
runs.

## A photo drop for a trip

After a weekend away, Sam, Bo and Dee each run:

```
mkdir -p ~/Pictures/lisbon
svrngs share --folder ~/Pictures/lisbon --group Trip
```

Everyone copies their photos into their own `lisbon` folder, and soon
each of them has everyone's. Nobody uploaded anything anywhere: the
photos went from one computer to the others.

## Who can do what

- **Anyone in the group** can add, change and delete files in the folder.
  A delete is everyone's: if Bo deletes a photo, it goes from every
  computer.
- **Nobody outside the group** receives a byte of it. svrngs gives a file
  only to a computer of someone in the group, asked fresh each time.
- **Someone you remove** keeps what they already have; nothing new reaches
  them.

## When two people change the same file

Bo and Cy both edit `setlist.txt` before either sees the other's change.
Neither edit is lost: one keeps the name `setlist.txt`, and the other is
kept beside it as `setlist (conflict Cy).txt`, named after whoever wrote
it. Every computer ends up with the same two files. Open both, merge them
by hand, and delete the one you don't need.

svrngs never writes over a file you've changed on your computer since it
last put it there.

## When a computer is off

Each computer gets a file from another computer in the group that has it.
So a new file reaches you once some computer that holds it is on, and
yours is on too. If Ana adds a song and closes the laptop straight away,
Bo gets it when Ana's laptop, or anyone else's that already has it, is on
again.

Changes you make while svrngs isn't running aren't lost: it notices them
when it starts and sends them then.

## Stop sharing a folder

There's no way to stop yet: no command takes a folder back out. Don't
empty the folder to stop sharing it, since what you delete there is
deleted for everyone.

## If something goes wrong

- *"svrngs is not running on this computer: start it, then run this
  again"*: open svrngs from your Applications (on Linux, your
  applications menu), then run the line again.
- *"you are in no group called Band on this computer"*: check the group's
  name as your home page shows it.
- *"the folder was refused: … is not synced: one directory keeps one
  group's folder, and this folder or this directory is kept already"*: this
  group already has a folder of that name, or that folder is already
  shared. Your group's own folder counts, so a folder named after the group
  is taken. Rename yours, and ask the others to use the same new name.
