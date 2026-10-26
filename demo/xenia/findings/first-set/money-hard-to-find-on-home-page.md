# Money is hard to find on the home page

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's): what they did and said. The words Kim read there before acting are hard faults, `wordy-kim-home-page.md`.

## Where

Kim's home page, House's section. Money's row ("Open Money. It reads and writes who paid for what, and who owes whom.") is in the table below "Show House on a screen", "Open this folder", "House’s folder is on this computer at …", the three link buttons and "Leave House", so off the first screen.

## Who, and how often

Kim, 4 of 6 accounts tapped "Open this folder" before Money, or instead of it (`kim` runs 1-2, `together` runs 2-3). In 1 of 6 (`together` run 3) she never found Money.

## What happened

The instrument's words, `together` run 3: "failed, stopped before their goal". Her acts: look · tap "Open this folder" · look · tell. In `kim` runs 1-2 and `together` run 2: tap "Open this folder" · look · scroll down · tap "Open Money".

## What the people said

- Kim, `together` run 3: "I found no place to record house money. The screen only shows the groups page with an 'Open this folder' button." Almost left: "After the folder opened and I found no money page, I nearly gave up."
- Kim, `together` run 3, expected: "I expected to find a house-money or expenses page where I could log the pizza."

## The product source

The order of a group's section, `crates/svrngs-node/src/home/home.js`: "Show … on a screen" `:100`, "Open this folder" `:106`, the link buttons `:120`, leaving `:128`, then the table of what is in the group `:149`, its rows' opener `:54` and words `question.js:31`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Should what the group has (its table) come before the folder and the invitations on the home page?
