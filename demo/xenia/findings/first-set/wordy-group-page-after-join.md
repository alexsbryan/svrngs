# Too many words before acting: the group's page after joining, on a phone

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's): `wordy`, words read on a screen before acting, past the person's budget.

## Where

The group's page on Ana's phone at the link, once joined: the shelf, "What acts as you" and the table, as `after-join-reads-as-permissions.md` describes.

## Who, and how often

Ana, 5 of 6 accounts: `ana` 3 of 3 runs, `together` 2 of 3 (run 3 never opened the link). Budget 20.

## What happened

The instrument's words, each `{kind: "wordy", screen, words, budget}`, with the last acts on that screen:

- `ana` run 1: 120 words. look · scroll down · say · wait 20.
- `ana` run 2: 63 words. scroll down · say · wait 30 · tell.
- `ana` run 3: 100 words. say · look · scroll down · tap "Bring the ticked ones to House".
- `together` run 1: 80 words. look · scroll down · look · tell.
- `together` run 2: 120 words. look · say · wait 30 · look.

## What the people said

- Ana, `ana` run 3: "The text was tiny and some of it was cut off, so I couldn't read all of it."
- Ana, `together` run 2: "The text was small and partly cut off, and it said there was more I'd skimmed past. I didn't finish reading it."

## The product source

`crates/svrngs-node/src/home/question.js:177-183` (the shelf), `:210-214` ("What acts as you"), `:31` (Money's row); on the phone `web/app.js:488`, `:502`, `:506`, `:539`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Which words on this screen does someone here for this one errand need before they act?
