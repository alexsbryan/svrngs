# Too many words before acting: House on a screen

- status: open, not seen since the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- kind: hard (the instrument's): `wordy`; never both

## Where

House on a screen. What was read there first: House / (yours, since 27 Oct) / Since this screen opened / Kim / 1 person · 1 on now · 0 things lent / on now

## Who, and how often

- `together`: 1 of 6 accounts (Kim run 2)

## What happened

- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: look · tap “Show House on a screen” · scroll · back

## What the people said

None.

## The product source

- `crates/svrngs-node/src/home/home.js:310` holds “Since this screen opened”
- `crates/svrngs-node/tests/browser/main.rs:3118` holds “Since this screen opened”
- `demo/xenia/findings/findings.json:442` holds “Since this screen opened”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
