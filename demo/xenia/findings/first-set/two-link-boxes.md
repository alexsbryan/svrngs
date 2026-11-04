# Two look-alike link boxes on the home page

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's): what she did and said. The instrument counted no fault for it.

## Where

June's home page, no group yet: "Open a link from your group", its field "The link: " and button "Open it", above "Make a group" and then "Join a group from a link", whose field is "The link someone sent you: ". In "Make a group" and "Join a group from a link" a second pair look alike too: both have a field "Your name in it: ".

## Who, and how often

June, 1 of 3 accounts (`newcomer` run 3) put Kim's link in the wrong box. In 1 of 3 (`newcomer` run 1) the two "Your name in it" fields stopped her; that run is also an instrument fault, `instrument/heading-forms-refused.md`.

## What happened

The instrument's words, `newcomer` run 3: "failed, ran out of patience". Her last acts: look · scroll down · scroll up · type the link into "The link" · tap "Open it" (the page: "That is not a link to something lent in your groups.") · scroll down · type the link into "The link someone sent you" (patience spent) · type "June" into "Your name in it".

## What the people said

- June, `newcomer` run 3: "I pasted it into the 'Open a link from your group' box and it said 'That is not a link'." Too much: "It has several boxes that look alike, and I couldn't tell which one the link belonged in." Almost left: "I almost left when the link was rejected as 'not a link'. I wasn't sure whether I'd done something wrong."
- June, `newcomer` run 1: "The page has two boxes both labelled 'Your name in it'. One is under 'Make a group' and the other is under 'Join a group from a link'." Too much: "The two boxes with the same label were confusing."

## The product source

`crates/svrngs-node/src/home/home.js`: "Open a link from your group" `:355` (its field "The link: "), `:359` ("That is not a link to something lent in your groups."), `:362` (its heading); "Make a group" `:367` (its "Your name in it: "), `:371`; "Join a group from a link" `:376` ("The link someone sent you: ", and a second "Your name in it: "), `:384`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Should a link pasted into "Open a link from your group" that is a join link be taken as one, or the box say where it goes?
- Should the join's name field have a label of its own?
