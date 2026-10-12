# One person, many devices — this host, 2026-10-12

Row bash-devices (e): Ana's computer, a second computer of hers and her
phone as one person, after (a)–(d) landed. Predictions first, in
`PREDICTIONS.md`, committed before any data (`102c8c5`). Every person is a
program; no people bar is judged (AGENTS.md 7). One host, one run: not a
rate.

## How it ran

```
W=/tmp/devices-run python3 demo/runs/devices/2026-10-12/run.py     # data/
sed 's/\x1b\[[0-9;]*m//g' /tmp/devices-run/sam.log \
  | grep -E "door: (let in|guessed in this|refused)" > data/sam-door.log
```

`run.py` and `phone.mjs` at this file's commit, as they ran. The build is
`102c8c5`'s (`data/events.jsonl`, `start`); the page built by
`scripts/build-web.sh` and served on this host by `web/tests/serve.mjs` at
`http://localhost:60387/`, nothing published; n0's public relays, the
default. Jellyfin 12.0.0 and Navidrome 0.64.2 (`data/summary.json`,
`apps`), Navidrome with `ND_EXTAUTH_USERHEADER=X-Svrngs-User`, as `svrngs
share 4533` printed (`summary.json`, `sam_shares`). The
1-minute load 10.1 at the start and 9.2 at the end (`events.jsonl`,
`start`; `summary.json`, `end_load`).

**Four takes; the fourth is every number here.** The first stopped before
anything ran (the driver's `http` shadowed Python's module). The second
stopped at Sam's shelf: `svrngs share <port> --name shelf --limit 1` was
refused, "svrngs: say what is on port …: `svrngs share … --kind <kind>`",
as it should be for a port no kind is known on, and the driver's phone log
line clashed on a key; the driver gives `--kind shelf` since. The third ran
through, but read the phone's shelf page while it still said "Opening…",
and after the removal asked Ana's computers for their own list of lends,
which no longer listed Sam's, rather than the door; the driver since waits
for the shelf's words on the phone and asks the origins each computer had
before the cut. Each take overwrote the one before's data.

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | each added by a device link, no name typed; Sam's register five keys, Ana's three one person | Ana's `svrngs link devices --device` said "A link that adds another computer or phone of yours to devices: open it there, and it joins as you, Ana, asked no name. Send it to nobody else: whoever opens it joins as you. …" and "A computer or phone already in devices on its own stays as it joined." (`summary.json`, `ana_device_link_said`). The laptop, `USER=analaptop`, no `--as`: "You're in devices, as Ana." in **1,015 ms** (`laptop_join`). The phone was never asked a name; it read "Opening your link…", "Adding this phone to devices as Ana’s…", "Reaching devices…", then "You’re in devices, as Ana", **5,184 ms** after opening the link (`phone_join`). Sam's register: five keys, Ana's three with one `person_key`, `289aba36…`, her computer's (`people_after_joins`) | held |
| 2 | one `x-svrngs-key` and one `x-svrngs-user` from all three; one account in Jellyfin and one in Navidrome | the shelf was told `x-svrngs-key: 289aba36…` and `x-svrngs-user: Ana-289aba36199a039f` by Ana's computer, her laptop and her phone alike (`shelf`). Jellyfin's users `["sam"]` before, `["Ana-289aba36199a039f", "sam"]` after; Navidrome's the same (`apps`, `accounts`). The phone's requests reached both as Ana: Sam's door let in 103 relayed requests of Ana's to Jellyfin and 5 to Navidrome (`data/sam-door.log`, `relayed=Some(true)`), beside the computers' 2 each | held |
| 3 | the sign-in asked once | the middleware's one ask, for `289aba36…` (`accounts.sign_in_asks`; `data/mw.log`), across Ana's 105 requests to Jellyfin | held |
| 4 | Ana's three let in, then Bo refused `full` | each of Ana's three let in (200), and **Bo let in too** (200, `shelf.bo`): Sam's door guessed Ana's two computers and Bo's in Sam's home — "the path ends at a private address", `192.168.1.4` — so counted none of them, and counted the phone, relayed, as `people=1` (`shelf.door`; `sam-door.log`) | could-not-judge |
| 5 | removed together within 30 s; Sam's register Sam and Bo | `svrngs remove Ana --group devices` said "Ana is no longer in devices. …" in 21 ms; the shelf's origin on Ana's computer and on her laptop answered 403 `removed`, "You are no longer in this group, so nothing of it is open to you.", at **0.03 s** and **0.04 s**; the phone, opening the page again, read "You are no longer in devices." at **1.23 s** (`removed`). Sam's register after: Bo and Sam (`people_after_remove`) | held |

## What it found

1. **On one host a lend's limit cannot count a computer.** Every
   computer's path to Sam's door ended at this host's private address,
   so the door guessed each was in Sam's home and counted nobody but the
   phone, whose path is a relay (DESIGN §8, "The local network is
   guessed"; `crates/svrngs-node/src/door/limit.rs`, `guess`). So "counted
   once by a lend's limit" is not judged here: it needs Ana's devices on
   paths the door counts — two phones, or computers in two homes. That one
   person on two devices is counted once is the door's test,
   `crates/svrngs-node/tests/door/limit.rs`,
   `one_person_on_two_devices_is_counted_once` (bash-devices (c)), red at
   the row's base and green since. A finding of the instrument, not a
   defect; nothing here changes it.
2. **Whoever holds a device link joins as its person.** The command says
   so ("Send it to nobody else"); nothing else stops it, as nothing stops
   a friend's link from reaching a stranger. Said, not solved.
3. **A device link closes its issuer's other link.** Ana's computer has
   one open link (DESIGN §8, "A new link closes the old one"), so the link
   she sent a friend stops letting anyone new in once she makes one for her
   devices, and the other way about; the command says it as it does for
   any link. Not measured here: Ana sent no other link.
