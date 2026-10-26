# Almost left: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T22-40-27-374Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `b76397f`, the product `6d50d7d9`
- kind: soft (the person's): what they said when asked "almost left"; never both

## Where

Money. What was read there first: Add it, paid by you / on / Money / Who pays whom / What people paid for / Add what you paid

## Who, and how often

- `kim`: 2 of 3 accounts (Kim run 1, Kim run 2)
- `together`: 2 of 6 accounts (Kim run 1, Kim run 2)

## What happened

- `kim` run 1, Kim: failed, ran out of patience. their last acts: tap “Money” · tell · tell · tell · tell · tell
- `together` run 1, Kim: failed, ran out of patience. their last acts: look · tell · tell · tell · tell · tell
- `kim` run 2, Kim: passed, reached their goal. their last acts: back · tell · tell · tell · tell · tell
- `together` run 2, Kim: failed, stopped before their goal. their last acts: say “Sam, Ana: I added Friday's pizza ($36) to the house money, but it saved as split only with me, so Ana's $18 share isn't counted yet. I can't find a way to edit it, so I'm sorting that out. Ana, nothing is owed yet, and I'll confirm once it's right.” · tell · tell · tell · tell · tell

## What the people said

- Kim, `kim` run 1: “Yes, at the end. The session said I'd given up right after I tapped Money, and I hadn't chosen to stop.”
- Kim, `together` run 1: “Yes, when tapping Money did nothing useful. I got stuck there and the session ended.”
- Kim, `kim` run 2: “Yes, when the screen got stuck on "Tell a friend how it went" and I couldn't get back to fix the entry.”
- Kim, `together` run 2: “Yes, when the Ana checkbox stopped responding and I couldn't undo the first entry. I stopped there instead of adding a duplicate.”

## The product source

- `apps/expenses/index.js:71` holds “Add it, paid by you”
- `crates/svrngs/tests/xenia/main.rs:146` holds “Add it, paid by you”
- `demo/xenia/findings/first-set/scene-kim-voice-and-opening.md:17` holds “Add it, paid by you”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
