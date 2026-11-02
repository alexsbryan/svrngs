# A group with the internet gone, on the shipped binary — predictions, before any data

The question: with the internet gone, as on a Lightning Mesh with its modem
off, do the shipped `svrngs` binaries keep a group working through a relay
on the local network? The shipped command turns on n0's address lookup and
pkarr publishing whenever it runs (`cli::SHIPPED = MeetAt::N0`,
`crates/svrngs/src/cli.rs:266`; `n0_services: at == MeetAt::N0`, `:797`),
even when every group it holds keeps its own relay. Offline those lookups
fail. Do they stall a dial, a join or a restart, or are they only noise?

Written and committed before the driver ran. One host, one run: not a rate
(AGENTS.md 7). No person did anything, so no people bar is judged.

## The setup

- **The binary**: `cargo build --release -p svrngs` in a clean detached
  worktree at `227193b1`, the commit that added `svrngs relay --at`. Every
  computer and the relay run that one binary as typed, no test harness: n0's
  lookup on, as shipped.
- **Modem off**: every `svrngs` (the relay's and each computer's) and each
  lent app runs under `sandbox-exec -f modem-off.sb`: everything allowed
  except outbound network, then outbound allowed to loopback and to unix
  sockets other than the resolver's, `/private/var/run/mDNSResponder`. So a
  name lookup and a connection off this host each fail at once, as on a mesh
  whose gateway withdrew its default route. The driver itself, which only
  reads each computer's API on loopback and its disk, runs outside it.
- **Validating the instrument first** (AGENTS.md 5, 7): inside the profile,
  `curl -m5 https://example.com`, `curl -m5 https://1.1.1.1`,
  `dig +time=2 +tries=1 example.com` and `host example.com` (the plants),
  and `curl` of a server on 127.0.0.1 (the control); outside it, the same
  `curl https://example.com`, to show this host's own network was up and the
  failure is the profile's.
- **The relay**: `svrngs relay --at 127.0.0.1:<port> --open`, a HOME of its
  own; the one port, chosen free at the start, for the whole run.
- **The computers**: Sam, Ana, Alex, then Dee, each a HOME and
  `SVRNGS_DIR` of their own, `RUST_LOG` at debug for `cli`, `transport`,
  `join`, `group`, `svrngs_node` and iroh's `address_lookup`,
  `socket::remote_map` and `net_report`. Sam: `svrngs new mesh --as Sam
  --relay http://127.0.0.1:<port>`; the others `svrngs join <the
  svrngs://join link> --as <name>`. No page is served: a phone on a mesh
  with no internet has no group page to open, so the phone is not run.
- **The lends**: a page served by `python3 -m http.server` on loopback,
  sandboxed: Ana's `notes` before the restart, Alex's `board` after, each
  `svrngs share <port> --kind page --name <name>`. A lend is *listed* when
  the computer's `GET /v1/lends?namespace=<group>` names it, and *fetched*
  when a `GET /` at its origin on that computer's own listener
  (`Host: <label>.localhost:<port>`, as a browser sends it) answers 200 with
  the page's own words.
- **Sampled**: every 2 s, `lsof -i -n -P` of every process the run starts,
  each new (process, protocol, remote) kept once.

## Predictions

### The instrument

0. **The plants fail and the control passes.** Inside the profile, `curl
   https://example.com` fails "Could not resolve host" within 1 s; `curl
   https://1.1.1.1` fails to connect within 1 s; `dig` and `host` reach no
   server; `curl` of 127.0.0.1 answers 200. Outside it, `curl
   https://example.com` answers 200. If a plant passes, the run stops: the
   instrument is not modem off.

### Before the restart

1. **The relay** says "Relaying at http://127.0.0.1:<port>/ for every
   computer and phone that reaches it, each sending at most 70 Mbit/s. Leave
   this running." within 2 s.
