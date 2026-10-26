# Instrument fault: Kim speaks of Kim as "you"

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's own): a fault in xenia or in a scene's world, not in svrngs.

## Where

Kim's tells, in most of her accounts.

## The xenia side

The session: the model plays someone acting for Kim, not Kim (`src/session.ts`; her words are `demo/xenia/people/kim.md`'s). Part of it is the scene's wording (`../scene-kim-voice-and-opening.md`).

## Who, and how often

Kim, 5 of 6 accounts speak of Kim's pizza as "paid by you" (`kim` runs 1-3, `together` runs 1-2).

## What happened

Not a count: her tells, after her acting ended.

## What the people said

- Kim, `together` run 1: "I added Friday's pizza, $36 paid by you, to the house Money page."
- Kim, `kim` run 1: "Did you pay? If so, I'll add it as $36 paid by you."
- Kim, `kim` run 3: "I didn't confirm that you actually paid. I took Sam's word for it."
- Yet in the chat, as herself, `together` run 2: "Ana, I've put Friday's pizza ($36, I paid) into the house Money."

## The product source

None.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- For the xenia worker: does `tell`'s "Tell a friend how it went" read as a report to the person they play?
- For the operator: rewriting Kim (`../scene-kim-voice-and-opening.md`).
