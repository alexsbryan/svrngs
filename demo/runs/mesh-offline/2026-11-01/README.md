# A group with the internet gone, on the shipped binary — this host, 2026-11-01

The question: with the internet gone, as on a Lightning Mesh with its modem
off, do the shipped `svrngs` binaries keep a group working through
`svrngs relay --at` on the local network, and do n0's lookups, which the
shipped command turns on whatever relay its groups keep
(`crates/svrngs/src/cli.rs:266`, `:797`), stall a dial, a join or a
restart? Predictions first, in `PREDICTIONS.md` (`f2f0cd2a`); the driver
and the profile committed before they ran (`dbb18666`), and again, fixed,
before take 2 ran (`8d86d0a6`). One host, one run: not a rate (AGENTS.md
7). Nobody did anything, so no people bar is judged.

**The answer: they are noise.** Every computer tried to publish to pkarr
and to look peers up at n0's DNS, and every attempt failed; no dial waited
on one, and every step held its bound, the cold restart and a newcomer
after it included. Two things are slower than they need be, neither of them
n0's: an address lookup takes about 3 s to fail offline, not at once
(finding 2), and every `new`, `join` and `link` at a relay `--at` waits 3 s
before it prints (finding 3).

## How it ran

```
git worktree add --detach /tmp/claude-502/mesh-offline 227193b1
cargo build --release -p svrngs       # in that worktree, CARGO_INCREMENTAL=0, 1m36s
BIN=/tmp/claude-502/mesh-offline/target/release/svrngs W=/tmp/mesh-offline-2026-11-01-take2 \
  python3 demo/runs/mesh-offline/2026-11-01/run.py      # data/, exit 0
```

Take 2 ran 00:01:39–00:02:39 local on 2026-11-02 (`data/events.jsonl`,
`start` and `end`); the build `227193b1` from a clean tree, the binary's
sha256 `6556ddc9…` (`summary.json`, `start`); macOS 27.0.1, 12 cores, the
1-minute load 4.7 at the start and 5.9 at the end (the 2026-10-13 run's was
34.4 and 17.6, so its timings and these are not compared as rates). Take 1,
the same build, 23:55:20–00:01:18, is kept in `take-1/data/`; its lend
fetches were not an instrument (below).

The relay: `svrngs relay --at 127.0.0.1:51447 --open`, both times "Relaying
at http://127.0.0.1:51447/ for every computer and phone that reaches it,
each sending at most 70 Mbit/s. Leave this running." (`summary.json`,
`relay`). Every `svrngs`, the relay's and each computer's, and each lent
app (`python3 -m http.server`) ran under `sandbox-exec -f modem-off.sb`.
`data/<who>-<n>.log.gz` is each start's trace, `<n>` counting the starts:
Sam's `new` (1), bare `svrngs` after the restart (2) and `link` (3).

## The instrument

Checked before anything ran (`summary.json`, `instrument`): inside the
profile `curl https://example.com` failed "Could not resolve host" in 21 ms;
`curl https://1.1.1.1` "Failed to connect … after 1 ms"; `dig` and `host`
"no servers could be reached" (3,022 and 6,037 ms, their own timeouts); a
UDP send to this host's resolver, 100.100.100.100:53,
`PermissionError(1, 'Operation not permitted')` in 0 ms; `curl` of the
loopback app 200. Outside it, `curl https://example.com` 200 in 93 ms. So
the failures are the profile's, and this host's network was up.

What it does **not** cut, found after the run (finding 4): the profile's
`localhost` admits this host's own addresses. A sandboxed `curl` of a server
on `0.0.0.0` answered 200 at 192.168.1.11 and at 100.104.36.28 (this host's
Tailscale address), and a UDP send to each went. So computers on this host
kept direct paths to each other, as computers on a mesh's LAN would;
everything off this host stayed cut.

## Against the predictions