2. **The group.** Sam's `new` prints a `svrngs://join/` link within 10 s
   (`ONLINE_WAIT`, cli.rs:270; the 2026-10-12 n0-off run's 3,348 ms). Each
   computer's trace names that relay alone as where it meets its groups
   ("cli: the relays this computer meets its groups at", `relay_urls` one
   entry) and its egress posture `n0_services=true`. Ana's and Alex's `join`
   each print "You're in mesh, as <name>." within 5 s of starting (n0-off's
   235–408 ms; the 2026-10-13 run's line, install included, 6.7–8.6 s).
3. **A file.** 1 MB put in Ana's folder is on Sam's and Alex's disks, the
   same bytes, within 10 s (2026-10-13: 0.52–0.62 s with computers reaching
   each other directly; here the profile denies this host's own LAN
   address, so every path is the relay's, and slower).
4. **A lend.** Ana's `share` exits 0 within 2 s; `notes` listed on Sam's and
   Alex's computers within 5 s of it (2026-10-13: 538–991 ms after the
   line's exit) and fetched there, 200 with its words, within 5 s more.

### The n0 lookups, as they run offline

5. **They are attempted, and fail fast.** Every computer's trace shows
   pkarr publishing attempted ("Publishing endpoint info to pkarr") and
   failing ("Failed to publish to pkarr", iroh-1.0.2
   `address_lookup/pkarr.rs:391`), each failure within 1 s of its attempt,
   retried after 1, 2, 3 … s; and an address lookup started beside each new
   dial (iroh `remote_state.rs:866`) failing within 1 s ("DNS lookup
   failed", "Address Lookup failed"). If no trace shows a lookup attempted,
   whether they stall anything is could-not-judge, said so.
6. **They are noise.** Every dial between members carries the relay the
   group names for it (`Contact::named`, svrngs-node `iroh.rs:111`), so it
   has a path and does not wait on a lookup (iroh `path_state.rs:161-166`);
   no trace says "a bare key — dialled through the endpoint's address
   lookup" of a member, except, if any, a newcomer's computer before its
   relay is in the record. No step below misses its bound by a lookup.

### The cold restart

7. **Stopping.** Ctrl-C (SIGINT) to Sam's, Ana's, Alex's `svrngs` and the
   relay: each exits within 10 s.
8. **Coming back**, the relay first at the same address, then the three
   computers' bare `svrngs` at once, all sandboxed:
   - each computer's API answers within 10 s of its start;
   - each computer's first round reaching both others ("ring sync: round"
     `peers_reached=2`) within 70 s of its start (a round as it starts, then
     one each 60 s, `ring_sync.rs:107`; a first round before the relay
     connection is up would reach nobody and wait for the next);
   - Ana's `notes` listed on Sam's and Alex's within 15 s of their starts,
     and fetched, 200, within 5 s more;
   - a new 1 MB file of Alex's on Sam's and Ana's disks within 15 s.
9. **A new lend after the restart**: Alex's `board` shared; listed on Sam's
   and Ana's within 5 s, fetched 200 within 5 s more.

### A newcomer offline

10. **Dee.** Sam's `svrngs` stopped and `svrngs link mesh` run in its
    place (it holds the data dir, so `link` needs it free, cli.rs:738; it
    stays running as Sam's computer): a new link within 10 s. Dee's `join`
    by it prints "You're in mesh, as Dee." within 5 s; Dee's computer lists
    `notes` and `board` within 30 s and fetches each, 200; both files on
    Dee's disk within 30 s of Dee being in.

### Off this host

11. **No socket of any sandboxed process to an address off loopback** in
    any sample — not n0's, not this network's router (the 2026-10-13 run saw
    iroh's port mapper reach 192.168.1.1:5000; the profile now denies it).
    The pkarr and lookup failures are the only trace of n0.

## What this does not model, each said

- **A blackholed path**: modem on, the ISP down, packets sent and never
  answered. A lookup then waits for its timeout instead of failing at once,
  and a bare-key dial waits with it. Modelling it needs `pf` and `sudo`, the
  operator's: never-ran.
- **The radio**: a mesh's loss, latency and hops across routers; every
  computer here is on one host's loopback.
- **Direct paths on the LAN**: computers on a mesh also reach each other
  directly; on one host the profile allows loopback alone, so everything
  rides the relay. Stricter than the mesh, not the same.
- **A phone**: with no internet it has no group page to open
  (`group.svrnme.sh`), so never-ran here.
