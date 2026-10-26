# Words June didn't know

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's): what she said.

## Where

June's home page: "What acts as you" with "Knows who you are"; "What you lend" with "You lend nothing yet."; "Already running here" with "Look for programs running on this computer"; "House’s folder is on this computer at …".

## Who, and how often

June, 3 of 3 accounts named one or more of these (`newcomer` runs 1-3).

## What happened

Not a fault the instrument counts. In run 1 she gave up at the name field; in run 2 she joined and touched nothing else; in run 3 her patience ran out at the link boxes.

## What the people said

- June, `newcomer` run 1: "I also didn't understand words like 'lend' and 'what acts as you', so I left those alone."
- June, `newcomer` run 2: "The page was a bit technical. It had a section about programs running on the computer, and it mentioned a folder." And: "There are also buttons labelled 'Show House on a screen' and 'Open this folder'. I didn't tap either of them because I don't know what they do."
- June, `newcomer` run 3: "It also talks about lending things and about programs running on the computer, which I don't understand."

## The product source

`crates/svrngs-node/src/home/question.js:214` ("What acts as you"), `:210` (`PARTS`, "Knows who you are"); `crates/svrngs-node/src/home/home.js:247` ("You lend nothing yet."), `:208` ("Already running here", "Look for programs running on this computer"), `:100` ("Show … on a screen"), `:106` ("Open this folder").

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Which of these does a newcomer with no group need on their first visit? "Lend" is DESIGN §1's word; is it one a newcomer should meet before they've joined anything?
