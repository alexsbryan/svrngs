# Too much: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`, folded again by xenia `b54b5b3`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

the group's page. What was read there first: Money / Bring more to House / What acts as you / You’re in House, as Ana / Who owes whom, and for what.

## Who, and how often

- `ana`: 1 of 3 accounts (Ana run 2)
- `together`: 1 of 6 accounts (Ana run 2)

## What happened

- `ana` run 2, Ana: failed, stopped before their goal. their last acts: tap “Say you were in on pizza” · tell · tell · tell · tell · tell
- `together` run 2, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell

## What the people said

- Ana, `ana` run 2: “Mildly, yes. The app said opening Money lets it read and write that as me. That's more access than I expected just to see my share. I'd have liked to be asked first.”
- Ana, `together` run 2: “Yes, one thing. Money wants to read and write what's owed, as me, as soon as I open it. That's more than I'd hand over without asking the others first.”

## The product source

- `demo/xenia/findings/findings.json:8` holds “Bring more to House”
- `demo/xenia/findings/findings.json:34` holds “Bring more to House”
- `demo/xenia/findings/findings.json:35` holds “Bring more to House”
- `RELEASES.md:40` holds “What acts as you”
- `apps/jellyfin/ROOM.md:17` holds “What acts as you”
- `crates/svrngs/tests/join_page.rs:206` holds “What acts as you”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
