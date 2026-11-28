# UX friction, as met

What made a real task harder than it should have been, one entry per run,
with what was typed and what svrngs said. Each entry names whether the
current source already fixes it (with the file or the release that does),
so a fix starts from the evidence, not from memory. Nothing here is a plan;
`PLAN.md` §1 holds the order of work.

## 2026-11-26: invite people to an old group and lend it Jellyfin

The task, as the operator asked it: "generate a group invite for the
oolong time group", then "make the group and share the jellyfin server on
it". It took eight commands, a stop and restart of the login service, and
reading the source to know which messages to believe. It should have taken
two (`svrngs link oolongtime`, `svrngs share 8096 --group oolongtime`).

This computer ran svrngs 1.11.1 (24b6ee7c) from `~/.local/bin/svrngs`, kept
running at login by `svrngs.service`; the source was at v1.13.3-17.

### 1. An old group can't be carried forward, only remade

`svrngs link oolongtime` said:

> oolongtime was made by an older svrngs, so nothing in it changes now. To
> carry on, make it again and invite people by link.

The group (made 6 Nov) is now read-only, and the only way on is a new,
empty group: its members, its folder and what was lent to it stay behind,
and each person must be invited again. Nothing says *what* is left behind,
or offers to bring the members, the folder's files or the lends across.
The sentence is `svrngs_core::made_older` (`crates/svrngs-core/src/lib.rs`
line ~330); still so in the current source.

Better: `svrngs new <name> --from <old group>` (or a button beside the
sentence on the home page) that makes the new group, copies the folder,
re-lends what was lent, and prints one link to send to the old group's
people, naming who they are.

### 2. `svrngs new` refused while svrngs ran at login

> svrngs: svrngs is already running on this computer
> (/home/alexbryan/.svrngs/lock is held); stop it, then run this again

So: `systemctl --user stop svrngs.service`, `svrngs new oolongtime` (which
then stays in the foreground, "Leave this running; Ctrl-C stops it"),
stop it, `systemctl --user reset-failed` (the unit had gone to *failed*
while the lock was taken), start the unit again. Every other group was
offline meanwhile.

**Fixed in V1.12.0** (`RELEASES.md`, "`svrngs new` works while svrngs
runs"; `crates/svrngs/src/cli.rs`, the `Verb::New` arm handed to the running
svrngs). This computer never got it: see 3.

The stop, the restart in the foreground and the reset of the unit after
an update: **Fixed in the release after V1.13.3**
(`crates/svrngs/src/cli.rs`, `Verb::Update` and `restart`): `svrngs
update` restarts svrngs onto the new one itself, by the login service
where it keeps svrngs, else by asking the svrngs running to stop
(`POST /v1/stop`, `crates/svrngs-node/src/surface.rs` `stop`) and starting
the new one apart from the terminal, and says "svrngs <version> is
running.".

### 3. This computer was two minor versions behind and nothing said so

1.11.1 ran here while 1.13.3 was released. Neither the refusal in 2 nor
`svrngs` ("svrngs is already running. Your home page is at …") said an
update was waiting, or that the update fixes the very refusal being shown.

Better: a refusal that a newer release removes says so ("svrngs 1.12 does
this while svrngs runs: `svrngs update`, then `svrngs restart`"), as
`Refused::Older` already does for the running process.

Where it applies: **Fixed in the release after V1.13.3**
(`crates/svrngs/src/cli.rs`, `Here::asked`). A command that finds another
build of svrngs running restarts it onto its own before it asks, and one
that finds a newer one says "run `svrngs update`, then this again"; bare
`svrngs` opens the home page in place of "already running". A refusal
that a release removes still doesn't say so.

### 4. Two groups with one name

`svrngs new oolongtime` said "This is a second group called oolongtime",
and put its folder at `~/svrngs/oolongtime-012085ec`. From then on the
name alone is ambiguous: the share and the link had to name the group by
the first 8 letters of its id (`--group 012085ec`), read off the folder
name. A group that can no longer change (1) still holds the name.

Better: when the same-named group is one an older svrngs made, the new
one takes the name and the old one is shown as "oolongtime (old)", so
`--group oolongtime` means the one that works.

### 5. The first link was spent by the second

`new` printed a link while running in the foreground. After the service was
restarted, `svrngs link 012085ec` was run to be sure the link was served by
the running svrngs, and that cancels every link printed before ("Any link
you sent before stops letting anyone new in"). Nobody had the first, but
nothing on screen said which of the two was live, nor whether a link
printed by a process now stopped still works.

Better: `svrngs link <group>` with no flag shows the live link, if one is,
and makes a new one only when asked (`--new`).

### 6. Lending Jellyfin ends with manual account work

`svrngs share 8096 --group 012085ec --program jellyfin` worked, then said to
make each person a Jellyfin account by hand (Dashboard, Users, +, a
password, tick libraries, send each person theirs). For a group of friends
this is the longest step and it happens outside svrngs. Navidrome and
Calibre-Web already take each person from svrngs (`svrngs --help`, `share`).

Better: Jellyfin signs people in from svrngs too (its SSO plugin takes an
OIDC client, which `--sign-in-with-svrngs` already makes), so lending it
needs no accounts.

### 7. A friend's Mac, running an older svrngs, could not join

Phoebe (macOS, x86_64) ran the installer with the link. It said "Stopped
the svrngs running here, to update it", "Updated svrngs in
/Users/phoebe/.local/bin", then:

> svrngs: cannot read or keep this computer's token,
> /Users/phoebe/.svrngs/token: No such file or directory (os error 2)

The likely path, from the source: her svrngs was older than the token
(`TOKEN_FILE`, added 662803be, 2026-10-25), and was running again over
`~/.svrngs` when `join --no-run` ran (`web/install.sh` line ~266; the login
service may restart it between the stop at line ~138 and the binary's swap
at line ~234). So `join` found the lock held and asked the running svrngs,
and `asked` in `crates/svrngs/src/cli.rs` (line ~1200) reads the token with
`?` before it looks for an older svrngs to restart: the missing file
refuses the join, and the restart meant for exactly this case (`Withheld::Older`
then `restart`) is never reached. Not yet reproduced; RUST_LOG=debug on her
`join` would say which.

Better: a missing token where the lock is held is an older svrngs
running, and goes the `Withheld::Older` way (restart it, ask again); and
the installer waits for the svrngs it stopped to be the new one before it
joins.

The first half: **Fixed in the release after V1.13.3**
(`crates/svrngs/src/cli.rs`, `Here::asked`, `stopped`): a data dir with no
token whose lock is held is a svrngs too old to keep one, restarted onto
this one as any other build is, by the login service where it keeps
svrngs; by hand, one that old can't be asked to stop, and the command
says the one step that stops it, then to run it again. The installer's
wait is not yet done (`web/install.sh`).
