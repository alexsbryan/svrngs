# A page never settled: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: hard (the instrument's): `unsettled`; never both

## Where

Money. What was read there first: nothing

## Who, and how often

- `kim`: 2 of 3 accounts (Kim run 1, Kim run 3)
- `together`: 1 of 6 accounts (Kim run 1)

## What happened

- `kim` run 1, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · scroll · look · scroll · scroll · tap “Open Money”
- `together` run 1, Kim: failed, ran out of patience. unsettled {}; their last acts: tap “Open Money” · scroll · tap “Open it” · scroll · scroll · tap “Open Money”
- `kim` run 3, Kim: failed, ran out of patience. unsettled {}; their last acts: back · scroll · scroll · tap “Bring things to the table” · scroll · tap “Open Money. It reads and writes who paid for what, and who owes whom.”

## What the people said

- Kim, `kim` run 1: “almost left: Yes. I nearly left when the Money link wouldn't open and the page was a long list of buttons with no clear way to the expense form.”
- Kim, `together` run 1: “almost left: Yes, when "Open Money" did nothing visible. I couldn't tell whether it had worked or whether I was missing a step, such as bringing Money to House first. Then the session ended.”
- Kim, `kim` run 3: “almost left: Yes. I got stuck looking for how to add an expense, and then the session ended after I tapped Open Money.”

## The product source

None found by its exact text.

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
