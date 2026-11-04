# Too many words before acting: June's home page, in no group

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's): `wordy`, words read on a screen before acting, past the person's budget.

## Where

June's home page before she has joined: "Your groups", "What acts as you", "Open a link from your group", "Make a group", "Join a group from a link", "Already running here", "What you lend".

## Who, and how often

June, 3 of 3 accounts (`newcomer` runs 1-3). Budget 40.

## What happened

The instrument's words, each `{kind: "wordy", screen, words, budget}`, with the last acts on that screen:

- `newcomer` run 1: 80 words. look · scroll down · type the link into "The link someone sent you".
- `newcomer` run 2: 80 words. look · scroll down · type the link into "The link someone sent you:".
- `newcomer` run 3: 120 words. look · scroll down · scroll up · type the link into "The link".

## What the people said

- June, `newcomer` run 3: "The page was confusing. It has several boxes that look alike."
- June, `newcomer` run 2: "The page was a bit technical."

## The product source

`crates/svrngs-node/src/home/home.js:337-386` (the page with no group, in that order), `crates/svrngs-node/src/home/index.html:15-17`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Which words on this screen does someone here for this one errand need before they act?
