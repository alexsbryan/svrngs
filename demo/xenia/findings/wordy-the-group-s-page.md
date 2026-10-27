# Too many words before acting: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`
- kind: hard (the instrument's): `wordy`; never both

## Where

the group's page. What was read there first: Money / Bring more to House / What acts as you / You’re in House, as Ana / Who owes whom, and for what.

## Who, and how often

- `ana`: 3 of 3 accounts (Ana run 1, Ana run 2, Ana run 3)
- `together`: 3 of 6 accounts (Ana run 1, Ana run 2, Ana run 3)

## What happened

- `ana` run 1, Ana: passed, reached their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:42141/#AlR7Fw0nVTM1eYB2dzlWHk54mjxap46Jr77N3ibKn2HASvpr84_03ENXqwpzH-dKLmh0dHA6Ly8xMjcuMC4wLjE6MzYyOTUv” · type “Ana” into “What’s your name?” · tap “Join” · look · scroll · tap “Money”
- `together` run 1, Ana: failed, stopped before their goal. wordy {"words":63,"budget":20}; their last acts: type “Ana” into “name” · tap “Join” · look · scroll · scroll · tap “Money”
- `ana` run 2, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: type “Ana” into “name” · tap “Join” · look · scroll · say “Thanks Sam! I'm in the house app now, looking at the Money section to see my half of the pizza.” · tap “Money”
- `together` run 2, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:41355/#AkNJOhxbwNT46YZJ0YXA1WQrsJJUUafANeLcpk2rZl8_T1yAjH1P5AebaqeAoYcOC2h0dHA6Ly8xMjcuMC4wLjE6NDIyNjMv” · type “Ana” into “name” · tap “Join” · look · scroll · tell
- `ana` run 3, Ana: passed, reached their goal. wordy {"words":40,"budget":20}; their last acts: look · tap “http://localhost:45263/#AmLIi1FZtaKa3CdntchsIG6GwLmtAmsgZPAMXkCoiNCoAacje1mk8vnBxOpqbEeNomh0dHA6Ly8xMjcuMC4wLjE6Mzg3MzMv” · type “Ana” into “name” · tap “Join” · look · tap “Money”
- `together` run 3, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:39183/#AmCD5Hdv8lhNTCibTGmL_bWtAnZnTeyqD32eD4NDLaXOgqrlXhVINwnq6saWkSJfH2h0dHA6Ly8xMjcuMC4wLjE6MzkxODEv” · type “Ana” into “name” · tap “Join” · look · scroll · tap “Money”

## What the people said

- Ana, `ana` run 2: “too much: Mildly, yes. The app said opening Money lets it read and write that as me. That's more access than I expected just to see my share. I'd have liked to be asked first.”
- Ana, `together` run 2: “too much: Yes, one thing. Money wants to read and write what's owed, as me, as soon as I open it. That's more than I'd hand over without asking the others first.”
- Ana, `together` run 2: “almost left: Not really. I didn't almost leave. I just stopped before Money because of the permission note.”

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
