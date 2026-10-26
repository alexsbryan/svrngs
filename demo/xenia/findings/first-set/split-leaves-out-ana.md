# Kim expected the pizza to be split with Ana

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's): what they said. The instrument counted no fault here.

## Where

Money on Kim's computer, after "Add it, paid by you": the line "You paid $36.00 for Friday's pizza, split between Kim." and "You’re square with everyone." / "Nobody owes anybody."

## Who, and how often

Kim, 4 of 4 accounts that added the pizza (`kim` runs 2-3, `together` runs 1-2), of 6. The other two never added it (`kim` run 1 waited to be told she paid, `scene-kim-voice-and-opening.md`; `together` run 3 never found Money, `money-hard-to-find-on-home-page.md`).

## What happened

The instrument's words: `kim` runs 2 and 3 "passed, reached their goal" (the goal is her own line, "You paid $36.00 for"); `together` runs 1-2 "failed, stopped before their goal" (its goal is Ana in on the pizza, which never came).

Her last acts, `together` run 1: look · scroll down · tap "Open Money" · type the pizza and 36 · tap "Add it, paid by you" · scroll down · say in the house chat · tell.

## What the people said

- Kim, `together` run 1: "I expected the pizza to be split between you and Ana when I added it. It was split only to you, so Ana has to opt in herself." Found: "I found no way to add Ana to the split from your side."
- Kim, `kim` run 2: "I expected the $36 to be split between you and Ana. It was split only to you, since Ana has to add herself, so nobody owes anything yet."
- Kim, `together` run 2: "It split only to you, because Ana has to say she was in on it herself."
- Kim, `kim` run 3: "The page didn't ask who was in on it, and I found no way to add Ana myself."
- Each then told Ana in the chat to say she was in on it, e.g. `kim` run 3: "It's split between me and whoever says they were in on it, so you'll need to say you were in on it to get your half."

## The product source

- Each expense, split between its payer and whoever says they were in on it: `apps/expenses/index.js:58` (the line), `:59` ("Say you were in on it", shown only to someone not yet in it), `:75` ("When someone adds what they paid, it is split between them and whoever says they were in on it."); the fold that starts an expense with the payer alone, `apps/expenses/expenses.js:261`.
- The form, with no field for who was in on it: `apps/expenses/index.js` `add` ("What it was for: ", "How much: ", "Add it, paid by you").

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Should adding an expense let its payer name who was in on it, or offer the house, with each person's own yes still theirs? (capability-money's rule, that a share is its own person's, is `apps/expenses/expenses.js`'s.)
