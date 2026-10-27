# Too many words before acting: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`
- kind: hard (the instrument's): `wordy`; never both

## Where

Home. What was read there first: Money / Open this folder / Invite someone to House / Add your phone or another computer / Have a guest / Your groups

## Who, and how often

- `kim`: 1 of 3 accounts (Kim run 2)

## What happened

- `kim` run 2, Kim: passed, reached their goal. wordy {"words":60,"budget":30}; their last acts: look · tap “Open this folder” · look · tap “Money”

## What the people said

None.

## The product source

- `crates/svrngs-node/src/home/home.js:130` holds “Open this folder”
- `crates/svrngs-node/tests/browser/main.rs:3504` holds “Open this folder”
- `crates/svrngs-node/tests/browser/main.rs:3543` holds “Open this folder”
- `RELEASES.md:10` holds “Invite someone to House”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
