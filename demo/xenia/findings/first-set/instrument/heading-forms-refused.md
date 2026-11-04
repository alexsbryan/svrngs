# Instrument fault: June named the heading, and the stage refused every form but one

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's own): a fault in xenia or in a scene's world, not in svrngs.

## Where

June's home page, the join's "Your name in it: " field.

## The xenia side

`src/stage.ts` at `57d0da1`: two fields of one name answer "There's more than one field here called “Your name in it”: one under “Make a group”, one under “Join a group from a link”. Say which, with its heading." Only the exact form `Your name in it: under Join a group from a link` is taken (a scripted June, `7341378c`'s body). June's ten other forms were each answered "There's no field here called …".

## Who, and how often

June, 1 of 3 accounts (`newcomer` run 1).

## What happened

"failed, gave up". Her acts after typing the link: type "June" into "Your name in it" (the stage's choices) · then into "Your name in it under \"Join a group from a link\"", "Join a group from a link", "Your name in it, under the heading “Join a group from a link”", "Your name in it (Join a group from a link)", "Join a group from a link / Your name in it", "Join a group from a link Your name in it", "Your name in it under Join House" · look · tap "Your name in it:" · type into "Your name in it, Join House" · tap "Join House" ("a name cannot be empty") · type into "Your name in it, under Join a group from a link", "Your name in it under “Join a group from a link”", "Your name in it under “Join House”" · give up.

## What the people said

- June, `newcomer` run 1, giving up: "There are two boxes called 'Your name in it', and every way I tried to say which one was refused."

## The product source

The twin labels are svrngs's (`../two-link-boxes.md`): `crates/svrngs-node/src/home/home.js:367`, `:376`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- For the xenia worker: take the heading however a person writes it (quotes, a comma, "under the heading", the heading first).