| | predicted | as it ran (take 2) | |
|---|---|---|---|
| 0 | the plants fail, the control passes | as above; every check `true` (`instrument.held`) | held |
| 1 | the relay's words within 2 s | 110 ms; again after the restart, 107 ms | held |
| 2 | the link within 10 s; one relay, `n0_services=true`; each join within 5 s | link 3,109 ms; every computer's trace "the relays this computer meets its groups at at=N0 unkept=false relay_urls=["http://127.0.0.1:51447/"]" and "iroh egress posture … n0_services=true" (`traces.<who>.relays_line`, `egress`); Ana in 3,118 ms, Alex 3,089 ms, "You're in mesh, as Ana." | held; the 3 s is finding 3 |
| 3 | 1 MB of Ana's on Sam's and Alex's disks within 10 s, all by the relay | 219 and 214 ms (`file_before`) | held; the reason was wrong, direct paths stayed open (finding 4) |
| 4 | `share` exit 0 within 2 s; listed within 5 s; fetched within 5 s more | exit 0 in 16 ms; listed on Sam's and Alex's 3 ms after it, fetched, 200 with its words, at 5 ms (`notes_before`) | held |
| 5 | pkarr and address lookups attempted, each failing within 1 s | **pkarr**: 46 publishes attempted, 46 failed, each in 0–10 ms, "dns error: Resolve failed, IPv4: Failed to resolve, IPv6: Failed to resolve", retried after 1, 2 … 8 s (`traces.*.pkarr_*`). **Address lookups**: 17 started at n0's DNS beside dials ("starting DNS lookup … origin_domain=dns.iroh.link."), 15 failed in **2,427–3,584 ms** "no calls succeeded: [Failed to resolve TXT record…]", 2 cut by Sam's stop (`dns_*`) | pkarr held; the lookups' 1 s **failed** (finding 2) |
| 6 | every member dialled with its relay; a bare key only of a newcomer; no bound missed by a lookup | three bare-key dials, all Sam's, each of a newcomer just after its join (Ana, Alex, Dee: "the group names no relay for this computer", svrngs-node `iroh.rs:114`), each answered in **0.9–1.0 ms** (`bare_key_dial_ms`), iroh already holding the newcomer's path from its own inbound dial; none after the restart; no bound missed | held |
| 7 | each exits within 10 s of Ctrl-C | 5, 5, 5 ms and the relay 2 ms, exit 0 (`stopped`) | held |
| 8 | API within 10 s; first round reaching both within 70 s; `notes` listed within 15 s, fetched within 5 more; Alex's file within 15 s | API 113–122 ms; the first round "peers_reached=2" at **91–99 ms** (`first_round_both_ms`); `notes` listed 117–124 ms, fetched 119–127 ms after each start; `after.bin` on Sam's and Ana's disks at 218 and 214 ms | held |
| 9 | `board` listed within 5 s, fetched within 5 more | listed 2 ms after the share, fetched 5 ms, on Sam's and Ana's (`board`) | held |
| 10 | a new link within 10 s; Dee in within 5 s; both lends within 30 s; both files within 30 s | Sam stopped in 1,035 ms; "A new link to mesh. …" at 3,056 ms; "You're in mesh, as Dee." at 3,136 ms; `notes` and `board` listed 13 ms and fetched 15 ms after; both files on Dee's disk at 18 ms (`link_again`, `dee`). The record's people at the end: Ana, Sam, Alex, Dee (`people`) | held |
| 11 | no sandboxed process's socket off loopback; n0 only in traces | 24 samples, every remote 127.0.0.1 (`lsof.jsonl`; `off_host`: `[]`); take 1's 167 samples the same. lsof sees connected sockets: an unconnected UDP send is not in it, and the direct path over 100.104.36.28 (finding 4) was such a send | held, as far as lsof sees |

Take 1 agrees wherever its instruments held (`take-1/data/events.jsonl`):
joins 3,126 and 3,108 ms, Dee's 3,108 ms; the file 207–208 ms; the first
rounds after the restart 99–1,102 ms; address lookups failing in
2,481–3,491 ms; Sam's bare-key dial of Alex answered 0.6 ms after it began
while its lookup ran on 3.2 s more (`take-1/data/sam-1.log.gz`, 07:55:39.857
to 07:55:43.033 UTC).

## What it found

Nothing here is fixed in this run's commit.

