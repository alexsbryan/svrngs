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

## (b) Green, and its plants

The test now waits on the Trip case with no round of Ana's second
computer's own: each computer of Ana's starts with its round a day apart
(`Node::start_every`, as tests/scale.rs does), so only a computer telling
it can reach it there; the off cases ask its rounds, as a computer back on
runs one.

`cargo test -p svrngs --test leave_page -- --nocapture` (`b-green.out`):
4 passed, both engines of both tests, 27.5 s.

| plant | run | fails at |
|---|---|---|
| the leave told only to the computers it did not cut, as at 1956f9bf (removal.rs `left != cut.contains(k)`) | `b-plant-filter.out` | "on, Ana's second computer says she left Trip" (Films passed, told by `told_cut`) |
| `told_cut` answering nothing (`return None` first) | `b-plant-told.out` | "back on, Ana's second computer says she left Films" |
| the router for a computer cut everywhere as at 1956f9bf (`door::gone` alone) | `b-plant-gone.out` | "back on, Ana's second computer says she left Walk" (Films and Trip passed) |

## (d) Glassbox (#181)

`cargo test -p svrngs --test join_by_link a_computer_the_group_cut`:
Hal's key, cut in house at Sam's, opens Sam's next link from a computer
whose copy of house cannot be read.

| run | what it says |
|---|---|
| red, join.rs and redeem.rs as at f53e06f6 (`d-red.out`) | `Err(LinkRefused)`: the link refused |
| plant, join.rs's `Err` arm tracing nothing (`d-plant-trace.out`) | fails on "join: whether this computer was cut in the group cannot be read here" |
| plant, the issuer's `Cut` arm tracing nothing (`d-plant-issuer-trace.out`) | fails on "redeem: refused — the group cut this key" |
| green | `Refusal::Left(house)`, both traces; join_by_link 11 passed |
