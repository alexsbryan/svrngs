# Too many words before acting: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: hard (the instrument's): `wordy`; never both

## Where

Home. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Copy its address for a program / Open it to this WiFi / Your groups / House / (yours, since

## Who, and how often

- `kim`: 3 of 3 accounts (Kim run 1, Kim run 2, Kim run 3)
- `newcomer`: 3 of 3 accounts (June run 1, June run 2, June run 3)
- `together`: 1 of 6 accounts (Kim run 3)

## What happened

- `kim` run 1, Kim: failed, ran out of patience. wordy {"words":60,"budget":30}; their last acts: look · tap “House” · scroll · tap “Open it to this WiFi”
- `kim` run 1, Kim: failed, ran out of patience. wordy {"words":59,"budget":30}; their last acts: tap “House” · scroll · tap “Open it to this WiFi” · scroll · scroll · tap “Money”
- `newcomer` run 1, June: passed, reached their goal. wordy {"words":80,"budget":40}; their last acts: look · scroll · type “http://localhost:42021/#AncAphQDBQZHoUHWAf5jLTVPhKeLgdCSwTL4Z1wqLFoDPCFXIi58CbOY09m49vJnZWh0dHA6Ly8xMjcuMC4wLjE6NDQ2Mjcv” into “The link someone sent you”
- `kim` run 2, Kim: failed, ran out of patience. wordy {"words":119,"budget":30}; their last acts: scroll · scroll · look · scroll · scroll · tap “Open Money”
- `newcomer` run 2, June: passed, reached their goal. wordy {"words":80,"budget":40}; their last acts: look · scroll · type “http://localhost:40751/#AhitqcfjyTSJaZlNWvsFFDVpDGZYmgswcySwod8KzqWgfp4FmnimxRwJ2roPbjGXcWh0dHA6Ly8xMjcuMC4wLjE6NDQ3NzUv” into “The link someone sent you:”
- `kim` run 3, Kim: failed, ran out of patience. wordy {"words":89,"budget":30}; their last acts: tap “Open Money in House” · scroll · scroll · scroll · scroll · tap “Open Money”
- `newcomer` run 3, June: passed, reached their goal. wordy {"words":80,"budget":40}; their last acts: look · scroll · type “http://localhost:43707/#Ai9-jtSZSzXzHiHvG-ISeYrvyOhrgDVdoM6zwWgWvtyDwl9kIPZBhqOEMbUcDL4tmmh0dHA6Ly8xMjcuMC4wLjE6NDI5Mzcv” into “The link someone sent you”
- `together` run 3, Kim: failed, ran out of patience. wordy {"words":60,"budget":30}; their last acts: look · tap “House” · scroll · tap “Show House on a screen”

## What the people said

- Kim, `kim` run 1: “too much: Yes, the WiFi tap. Opening Money to the WiFi wasn't part of what Sam asked, and I did it without knowing what it would do. I shouldn't have tapped it.”
- Kim, `kim` run 1: “almost left: Yes, when I got lost in the groups page and couldn't find an expense form. That's when I stopped.”
- Kim, `kim` run 1: “too much: Yes, the WiFi tap. Opening Money to the WiFi wasn't part of what Sam asked, and I did it without knowing what it would do. I shouldn't have tapped it.”
- Kim, `kim` run 1: “almost left: Yes, when I got lost in the groups page and couldn't find an expense form. That's when I stopped.”
- June, `newcomer` run 1: “too much: The "Copy its address for a program" button and the idea of programs reading and writing things were confusing. I'd want to ask Kim before doing anything with them.”
- June, `newcomer` run 1: “almost left: Not really. The form asking for "Your name in it" appeared twice, so I was a bit unsure at first which one to use. I went with the one under "Join a group from a link".”
- Kim, `kim` run 2: “almost left: Not really. The session ended right after I tapped "Open Money", so I didn't get to decide anything.”
- June, `newcomer` run 2: “almost left: Yes, a little. The page asks you to paste a link and choose a name, and it wasn't obvious which of the two "Your name in it" boxes to use. I carried on because the link came from Kim in the house chat. If it had come from a stranger, I would have stopped.”
- Kim, `kim` run 3: “almost left: Yes. I almost left when I couldn't find how to open Money for the House group. The page was long, and the "Open Money" link wasn't easy to tell apart from the other things on it.”
- June, `newcomer` run 3: “too much: The page was a bit much. It had lots of unfamiliar wording, like "lend", "programs running here" and "copy address for a program". I didn't know what those meant, so I left them alone.”
- June, `newcomer` run 3: “almost left: Not really. I hesitated a little before typing the link in, because I wasn't sure what joining would do. It was Kim's link in our house chat, so I went ahead.”
- Kim, `together` run 3: “almost left: Not really. I was unsure about tapping "Bring the linked ones to House" without knowing what it would do. I tapped it anyway, and the session then ended.”

## The product source

- `crates/svrngs-node/tests/browser/carried.rs:194` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `crates/svrngs-node/tests/browser/opens.rs:147` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:10` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `crates/svrngs-node/src/home/home.js:168` holds “Copy its address for a program”
- `crates/svrngs-node/src/home/index.html:18` holds “Copy its address for a program”
- `crates/svrngs-node/tests/browser/main.rs:2736` holds “Copy its address for a program”
- `ARCHITECTURE.md:538` holds “Open it to this WiFi”
- `apps/jellyfin/ROOM.md:10` holds “Open it to this WiFi”
- `crates/svrngs-node/src/home/home.js:76` holds “Open it to this WiFi”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
