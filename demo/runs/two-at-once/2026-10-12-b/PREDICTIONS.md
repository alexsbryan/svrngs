# Two of one person's devices at once, after (b) — predictions, before any data

Row bash-two-at-once (b) (`ralph/next/bash/STATE.md`): `../2026-10-12/`'s
steps again, on `0e04c99`, which tells a lend the device asking
(`x-svrngs-device`, `crates/svrngs-node/src/door.rs`, `DEVICE_HEADER`),
keeps a sign-in's answer a device (`door/sign_in.rs`, `Whose`), and names
each Jellyfin device from it (`apps/jellyfin/mw/src/person.rs`,
`Device::of`). `../2026-10-12/` is the red; this run is to be the green.
The setup is that run's, its driver copied here (`run.py`, `two.mjs`),
changed only in the trace words it counts. Every person is a program; one
host, one run: not a rate. Written and committed before the driver ran.

## Predictions

1. **Both in by Ana's device link, no name typed**, as before: the laptop
   says "You're in two, as Ana." within 60 s, the phone reads "You’re in
   two, as Ana" within 60 s, never asked a name; neither asked a sign-in.
   Now each page's stored device (`_deviceId2`) is **its own**: the
   laptop's `svrngs-<first 16 hex of the laptop's key>`, the phone's
   `svrngs-<first 16 hex of the phone's key>`, each a key Sam's register
   lists for Ana, neither the other's.
2. **Each film plays** within 30 s of its play button.
3. **Neither takes the other's film**: outside each page's own seeks, no
   `emptied`, `loadstart` or new element, no still of 5 s; each of the
   eight seeks plays again within 30 s.
4. **Jellyfin sees two devices, each its own film.** Every `/Sessions`
   read in the window lists two sessions of Ana's account, one for each
   device of prediction 1, and each that names a film names its own
   device's film — never the other's.
5. **Stopping one leaves the other**: after the laptop's film stops, the
   phone's plays on 20 s with no `emptied` or `loadstart`, at least 15 s
   played, and every read of the phone's session names its film.
6. **Each film's place kept as its own**: within 15 s of where its player
   stood when stopped.
7. **One account, asked once a device.** Jellyfin's users after: `sam`
   and one `Ana-<16 hex>`; the middleware asked twice for Ana's key, once
   with each device's key.

A step the run cannot finish is could-not-judge, never a pass (AGENTS.md
6). If 4 does not hold — one session, or a session naming the other
device's film — (b) has not closed what (a) found.
