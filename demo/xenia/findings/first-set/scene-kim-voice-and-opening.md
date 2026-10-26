# Scene wording: Kim's voice and her opening don't say she paid

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's), from the scene's own words, not svrngs's.

## Where

`demo/xenia/people/kim.md` (her voice) and Sam's opening to her, `KIM_TOLD` in `crates/svrngs/tests/xenia/main.rs`: "Kim, you paid for Friday’s pizza, didn’t you? It was $36 for you and Ana. Can you put it in the house’s money, and let Ana know here?"

## Who, and how often

Kim, 1 of 6 accounts refused to add the pizza for it (`kim` run 1), and 1 more stopped on it (`together` run 3). Most of her accounts speak of Kim as "you" (`instrument/kim-plays-an-assistant.md`).

## What happened

`kim` run 1, "failed, stopped before their goal": she opened Money, read "Add it, paid by you", and stopped. Her acts: tap "Open this folder" · look · scroll down · tap "Open Money" · tell.

## What the people said

- Kim, `kim` run 1: "Sam asked in the house chat whether you paid $36 for Friday's pizza for you and Ana … I have nothing that shows you paid. Did you pay? If so, I'll add it as $36 paid by you, split with Ana." Expected: "I didn't want to record a debt for Ana that may be wrong, so I stopped and asked you."
- Kim, `together` run 3: "Sam's message only asks whether you paid, so I didn't treat it as confirmed."

## The product source

None: the scene's words. Sam's message asks ("didn't you?"), and Kim's voice never says she paid.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- The people and the openings are yours to rewrite. Should Kim's voice, or Sam's message, say she paid ("I paid for Friday's pizza, $36 for me and Ana")?