1. **n0's lookups are noise on a mesh with the modem off.** They are
   attempted — pkarr's publisher from the start of every computer
   (iroh-1.0.2 `address_lookup/pkarr.rs:376-417`), and an address lookup
   beside each dial iroh holds no selected path for
   (`socket/remote_map/remote_state.rs:866-880`) — and every one fails. A
   dial waits on its lookup only when iroh knows no path at all
   (`remote_state/path_state.rs:161-166`); here every dial had one: the
   relay the group's record names for the member (`Contact::named`,
   svrngs-node `iroh.rs:111`), or for a newcomer just in, the path its own
   dial left. So the cost is log lines, 46 warnings in a minute across four
   computers, and no step waited on n0.
2. **An address lookup takes about 3 s to fail offline**, 2,427–3,584 ms in
   both takes, though each send to the resolver is refused at once
   (`instrument`, 0 ms). A dial that does wait on one — a bare key iroh holds
   no path for, which svrngs dials only through the lookup (svrngs-node
   `iroh.rs:263`) — would wait that long and then fail. The window is a
   computer dialling a member its record names no relay for: Sam's three,
   just after each join, had paths; one with none (a computer restarted
   before a newcomer's relay reached its record) never-ran here, so whether
   it costs a join or a round is could-not-judge. With the modem on and the
   ISP down, the wait would be the resolver's timeout instead: never-ran.
3. **Every `new`, `join` and `link` at a relay `--at` waits 3.0 s before it
   prints, and n0 is not why.** In each trace iroh's report probes the
   relay's QUIC address discovery, "QADv4: probe timed out" 3.0 s after the
   start, then "home is now relay http://127.0.0.1:51447/", then "cli:
   waited for a relay online=true" (`sam-1.log.gz` 08:01:49.516 →
   08:01:52.517 → 08:01:52.518 UTC; `ana-1`, `sam-3`, `dee-1` the same).
   Read from the code, not tried: `--at` serves no QUIC
   (`crates/svrngs/src/relay.rs:146-148`), so the probe waits out iroh's
   `PROBES_TIMEOUT`, 3 s (iroh-1.0.2 `net_report/defaults.rs:23`), and
   `cli::wait_online` (`cli.rs:1800`) waits for the home relay the report
   sets. The bare `svrngs` does not wait: its first round reached both
   others within 99 ms.
4. **The instrument's `localhost` is this host, not loopback.** The profile
   cut everything off this host but left this host's own LAN and Tailscale
   addresses open, and iroh used one: the folder's fetch after the restart
   dialled Alex with "Ip(100.104.36.28:53057) Active" beside the relay
   (`ana-2.log.gz`, 08:02:07.435 UTC). `PREDICTIONS.md` said every path
   would be the relay's; it was not. On a mesh, computers on one LAN have
   such direct paths too, so the run stands for the mesh; it does not stand
   for "through the relay alone".
5. **The relay's own start lists n0's relays in its trace**: `cli::reach`
   runs for every verb (`crates/svrngs/src/main.rs:54`), and a relay's data
   dir holds no group, so it names n0's four ("relay: the host's trailing
   dot dropped url=https://aps1-1.relay.n0.iroh.link/", `relay-1.log.gz`).
   `svrngs relay` builds no endpoint (`cli.rs:809`): a trace, nothing
   dialled.

Take 1's driver was not an instrument for the lends: it fetched each lend
with no token, refused every time at svrngs-node `origin.rs:189` ("origin:
refused — neither this origin's session nor the token, and no whole tab's
navigation", `take-1/data/sam-1.log.gz`), and it waited for one computer
after another from one start, so Alex's 60 s went to Sam's wait (its
`notes-before`: Alex `listed_ms` null, while `alex-1.log.gz` says "lent=1"
at 07:56:40). Take 2 sends the computer's token, as a program on it does
(`origin.rs:164-165`), checks the lender's own fetch first, and waits for
each at once.

## What it leaves out, each said

- **A blackholed path** — modem on, ISP down, packets unanswered: needs
  `pf` and `sudo`, the operator's. Never-ran.
- **The radio**: a mesh's loss, latency and hops between routers; every
  computer here was on one host.
- **The relay on another box**: here it was on the computers' own host, at
  127.0.0.1; on a mesh it is at a LAN address, `--at 10.x.y.z:<port>`.
- **A phone**: with no internet it has no group page to open
  (`group.svrnme.sh`). Never-ran.
- **A DNS name asked**: the traces name each lookup; a packet capture would
  confirm none left, and needs `sudo`.
