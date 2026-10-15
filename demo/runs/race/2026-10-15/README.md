# v1ui-race (a): the phone dials every computer at once

One host, one session, 2026-10-15, load 3.7-5.2 on 12 cores. One host is not
a rate (AGENTS.md 7). Each file here is the output of the command it names,
kept as it ran.

## What changed

Before, the page dialled the computer that made its link and waited up to its
cap, 10 s (`dial::LINK_WITHIN`), before it dialled anyone else
(`web/app.js` `reachGroup` at e3cdb97). Now it dials that computer and every
lender it knows at once, from its one endpoint, and goes through the first to
answer. The link's computer keeps its 10 s and each lender keeps its 4 s.
A dial that answers late is closed.

## With the link's computer asleep

`cargo test -p svrngs-node --test browser -- the_phone_reaches_whoever_s_awake
the_phone_goes_through an_app_one_computer_brought a_computer_that_did_not_answer
--nocapture`

| | at e3cdb97 (`red-at-head.out`) | after (`green.out`) |
|---|---|---|
| cy's shelf listed, chromium / webkit | 13,959 / 13,992 ms | 3,364 / 3,395 ms |
| cy's shelf opened | 14,407 / 14,553 ms | 3,809 / 3,962 ms |
| Money opened and wrote, through cy's | 14,470 / 14,606 ms | 3,820 / 3,942 ms |
| both computers asleep, told | (not taken) | 10,439 / 10,544 ms |

The bar is 15 s (V1_UI_BUILD §7, 3.3). The test now asks for 10 s, the link's
cap, which a page dialling one computer after another cannot meet: it waits
all 10 s before it asks the next. Of what is left after, 3 s is cy's computer
asking ada's, which does not answer (`node::ANSWER_WITHIN`).

## With the link's computer awake

Three takes each, before (`awake-at-head.out`) and after (`awake-after.out`).
The first case is `the_phone_goes_through_the_link_s_computer_while_it_answers`.
The second is the ignored instrument `the_dials_cost_with_everyone_awake`, run
with `--ignored`.

| | before | after |
|---|---|---|
| cy's asleep: listed, chromium | 3,334-3,342 ms | 3,334-3,347 ms |
| cy's asleep: listed, webkit | 3,359-3,367 ms | 3,365-3,385 ms |
| cy's asleep: through the relay | 23.9-25.6 KB, 37-41 packets, 4-5 dropped | 27.5-31.1 KB, 40-44 packets, 7-10 dropped |
| everyone awake: listed, chromium | 818-822 ms | 816-822 ms |
| everyone awake: listed, webkit | 325-340 ms | 336-351 ms |
| everyone awake: through the relay | 34.2-38.6 KB, 62-67 packets | 42.2-48.1 KB, 76-87 packets |

While the link's computer answers and cy's does not, the phone goes through
the link's computer every time. Chromium is as fast as it was before. WebKit
is up to 18 ms slower than its slowest take before. `link-only.out` shows
where that time goes: with the lenders' dials taken out and the link's dial
still from the shared endpoint, chromium lists in 3,338-3,342 ms, the same as
before. So the time is the second handshake running in the page's one thread.
It is not a wait on any computer.

With everyone awake, the page goes through whichever computer answers first:
cy's in 4 of the 6 takes after. The status says whose. The row says "the first
that answers", so the page does not prefer the link's computer, and a person
who joined by ada's link can read "through cy's computer" while ada's is on.

What the extra dials cost through the relay: 4-6 KB and 3-5 dropped packets
when a lender is asleep, which is the handshake's first packets sent to a key
the relay no longer holds. When every lender is awake, the cost is 7-10 KB
and about 15 packets, which is one more handshake and its close. These are
counted over the whole second visit (iroh-relay's `bytes_recv`,
`send_packets_recv`, `send_packets_dropped`), so the computers' own traffic
in that span is in both columns.

Not measured: n0's relays, a phone on cellular, a group with many lenders
(each one adds a handshake), a second host. Those are the operator's.

# v1ui-race (b): a page opened with no link

`cargo test -p svrngs-node --test browser --
the_page_opened_with_no_link_offers_a_founder_s_line`. Before the fix
(`b-red.out`, at e23af90), the page read `asked.has` with nothing asked
(web/app.js `opening`, db36fec's line). It caught the TypeError and told the
person "This browser could not open the page.", on a computer and on a phone,
in both engines. After the fix (`b-green.out`, at 58eb820), it says "Open
this page from your group’s link." 58eb820's body cites the red commit as
"85df…". That is wrong: the red commit is e23af90.

# The close: scripts/check.sh

`check-1.out`: `scripts/check.sh` at 0b61d06, both parts in, exit 0 after
12m15s. Every step passed, the browser checks and the conformance canary
with them. `python3 scripts/lean_check.py` beside it: 16,875 of 16,876 for
the repository, 3,149 of 3,150 for svrngs-core. One host, one run: not a
rate (AGENTS.md 7).
