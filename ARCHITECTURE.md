# Architecture

The map of svrngs as it is built today. What it is built toward is
`docs/DESIGN.md`. This file says what exists, and it changes in the same commit
as what it describes.

## Bird's eye

svrngs owns the group's code, forked from commonwealth-ai's ring slice (DESIGN
§4). The record and the journal on disk have landed; sync, the routes and the
member door come next, forked the same way (DESIGN §8, U1-U3); of sync, the
dialling and serving halves over iroh have landed, and the round and its
route, and the node that runs them. Nothing runs as a program yet: the node
is a library type, and the `svrngs` command that starts one is U2's.

## Code map

**`crates/svrngs-core`**: the group's record — signed acts (`Record`,
`Correct`, `Seal`, `Admit`, `Remove`), admission into one total order
(`admit.rs`), membership (`membership.rs`), the per-actor sync digest
(`sync.rs`, `view.rs`), attestation and introductions. Pure: no filesystem,
clock or network; time and keys arrive as arguments. It is commonwealth-ai's
`commonwealth-rail-core` at `e1a43bcd2`, with `oplog-types` (`op.rs`) and
`ContentHash` (`hash.rs`) inlined, and `hash.rs`'s wire test asserting the
JSON form directly rather than through kernel-types' `wire` module, which the
fork does not carry. Every source file carries its `ported from` line (the
JSON fixtures came over unchanged);
`scripts/ports.py` lists each beside its original. It keeps upstream's edition,
rustfmt policy and clippy allowances so a diff against upstream shows only
what svrngs changed.

**`crates/svrngs-node`**: a group's journal on disk — one append-only JSONL
log per group under `<root>/rings/<ns>/` (`journal.rs`), the single-writer
door that signs and the peer-ingest path that does not, roster sources, and
the checkpoint (`ring_checkpoint.rs`). The writer underneath is `oplog.rs`. It
is commonwealth-ai's `commonwealth-rail` and `oplog` at `e1a43bcd2`, forked the
same way as the core; its one edit is gating `oplog.rs`'s test-only serde
import. Upstream's shape stays: `RingRail` keeps its open journals in a map
behind a `Mutex` (`lib.rs`), which an actor per journal replaces when the node
becomes ours (DESIGN §5).

Beside the journals, `svrngs-node` holds the dialling half of sync over iroh
(DESIGN §8, U1's inventory), forked at `e1a43bcd2` the same way:
`peer_wire.rs` is `sovereign-peer-wire` whole (the sync bodies, the body limit
and the exchange budget); `alpn.rs` is `mesh-reach`'s `alpn.rs` with its one
`ALPN`; `guest.rs` and `guest/tunnel.rs` are `mesh-reach`'s guest dialer, the
relayed endpoint and the one loopback bridge (`HttpBridge`, `pump`); `iroh.rs`
is `commonwealth-transport`'s client side (`IrohTransport`,
`format_dial_string`, `preferred_bridge_port`). Its edits: no IP transport;
one ALPN, so the per-class ALPNs (`RPC_ALPN`, `CLIENT_ALPN`, `GUEST_ALPN`,
`OFFER_ALPN`, `MEDIA_ALPN`, `APP_ALPN`) and `alpn_for_class` go, and the guest
tunnel dials `ALPN`; no bench relay pin (`relay_pin_active`,
`build_relay_only_endpoint`); no `PeerTransport`, `PeerContact`,
`PeerEndpoint` or `TrafficClass`, so the transport is a concrete type whose
`base_url` takes a `Contact` (a key with optional dial hints) and its bridges
are keyed by peer alone; `NodePubkey` becomes iroh's `PublicKey`; the sync
bodies import the record from `svrngs-core`. `iroh.rs` also holds the
serving half, upstream's accept loop (`IrohAcceptor`) forked without its
`Forward` kinds and then changed: `IrohAcceptor::spawn` takes an admit decision
on each connection's ALPN and verified key (`Admit`), closing a refused
connection, and an `axum::Router` that hyper's HTTP/1 server serves on every
bi-stream of an admitted one, each request carrying the verified key as a
`Dialer` extension. There is no loopback listener on the serving side and no
request head parsed but by hyper. The bridge's own tests splice through a test
stand-in, since they test the bridge, not what the acceptor serves;
`tests/accept.rs` tests the acceptor.
iroh is pinned at `=1.0.2`, and its tree locked at upstream's versions, so the
fork runs on what upstream tested.

