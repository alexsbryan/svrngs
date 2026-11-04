# A group's relays in tiers, its laptop relay stopped and started: predictions, before any data

The question (svrn-docs `rings/current/GROUP_RELAYS.md` §7, the operator's
"Tiers", 2026-11-02): a group whose keeper names two relays of its own, as
two laptops' `svrngs relay --at` on a local network would be, beside the
build's relay. Does it keep working through the second when the first
stops, does a computer still reach a member that has moved, and does it go
to the build's relay only when both stop, and come back when they return?
How long does each move take?

Written and committed before the driver runs. One host, one run, so not a
rate (AGENTS.md 7). Nobody acts in it, so no people bar is judged.

## The setup

- **The binary**: `cargo build --release -p svrngs` at the commit that
  lands the tiers, from a clean tree. `cli::SHIPPED` is n0's, so the build's
  relay is n0's four relays with n0's lookup, online.
- **No direct path** (`no-direct.sb`): every `svrngs`, its relays and the lent
  app run under `sandbox-exec`. Everything is allowed except UDP sent to any of
  this host's own addresses. The 2026-11-01 run found the profile's
  `localhost` covers loopback, the LAN address and the Tailscale address
  (`demo/runs/mesh-offline/2026-11-01/README.md`, finding 4). iroh's direct
  paths are UDP and its relays are TCP, so two computers here reach each
  other only through a relay. UDP off this host still goes, n0's address
  discovery included.
- **Validating the instrument first** (AGENTS.md 5, 7): inside the profile,
  a UDP send to each of this host's addresses must be refused (the plants).
  The controls must pass: a UDP send to 1.1.1.1, a TCP fetch of the lent app
  on 127.0.0.1, and an HTTPS fetch of n0's `usw1-1/ping`. If any plant is
  not refused, or any control fails, the run stops.
- **The relays**: A and B, each `svrngs relay --at 127.0.0.1:<port> --open`,
  each in its own HOME.
- **The computers**: Sam runs `svrngs new house --as Sam --relay <A>
  --relay <B>`; Ana and Alex `svrngs join <link>`. Each has its own HOME and
  data dir. `RUST_LOG` is debug for `tiers`, `transport`, `cli`, `join`,
  `group`, `origin`, `door` and iroh's `net_report`.
- **The lend**: Ana's `notes`, a page from `python3 -m http.server`. It
  *works* on a computer when a `GET /` at its origin, on that computer's own
  listener with its token, answers 200 with the page's words. The driver
  asks every 0.5 s, each ask with a 12 s timeout, on Sam's and Alex's
  computers at once.
- **Home and tier**, per computer, come from its trace: `tiers: this
  computer meets its groups at another relay now … to=` and `… at another
  tier now from= to=` (svrngs-node `tiers.rs`).

## Predictions

0. **The instrument holds**: every plant refused, every control through.

1. **Before.** Sam's link within 15 s; each join within 15 s. `notes` is
   listed on Sam's and Alex's computers within 60 s of the share, and works
   there. Every computer is at tier `Own`, homed at A or B. The n0 relays are
   not in its endpoint: at `Own` the endpoint holds the group's own relays
   alone. It still probes nothing of n0's relays, though pkarr and the
   address lookup stay on.

2. **The relay Ana is homed at stops** (Ctrl-C).
   - The tier stays `Own`, since the other still answers.
   - Ana's home moves to the other within 30 s: iroh replaces a home that
     stops answering at its next report, every 20–26 s
     (iroh-1.0.2 socket.rs:2004-2018).
   - The lend works again on Sam's and on Alex's computers within 60 s of
     the stop. The connection kept through the dead relay fails a request,
     `fetch` dials afresh (svrngs-node `iroh.rs` `fetch`, `close_every`), and
     the fresh dial reaches Ana at the other relay because a dial carries
     every relay of both tiers (`Contact::named`). Nothing else can carry it:
     no direct path, and n0 is not where Ana is. So a lend that works again
     is the dial reaching a member that moved.
   - Some fetches in between fail, or take up to their 12 s timeout.

3. **It starts again at its address.**
   - Homes do not return to it within 90 s. Both relays are on this host and
     equally near, and iroh moves home only to one a third faster
     (net_report.rs:810-816), so the second stays home. If one does return,
     it is iroh's report that moves it, within 26 s.
   - The lend still works.

4. **Both stop.**
   - Every computer goes to tier `Unanswered` within 13 s of the stops: one
     ask (`tiers::ASKED_EVERY`, 10 s) and its time to answer
     (`ANSWERS_WITHIN`, 3 s).
   - Each is homed at one of n0's relays within 30 s of the stops: adding a
     relay starts a report at once.
   - The lend works again through n0 within 90 s on both computers. It waits
     for Ana's home to move and for the failing connection to be let go, as
     in 2.

5. **Both start again.**
   - Every computer is back at tier `Own` within 13 s of the restart, with
     n0's relays taken from its endpoint.
   - Each is homed at A or B within 30 s of the restart.
   - The lend works within 60 s.

## The modem-off rerun

`modem-off/` holds the 2026-11-01 driver and profile, byte for byte. It
runs with this build, its own predictions taken as that run's
(`demo/runs/mesh-offline/2026-11-01/PREDICTIONS.md`), with one addition:

6. **The build's relay is noise offline.** Every bound that run held still
   holds. Sam's `new --relay`, each join and the bare restart reach tier
   `Own` at the group's relay, with n0's relays out of the endpoint.
   pkarr's failed publishes and n0's failed lookups are the only trace of
   n0, and no step waits on them. The 3 s at `new`, `join` and `link`
   (that run's finding 3) stays.

## What this does not model, each said

- **The radio and real laptops**: every relay and computer is on one host's
  loopback.
- **A phone**: it binds one relay of its link. Not in this slice.
- **A relay with QUIC**: `svrngs relay <name>` serves QUIC address
  discovery, so iroh would measure it properly. Not run here.
- **A group with no relays of its own beside this one** (tier `Unowned`): the
  driver holds one group. The home page names that case.
