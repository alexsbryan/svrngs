# Demo B, the operator's run — u6-9's package

`operator-only: anything that needs a service this host cannot run` —
Dee's computer and Jellyfin's app on a real TV, Sam's library on another
network, five people.

The dry run on this host (`README.md` beside this file) took Demo B's
steps on one computer: five browser profiles as phones each got from the
link to their film on a TV stand-in in 25–39 s, the TV reaching Sam's
Jellyfin only through the room door. What it cannot say is what this
package is for: whether five people find those steps, and what Jellyfin's
own app on a real TV does. `demo/b.md` is the run-of-show; its gate is
unchanged.

## Before the evening (not timed)

**Sam**, at home, on a computer running Jellyfin with at least three
films, on a network that is not Dee's:

```sh
cargo build --release -p svrngs
RUST_LOG=info target/release/svrngs new House --as Sam --origin https://group.svrnme.sh/ 2>sam-run.log
# beside it, in another terminal:
target/release/svrngs share 8096
```

`new` keeps running all evening, and prints the link twice: `svrngs://…`
for Dee's computer, `https://group.svrnme.sh/#…` for the phones. `share`
prints how to make an account in Jellyfin; make one for the room as
`apps/jellyfin/ROOM.md` says ("The simplest way: one account for the
room": a name such as `room`, a password, every library ticked). Then
export Jellyfin's users, as Demo A's package does (Dashboard → API Keys →
+, then):

```sh
curl -s http://127.0.0.1:8096/Users \
  -H 'Authorization: MediaBrowser Client="export", Device="Sam", DeviceId="export", Version="1", Token="<the key>"' \
  | python3 -c 'import json,sys;print(json.dumps(sorted(u["Name"] for u in json.load(sys.stdin))))' >users-before.json
```

**Dee**, at home, on a computer on the WiFi the TV is on, with svrngs put
there beforehand:

```sh
target/release/svrngs join '<the svrngs:// link Sam sent>' --as Dee
RUST_LOG=info,room=debug,door=debug target/release/svrngs room jellyfin 2>dee-room.log
```

`room` prints `Sam's jellyfin: http://<an address on Dee's WiFi>:<port>/`
and keeps running all evening; the address is the same each time it runs.
Then, as `apps/jellyfin/ROOM.md` says: on the TV, Jellyfin's own app from
its store, that address typed into it once; on the TV, "Use Quick
Connect"; on Dee's phone, the `https://group.svrnme.sh/#…` link, Sam's
jellyfin from the list, signed in as the room's account, then the picture
at the top right, Quick Connect, and the TV's code. Write down, in the
notes, everything Dee typed and where Dee got stuck.

## The evening

Sam sends the five the `https://group.svrnme.sh/#…` link and the room's
name and password. Each of the five is told only what `demo/b.md` says,
and goes one at a time, the TV playing one film at a time. Every phone's
screen and the TV are recorded (the TV by a phone on a stand). The
operator says nothing; any help ends that person's attempt and goes in
the notes.

## What to record

- Each person's clock, from opening the link to the film they picked
  playing on the TV, read from the recordings; who met three minutes.
- `sam-run.log`, `dee-room.log` and Jellyfin's log (Dashboard → Logs,
  the evening's file), kept.
- Jellyfin's Dashboard → Playback, while each film plays on the TV:
  played as it is or converted.
- After: export the users again (the command above, to
  `users-after.json`) and compare; write `demo/runs/b/<date>.json` as
  `demo/README.md` says, before anything else — `participants` 5,
  `succeeded_in_time`, `accounts_created`, `key_prompts` (any time a phone
  asked for anything DESIGN §1 keeps from people), `assisted` — then the
  notes.

## What only the real TV says (u6-2 and this run left these open)

1. **What the TV is called in "Play on".** The stand-in was "Chrome", one
   item among every phone signed in as the room (finding 3). Note what
   Jellyfin's app on the TV is called there, and whether anyone picked a
   phone instead of it.
2. **Whether the TV's app keeps to the room address** with Dee's
   computer asleep or `room` stopped a while, and asks no other address
   (u6-2's finding 4 left the app's own shell unread).
3. **Whether the TV asks Google's `cast_sender.js`** (u6-2, finding 1):
   it does no harm if refused; note it if the app stalls on it.
4. **Each film played as it is or converted on the TV**, and whether a
   phone's seek moves it (finding 4: a phone dragging before its Remote
   Control showed the film's length sent the stand-in to the start).
5. **Starting a film someone already started**: it resumes where they
   left it (finding 3); note whether anyone was surprised.

## Nothing to decide first

The dry run passed every bar it read, and no finding moves a demo before
this run: findings 3 and 4 are Jellyfin's, for one account and for a
phone's own Remote Control, and the operator's run reads them.
