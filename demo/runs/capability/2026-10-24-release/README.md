# capability-release, 2026-10-24

The queue's last row (`ralph/next/capability/STATE.md`): DESIGN's rules this
queue changed, rewritten in place; PLAN §2's Waits; the demo scenes on
today's paths; `RELEASES.md`; and the publish, written for the operator in
`NEEDS_HUMAN.md`, never run.

## What changed, by commit

- 7e2e3408: the trust floor's header and DESIGN §3's containment promise
  say what the floor keeps out, and what it does not (WHAT_CAN_ACT_AS_YOU
  §5). Only comment lines in the floor; `scripts/trust_check.py` passed
  after it.
- 939b0c46: DESIGN §1, §3 and §6: an entry's author is its signer, the
  keeper's cutover, a yes by first key, each yes under "What acts as you".
- 9970903a: PLAN §1 says the queue landed; §2 gains its four Waits, each
  with its trigger, and loses the phone's-write row it built.
- b52aac49: `demo/a.md`, `b.md`, `c.md`, `apps/jellyfin/ROOM.md` on
  today's Jellyfin paths; `demo/events.md` step 12 says why no answer
  comes "through Sam's computer".
- a652cc0f: `RELEASES.md`. Its body cites each claim's commit; one
  citation there is wrong: the trade's claim names "64c4e0e8", which is no
  commit. It is 7e2e3408 (above).

## Each claim in the release notes, and its row's commit

| The notes say | Row | Commit |
|---|---|---|
| the open says what an app does, and is the yes | capability-open | d55b5506 |
| "Say yes on your home page", one tap there | capability-open | d55b5506 |
| "What acts as you", each line's Stop, 12 hours on a phone | capability-what-acts | 36aa173f |
| "1 app changed. See what’s new", "Open the new version" | capability-what-acts | 0573f402, 36aa173f |
| only your own phone or computer writes as you | capability-signed | 686cc567, 1dac79bf, d7bf8f16 |
| nobody else's computer chooses what runs as you | capability-signed | dd932d60 |
| an older svrngs's entries still read "through cy’s computer", and count | capability-signed | 88353ae4, 1dac79bf |
| a phone may ask once; nobody on a computer is asked | capability-signed | dd932d60, d7bf8f16 |
| two people of one name stay two, "Ana (joined 3 Oct)" | capability-keys | a809c6d5, f7b28d01 |
| Money counts a correction only from its author | capability-money, capability-keys | fd3347ac, 1bd20af4 |
| the phone's key stays where an app's pages can't copy it | capability-signed | 0c9032ed |
| a frame or a form held to its page's rule | capability-navigation | 8185a48d, c08db75d |
| what an app you open can do, the trade | capability-release | 7e2e3408 |
| an older friend's computer: writes wait, and the words | capability-signed | 0c9032ed |
| update every computer | capability-signed | dcd8c4a3, 3caa06f5 |
| a phone's House rules agreement signed on the phone | capability-signed | 686cc567 |
| `person_key`; the record API only as you; `x-svrngs-append` | capability-keys, capability-signed | a809c6d5, 1dac79bf, ca4327a5 |
| every scene on each check | capability-scenes | f15184ce |
| the last full check, 838 s | capability-keys | c7e81354 |
| about 4 ms more on 72 ms per 2,000 entries | capability-signed | 0e20661a (`../../authorship/2026-10-24/COST.md`) |

## The size budget

`python3 scripts/lean_check.py` after the last commit here, inside the
operator's ceiling of 18,325 and 3,330 (CHARTER): this row changes no
counted line.

```
crates/svrngs-core     3298 / 3299 lines
.                     18284 / 18285 lines
```

## The close

`scripts/check.sh`, once, at the close: below, from its await log.
