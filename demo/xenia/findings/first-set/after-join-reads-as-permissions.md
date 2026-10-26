# The page after joining reads as a permissions grant

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's): what they said. No hard fault the instrument counted is this finding; the words Ana read before acting there are `wordy-group-page-after-join.md`.

## Where

The group's page on Ana's phone, just after she taps Join: "You’re in House (Kim’s, since 26 Oct), as Ana", above the sections "Bring things to the table" (the shelf, its boxes ticked, and the button "Bring the ticked ones to House") and "What acts as you" ("Reads and writes for you", "Knows who you are", "Can decide for you"). Money's own row, "Open Money. It reads and writes who paid for what, and who owes whom.", is on the same screen, and is the first thing she reads.

## Who, and how often

Ana, 5 of 6 accounts (`ana` runs 1-3, `together` runs 1-2). The sixth, `together` run 3, never opened the link (`instrument/ana-never-tapped-the-link.md`). In none of the 6 did Ana open Money.

## What happened

The instrument's words: `ana` run 1 "failed, ran out of patience"; runs 2 and 3 "failed, stopped before their goal"; `together` run 1 "stopped before their goal", run 2 "ran out of patience".

What she read first after joining, in every one of the five (trace, the look after Join): `Open Money. It reads and writes who paid for what, and who owes whom. / Bring the ticked ones to House / on / on / You’re in House (Kim’s, since 26 Oct), as Ana / Bring things to the table / What acts as you`.

Her last acts, `ana` run 1: look · tap the link · type "Ana" into "What’s your name?" · look · tap "Join" · look · scroll down · look · scroll down · say in the house chat · wait 20. In `ana` run 3 she tapped "Bring the ticked ones to House" before stopping.

## What the people said

- Ana, `ana` run 1: "It then asked me to let Money and House read and write for me and 'decide for me', with those toggles already on. I didn't agree to that." Too much: "The permissions were too much for seeing one bill. They were switched on already, and they covered writing on my behalf and deciding for me." Almost left: "Yes, at the permission screen. I wasn't sure what I was agreeing to, so I stopped and asked in the chat."
- Ana, `ana` run 2: "'Can decide for you' and 'Knows who you are' seemed like too much just to see a bill split. The screen also says everyone gets access when one is brought, and I couldn't read all of it."
- Ana, `ana` run 3: "Write access to who owes whom, and 'can decide for you', are more than I need to view one bill. I'd want read-only. I also couldn't tell which toggles were already switched on, and 'Bring the ticked ones' may have sent some of them already."
- Ana, `together` run 1: "a page asked me to 'bring' Money to House, with two options already ticked … That's a lot to hand over on the first screen after joining."
- Ana, `together` run 2: "'Can decide for you' and write access to the money records felt like too much for joining a house. They were already switched on, and the text was hard to read. I'd want someone to explain it before I agree."
- Ana in the house chat, `ana` run 2: "it's asking me to bring Money to House, with things like 'reads and writes for you' and 'can decide for you' ticked. Is that normal? Which should I leave on just to see my half of the pizza?"

## The product source

- The shelf, its boxes ticked by default: `crates/svrngs-node/src/home/question.js:177` (`checked: true`), its heading `:179` ("Bring things to the table"), its button `:183` ("Bring the ticked ones to ${name}"); drawn on the phone by `web/app.js:502`. Money is on House's table already, so what is ticked is the other shelf apps, which Ana read as Money.
- "What acts as you" and its parts: `question.js:214` (the heading) and `:210` (`PARTS`: "Reads and writes for you", "Opens friends’ things as you", "Knows who you are", "Open to this WiFi", "Can decide for you"); built on the phone at `web/app.js:506`.
- Money's row: `question.js:31` (`agreed`: "Open ${named(l)}. It reads and writes …"), its row on the phone `web/app.js:539`.
- "You’re in …": `web/app.js:488`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Should a newcomer's first screen after joining show the shelf and "What acts as you" at all, or lead with what the group has for them?
- Should the shelf's boxes start ticked?
- Is "It reads and writes who paid for what" on Money's own button read as a request for permission? Ana read it so, 5 of 5 times she saw it.