Ring sync came over the same way: `ring_sync.rs` and `ring_sync/journal.rs`
are `commonwealth-rails`' round, chunked exchange and responder (`answer`),
and the seam the round reads (`RingSyncHost`, `RingSyncJournal`, implemented
for `RingRail`); `ring_routes.rs` is its sync route, with `rail.rs`'s `err`.
Their edits cut the mesh ties: `RoundMembers` loses its walk of the mesh
(`RoundMembers::of`), and is built from named keys instead, which the node
reads from the seeds until the register answers (DESIGN §8, U1); the mesh proof stamp the exchange
carried goes; `RingSyncHost::transport` is the concrete `IrohTransport`, and a
contact's addresses are its one `base_url`; `roster_names` reads iroh's
`PublicKey`; `peer_client` is `pub`, since its one upstream caller,
`RailsDaemon`'s `impl RingSyncHost`, did not come over, nor did the append
wrapper that nudged the round. The route's live and checkpoint siblings stay
behind, and with them `RingInbound`'s live buffer and origin registry; the
route is an `axum::Router` where upstream's was a `RouteBundle`. Then it
changed: the route reads the asker from the acceptor's `Dialer` extension,
where upstream read an `X-Mesh-Pubkey` header, and refuses (403) a request
with no `Dialer`, where upstream served it as a local process — the route is
served only on the iroh stream. Its tests are served there too, through
`iroh::tests::serve_on_stream`. Of the tests,
`ring_sync/tests.rs` came over but for the derived-roster prune test, which
installs upstream's `MembershipRosterSource` over a mesh, and
`ring_routes/tests.rs` kept only its sync test; `round_tests.rs` is U1's to
rewrite.

What runs is the node (`node.rs`), new code in place of upstream's
`RailsDaemon` and `RailsNode` wiring, which did not come over. `Node::start`
takes a data dir, a signing key and a `RelayConfig`; it binds the endpoint
with that key, so the key that signs a group's acts is the key iroh verifies
when the node dials (DESIGN §8, "Four facts"), and holds the `RingRail`
signed by it, an `IrohTransport` over the endpoint, the accept loop serving
the sync route's router on the one ALPN, and the round, which runs at start
and every sixty seconds (`Node::round` runs one now). Until the register
answers (DESIGN §8, U1) the seed — each group's `roster.json` — does: the
round dials every key the seeds of the groups the node holds name, by key,
through addresses a caller seeds on the transport or iroh's lookup finds; the
accept loop admits any dialer; and the sync route refuses (403) a dialer its
group's seed does not name. `tests/node_sync.rs` runs two nodes and a
stranger in one process. The round reads the node's `Host` and the
`RingRail` directly: upstream's round read them through the `RingSyncHost`
and `RingSyncJournal` traits, and with one implementor of each here the
traits went (RUST_ARCH 7); `ring_sync::roster_of` is what remains of the
second.

**`crates/svrngs-engine`**: the `Engine` trait and QuickJS behind it — a fold
engine that waits in DESIGN §9 until an app needs its state offline on a device
that is not its host.

**`apps/expenses/`**: the Expenses fold (`expenses.js`) and its tests, from E0,
the bot at tag `e0-bot`. Its reducer (`init`, `reduce`) is what the localhost
Expenses app of DESIGN U3 builds on; `propose`, `say` and `due` answered chat
commands and go with the bot.

**`apps/films/`, `apps/tap/`**: so far only each app's `nouns.txt` — the words
the infra may never use. Films is DESIGN U5; Tap is a canary (DESIGN §8).

**`bridge/`**: the Discord door's first code — `setup.mjs` and `discord.mjs`
(the bot's application, invite and channels), `words.json` (the plain-words
list), and `bridge.mjs`, E0's bridge core, kept for its log and tests until the
door (DESIGN U4) replaces it.

**`demo/`**: how a demo is recorded. **`ralph/`**: the work loop. A queue is a
directory `ralph/next/<queue>/` holding `queue.toml` (models, timeouts, the
launch line), `STATE.md` (its rows) and `PROMPT.addendum.md`, which the driver
renders over `ralph/PROMPT.base.md`; E0's queue at tag `e0-bot`
(`ralph/next/e0/`) is a working example. The driver is commonwealth-ai's
`scripts/ralph.py`, run as `python3 ~/dev/commonwealth-ai/scripts/ralph.py run
--workdir ~/dev/svrngs --queue <queue>` — process tooling, not code svrngs
links; `scripts/ralph-mark.sh` expects to run under it. U1's queue is
`ralph/next/u1/`.

**`scripts/`**: `check.sh` is the definition of done; `lean_check.py` keeps the
repository standalone and within budget; `vocab_check.py` keeps every app's
nouns (`apps/<app>/nouns.txt`) out of infra code; `ports.py` lists the fork's pairs;
`principles_check.py` keeps `docs/principles/` pinned; `ralph-mark.sh` marks a
queue row done.
