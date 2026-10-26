# Too many words before acting: House on a screen

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T22-40-27-374Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `b76397f`, the product `6d50d7d9`
- kind: hard (the instrument's): `wordy`; never both

## Where

House on a screen. What was read there first: House / (yours, since 26 Oct) / Since this screen opened / Kim / 1 person · 1 on now · 0 things lent / on now

## Who, and how often

- `together`: 1 of 6 accounts (Kim run 2)

## What happened

- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: look · tap “House” · scroll · back

## What the people said

- Kim, `together` run 2: “almost left: Yes, when the Ana checkbox stopped responding and I couldn't undo the first entry. I stopped there instead of adding a duplicate.”

## The product source

- `demo/xenia/findings/findings.json:462` holds “(yours, since 26 Oct)”
- `demo/xenia/findings/findings.json:493` holds “(yours, since 26 Oct)”
- `demo/xenia/findings/said-almost-left-house-on-a-screen.md:10` holds “(yours, since 26 Oct)”
- `crates/svrngs-node/src/home/home.js:294` holds “Since this screen opened”
- `crates/svrngs-node/tests/browser/main.rs:3113` holds “Since this screen opened”
- `demo/xenia/findings/findings.json:462` holds “Since this screen opened”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
