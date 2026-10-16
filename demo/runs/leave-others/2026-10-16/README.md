# groups-leave-others: Ana's other computer (#179)

One host, one run of each: not a rate (AGENTS.md 7).

## (a) Red at 1956f9bf

`cargo test -p svrngs --test leave_page --
a_leave_reaches_the_leavers_other_computer::chromium --nocapture`, with the
test's three `until_left` asserts and its last loop's printing their
verdicts instead of stopping at the first, so all three cases show in one
run (`a-red.out`). The test as committed asserts them; run as committed it
stops at the first, "back on, Ana's second computer says she left Films"
(leave_page.rs, 103.6 s).

Sam's computer makes Films, Trip and Walk, all on this host at a relay on
127.0.0.1. Ana joins each from her first computer, and her second joins
each from her first's links for her devices, as Ana.

| case | Ana's second computer | within | verdict |
|---|---|---:|---|
| Films, left by `POST /v1/leaves` (as `svrngs leave` asks), second off, in Trip and Walk at Sam's | `/v1/groups` says left | 90 s | **false** |
| Trip, left by "Leave Trip" on her first computer's home page, second on | `/v1/groups` says left | 30 s | **false** |
| Walk, left by `POST /v1/leaves`, second off, in no group at Sam's | `/v1/groups` says left | 90 s | **false** |
| its home page, `#left p` | lists Films, Trip, Walk under "Groups you left" | — | **none listed**; its heading is still "Trip (Sam’s, since 16 Oct)" |

Her first computer's page said "You left Trip. Rejoining needs a new
invitation, and this computer can’t rejoin yet." Sam's computer held her
second out of Films and Walk within 30 s each (the test's controls).

Why, by the code at 1956f9bf:
- crates/svrngs-node/src/removal.rs `cut`, `left != cut.contains(k)`: a
  leave is told to the computers it did not cut, so not to the leaver's
  own other computers, which it cut.
- node.rs's acceptor serves a dialer cut everywhere here `door::gone()`
  on every route; ring_routes.rs `standing_asker` refuses a dialer not
  standing in the group it asks. Neither tells it its cut.

#179 reproduces.
