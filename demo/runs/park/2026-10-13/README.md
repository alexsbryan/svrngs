# relay-park (c)'s run — the shipped command's link, through n0, in both engines

**Passed, on one host, one run of each engine. That is not a rate
(AGENTS.md 7).** The command as shipped (`pub const SHIPPED: MeetAt =
MeetAt::N0;`) ran `new` and printed a link. The link carried one of n0's
relays. The page, in Chromium and in WebKit as an iPhone, joined through
that relay and listed the group.

Driver: `run.py`, at 46b3b59. The predictions were committed before any
data, at a0663e1 (`PREDICTIONS.md`). Data: `data/`. Run on 2026-10-13
from 07:34:48 to 07:35:05, local time.

## The first attempt

The first attempt ran at a0663e1 and judged nothing. Its `summary.json`
said `"could-not-judge"`, `"the binary listed no n0 relay"`. The cause
was in the driver: the binary's traces carry ANSI colours even into a
pipe, so the driver never matched `relay_urls=`. 46b3b59 strips the
colours and changes nothing else. That attempt's two files are not kept.

## What it showed, against each prediction

1. **n0 answers: held.** All four of n0's relays answered `/ping`
   (`data/n0.json`: `answered`, each `true`). They are listed as the
   binary lists them, dotless, with the address each resolved to:

   | relay | address |
   |---|---|
   | aps1-1.relay.n0.iroh.link | 5.223.65.62 |
   | euc1-1.relay.n0.iroh.link | 91.99.237.97 |
   | use1-1.relay.n0.iroh.link | 87.99.151.110 |
   | usw1-1.relay.n0.iroh.link | 5.78.69.43 |

2. **The link carries one of n0's relays: held.** In both engines' runs,
   the relay read from the token was `https://usw1-1.relay.n0.iroh.link/`,
   which was predicted (`data/summary.json`: `carried`, with
   `carried_is_n0` `true`). The binary's own trace said the same
   (`data/chromium.svrngs.err`):

   ```text
   DEBUG cli: cli: the relays this computer meets its groups at at=N0 unkept=false relay_urls=["https://aps1-1.relay.n0.iroh.link/", ...]
   DEBUG cli: cli: waited for a relay online=true tried=[...]
   DEBUG cli: cli: the link carries this computer's relay relay=https://usw1-1.relay.n0.iroh.link/ n0=true
   ```

   The link was printed 1,191 ms after the start in Chromium's run and
   1,003 ms in WebKit's (`link_ms`), against a prediction of under 15 s.
   The command printed (`data/chromium.svrngs.out`):

   ```text
   Made house, with you as Sam.
   Send this link to the people you want in it. It works for a day, while this computer is on:

     svrngs://join/AjPHbsFooVGHZuzpbQ6LJhOlAx-gDEoLdngFbGIgM_SHzeahnmVJS-ZLDV7abpTzs2h0dHBzOi8vdXN3MS0xLnJlbGF5Lm4wLmlyb2gubGluay8

   or, to open it in a browser:

     http://localhost:50360/#AjPHbsFooVGHZuzpbQ6LJhOlAx-gDEoLdngFbGIgM_SHzeahnmVJS-ZLDV7abpTzs2h0dHBzOi8vdXN3MS0xLnJlbGF5Lm4wLmlyb2gubGluay8
   ```

   The token's tail is the base64url of `https://usw1-1.relay.n0.iroh.link/`.

3. **The page joins through it, in both engines: held.** Each engine was
   asked for a name, was given Bo, and said each of these steps in turn
   (`said`):

   > Opening… · Opening your link… · Join house · Joining house… ·
   > Reaching house… · You’re in house, as Bo

   Opened again by the link, and then alone, the page said "You’re in
   house, as Bo" without asking again (`again`, `alone`). From the link
   handed over to the page's last line took 5,256 ms in Chromium and
   5,608 ms in WebKit (`page_ms`), against a prediction of under 30 s.
   `page_exit` was 0 and `svrngs_exit` 0 in both runs. The page's code is
   ab99c34's: relay-park changed none of it.

4. **Sam's computer is reached through n0: held.** The svrngs process was
   sampled five times in each run. It opened two remote addresses
   (`remote`):

   | address | what it is |
   |---|---|
   | 5.78.69.43:443 | usw1-1, the n0 relay its link carries (`n0_hits`) |
   | 49.13.207.244:443 | dns.iroh.link (`dig +short dns.iroh.link` answered 49.13.207.244 after the run): n0's lookup, which this computer publishes to |

   It opened nothing else. The page's own connections were not sampled:
   `lsof` cannot see a browser's network process (as
   demo/runs/c/no-n0-2026-10-13/ found), and this run did not route the
   page's requests.

## What this run does not show

- That a phone's page asks no n0 lookup. That is what the page's code
  does, and it is unchanged (`web/src/lib.rs` `bind_for`; the browser
  test `away`, relay-default (b)). This run did not measure it.
- A computer joining by the link, and a lend through n0's relay. Demo C's
  dry runs, through n0, did those (`demo/runs/c/dry-2026-10-12-rerun/`).
- Any rate. One host and one run of each engine are not a rate.
