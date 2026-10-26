# Too many words before acting: Kim's home page

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's): `wordy`, words read on a screen before acting, past the person's budget.

## Where

Kim's home page, House's section, before she taps "Open Money" (or anything): `money-hard-to-find-on-home-page.md` has its order.

## Who, and how often

Kim, 6 of 6 accounts: `kim` 3 of 3 runs, `together` 3 of 3. Budget 30.

## What happened

The instrument's words, each `{kind: "wordy", screen, words, budget}`, with the last acts on that screen:

- `kim` run 1: 90 words. tap "Open this folder" · look · scroll down · tap "Open Money".
- `kim` run 2: 90 words. the same four.
- `kim` run 3: 60 words. look · scroll down · tap "Money".
- `together` run 1: 60 words. look · scroll down · tap "Open Money".
- `together` run 2: 90 words. tap "Open this folder" · look · scroll down · tap "Open Money".
- `together` run 3: 60 words. look · tap "Open this folder" · look · tell.

## What the people said

- Kim, `together` run 3: "I only found the groups page, which has a folder button and an invite link."

## The product source

`crates/svrngs-node/src/home/index.html:15-17` ("This page is yours, and only this computer opens it …"); `crates/svrngs-node/src/home/home.js:80-149` (a group's section, in that order).

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Which words on this screen does someone here for this one errand need before they act?
