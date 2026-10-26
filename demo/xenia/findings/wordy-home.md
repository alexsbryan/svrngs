# Too many words before acting: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T22-40-27-374Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `b76397f`, the product `6d50d7d9`
- kind: hard (the instrument's): `wordy`; never both

## Where

Home. What was read there first: Money / Copy its address for a program / Show House on a screen / Your groups / House / (yours, since 26 Oct)

## Who, and how often

- `kim`: 3 of 3 accounts (Kim run 1, Kim run 2, Kim run 3)
- `newcomer`: 3 of 3 accounts (June run 1, June run 2, June run 3)
- `together`: 1 of 6 accounts (Kim run 1)

## What happened

- `kim` run 1, Kim: failed, ran out of patience. wordy {"words":90,"budget":30}; their last acts: look · scroll · scroll · tap “Show House on a screen”
- `newcomer` run 1, June: passed, reached their goal. wordy {"words":80,"budget":40}; their last acts: look · scroll · type “http://localhost:40745/#AloVQiHslBLAItxC4hMKre-jUl11sbRquQKzvb2rs6TwDm4lKh9RM5JgdAaoWXPf22h0dHA6Ly8xMjcuMC4wLjE6NDA1Njcv” into “The link someone sent you:”
- `together` run 1, Kim: failed, ran out of patience. wordy {"words":60,"budget":30}; their last acts: look · scroll · tap “Money”
- `together` run 1, Kim: failed, ran out of patience. wordy {"words":60,"budget":30}; their last acts: look · scroll · tap “Money” · scroll · tap “Show House on a screen”
- `kim` run 2, Kim: passed, reached their goal. wordy {"words":90,"budget":30}; their last acts: look · scroll · scroll · tap “Money”
- `newcomer` run 2, June: passed, reached their goal. wordy {"words":80,"budget":40}; their last acts: look · scroll · type “http://localhost:46467/#AkRd_haaf5hZeAsdHEU-xGTyZpgECWQmZMO2cJo42lR6AD5L3zlKmGZuCHzSIHlmYWh0dHA6Ly8xMjcuMC4wLjE6MzkxNjkv” into “The link someone sent you”
- `kim` run 3, Kim: passed, reached their goal. wordy {"words":90,"budget":30}; their last acts: look · scroll · scroll · tap “Money”
- `newcomer` run 3, June: failed, stopped before their goal. wordy {"words":80,"budget":40}; their last acts: look · type “http://localhost:37133/#AgK3NNRdhLN3R1hgIiYM5M9DcC1v_PvJ88leNIiSyDpVkAaqJVBmCioBsCLfMq2CjWh0dHA6Ly8xMjcuMC4wLjE6MzgzNTMv” into “The link” · tap “Open it” · look · say “Hi Kim, thank you. I pasted your link into the page and pressed Open it, but it says "That is not a link to something lent in your group." Could you check it and send it again, or show me what to do? I'd rather not guess.” · tell

## What the people said

- Kim, `kim` run 1: “almost left: Yes, at the end. The session said I'd given up right after I tapped Money, and I hadn't chosen to stop.”
- Kim, `together` run 1: “almost left: Yes, when tapping Money did nothing useful. I got stuck there and the session ended.”
- Kim, `together` run 1: “almost left: Yes, when tapping Money did nothing useful. I got stuck there and the session ended.”
- Kim, `kim` run 2: “almost left: Yes, when the screen got stuck on "Tell a friend how it went" and I couldn't get back to fix the entry.”
- June, `newcomer` run 2: “too much: The page had a lot of unfamiliar wording, like "What you lend", "Look for programs running on this computer" and "Copy its address for a program". I left all of that alone.”
- June, `newcomer` run 2: “almost left: Not really. I hesitated a little at the confusing wording, but I didn't need any of it, so I carried on and stopped once I'd joined.”
- June, `newcomer` run 3: “too much: The page is wordy and I don't fully understand it (lending, groups, programs). That made me nervous, so I stopped rather than try anything else.”
- June, `newcomer` run 3: “almost left: Yes, when the error appeared. I didn't know whether the link was wrong or I'd done something wrong. I asked Kim and stopped.”

## The product source

- `crates/svrngs-node/src/home/home.js:172` holds “Copy its address for a program”
- `crates/svrngs-node/src/home/index.html:19` holds “Copy its address for a program”
- `crates/svrngs-node/tests/browser/main.rs:2738` holds “Copy its address for a program”
- `crates/svrngs/tests/xenia/main.rs:116` holds “Show House on a screen”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:18` holds “Show House on a screen”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:19` holds “Show House on a screen”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
