# svrngs relay — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so.

## What the operator decided, 2026-10-12

- "I do think we need to host our own relay (can't rely on n0) before our
  v1 ship. And ideally however we do that is something packageable to make
  it easy for anyone who wants to run it themselves".
- Packaged as **a `svrngs relay` verb** in the svrngs binary, not n0's
  `iroh-relay` binary beside it: one install line, version-locked to the
  computers that dial it, and the relay svrngs runs for everyone is the one
  anyone runs for themselves.
- **n0's address lookup goes too**, not its relays alone: "setting
  `relay_urls` alone does NOT stop the n0 DNS lookup"
  (`crates/svrngs-node/src/guest.rs:52`), and DESIGN §11 risk 5 is that
  lookup (`docs/DESIGN.md:2269-2272`). Each computer's relay is learned from
  its group's record — DESIGN's own open question, "computers learning each
  other's relay from the group's record" (`docs/DESIGN.md:2040`).
- **The budget is raised for this queue**: the repository's cap from 15,350
  to 15,800 and svrngs-core's from 3,080 to 3,130 — each raise in its own
  commit, with the operator's words and the counts, taken by the first row
  that needs it.

## What the operator decided, 2026-10-13

- Asked what DigitalOcean account `scripts/deploy-relay.sh` needs: "Can we
  actually just park this here right at the gate and keep using n0 until
  I'm actually ready to start paying / flip the switch for production?"
- So **svrngs's relay is built and parked at the gate**: n0's relays and
  address lookup are the command's default again, chosen, until the
  operator flips the switch. Nothing this queue built goes: `svrngs relay`,
  each computer's relay in its group's record, `new --relay`, the deploy
  scripts and the guard all stay, and a group that keeps its own relay is
  still met there.
- **The switch is one constant**, a closed set (n0's, or svrngs's at
  `relay.svrnme.sh`). Flipping it, then `deploy-relay.sh`, then
  `deploy-page.sh`, is the operator's, in that order.
- **A link carries its issuer's relay in either posture**, so the phone's
  page dials only the relays its link and its group name, and never asks
  n0's lookup, before the switch or after it.

## Why

A browser's iroh is relay-only (`docs/DESIGN.md:298`), so every byte a phone
moves crosses a relay — today n0's, under n0's terms for production use
(`:2040`), at 8.1 Mbit/s measured (`:670`). And members find each other's
relay through n0's DNS by default, "and a group is unreachable through it
while it is down" (`:2269-2272`). The design is svrn-docs
`rings/current/GROUP_RELAYS.md`: a relay is something a group's members can
run; svrngs's own is the default, a group's own overrides it.

## What exists to build on

- `RelayConfig { relay_urls, n0_services, loopback_only }`
  (`crates/svrngs-node/src/guest.rs:41-74`): without `n0_services` the
  endpoint "reaches peers by gossiped direct addrs and/or self-hosted
  `relay_urls` alone" (`:86-94`). The command builds `RelayConfig::default()`,
  n0's, with no option (`crates/svrngs/src/main.rs:46`).
- A link already carries its issuer's home relay when lookup is off
  (`crates/svrngs/src/cli.rs:1695-1725`, `link_relay`;
  `crates/svrngs-core/src/link.rs:90-112`).
- Every relay URL is spelled dotless for WebKit (`crates/svrngs-node/src/relay.rs:2-13`).
- iroh-relay 1.0.2's server is a dev-dependency the tests run
  (`crates/svrngs-node/Cargo.toml:72`, `crates/svrngs/Cargo.toml:42`), with
  `AccessControl::on_connect` (`server.rs:285-293`), `CertConfig::LetsEncrypt`
  (`:527-537`), `QuicConfig` (`:430-446`, port 7842 by `defaults.rs:7`) and
  `Limits`/`ClientRateLimit` (`:489-510`). Its ACME client is
  tokio-rustls-acme 0.9.1; whether it can ask Let's Encrypt's `shortlived`
  profile, which a bare-IP certificate requires (GA 2026-01-15), is unread.
- install.sh keeps svrngs running at login: a launchd agent on macOS, a
  `systemd --user` unit on Linux (`web/install.sh:61-114`).

## Decide

Row order, splitting and re-scoping a row that cannot execute; where a
computer's relay lives in the record (on its `Admit`, an act of its own, or
both) by what the round and a phone each need; the verb's flags, in DESIGN
§1's words for the person who runs it; how a relay on a server keeps
running (a system unit printed for its operator to install, never `sudo`
run here); every rate limit from a measurement, said — never a number
nobody chose (the operator, 2026-10-12, on limits); the smaller reversible
step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- deploying `relay.svrnme.sh` or any server, its DNS, its account, its
  cost — the queue writes `scripts/deploy-relay.sh` and never runs it;
- flipping the switch to svrngs's relay (the operator, 2026-10-13);
- publishing a build whose default relay is one not yet answering;
- `sudo`, a system service installed on this host, or opening a port on
  this host's network;
- a second ACME client beside iroh-relay's, or any service run by a third
  party in the relay's path;
- the budget past 15,800, or svrngs-core's past 3,130;
- a new crate, or a dependency on a commonwealth-ai or sovereign crate;
- docker, OrbStack or any tool that is not open software;
- anything that needs a second computer or a public address this host
  cannot give (say what, and the command that would run it);
- pushing, history rewrites, `--no-verify`;
- anything whose evidence you could not reproduce.

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
