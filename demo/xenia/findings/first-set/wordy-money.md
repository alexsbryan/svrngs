# Too many words before acting: Money, on Kim's computer

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's): `wordy`, words read on a screen before acting, past the person's budget.

## Where

Money in its own tab on Kim's computer: "Money", "Who pays whom", "What people paid for", the sentence "When someone adds what they paid …", "Add what you paid" and its form.

## Who, and how often

Kim, 3 of 6 accounts: `kim` 1 of 3 runs (run 3), `together` 2 of 3 (runs 1-2). Budget 30.

## What happened

The instrument's words, each `{kind: "wordy", screen, words, budget}`, with the last acts on that screen:

- `kim` run 3: 60 words. tap "Money" · scroll down · type "Friday's pizza (Kim and Ana)" into "What's it for".
- `together` run 1: 60 words. tap "Add it, paid by you" · scroll down · say · tell.
- `together` run 2: 60 words. tap "Add it, paid by you" · look · say · tell.

## What the people said

None said Money was too much: "None" in all three.

## The product source

`apps/expenses/index.js:68-77` (the page as drawn), `:75` (the sentence on how a split works).

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Which words on this screen does someone here for this one errand need before they act?
