# Architecture

The map of svrngs as it is built today. What it is built toward is
`docs/DESIGN.md`. This file says what exists, and it changes in the same commit
as what it describes.

## Bird's eye

svrngs owns the group's code, forked from commonwealth-ai's ring slice (DESIGN
§4). The record and the journal on disk have landed; sync, the routes and the
member door come next, forked the same way (DESIGN §8, U1-U3); of sync, the
dialling and serving halves over iroh have landed, and the round and its
route, and the node that runs them. The `svrngs` command runs a node: it
makes a group, mints its link and runs for every group the computer holds,
serving the record to the computer's apps over loopback (U3, `rail.rs`);
joining by that link is next (U2).

## Code map

**`crates/svrngs-core`**: the group's record — signed acts (`Record`,
`Correct`, `Seal`, `Admit`, `Remove`, `Invite`), the lines a computer holds as
a set (`ActSet`, `act_set.rs`), admission of that set into one total order
(`admit.rs`; only `admit` makes an `Admission`), membership (`membership.rs`), the per-actor sync digest
(`sync.rs`, `view.rs`), attestation and introductions. `no_std`, so the
compiler refuses filesystem, clock, network, threads and `HashMap`; time and
keys arrive as arguments. It is commonwealth-ai's
`commonwealth-rail-core` at `e1a43bcd2`, with `oplog-types` (`op.rs`) and
`ContentHash` (`hash.rs`) inlined, and `hash.rs`'s wire test asserting the
JSON form directly rather than through kernel-types' `wire` module, which the
fork does not carry. Every source file carries its `ported from` line (the
JSON fixtures came over unchanged);
`scripts/ports.py` lists each beside its original. It keeps upstream's edition,
rustfmt policy and clippy allowances so a diff against upstream shows only
what svrngs changed. Then it changed (U2, DESIGN §6 "Ids cover everything
signed"): an act's id is the whole BLAKE3 hash, 64 hex, of its namespace,
actor, seq, previous act, timestamp and body (`sig.rs`'s `act_id`), and the
signature is over the id, where upstream signed the fields and kept 64 bits
of a hash of the line, signature included; `OpId` holds the whole hash, so a
line whose id is not 64 hex does not parse. Each act names its author's act
at seq − 1 (`SignedOp::prev`), and admission names one that does not a
`WrongPrevious` gap and admits it not. Membership, the view and sync take the
id admission derived (`derive_id`), never the line's. A link is in the record
(DESIGN §6 "A link admits each person once"): an `Invite` names its key and
lapse, and `membership` counts an `Admit` redeeming one only under the rule
in `membership.rs`'s module docs, naming each refusal a `RefusedRedemption`.
An issuer's newer `Invite` closes their older one from its place in the
order, so what the older link admitted keeps standing.

**`crates/svrngs-node`**: a group's journal on disk — one append-only JSONL
log per group under `<root>/rings/<ns>/` (`journal.rs`), the single-writer
door that signs and the peer-ingest path that does not, and
the checkpoint (`ring_checkpoint.rs`). The writer underneath is `oplog.rs`. It
is commonwealth-ai's `commonwealth-rail` and `oplog` at `e1a43bcd2`, forked the
same way as the core; its one edit is gating `oplog.rs`'s test-only serde
import. Upstream's shape stays: `RingRail` keeps its open journals in a map
behind a `Mutex` (`lib.rs`), which an actor per journal replaces when the node
becomes ours (DESIGN §5). Then it changed (U2): the write index holds the
ids each act's content derives, so a line claiming a held act's id is not
skipped as held; the door names, as the previous act, the line at its newest
seq whose signature verifies under its own key; compaction keeps a cited
act by its derived id; and `RingJournal::append_if` signs one act and writes
it only when a caller's judgement of the journal with the act in it keeps
it, both under the write lock.

Beside the journals, `svrngs-node` holds the dialling half of sync over iroh
(DESIGN §8, U1's inventory), forked at `e1a43bcd2` the same way:
`peer_wire.rs` is `sovereign-peer-wire` whole (the sync bodies, the body limit
and the exchange budget); `alpn.rs` is `mesh-reach`'s `alpn.rs` with its one
`ALPN`, then the enum it promised (`Alpn`, U2); `guest.rs` and `guest/tunnel.rs` are `mesh-reach`'s guest dialer, the
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
bodies import the record from `svrngs-core`. Then it changed: a bare key, with
no address in its contact or seeded, is dialled when the endpoint has an
address lookup (n0's DNS under `RelayConfig::n0_services`), where upstream
refused it (DESIGN §8, "Four facts"). `iroh.rs` also holds the
serving half, upstream's accept loop (`IrohAcceptor`) forked without its
`Forward` kinds and then changed: `IrohAcceptor::spawn` takes an admit decision
on each connection's ALPN and verified key (`Admit`), closing a refused
connection and naming, for an admitted one, the `axum::Router` that hyper's
HTTP/1 server serves on its every bi-stream, each request carrying the verified key as a
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
reads from the register (DESIGN §8, U1); the mesh proof stamp the exchange
carried goes; `RingSyncHost::transport` is the concrete `IrohTransport`, and a
contact's addresses are its one `base_url`; `peer_client` is `pub`, since its one upstream caller,
`RailsDaemon`'s `impl RingSyncHost`, did not come over. The append
wrapper that nudges the round (`ring_sync::append`) came over in U3 with the
record API, which routes the append door through it. The live route
(`ring_live`) came over in U3, with `RingInbound`'s live buffer; it refused
an envelope for a group the rail does not hold, where upstream asked the
origin registry's live registrations, which did not come over. Then it
changed: it asks the sync route's question (`standing_refusal`), so a dialer
not standing in the group its envelope names — one the acceptor admitted
for another group — is refused 403, as is a request off the stream. The
checkpoint route stays behind; the routes are an `axum::Router` where
upstream's was a `RouteBundle`. Then it
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
takes a data dir, a signing key, a `RelayConfig` and the loopback address
it serves the record to this computer's apps on (`api::bind`, refusing any
other, before the endpoint; `api::serve` once the endpoint's transport,
which the live lane dials through, exists; `Node::api_addr` answers the port bound); it binds the endpoint
with that key, so the key that signs a group's acts is the key iroh verifies
when the node dials (DESIGN §8, "Four facts"), and holds the `RingRail`
signed by it, an `IrohTransport` over the endpoint, which the round and the
live lane share, the accept loop serving
the sync route's router on the members' ALPN and the redemption route on
the second, and the round, which runs at start, every sixty seconds and at
once after an app's append (one `Notify`, held by the round and the record
API)
(`Node::round` runs one now; `Node::sync_with` runs one exchange with a
peer the register may not name yet).

Redemption is `redeem.rs` (DESIGN §6, U2), new code in the shape of
commonwealth-rails' mesh join. A link carries only the issuer's key and the
link's secret, so the joiner first asks: `AskRequest`, the link's key and
its own, signed by the link's key, answered (`Asked`) with the group whose
`Invite` of this node's names that key, its seed as the command keeps it
(read through the `SeedText` function `Node::start` is handed, since the
seed is the command's) and the `Invite`'s id. Then `RedeemRequest` — the group's id, the
`Invite`'s id, a person's name, the new key, its consent and the link's
key, signed by the link's secret — posted on `Alpn::Redeem`, which the
accept loop serves to any dialer with these two routes. Each route is an
adapter around pure deciders: `parse_redeem` and `parse_ask` on what arrived
(the dialer is the new key and the link's key signed), so a refused request
reads nothing; `decide_admit` and `own_invite` on the group's acts (the
`Invite` is held, is this node's and names the link's key). `append_if`
writes the redeeming `Admit` only when `judge`, run again under the write
lock, finds the key not standing already and `membership`
(`register::membership_in`) counting the `Admit`.
Every refusal answers `REFUSAL`, one sentence naming no group; a key that
already stands is answered `AlreadyIn`, nothing written.

Who is in a group is the register's (`register.rs`, DESIGN §6, U1):
`RingRail::standing` answers a group's standing keys from `membership` over
its seed — the group's `roster.json`, the founder's key alone — and the acts
admission authenticates; a group the node does not hold answers `NotHeld`,
and one with no seed `NoSeed`, which refuses everyone. Its four callers: the
round dials every key standing in a group the node holds, by key, through
addresses a caller seeds on the transport or iroh's lookup finds, and offers
each group to its own; the accept loop admits a dialer standing in any group
the node holds and closes the rest; the sync route refuses (403) a dialer not
standing in the group its request names, and a group the node does not hold
in the same words; and the door (`RingJournal::append`) signs only for a key
standing in the seed and the acts it holds, under its write lock.
`tests/group_not_mesh.rs` runs a founder, a member it admits, a stranger and
a member of another group in one process. The round reads the node's `Host` and the
`RingRail` directly: upstream's round read them through the `RingSyncHost`
and `RingSyncJournal` traits, and with one implementor of each here the
traits went (RUST_ARCH 7). Upstream's other answers to who is in a ring —
`RosterSource`, `RingRail::derive_roster` and `default_roster`, which let a
ring's roster be derived from the mesh, `RingRail::roster` and
`roster_origin`, which chose between them and the file, the core's
`RosterOrigin`, and `ring_sync::roster_names` — went once the register had
taken their callers, and a peer's seal is pruned against the group's seed
file.

The record over loopback (DESIGN §8, U3's inventory, the rail.rs row) is
`rail.rs` and `api.rs`, commonwealth-rails' `rail.rs` and `api.rs` at
`e1a43bcd2`: the append door (`append_act`) and the log (`log_answer`), with
`RailQuery`, `namespace_of` and `journal_of`, served as an `axum::Router`
(`api::router`) by `api::serve`, which refuses any address but loopback
(`check_loopback`). Then it changed: every request's caller is asked the
same (`from_this_computer`), so a caller elsewhere, or one whose address is
unknown, is refused 403 `not_loopback`, naming it. Its edits: the doors read the journals from a `RailApi`
where upstream's read `RailsDaemon`; the roster they read is the group's
seed file, since upstream's `RingRail::roster` went in U1, and so the
not-in-roster sentence is the rail's own, where upstream chose it by the
roster's origin; `err` is the one `ring_routes.rs` carries; `serve`
answers the address it bound with the task (`Serving`, which stops the
listener when dropped), where upstream's answered the task alone. The live
lane came over in U3: `LiveBuffer`, bounded per group, which the live route
fills and `GET /v1/rail/live` drains, and `POST /v1/rail/live`; `live.rs`
is the inference daemon's fan-out (`routes_rail_live.rs`, `push_ephemeral`
and `offer_peer`), its walk of the mesh's Online members cut, so its caller
names the peers (`LivePeer`), each dialled at its one `base_url`. Then it
changed: the push door, which buffered locally and answered `peers: []`,
offers the payload to every key standing in the group but this node's
(`live_peers`, from the register), over the node's `IrohTransport`, held
by `RailApi`, and answers each peer and whether it took it. The sync
doors and `attest` stay behind, and with them the other routes beside them
in upstream's `api.rs`. Then it changed: the append door's
attestation branch went with `attest`. An append's `on_behalf_of` is the
person an app names when it serves someone else — whom the member door
verified (DESIGN §8, "Localhost is the trust boundary") — signed by this
computer, and refused (`not_a_name`, `not_in_group`) unless it parses as a
person standing in the group; upstream dropped a bare one.
The register stands where the roster stood: `journal_of` asks
`RingRail::standing` before a journal is opened, refusing a group this
computer does not hold (`not_held`, 404) and one held with no seed
(`no_seed`, 409); the door's own fold refuses a computer that does not
stand (`not_in_roster`, in DESIGN §1's words); and the log answers the
people standing after its one read (`people_json`: each person and their
key), where upstream's answered the roster. `GET /v1/rail/people` is new:
the same people, through the same gate (`standing_of`), so an app needs no
second source for who is in its group or a key to seal to (DESIGN §3).

The member door (DESIGN §8, U3's inventory) starts from `declared.rs`,
commonwealth-media's `declared.rs` at `e1a43bcd2` (:51-77, 164-247): a
lender's credential for its own service, a header value in a 0600 file
under a 0700 dir, the file named by its header, never logged. Its house
store (`house_dir_under`, `read_house_in`, `write_viewer_in`) and rails'
data-dir helpers stay behind with their two tests, and with `dir_under` the
`PathBuf` import.
Then it changed: a hardening that fails fails the write, where upstream
ignored it and reported a secret readable by others as stored; the file is
created 0600; and that a credential is set is traced at debug, since the door
reads the store on every request.

The door is `door.rs`, new code holding upstream's rules (DESIGN §8, U3's
inventory, its identity and forward_stream rows), merged into the members'
protocol's router beside sync and live. `/lend/<lend>/<path>` reaches the
lend's port on 127.0.0.1 as `/<path>`, one hyper HTTP/1 connection per
request, both bodies streamed and neither buffered. It strips the hop-by-hop
headers both ways, every client header under `x-svrngs-` and every one the
lend's secret names (compared as `rewrite_head` compared,
`iroh_identity_forward.rs:297-328`), then sets `x-svrngs-person` — the
dialer's name in the first group held that they stand in — and
`x-svrngs-key` from the `Dialer`, then the secret; every other header,
`Range`, `If-Range` and `Content-Range` among them, passes untouched. A
response header holding the secret is dropped. It refuses, by name, a
request off the stream, a dialer standing in no group held, a path with a
`..` segment or an encoded `.` or `/` (`route_by_prefix`'s guard), a lend
not declared, and a path the lend refuses. A lend is `lend.rs`: its
declaration, `<data dir>/lends/<lend>.json` (the port and the refused
paths, unknown fields refused), read per request, so a running node sees a
lend come and go; its secret is `declared.rs`'s store under
`<data dir>/secrets/lends/<lend>/`. A refused path covers what is under it
at a segment boundary, as upstream's `covers`, compared percent-decoded,
case-blind and with repeated slashes as one. `tests/door/` runs a lender
and a member through it, with upstream's `picky_origin` and Range origin
forked as fixtures.

**`crates/svrngs`**: the command (DESIGN §5), the one crate with a binary.
`identity.rs` is the device key, commonwealth-transport's `identity.rs`
:1-126 at `e1a43bcd2`, its join proof and dial-info signing left behind, its
`node_pubkey` answering hex until svrngs-core's `actor_of`, the one hex
spelling of a key, took its callers; then changed: a key file that cannot be read,
and a fresh key that cannot be written, refuse naming the file, where
upstream fell back to a key nobody keeps, and the file is `identity`, a word
a person may read. `link.rs` is mesh-join-vocab's https guest link
(`deep_link.rs:228-300`) and its `sovereign://join/` form (`:421-436`),
made one short base64url `Token` (u2-8): a `Format` byte, the issuer's key,
16 secret bytes the link's signing key is derived from, and the issuer's
relay only from a computer without address lookup. It is printed as the
client's link, `svrngs://join/<token>`, and as `<origin>#<token>` beside
it for a group that keeps a page origin; `Link::parse` reads either or the
bare token. `group.rs` is new: a group's `Seed` — its
name, the founder's name and key, 16 random bytes — whose BLAKE3 hash is
the group's id and so its directory; `seed.json` holds it beside the
record, and `Seed::roster` writes the register's seed, `roster.json`, the
founder's key bound to the founder's name; `group::answered` believes a
seed an issuer answers only when it hashes to the group named. `cli.rs` is new, in the shape of
commonwealth-rails' `cli.rs` and `main.rs`: verbs parsed by hand (`svrngs`,
`new <name> [--as] [--origin]`, `join <link> [--as]`, `link <group>
[--origin]`); one svrngs per
data dir (`$SVRNGS_DIR`, else `~/.svrngs`), held by a lock on
`<dir>/lock`; `new` and `link` append an `Invite` through the group's door
that lapses in a day and print the link; `join` asks a link's issuer what
it is for and redeems it there (`svrngs_node::redeem`), writes the group's
seed only once let in, and the origin only of an https link, syncs once from the issuer (`Node::sync_with`) and prints "You're in
<group>"; each verb then runs the node until Ctrl-C, serving the record
on `127.0.0.1` at a free port (`cli::API`), which the bare command prints. What a verb prints is a
closed `Said`, and refusals a closed `Refusal`, both in a person's words,
exit 1 for a refusal and 3 for a precondition absent. `tests/new.rs` runs
one node in process; `tests/join_by_link.rs`, U2's bar, runs Sam and six
others.

**`apps/expenses/`**: the Expenses fold (`expenses.js`) and its tests, from E0,
the bot at tag `e0-bot`. Its reducer (`init`, `reduce`) is what the localhost
Expenses app of DESIGN U3 builds on. Its chat handlers went with the bot and
are at that tag.

**`apps/films/`, `apps/tap/`**: so far only each app's `nouns.txt` — the words
the infra may never use. Films is DESIGN U5; Tap is a canary (DESIGN §8).

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
