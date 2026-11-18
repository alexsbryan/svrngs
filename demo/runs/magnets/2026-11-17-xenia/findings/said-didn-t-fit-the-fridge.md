# Didn't fit together: the fridge

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-18T00-56-56-346Z`, 1 run(s) by claude-sonnet-5-5, as claude -p, xenia `df729c3`, the product `ba3e9cff`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-18T00-56-56-346Z`, 1 run(s) by claude-sonnet-5-5, as claude -p, xenia `df729c3`, the product `ba3e9cff`
- kind: soft (the person's): what they said when asked "didn't fit"; never both

## Where

the fridge. What was read there first: goose / invoice / moon / group chat / landlord / lasagna

## Who, and how often

- `solo`: 1 of 3 accounts (Player run 1)

## What happened

- `solo` run 1, Player: passed, explored, and said they were done. their last acts: tap “situation” · tell · tell · tell · tell · tell
- `solo` run 1, Player: passed, explored, and said they were done. their last acts: tap “somehow” · tell · tell · tell · tell · tell
- `solo` run 1, Player: passed, explored, and said they were done. their last acts: scroll · tell · tell · tell · tell · tell

## What the people said

- Player, `solo` run 1: “Nothing really. The word tiles kept reordering after each tap and the sentence box was a bit small, but it worked.”
- Player, `solo` run 1: “Nothing really broke. "somehow" after "situation" was a bit odd, and there was no clear way to finish or send the sentence.”
- Player, `solo` run 1: “The odd thing was that Sam's dinner message turned up on top of a word game. It's from the house chat but sits on the magnets screen. Also, the sentence is shared, so a tap would change it for everyone, and nothing said so until I scrolled.”

## The product source

- `apps/magnets/pack.json:29` holds “somehow”
- `crates/svrngs-core/src/admit.rs:716` holds “somehow”
- `ralph/next/canon/ctl/log/sessions/canon-pin-1.out:1222` holds “somehow”
- `apps/magnets/pack.json:14` holds “situation”
- `ralph/next/oidc/ctl/log/sessions/REVIEW-audit-oidc-auto-1-1.out:3107` holds “situation”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/apps/magnets/tests/xenia --runs 1 --model claude-sonnet-5-5 --people /Users/alexsbryan/dev/svrngs/apps/magnets/tests/xenia/people --findings /Users/alexsbryan/dev/svrngs/demo/runs/magnets/2026-11-17-xenia/findings --source /Users/alexsbryan/dev/svrngs
```
