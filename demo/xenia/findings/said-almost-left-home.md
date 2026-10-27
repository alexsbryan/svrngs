# Almost left: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`
- kind: soft (the person's): what they said when asked "almost left"; never both

## Where

Home. What was read there first: The link someone sent you: / Your name in it: / A name for the group: / Your name in it: / Make a group / Your groups

## Who, and how often

- `newcomer`: 1 of 3 accounts (June run 3)

## What happened

- `newcomer` run 3, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell

## What the people said

- June, `newcomer` run 3: “No. Kim had sent the link in the house chat and said she'd set this up, so I wasn't worried.”

## The product source

- `crates/svrngs-node/src/home/home.js:408` holds “The link someone sent you:”
- `crates/svrngs-node/tests/browser/main.rs:2697` holds “The link someone sent you:”
- `demo/xenia/findings/findings.json:173` holds “The link someone sent you:”
- `crates/svrngs-node/src/home/home.js:408` holds “Your name in it:”
- `crates/svrngs-node/src/home/home.js:421` holds “Your name in it:”
- `crates/svrngs-node/tests/browser/main.rs:2697` holds “Your name in it:”
- `crates/svrngs-node/src/home/home.js:421` holds “A name for the group:”
- `ralph/next/arch/arch-command/b-wip.patch:241` holds “A name for the group:”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
