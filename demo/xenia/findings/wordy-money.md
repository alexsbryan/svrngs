# Too many words before acting: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`
- kind: hard (the instrument's): `wordy`; never both

## Where

Money. What was read there first: Say you were in on pizza / What it was for: / How much: / Money / Who pays whom / What people paid for

## Who, and how often

- `ana`: 3 of 3 accounts (Ana run 1, Ana run 2, Ana run 3)
- `together`: 5 of 6 accounts (Kim run 1, Ana run 1, Kim run 2, Kim run 3, Ana run 3)

## What happened

- `ana` run 1, Ana: passed, reached their goal. wordy {"words":60,"budget":20}; their last acts: look · scroll · tap “Money” · scroll · scroll · tap “Say you were in on pizza”
- `together` run 1, Kim: failed, stopped before their goal. wordy {"words":36,"budget":30}; their last acts: look · tap “Money” · type “Friday's pizza” into “What it was for” · type “36” into “How much” · scroll · tap “Add it, paid by you”
- `together` run 1, Kim: failed, stopped before their goal. wordy {"words":90,"budget":30}; their last acts: type “36” into “How much” · scroll · tap “Add it, paid by you” · look · scroll · type “Friday's pizza (with Ana)” into “What it was for”
- `together` run 1, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: tap “Ana (not ticked)” · tap “Ana” · tap “Add it, paid by you” · look · say “Hi Ana, Sam: I paid for Friday's pizza, $36 for me and Ana. It's in the house Money page. Ana, your share is $18. It shows as owed once you add yourself as in on it. Heads up: there's also an earlier entry called just "Friday's pizza" that I added by mistake with only me on it. It costs nobody anything, but ignore it. The one marked "(with Ana)" is the real one.” · tell
- `together` run 1, Ana: failed, stopped before their goal. wordy {"words":73,"budget":20}; their last acts: tap “Money” · scroll · scroll · say “Thanks Sam, I'm in! I see the Money page has Friday's pizza on it. Who paid, and how much was it? I'll add myself once I know what my share is.” · wait · tell
- `ana` run 2, Ana: failed, stopped before their goal. wordy {"words":40,"budget":20}; their last acts: look · scroll · say “Thanks Sam! I'm in the house app now, looking at the Money section to see my half of the pizza.” · tap “Money” · scroll · tap “Say you were in on pizza”
- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: look · tap “Money” · scroll · type “Friday's pizza (Kim and Ana)” into “What it was for”
- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":90,"budget":30}; their last acts: tap “Ana” · tap “Add it, paid by you” · look · scroll · say “Hi Ana, I've put Friday's pizza ($36, split between us) into the house money. You owe me $18.00. It will show as owed once you open Money and say you were in on it. Thanks! Sam, it's in.” · tell
- `ana` run 3, Ana: passed, reached their goal. wordy {"words":60,"budget":20}; their last acts: look · tap “Money” · say “Hi all! I joined the house link and I'm looking at the Money page to see my half of the pizza.” · scroll · scroll · tap “Say you were in on pizza”
- `together` run 3, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: look · tap “Money” · scroll · back
- `together` run 3, Kim: failed, stopped before their goal. wordy {"words":85,"budget":30}; their last acts: tap “Ana” · tap “Add it, paid by you” · look · scroll · say “Ana, I've put Friday's pizza ($36) into the house money. Your half is $18. It only counts once you say you were in on it, so can you open Money and confirm? Thanks! Sam, it's in.” · tell
- `together` run 3, Ana: failed, stopped before their goal. wordy {"words":34,"budget":20}; their last acts: tap “Join” · look · scroll · tap “Money” · scroll · tell

## What the people said

- Ana, `ana` run 2: “too much: Mildly, yes. The app said opening Money lets it read and write that as me. That's more access than I expected just to see my share. I'd have liked to be asked first.”

## The product source

- `apps/expenses/index.js:93` holds “What it was for:”
- `demo/xenia/findings/findings.json:48` holds “What it was for:”
- `demo/xenia/findings/findings.json:67` holds “What it was for:”
- `apps/expenses/index.js:93` holds “How much:”
- `demo/xenia/findings/findings.json:48` holds “How much:”
- `demo/xenia/findings/findings.json:69` holds “How much:”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
